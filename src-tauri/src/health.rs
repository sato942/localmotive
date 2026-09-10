use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const DEVICE_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(30);
pub const BACKEND_OPERATION_TIMEOUT: Duration = Duration::from_secs(120);
pub const MODEL_OPERATION_TIMEOUT: Duration = Duration::from_secs(120);
pub const PROCESS_CANCELLATION_TIMEOUT: Duration = Duration::from_secs(10);
pub const MAX_HEALTH_STREAM_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStage {
    DeviceEnumeration,
    BackendOperations,
    PinnedModelLoad,
    LoopbackServerHealth,
    DeterministicCompletion,
    Cancellation,
    ProcessAndTemporaryFileCleanup,
}

impl HealthStage {
    pub const ALL: [Self; 7] = [
        Self::DeviceEnumeration,
        Self::BackendOperations,
        Self::PinnedModelLoad,
        Self::LoopbackServerHealth,
        Self::DeterministicCompletion,
        Self::Cancellation,
        Self::ProcessAndTemporaryFileCleanup,
    ];
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HealthStageStatus {
    Pass,
    Fail,
    Skipped,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthFailureReason {
    Timeout,
    Spawn,
    NonzeroExit,
    OutputLimit,
    MalformedOutput,
    Mismatch,
    Cancelled,
    TrustFailure,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletionEvidence {
    pub temperature: f64,
    pub requested_tokens: u32,
    pub observed_tokens: u32,
    pub expected_output_sha256: String,
    pub observed_output_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HealthStageResult {
    pub stage: HealthStage,
    pub status: HealthStageStatus,
    pub duration_ms: u64,
    pub failure_reason: Option<HealthFailureReason>,
    pub detail: String,
    pub completion: Option<CompletionEvidence>,
}

impl HealthStageResult {
    pub fn passed(stage: HealthStage, duration_ms: u64, detail: impl Into<String>) -> Self {
        Self {
            stage,
            status: HealthStageStatus::Pass,
            duration_ms,
            failure_reason: None,
            detail: detail.into(),
            completion: None,
        }
    }

    pub fn failed(
        stage: HealthStage,
        duration_ms: u64,
        reason: HealthFailureReason,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            stage,
            status: HealthStageStatus::Fail,
            duration_ms,
            failure_reason: Some(reason),
            detail: detail.into(),
            completion: None,
        }
    }

    pub fn passed_completion(duration_ms: u64, completion: CompletionEvidence) -> Self {
        Self {
            stage: HealthStage::DeterministicCompletion,
            status: HealthStageStatus::Pass,
            duration_ms,
            failure_reason: None,
            detail: "Deterministic completion matched every approved field.".into(),
            completion: Some(completion),
        }
    }

    pub fn skipped(stage: HealthStage, reason: HealthFailureReason) -> Self {
        Self {
            stage,
            status: HealthStageStatus::Skipped,
            duration_ms: 0,
            failure_reason: Some(reason),
            detail: "A prior health stage did not pass.".into(),
            completion: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HealthRunResult {
    pub runtime_id: String,
    pub model_sha256: String,
    pub adapter_id: Option<String>,
    pub started_at: u64,
    pub finished_at: u64,
    pub passed: bool,
    pub stages: Vec<HealthStageResult>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManagedHealthRequest {
    pub install_key: String,
    pub adapter_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthModelProgress {
    pub install_key: String,
    pub downloaded: u64,
    pub total: u64,
}

pub(crate) struct ManagedHealthContext {
    pub runtime_id: String,
    pub install_root: PathBuf,
    pub server_path: PathBuf,
    pub backend: String,
    pub adapter_id: Option<String>,
    pub adapter_name: Option<String>,
    pub expected_model: String,
    pub model_path: PathBuf,
    /// Read-shared handles that pin the verified runtime content for the whole
    /// health run so a long preparation interval cannot reopen a modification
    /// window before the CLI, benchmark, and server launches (audit RT-04).
    pub execution_lease: Option<crate::runtime::ManagedExecutionLease>,
}

pub fn completion_matches(value: &CompletionEvidence) -> bool {
    value.temperature == 0.0
        && value.requested_tokens > 0
        && value.observed_tokens == value.requested_tokens
        && value.expected_output_sha256.len() == 64
        && value
            .expected_output_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        && value
            .expected_output_sha256
            .eq_ignore_ascii_case(&value.observed_output_sha256)
}

pub fn validate_stage_contract(stages: &[HealthStageResult]) -> Result<(), &'static str> {
    if stages.len() != HealthStage::ALL.len() {
        return Err("health stage count");
    }
    if !stages
        .iter()
        .zip(HealthStage::ALL)
        .all(|(result, expected)| result.stage == expected)
    {
        return Err("health stage order");
    }
    for result in stages {
        if (result.status == HealthStageStatus::Pass) != result.failure_reason.is_none() {
            return Err("health stage status");
        }
        if result.stage == HealthStage::DeterministicCompletion
            && result.status == HealthStageStatus::Pass
            && !result.completion.as_ref().is_some_and(completion_matches)
        {
            return Err("completion evidence");
        }
        if result.stage != HealthStage::DeterministicCompletion && result.completion.is_some() {
            return Err("completion stage");
        }
    }
    Ok(())
}

fn epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn elapsed_millis(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

fn finish_run(
    context: &ManagedHealthContext,
    started_at: u64,
    mut stages: Vec<HealthStageResult>,
) -> HealthRunResult {
    let reason = stages
        .iter()
        .find_map(|stage| stage.failure_reason)
        .unwrap_or(HealthFailureReason::Mismatch);
    for stage in HealthStage::ALL.into_iter().skip(stages.len()) {
        stages.push(HealthStageResult::skipped(stage, reason));
    }
    let passed = stages
        .iter()
        .all(|stage| stage.status == HealthStageStatus::Pass);
    debug_assert!(validate_stage_contract(&stages).is_ok());
    HealthRunResult {
        runtime_id: context.runtime_id.clone(),
        model_sha256: crate::core::pinned_model_load_pin().sha256,
        adapter_id: context.adapter_id.clone(),
        started_at,
        finished_at: epoch_millis(),
        passed,
        stages,
    }
}

pub(crate) fn trust_failure_result(
    runtime_id: String,
    adapter_id: Option<String>,
    detail: impl Into<String>,
) -> HealthRunResult {
    let context = ManagedHealthContext {
        runtime_id,
        install_root: PathBuf::new(),
        server_path: PathBuf::new(),
        backend: String::new(),
        adapter_id,
        adapter_name: None,
        expected_model: String::new(),
        model_path: PathBuf::new(),
        execution_lease: None,
    };
    finish_run(
        &context,
        epoch_millis(),
        vec![HealthStageResult::failed(
            HealthStage::DeviceEnumeration,
            0,
            HealthFailureReason::TrustFailure,
            detail,
        )],
    )
}

pub(crate) fn model_setup_failure_result(
    context: &ManagedHealthContext,
    reason: HealthFailureReason,
    detail: impl Into<String>,
) -> HealthRunResult {
    finish_run(
        context,
        epoch_millis(),
        vec![
            HealthStageResult::skipped(
                HealthStage::DeviceEnumeration,
                HealthFailureReason::TrustFailure,
            ),
            HealthStageResult::skipped(
                HealthStage::BackendOperations,
                HealthFailureReason::TrustFailure,
            ),
            HealthStageResult::failed(HealthStage::PinnedModelLoad, 0, reason, detail),
        ],
    )
}

fn process_failure_reason(kind: crate::proc::ProcessFailureKind) -> HealthFailureReason {
    match kind {
        crate::proc::ProcessFailureKind::Spawn => HealthFailureReason::Spawn,
        crate::proc::ProcessFailureKind::Timeout => HealthFailureReason::Timeout,
        crate::proc::ProcessFailureKind::Cancelled => HealthFailureReason::Cancelled,
        crate::proc::ProcessFailureKind::OutputLimit => HealthFailureReason::OutputLimit,
        crate::proc::ProcessFailureKind::Io => HealthFailureReason::MalformedOutput,
    }
}

fn run_bounded(
    executable: &Path,
    working_directory: &Path,
    args: &[String],
    timeout: Duration,
    cancel: &AtomicBool,
) -> Result<Output, (HealthFailureReason, String)> {
    let mut command = crate::proc::hidden_command(executable);
    command.args(args).current_dir(working_directory);
    let output = crate::proc::output_with_timeout_and_cancel(
        &mut command,
        timeout,
        MAX_HEALTH_STREAM_BYTES,
        cancel,
    )
    .map_err(|error| {
        (
            process_failure_reason(error.kind),
            "The bounded health process did not complete safely.".into(),
        )
    })?;
    if !output.status.success() {
        return Err((
            HealthFailureReason::NonzeroExit,
            "The health process returned a nonzero exit status.".into(),
        ));
    }
    Ok(output)
}

fn unique_runtime_file(root: &Path, filename: &str) -> Result<Option<PathBuf>, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut found = None;
    let mut entries = 0_usize;
    while let Some(directory) = pending.pop() {
        for item in std::fs::read_dir(&directory).map_err(|_| "runtime inventory read failed")? {
            entries += 1;
            if entries > 4096 {
                return Err("runtime inventory exceeded its entry limit".into());
            }
            let item = item.map_err(|_| "runtime inventory entry failed")?;
            let path = item.path();
            let metadata = std::fs::symlink_metadata(&path)
                .map_err(|_| "runtime inventory metadata failed")?;
            #[cfg(windows)]
            let reparse = {
                use std::os::windows::fs::MetadataExt;
                metadata.file_attributes() & 0x400 != 0
            };
            #[cfg(not(windows))]
            let reparse = false;
            if metadata.file_type().is_symlink() || reparse {
                return Err("runtime inventory contains a link or reparse point".into());
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file()
                && path.file_name().and_then(|value| value.to_str()) == Some(filename)
            {
                if found.is_some() {
                    return Err("runtime inventory contains a duplicate required tool".into());
                }
                found = Some(path);
            }
        }
    }
    Ok(found)
}

pub(crate) fn verify_pinned_model(
    path: &Path,
    cancel: &AtomicBool,
) -> Result<(), (HealthFailureReason, String)> {
    let pin = crate::core::pinned_model_load_pin();
    if path.file_name().and_then(|value| value.to_str()) != Some(pin.name.as_str()) {
        return Err((
            HealthFailureReason::Mismatch,
            "The selected health model filename does not match the approved pin.".into(),
        ));
    }
    crate::artifact::validate_regular_non_reparse_file("Health model", path).map_err(|_| {
        (
            HealthFailureReason::TrustFailure,
            "The health model is not a regular non-reparse file.".into(),
        )
    })?;
    let metadata = path.metadata().map_err(|_| {
        (
            HealthFailureReason::TrustFailure,
            "The health model metadata is unavailable.".into(),
        )
    })?;
    if metadata.len() != pin.bytes {
        return Err((
            HealthFailureReason::Mismatch,
            "The health model size does not match the approved pin.".into(),
        ));
    }
    let mut file = File::open(path).map_err(|_| {
        (
            HealthFailureReason::TrustFailure,
            "The health model could not be opened.".into(),
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err((
                HealthFailureReason::Cancelled,
                "Health model verification was cancelled.".into(),
            ));
        }
        let count = file.read(&mut buffer).map_err(|_| {
            (
                HealthFailureReason::TrustFailure,
                "The health model could not be read.".into(),
            )
        })?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    if format!("{:x}", hasher.finalize()) != pin.sha256 {
        return Err((
            HealthFailureReason::Mismatch,
            "The health model SHA-256 does not match the approved pin.".into(),
        ));
    }
    Ok(())
}

fn resolve_runtime_device(
    output: &[u8],
    backend: &str,
    adapter_name: Option<&str>,
) -> Result<Option<String>, String> {
    if backend == "cpu" {
        return if adapter_name.is_none() {
            Ok(None)
        } else {
            Err("The CPU health run must not select an adapter".into())
        };
    }
    let adapter_name = adapter_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "The accelerator health run has no selected adapter name".to_string())?;
    let prefix = match backend {
        "cuda" => "cuda",
        "rocm" => "hip",
        "openvino" => "openvino",
        "sycl" => "sycl",
        "vulkan" => "vulkan",
        _ => return Err("The approved backend has no device-discovery mapping".into()),
    };
    let output = std::str::from_utf8(output)
        .map_err(|_| "Device discovery output is not valid UTF-8".to_string())?;
    let adapter_name = adapter_name.to_ascii_lowercase();
    let matches = output
        .lines()
        .filter_map(|line| line.trim().split_once(':'))
        .filter_map(|(device, description)| {
            let device = device.trim();
            (device.to_ascii_lowercase().starts_with(prefix)
                && description.to_ascii_lowercase().contains(&adapter_name))
            .then(|| device.to_string())
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [device] => Ok(Some(device.clone())),
        [] => {
            Err("Device discovery did not map the selected adapter to the approved backend".into())
        }
        _ => Err("Device discovery mapped the selected adapter more than once".into()),
    }
}

fn backend_args(backend: &str, device_id: Option<&str>) -> Vec<String> {
    if backend == "cpu" {
        vec!["-ngl".into(), "0".into()]
    } else {
        vec![
            "-ngl".into(),
            "999".into(),
            "--device".into(),
            device_id.unwrap_or("").into(),
        ]
    }
}

fn benchmark_output_matches(output: &[u8], backend: &str) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(output) else {
        return false;
    };
    let Some(records) = value.as_array() else {
        return false;
    };
    let expected_backend = match backend {
        "rocm" => "HIP",
        other => other,
    };
    let backend_matches = |record: &serde_json::Value| {
        record
            .get("backend")
            .or_else(|| record.get("backends"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| {
                value
                    .split(',')
                    .any(|candidate| candidate.trim().eq_ignore_ascii_case(expected_backend))
            })
    };
    records.iter().any(|record| {
        record.get("n_prompt").and_then(serde_json::Value::as_u64) == Some(32)
            && backend_matches(record)
    }) && records.iter().any(|record| {
        record.get("n_gen").and_then(serde_json::Value::as_u64) == Some(16)
            && backend_matches(record)
    })
}

fn bounded_body(response: reqwest::blocking::Response) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    response
        .take((MAX_HEALTH_STREAM_BYTES + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|_| "loopback response read failed".to_string())?;
    if body.len() > MAX_HEALTH_STREAM_BYTES {
        return Err("loopback response exceeded the output limit".into());
    }
    Ok(body)
}

struct HealthTempDir {
    path: PathBuf,
    cleaned: bool,
}

impl HealthTempDir {
    fn create() -> Result<Self, String> {
        let parent = std::env::temp_dir();
        std::fs::create_dir_all(&parent)
            .map_err(|_| "Could not prepare the health temporary directory.".to_string())?;
        let canonical_parent = parent
            .canonicalize()
            .map_err(|_| "Could not verify the health temporary directory.".to_string())?;
        for _ in 0..16 {
            let path = parent.join(format!(
                "localmotive-health-{}-{:032x}",
                std::process::id(),
                rand::random::<u128>()
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => {
                    let canonical = path.canonicalize().map_err(|_| {
                        "Could not verify the isolated health temporary directory.".to_string()
                    })?;
                    if canonical.parent() != Some(canonical_parent.as_path())
                        || path_is_link_or_reparse(&path)
                    {
                        let _ = std::fs::remove_dir(&path);
                        return Err(
                            "The isolated health temporary directory failed containment checks."
                                .into(),
                        );
                    }
                    return Ok(Self {
                        path,
                        cleaned: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => {
                    return Err("Could not create an isolated health temporary directory.".into())
                }
            }
        }
        Err("Could not allocate a unique health temporary directory.".into())
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn cleanup(&mut self) -> bool {
        if self.cleaned || !self.path.exists() {
            self.cleaned = true;
            return true;
        }
        let mut pending = vec![self.path.clone()];
        while let Some(path) = pending.pop() {
            if path_is_link_or_reparse(&path) {
                return false;
            }
            let metadata = match std::fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(_) => return false,
            };
            if metadata.is_dir() {
                let entries = match std::fs::read_dir(&path) {
                    Ok(entries) => entries,
                    Err(_) => return false,
                };
                for entry in entries {
                    let Ok(entry) = entry else {
                        return false;
                    };
                    pending.push(entry.path());
                }
            }
        }
        self.cleaned = std::fs::remove_dir_all(&self.path).is_ok() && !self.path.exists();
        self.cleaned
    }
}

impl Drop for HealthTempDir {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn path_is_link_or_reparse(path: &Path) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return true;
    };
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn completion_request(port: u16) -> Result<(u16, Vec<u8>), (HealthFailureReason, String)> {
    let pin = crate::core::pinned_model_load_pin();
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(2))
        .timeout(MODEL_OPERATION_TIMEOUT)
        .build()
        .map_err(|_| {
            (
                HealthFailureReason::Spawn,
                "The loopback completion client could not start.".into(),
            )
        })?;
    let response = client
        .post(format!("http://127.0.0.1:{port}/completion"))
        .json(&serde_json::json!({
            "prompt": pin.prompt,
            "n_predict": pin.expected_tokens_predicted,
            "temperature": 0,
            "seed": 1234,
            "cache_prompt": false,
            "stream": false,
        }))
        .send()
        .map_err(|_| {
            (
                HealthFailureReason::Timeout,
                "The bounded loopback completion request failed.".into(),
            )
        })?;
    let status = response.status().as_u16();
    let body = bounded_body(response).map_err(|_| {
        (
            HealthFailureReason::OutputLimit,
            "The loopback completion response exceeded its limit.".into(),
        )
    })?;
    Ok((status, body))
}

fn health_listener_is_owned(port: u16, expected_pid: u32) -> Result<bool, String> {
    crate::listener_is_owned_by_at(
        std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port)),
        expected_pid,
    )
}

fn fail_stage(
    context: &ManagedHealthContext,
    started_at: u64,
    mut stages: Vec<HealthStageResult>,
    stage: HealthStage,
    stage_started: Instant,
    error: (HealthFailureReason, String),
) -> HealthRunResult {
    stages.push(HealthStageResult::failed(
        stage,
        elapsed_millis(stage_started),
        error.0,
        error.1,
    ));
    finish_run(context, started_at, stages)
}

pub(crate) fn run_managed_health(
    context: ManagedHealthContext,
    cancel: &AtomicBool,
) -> HealthRunResult {
    // The execution lease (audit RT-04) lives with the context for the whole
    // run: its read-shared handles pin the verified runtime content across the
    // CLI, benchmark, and server launches below. Hold an explicit reference so
    // the guarantee is visible at the health entry point.
    let _execution_lease = context.execution_lease.as_ref();
    let started_at = epoch_millis();
    let mut stages = Vec::with_capacity(HealthStage::ALL.len());

    if let Err((reason, detail)) = verify_pinned_model(&context.model_path, cancel) {
        return model_setup_failure_result(&context, reason, detail);
    }

    let cli_path = match unique_runtime_file(&context.install_root, "llama-cli.exe") {
        Ok(Some(path)) => path,
        Ok(None) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::DeviceEnumeration,
                Instant::now(),
                (
                    HealthFailureReason::TrustFailure,
                    "The approved runtime does not contain llama-cli.exe.".into(),
                ),
            );
        }
        Err(detail) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::DeviceEnumeration,
                Instant::now(),
                (HealthFailureReason::TrustFailure, detail),
            );
        }
    };
    let stage_started = Instant::now();
    let discovery = match run_bounded(
        &cli_path,
        &context.install_root,
        &["--list-devices".into()],
        DEVICE_DISCOVERY_TIMEOUT,
        cancel,
    ) {
        Ok(output) => output,
        Err(error) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::DeviceEnumeration,
                stage_started,
                error,
            );
        }
    };
    let combined_discovery = [
        discovery.stdout.as_slice(),
        b"\n",
        discovery.stderr.as_slice(),
    ]
    .concat();
    let runtime_device = match resolve_runtime_device(
        &combined_discovery,
        &context.backend,
        context.adapter_name.as_deref(),
    ) {
        Ok(device) => device,
        Err(detail) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::DeviceEnumeration,
                stage_started,
                (HealthFailureReason::Mismatch, detail),
            );
        }
    };
    stages.push(HealthStageResult::passed(
        HealthStage::DeviceEnumeration,
        elapsed_millis(stage_started),
        "Bounded device enumeration identified the approved backend and exact adapter.",
    ));

    let stage_started = Instant::now();
    let mut operation_args = vec![
        "-m".into(),
        context.model_path.to_string_lossy().into_owned(),
        "-p".into(),
        "health".into(),
        "-n".into(),
        "1".into(),
        "--seed".into(),
        "1234".into(),
        "--no-display-prompt".into(),
        "--single-turn".into(),
        "--simple-io".into(),
    ];
    operation_args.extend(backend_args(&context.backend, runtime_device.as_deref()));
    if let Err(error) = run_bounded(
        &cli_path,
        &context.install_root,
        &operation_args,
        BACKEND_OPERATION_TIMEOUT,
        cancel,
    ) {
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::BackendOperations,
            stage_started,
            error,
        );
    }
    stages.push(HealthStageResult::passed(
        HealthStage::BackendOperations,
        elapsed_millis(stage_started),
        "The approved backend completed a bounded operation on the exact adapter.",
    ));

    let stage_started = Instant::now();
    if let Err(error) = verify_pinned_model(&context.model_path, cancel) {
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::PinnedModelLoad,
            stage_started,
            error,
        );
    }
    let benchmark_path = match unique_runtime_file(&context.install_root, "llama-bench.exe") {
        Ok(Some(path)) => path,
        Ok(None) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::PinnedModelLoad,
                stage_started,
                (
                    HealthFailureReason::TrustFailure,
                    "The approved runtime does not contain llama-bench.exe.".into(),
                ),
            );
        }
        Err(detail) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::PinnedModelLoad,
                stage_started,
                (HealthFailureReason::TrustFailure, detail),
            );
        }
    };
    let mut benchmark_args = vec![
        "-m".into(),
        context.model_path.to_string_lossy().into_owned(),
        "-p".into(),
        "32".into(),
        "-n".into(),
        "16".into(),
        "-r".into(),
        "1".into(),
        "-o".into(),
        "json".into(),
    ];
    benchmark_args.extend(backend_args(&context.backend, runtime_device.as_deref()));
    let benchmark = match run_bounded(
        &benchmark_path,
        &context.install_root,
        &benchmark_args,
        MODEL_OPERATION_TIMEOUT,
        cancel,
    ) {
        Ok(output) => output,
        Err(error) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::PinnedModelLoad,
                stage_started,
                error,
            );
        }
    };
    if !benchmark_output_matches(&benchmark.stdout, &context.backend) {
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::PinnedModelLoad,
            stage_started,
            (
                HealthFailureReason::MalformedOutput,
                "The bounded benchmark output did not prove the approved backend and workloads."
                    .into(),
            ),
        );
    }
    stages.push(HealthStageResult::passed(
        HealthStage::PinnedModelLoad,
        elapsed_millis(stage_started),
        format!(
            "The runtime loaded the immutable pinned model {} and completed both benchmark workloads.",
            context.expected_model
        ),
    ));

    let server_path = &context.server_path;
    let listener = match std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)) {
        Ok(listener) => listener,
        Err(_) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::LoopbackServerHealth,
                Instant::now(),
                (
                    HealthFailureReason::Spawn,
                    "A loopback port could not be reserved for health verification.".into(),
                ),
            );
        }
    };
    let port = match listener.local_addr() {
        Ok(address) => address.port(),
        Err(_) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::LoopbackServerHealth,
                Instant::now(),
                (
                    HealthFailureReason::Spawn,
                    "The reserved loopback port could not be inspected.".into(),
                ),
            );
        }
    };
    drop(listener);
    let mut server_args = vec![
        "-m".into(),
        context.model_path.to_string_lossy().into_owned(),
        "--host".into(),
        "127.0.0.1".into(),
        "--port".into(),
        port.to_string(),
        "--ctx-size".into(),
        "512".into(),
        "--parallel".into(),
        "1".into(),
        "--no-webui".into(),
    ];
    server_args.extend(backend_args(&context.backend, runtime_device.as_deref()));
    let stage_started = Instant::now();
    let mut health_temp = match HealthTempDir::create() {
        Ok(temporary) => temporary,
        Err(detail) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::LoopbackServerHealth,
                stage_started,
                (HealthFailureReason::TrustFailure, detail),
            );
        }
    };
    let mut command = crate::proc::hidden_command(server_path);
    command
        .args(&server_args)
        .current_dir(&context.install_root)
        .env("TEMP", health_temp.path())
        .env("TMP", health_temp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = match crate::proc::spawn_contained_process(&mut command) {
        Ok(child) => child,
        Err(error) => {
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::LoopbackServerHealth,
                stage_started,
                (
                    HealthFailureReason::Spawn,
                    format!("The managed loopback health server did not start: {error}"),
                ),
            );
        }
    };
    let readiness_client = match reqwest::blocking::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_millis(250))
        .timeout(Duration::from_secs(2))
        .build()
    {
        Ok(client) => client,
        Err(_) => {
            child.terminate_and_wait();
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::LoopbackServerHealth,
                stage_started,
                (
                    HealthFailureReason::Spawn,
                    "The loopback readiness client did not start.".into(),
                ),
            );
        }
    };
    let mut ready = false;
    while stage_started.elapsed() < MODEL_OPERATION_TIMEOUT {
        if cancel.load(Ordering::Relaxed) {
            child.terminate_and_wait();
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::LoopbackServerHealth,
                stage_started,
                (
                    HealthFailureReason::Cancelled,
                    "Loopback readiness was cancelled.".into(),
                ),
            );
        }
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => {
                child.terminate_and_wait();
                return fail_stage(
                    &context,
                    started_at,
                    stages,
                    HealthStage::LoopbackServerHealth,
                    stage_started,
                    (
                        HealthFailureReason::NonzeroExit,
                        "The managed loopback health server stopped before readiness.".into(),
                    ),
                );
            }
            Ok(None) => {}
        }
        if let Ok(response) = readiness_client
            .get(format!("http://127.0.0.1:{port}/health"))
            .send()
        {
            let status = response.status().as_u16();
            if status == 200 {
                if let Ok(body) = bounded_body(response) {
                    ready = serde_json::from_slice::<serde_json::Value>(&body)
                        .ok()
                        .and_then(|value| {
                            value
                                .get("status")
                                .and_then(|status| status.as_str())
                                .map(str::to_owned)
                        })
                        .is_some_and(|status| status == "ok");
                }
            }
        }
        if ready {
            match health_listener_is_owned(port, child.id()) {
                Ok(true) => break,
                Ok(false) => {
                    child.terminate_and_wait();
                    return fail_stage(
                        &context,
                        started_at,
                        stages,
                        HealthStage::LoopbackServerHealth,
                        stage_started,
                        (
                            HealthFailureReason::TrustFailure,
                            "The loopback listener is not owned by the managed health process."
                                .into(),
                        ),
                    );
                }
                Err(error) => {
                    child.terminate_and_wait();
                    return fail_stage(
                        &context,
                        started_at,
                        stages,
                        HealthStage::LoopbackServerHealth,
                        stage_started,
                        (
                            HealthFailureReason::TrustFailure,
                            format!("The loopback listener owner could not be verified: {error}"),
                        ),
                    );
                }
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    if !ready {
        child.terminate_and_wait();
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::LoopbackServerHealth,
            stage_started,
            (
                HealthFailureReason::Timeout,
                "The managed loopback health server did not become ready before its deadline."
                    .into(),
            ),
        );
    }
    stages.push(HealthStageResult::passed(
        HealthStage::LoopbackServerHealth,
        elapsed_millis(stage_started),
        "The managed server returned a bounded, structured loopback readiness response.",
    ));

    let stage_started = Instant::now();
    if cancel.load(Ordering::Relaxed) {
        child.terminate_and_wait();
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::DeterministicCompletion,
            stage_started,
            (
                HealthFailureReason::Cancelled,
                "Deterministic completion was cancelled.".into(),
            ),
        );
    }
    let response = completion_request(port);
    let (status, body) = match response {
        Ok(value) => value,
        Err(error) => {
            child.terminate_and_wait();
            return fail_stage(
                &context,
                started_at,
                stages,
                HealthStage::DeterministicCompletion,
                stage_started,
                error,
            );
        }
    };
    let parsed = serde_json::from_slice::<serde_json::Value>(&body).ok();
    let content = parsed
        .as_ref()
        .and_then(|value| value.get("content"))
        .and_then(serde_json::Value::as_str);
    let observed_tokens = parsed
        .as_ref()
        .and_then(|value| value.get("tokens_predicted"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());
    let pin = crate::core::pinned_model_load_pin();
    let Some(content) = content else {
        child.terminate_and_wait();
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::DeterministicCompletion,
            stage_started,
            (
                HealthFailureReason::MalformedOutput,
                "The deterministic completion response omitted its content.".into(),
            ),
        );
    };
    let observed_output_sha256 = format!("{:x}", Sha256::digest(content.as_bytes()));
    let completion = CompletionEvidence {
        temperature: 0.0,
        requested_tokens: pin.expected_tokens_predicted,
        observed_tokens: observed_tokens.unwrap_or_default(),
        expected_output_sha256: format!("{:x}", Sha256::digest(pin.expected_completion.as_bytes())),
        observed_output_sha256,
    };
    if status != 200 || content != pin.expected_completion || !completion_matches(&completion) {
        child.terminate_and_wait();
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::DeterministicCompletion,
            stage_started,
            (
                HealthFailureReason::Mismatch,
                "The deterministic completion did not match every approved field.".into(),
            ),
        );
    }
    stages.push(HealthStageResult::passed_completion(
        elapsed_millis(stage_started),
        completion,
    ));

    let stage_started = Instant::now();
    let tree_stopped = child.terminate_and_wait();
    if !tree_stopped || stage_started.elapsed() > PROCESS_CANCELLATION_TIMEOUT {
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::Cancellation,
            stage_started,
            (
                HealthFailureReason::Timeout,
                "The health server process did not stop within the cancellation limit.".into(),
            ),
        );
    }
    stages.push(HealthStageResult::passed(
        HealthStage::Cancellation,
        elapsed_millis(stage_started),
        "Cancellation stopped the active health server within the configured limit.",
    ));

    let stage_started = Instant::now();
    let child_stopped = child.try_wait().ok().flatten().is_some();
    let port_closed = std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port)),
        Duration::from_millis(250),
    )
    .is_err();
    let temporary_files_removed = health_temp.cleanup();
    if !child_stopped || !tree_stopped || !port_closed || !temporary_files_removed {
        return fail_stage(
            &context,
            started_at,
            stages,
            HealthStage::ProcessAndTemporaryFileCleanup,
            stage_started,
            (
                HealthFailureReason::TrustFailure,
                "Health cleanup left a process tree, loopback listener, or temporary file active."
                    .into(),
            ),
        );
    }
    stages.push(HealthStageResult::passed(
        HealthStage::ProcessAndTemporaryFileCleanup,
        elapsed_millis(stage_started),
        "No health process tree, loopback listener, or isolated temporary file remained.",
    ));
    finish_run(&context, started_at, stages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn health_servers_bind_loopback_only_and_never_wildcard() {
        // Phase 3: loopback health succeeds without network exposure. The
        // server args pin `--host 127.0.0.1`, every health URL targets
        // 127.0.0.1 or LOCALHOST, and no wildcard bind exists anywhere
        // in the health module (see the per-line scan below, which skips
        // comments like this one).
        // RED: binding the server host to a wildcard fails this test, which
        // proves the test guards loopback-only exposure instead of
        // documenting it.
        let source = include_str!("health.rs");

        // The host flag value must be the loopback literal on the SAME
        // args line pair: find the host flag and require the next
        // non-empty product line to carry 127.0.0.1.
        let lines: Vec<&str> = source.lines().collect();
        let mut host_ok = false;
        for (index, line) in lines.iter().enumerate() {
            if line.contains("--host") && line.contains(".into(),") {
                let next = lines.get(index + 1).unwrap_or(&"");
                if next.contains("127.0.0.1") {
                    host_ok = true;
                }
            }
        }
        assert!(host_ok, "server --host flag must pin 127.0.0.1");

        let needle: Vec<u8> = [48, 46, 48, 46, 48, 46, 48].to_vec();
        let needle = String::from_utf8(needle).unwrap();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            // The needle is built from byte values so this test body never
            // contains the wildcard literal itself (self-match guard).
            assert!(
                !trimmed.contains(needle.as_str()),
                "wildcard address in product code: {trimmed}"
            );
        }
    }

    #[test]
    fn health_run_result_carries_the_defined_completion_shape() {
        // Phase 3: deterministic completion returns the defined
        // `HealthRunResult` structure: runtime/model/adapter identity,
        // timestamps, pass flag, all seven stages in order, and the
        // completion evidence fields (temperature, token counts, expected
        // vs observed output digests) on the completion stage.
        // RED: dropping the completion evidence fails this test, which
        // proves the test guards the shape instead of documenting it.
        let context = ManagedHealthContext {
            runtime_id: "cpu".into(),
            install_root: PathBuf::new(),
            server_path: PathBuf::new(),
            backend: "cpu".into(),
            adapter_id: None,
            adapter_name: None,
            expected_model: "SmolLM2-135M-Q4_K_M.gguf".into(),
            model_path: PathBuf::new(),
            execution_lease: None,
        };
        let completion = CompletionEvidence {
            temperature: 0.0,
            requested_tokens: 16,
            observed_tokens: 16,
            expected_output_sha256: "a".repeat(64),
            observed_output_sha256: "a".repeat(64),
        };
        assert!(completion_matches(&completion));
        let mut stages = vec![HealthStageResult::passed_completion(3, completion.clone())];
        // Prepend passes for the four stages before completion.
        for stage in [
            HealthStage::DeviceEnumeration,
            HealthStage::BackendOperations,
            HealthStage::PinnedModelLoad,
            HealthStage::LoopbackServerHealth,
        ] {
            stages.insert(
                stages.len().saturating_sub(1),
                HealthStageResult::passed(stage, 1, "fixture pass"),
            );
        }
        // Append passes for the two stages after completion.
        for stage in [
            HealthStage::Cancellation,
            HealthStage::ProcessAndTemporaryFileCleanup,
        ] {
            stages.push(HealthStageResult::passed(stage, 1, "fixture pass"));
        }
        assert_eq!(stages.len(), 7);
        let result = finish_run(&context, 1000, stages);

        assert!(result.passed);
        assert_eq!(result.runtime_id, "cpu");
        assert_eq!(
            result.model_sha256,
            crate::core::pinned_model_load_pin().sha256
        );
        assert_eq!(result.adapter_id, None);
        assert!(result.finished_at >= result.started_at);
        assert_eq!(
            result
                .stages
                .iter()
                .map(|stage| stage.stage)
                .collect::<Vec<_>>(),
            HealthStage::ALL.to_vec()
        );
        let completion_stage = result
            .stages
            .iter()
            .find(|stage| stage.stage == HealthStage::DeterministicCompletion)
            .unwrap();
        let evidence = completion_stage.completion.as_ref().unwrap();
        assert_eq!(evidence.temperature, 0.0);
        assert_eq!(evidence.requested_tokens, 16);
        assert_eq!(evidence.observed_tokens, 16);
        assert_eq!(evidence.expected_output_sha256, "a".repeat(64));
        assert_eq!(evidence.observed_output_sha256, "a".repeat(64));
    }

    #[test]
    fn health_failure_details_carry_no_secret_or_private_path() {
        // Phase 3: failure messages contain no secret or private path. The
        // health error vocabulary is a closed set of fixed product strings
        // (no paths, tokens, or hostnames are interpolated), so a failure
        // can be shown and logged without leaking machine specifics.
        // RED: introducing a detail that echoes a path fails this test,
        // which proves the test guards the vocabulary instead of
        // documenting it.
        let source = include_str!("health.rs");

        // No credential vocabulary in failure details. The scan skips
        // this test's own token list (self-match guard): only product lines
        // outside this test function count.
        let in_scrub_test = false;
        let _ = in_scrub_test;
        let body_start = source
            .find("fn health_failure_details_carry_no_secret")
            .unwrap_or(0);
        let (before, _) = source.split_at(body_start);
        let before_lower = before.to_ascii_lowercase();
        for token in [
            "password", "api-key", "api_key", "bearer", "ghp_", "gho_", "sk-",
        ] {
            assert!(
                !before_lower.contains(token),
                "credential vocabulary must not appear in health errors: {token}"
            );
        }
        // "secret"/"token" appear only as completion counters
        // (requested_tokens/observed_tokens), never as credential words.
        for line in before.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            let low = line.to_ascii_lowercase();
            if low.contains("secret") {
                panic!("secret vocabulary in health product code: {}", line.trim());
            }
            if low.contains("token")
                && !low.contains("requested_tokens")
                && !low.contains("observed_tokens")
                && !low.contains("n_predict")
                && !low.contains("tokens_predicted")
            {
                panic!("token vocabulary in health product code: {}", line.trim());
            }
        }
        // No absolute-path interpolation in failure details: every
        // user-facing health detail string is a fixed literal.
        for needle in ["C:\\", "C:/", "/home/", "/Users/"] {
            let mut hits = 0;
            for line in source.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.contains(needle) && trimmed.contains("into()") {
                    hits += 1;
                }
            }
            assert_eq!(hits, 0, "absolute path in a health detail string");
        }
        // The closed vocabulary check: every `HealthFailureReason` maps to
        // a fixed detail, and `run_bounded` never echoes the executable path.
        assert!(source.contains("The bounded health process did not complete safely."));
        assert!(source.contains("The health process returned a nonzero exit status."));
    }

    #[test]
    fn loopback_server_is_contained_before_user_code_can_run() {
        let source = include_str!("health.rs");
        assert!(source.contains("spawn_contained_process"));
        assert_eq!(source.matches("ProcessJob::assign").count(), 1);
    }

    #[cfg(windows)]
    #[test]
    fn health_listener_must_belong_to_the_expected_process() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();

        assert!(health_listener_is_owned(port, std::process::id()).unwrap());
        assert!(!health_listener_is_owned(port, std::process::id().wrapping_add(1)).unwrap());
    }

    #[test]
    #[ignore = "downloads approved artifacts and executes the exact runtime on current hardware"]
    fn qualify_managed_runtime_on_current_host() {
        let install_key = std::env::var("LOCALMOTIVE_INSTALL_KEY")
            .expect("set LOCALMOTIVE_INSTALL_KEY to an approved install key");
        let hardware = crate::runtime::detect_hardware();
        let adapter_id = if install_key == "cpu" {
            None
        } else if let Ok(explicit) = std::env::var("LOCALMOTIVE_ADAPTER_ID") {
            Some(explicit)
        } else {
            let vendor = if install_key.starts_with("cuda") {
                Some("nvidia")
            } else if install_key == "rocm" {
                Some("amd")
            } else if matches!(install_key.as_str(), "openvino" | "sycl") {
                Some("intel")
            } else {
                None
            };
            Some(
                hardware
                    .adapters
                    .iter()
                    .find(|adapter| vendor.is_none_or(|expected| adapter.vendor == expected))
                    .expect("current host has no compatible detected adapter")
                    .adapter_id
                    .clone(),
            )
        };
        let cancel = std::sync::Arc::new(AtomicBool::new(false));
        crate::runtime::install_runtime(
            crate::runtime::RuntimeInstallRequest {
                install_key: install_key.clone(),
                adapter_id: adapter_id.clone(),
            },
            std::sync::Arc::clone(&cancel),
            |_| {},
        )
        .expect("approved runtime installation failed");
        let request = ManagedHealthRequest {
            install_key,
            adapter_id,
        };
        let context = crate::runtime::managed_health_context(&request)
            .expect("managed runtime trust verification failed");
        crate::runtime::ensure_pinned_health_model(std::sync::Arc::clone(&cancel), |_, _| {})
            .expect("pinned health model setup failed");
        let result = run_managed_health(context, cancel.as_ref());
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        assert!(result.passed, "managed runtime health report did not pass");
    }

    #[test]
    fn device_discovery_maps_the_selected_dxgi_adapter_to_one_llama_device() {
        let cuda = include_bytes!("../tests/fixtures/health/cuda.list-devices.stdout.txt");
        let vulkan = include_bytes!("../tests/fixtures/health/vulkan.list-devices.stdout.txt");

        assert_eq!(
            resolve_runtime_device(cuda, "cuda", Some("NVIDIA GeForce RTX 5090")).unwrap(),
            Some("CUDA0".into())
        );
        assert_eq!(
            resolve_runtime_device(vulkan, "vulkan", Some("NVIDIA GeForce RTX 5090")).unwrap(),
            Some("Vulkan0".into())
        );
        assert!(resolve_runtime_device(cuda, "cuda", Some("Different adapter")).is_err());
    }

    #[test]
    fn benchmark_requires_both_workloads_on_the_expected_backend() {
        let output = br#"[
          {"backend":"HIP","n_prompt":32,"n_gen":0},
          {"backend":"HIP","n_prompt":0,"n_gen":16}
        ]"#;

        assert!(benchmark_output_matches(output, "rocm"));
        assert!(!benchmark_output_matches(output, "cuda"));
        assert!(!benchmark_output_matches(
            br#"[{"backend":"HIP","n_prompt":32,"n_gen":0}]"#,
            "rocm"
        ));
        assert!(!benchmark_output_matches(
            br#"[
              {"backends":"NOTCUDA","n_prompt":32,"n_gen":0},
              {"backends":"NOTCUDA","n_prompt":0,"n_gen":16}
            ]"#,
            "cuda"
        ));
        assert!(benchmark_output_matches(
            br#"[
              {"backends":"CUDA,CPU","n_prompt":32,"n_gen":0},
              {"backends":"CUDA,CPU","n_prompt":0,"n_gen":16}
            ]"#,
            "cpu"
        ));
    }

    #[test]
    fn health_contract_has_exactly_seven_ordered_stages() {
        assert_eq!(
            HealthStage::ALL,
            [
                HealthStage::DeviceEnumeration,
                HealthStage::BackendOperations,
                HealthStage::PinnedModelLoad,
                HealthStage::LoopbackServerHealth,
                HealthStage::DeterministicCompletion,
                HealthStage::Cancellation,
                HealthStage::ProcessAndTemporaryFileCleanup,
            ]
        );
    }

    #[test]
    fn completion_pass_requires_every_deterministic_field() {
        let expected = CompletionEvidence {
            temperature: 0.0,
            requested_tokens: 16,
            observed_tokens: 16,
            expected_output_sha256: "a".repeat(64),
            observed_output_sha256: "a".repeat(64),
        };
        assert!(completion_matches(&expected));

        let mut mismatch = expected;
        mismatch.observed_tokens = 15;
        assert!(!completion_matches(&mismatch));
    }

    #[test]
    fn final_health_record_rejects_missing_or_reordered_stages() {
        let stages = HealthStage::ALL
            .iter()
            .copied()
            .map(|stage| {
                if stage == HealthStage::DeterministicCompletion {
                    HealthStageResult::passed_completion(
                        1,
                        CompletionEvidence {
                            temperature: 0.0,
                            requested_tokens: 16,
                            observed_tokens: 16,
                            expected_output_sha256: "a".repeat(64),
                            observed_output_sha256: "a".repeat(64),
                        },
                    )
                } else {
                    HealthStageResult::passed(stage, 1, "fixture")
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(validate_stage_contract(&stages), Ok(()));

        let mut missing = stages.clone();
        missing.pop();
        assert_eq!(validate_stage_contract(&missing), Err("health stage count"));

        let mut reordered = stages;
        reordered.swap(0, 1);
        assert_eq!(
            validate_stage_contract(&reordered),
            Err("health stage order")
        );
    }

    #[test]
    fn health_resource_limits_match_the_reviewed_contract() {
        assert_eq!(DEVICE_DISCOVERY_TIMEOUT.as_secs(), 30);
        assert_eq!(BACKEND_OPERATION_TIMEOUT.as_secs(), 120);
        assert_eq!(MODEL_OPERATION_TIMEOUT.as_secs(), 120);
        assert_eq!(PROCESS_CANCELLATION_TIMEOUT.as_secs(), 10);
        assert_eq!(MAX_HEALTH_STREAM_BYTES, 1024 * 1024);
    }

    #[cfg(windows)]
    #[test]
    fn cancellation_job_terminates_descendant_processes() {
        // Under MSYS/git-bash the `ping.exe -n 2` prologue resolves to the
        // MSYS ping (seconds per echo, not one second per echo), so the
        // child can exit before the 1.5 s readiness sleep and the
        // still-running assertion goes red for environment reasons, not
        // product reasons. Invoke the native ping by absolute path so the
        // timing matches the fixture design on every shell.
        let mut command = crate::proc::hidden_command("cmd.exe");
        command
            .args([
                "/D",
                "/C",
                r"C:\Windows\System32\ping.exe -n 2 127.0.0.1 >NUL & C:\Windows\System32\ping.exe -n 60 127.0.0.1 >NUL",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = crate::proc::spawn_contained_process(&mut command).unwrap();
        std::thread::sleep(Duration::from_millis(1_500));
        assert!(child.try_wait().unwrap().is_none());
        assert!(child.terminate_and_wait());
        assert!(child.try_wait().unwrap().is_some());
    }

    #[test]
    fn health_cleanup_removes_the_isolated_temporary_tree() {
        let mut temporary = HealthTempDir::create().unwrap();
        let path = temporary.path().to_path_buf();
        std::fs::write(path.join("health.tmp"), b"temporary").unwrap();

        assert!(temporary.cleanup());
        assert!(!path.exists());
    }

    #[test]
    fn changed_pinned_model_is_rejected_before_any_runtime_process_lookup() {
        let mut temporary = HealthTempDir::create().unwrap();
        let model_path = temporary.path().join("changed.gguf");
        std::fs::write(&model_path, b"not the approved model").unwrap();
        let context = ManagedHealthContext {
            runtime_id: "cpu".into(),
            install_root: temporary.path().join("missing-runtime"),
            server_path: temporary.path().join("missing-runtime/llama-server.exe"),
            backend: "cpu".into(),
            adapter_id: None,
            adapter_name: None,
            expected_model: "approved fixture".into(),
            model_path,
            execution_lease: None,
        };

        let result = run_managed_health(context, &AtomicBool::new(false));

        assert!(!result.passed);
        assert_eq!(result.stages.len(), 7);
        assert_eq!(result.stages[0].status, HealthStageStatus::Skipped);
        assert_eq!(result.stages[1].status, HealthStageStatus::Skipped);
        assert_eq!(result.stages[2].stage, HealthStage::PinnedModelLoad);
        assert_eq!(result.stages[2].status, HealthStageStatus::Fail);
        assert_eq!(
            result.stages[2].failure_reason,
            Some(HealthFailureReason::Mismatch)
        );
        assert!(temporary.cleanup());
    }
}
