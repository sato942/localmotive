pub mod artifact;
pub mod calibration;
mod catalog;
mod catalog_db;
mod cloud;
mod core;
mod download;
pub mod evidence;
mod gguf;
mod health;
mod local_client;
pub mod measurement;
pub mod preflight;
mod proc;
pub mod recommend;
mod runtime;
pub mod sharing;
mod tune;

use core::{BenchmarkSummary, LaunchProfile, LogicalModel, RuntimeCapabilities};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
#[cfg(test)]
use std::net::TcpListener;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};

struct ManagedServer {
    child: proc::ContainedProcess,
    profile: LaunchProfile,
    command: String,
    validation: LaunchValidation,
    log_path: String,
    started_at: u64,
    /// Read-shared handles that pin the verified managed runtime content for
    /// the whole server lifetime (audit RT-04). Dropped when the server is
    /// stopped or replaced.
    #[allow(dead_code)]
    runtime_lease: Option<runtime::ManagedExecutionLease>,
}

trait HealthProcess {
    fn id(&self) -> u32;
    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>>;
}

impl HealthProcess for Child {
    fn id(&self) -> u32 {
        Child::id(self)
    }

    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        Child::try_wait(self)
    }
}

impl HealthProcess for proc::ContainedProcess {
    fn id(&self) -> u32 {
        proc::ContainedProcess::id(self)
    }

    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        proc::ContainedProcess::try_wait(self)
    }
}

struct ExclusiveOperation<'a> {
    active: &'a AtomicBool,
}

impl<'a> ExclusiveOperation<'a> {
    fn acquire(active: &'a AtomicBool, operation: &str) -> Result<Self, String> {
        active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| format!("A {operation} request is already active"))?;
        Ok(Self { active })
    }
}

impl Drop for ExclusiveOperation<'_> {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

#[derive(Default)]
struct AppState {
    server: Mutex<Option<ManagedServer>>,
    tuning: Mutex<Option<Arc<AtomicBool>>>,
    benchmark: Mutex<Option<Arc<AtomicBool>>>,
    /// Last validated catalog shown to the frontend. `None` means use bundled.
    catalog: Mutex<Option<catalog::Catalog>>,
    /// Cancel flags for in-flight downloads, keyed by normalized target path.
    downloads: Mutex<std::collections::HashMap<String, Arc<AtomicBool>>>,
    /// One managed-runtime installation may run at a time.
    runtime_install: Mutex<Option<Arc<AtomicBool>>>,
    /// One approved runtime catalog request may run at a time.
    runtime_catalog: AtomicBool,
    /// One cancellable GGUF metadata read may run at a time.
    gguf_read: Mutex<Option<Arc<AtomicBool>>>,
    /// The in-flight managed-server start, when one is pending: the child
    /// stays reachable by Stop through its cancellation signal (audit
    /// IPC-01), and only the worker that owns this operation ID may commit
    /// the server slot.
    starting: Mutex<Option<StartingServer>>,
    /// One managed-runtime seven-stage health run may execute at a time.
    runtime_health: Mutex<Option<Arc<AtomicBool>>>,
    /// One managed-inference operation at a time, with process generations
    /// (audit MT-05).
    operations: Mutex<OperationCoordinator>,
}

/// Which managed-inference operation currently owns the machine. Every
/// subsystem that launches or drives a llama-server — ordinary startup, warm
/// benchmarks, cold attempts, tuning sessions, and quality suites — reserves
/// here first, so two owners can never run concurrently and every finalizer
/// can detect that its server identity was replaced (audit MT-05).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperationOwner {
    Server,
    Benchmark,
    ColdBenchmark,
    Tuning,
    Quality,
}

impl OperationOwner {
    fn label(self) -> &'static str {
        match self {
            OperationOwner::Server => "a running server",
            OperationOwner::Benchmark => "a warm-cache benchmark",
            OperationOwner::ColdBenchmark => "a cold-cache benchmark",
            OperationOwner::Tuning => "a tuning session",
            OperationOwner::Quality => "a quality suite",
        }
    }
}

#[derive(Debug, Default)]
struct OperationCoordinator {
    /// The current (owner, generation) pair. Only one at a time.
    active: Option<(OperationOwner, u64)>,
    /// Increments on every successful reservation, so stale work can notice
    /// that the machine state it was measured against is gone.
    generation: u64,
}

/// RAII reservation: released on drop, and only when this exact generation
/// is still the active one, so one operation's completion cannot release a
/// replacement's ownership (audit MT-05 I2).
#[derive(Debug)]
struct OperationReservation<'a> {
    coordinator: &'a Mutex<OperationCoordinator>,
    owner: OperationOwner,
    generation: u64,
}

impl OperationReservation<'_> {
    /// True while this reservation still owns the machine generation. A
    /// false value means results must not be finalized under the original
    /// server identity (audit MT-05 I3).
    fn is_current(&self) -> bool {
        self.coordinator
            .lock()
            .map(|coordinator| coordinator.active == Some((self.owner, self.generation)))
            .unwrap_or(false)
    }
}

impl Drop for OperationReservation<'_> {
    fn drop(&mut self) {
        if let Ok(mut coordinator) = self.coordinator.lock() {
            if coordinator.active == Some((self.owner, self.generation)) {
                coordinator.active = None;
            }
        }
    }
}

fn reserve_operation(
    coordinator: &Mutex<OperationCoordinator>,
    owner: OperationOwner,
) -> Result<OperationReservation<'_>, String> {
    let mut state = coordinator
        .lock()
        .map_err(|_| "Operation state is unavailable".to_string())?;
    if let Some((current, _)) = state.active {
        return Err(format!(
            "{} is already active; finish or cancel it first.",
            current.label()
        ));
    }
    state.generation = state.generation.wrapping_add(1);
    let generation = state.generation;
    state.active = Some((owner, generation));
    Ok(OperationReservation {
        coordinator,
        owner,
        generation,
    })
}

fn active_operation_owner(coordinator: &Mutex<OperationCoordinator>) -> Option<OperationOwner> {
    coordinator
        .lock()
        .ok()
        .and_then(|state| state.active.map(|(owner, _)| owner))
}

/// A Stop request may only act on the ordinary server owner: anything else
/// owns the machine right now, and its own cancel control must run first
/// (audit MT-05 I3).
fn stop_owner_conflict(active: Option<OperationOwner>) -> Option<String> {
    match active {
        Some(OperationOwner::Server) | None => None,
        Some(owner) => Some(format!(
            "{} currently owns the managed server; cancel it before stopping.",
            owner.label()
        )),
    }
}

/// A managed-server start in flight (audit IPC-01). The cancellation signal
/// is the handle Stop uses while readiness is pending; the worker that owns
/// `operation_id` is the only writer allowed to publish the running server.
struct StartingServer {
    operation_id: u64,
    cancel: Arc<AtomicBool>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerStatus {
    running: bool,
    /// Explicit lifecycle phase: idle, starting, running, or stopping
    /// (audit IPC-01 I1).
    phase: String,
    pid: Option<u32>,
    profile_name: Option<String>,
    alias: Option<String>,
    port: Option<u16>,
    command: Option<String>,
    log_path: Option<String>,
    started_at: Option<u64>,
    exit_code: Option<i32>,
    /// Speculative strategy of the LAUNCHED configuration (snapshot), so the
    /// running identity never follows the editable draft (audit FE-16).
    spec_type: Option<String>,
    companion_linked: Option<bool>,
    result_class: evidence::FitClass,
    validation: Option<LaunchValidation>,
    failure: Option<LaunchFailureEvidence>,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self {
            running: false,
            phase: "idle".to_string(),
            pid: None,
            profile_name: None,
            alias: None,
            port: None,
            command: None,
            log_path: None,
            started_at: None,
            exit_code: None,
            spec_type: None,
            companion_linked: None,
            result_class: evidence::FitClass::Unknown,
            validation: None,
            failure: None,
        }
    }
}

fn status_from(slot: &mut Option<ManagedServer>) -> ServerStatus {
    if let Some(server) = slot.as_mut() {
        match server.child.try_wait() {
            Ok(Some(code)) => {
                let exit_code = code.code();
                let status = ServerStatus {
                    running: false,
                    phase: "idle".to_string(),
                    pid: None,
                    profile_name: Some(server.profile.name.clone()),
                    alias: Some(server.profile.alias.clone()),
                    port: Some(server.profile.port),
                    command: Some(server.command.clone()),
                    log_path: Some(server.log_path.clone()),
                    started_at: Some(server.started_at),
                    exit_code,
                    spec_type: Some(server.profile.spec_type.clone()),
                    companion_linked: Some(server.profile.draft_model.is_some()),
                    result_class: evidence::FitClass::Failed,
                    validation: None,
                    failure: Some(launch_failure_evidence(
                        "runtime_exit",
                        format!(
                            "llama-server exited after health validation with {}",
                            exit_code
                                .map(|value| format!("code {value}"))
                                .unwrap_or_else(|| "a signal".into())
                        ),
                        &server.log_path,
                        exit_code,
                        false,
                    )),
                };
                *slot = None;
                status
            }
            Ok(None) => ServerStatus {
                running: true,
                phase: "running".to_string(),
                pid: Some(server.child.id()),
                profile_name: Some(server.profile.name.clone()),
                alias: Some(server.profile.alias.clone()),
                port: Some(server.profile.port),
                command: Some(server.command.clone()),
                log_path: Some(server.log_path.clone()),
                started_at: Some(server.started_at),
                exit_code: None,
                spec_type: Some(server.profile.spec_type.clone()),
                companion_linked: Some(server.profile.draft_model.is_some()),
                result_class: evidence::FitClass::LaunchValidated,
                validation: Some(server.validation.clone()),
                failure: None,
            },
            Err(error) => ServerStatus {
                running: false,
                phase: "idle".to_string(),
                pid: None,
                profile_name: Some(server.profile.name.clone()),
                alias: Some(server.profile.alias.clone()),
                port: Some(server.profile.port),
                command: Some(server.command.clone()),
                log_path: Some(server.log_path.clone()),
                started_at: Some(server.started_at),
                exit_code: None,
                spec_type: Some(server.profile.spec_type.clone()),
                companion_linked: Some(server.profile.draft_model.is_some()),
                result_class: evidence::FitClass::Failed,
                validation: None,
                failure: Some(launch_failure_evidence(
                    "runtime_status",
                    format!("Could not read llama-server exit status: {error}"),
                    &server.log_path,
                    None,
                    false,
                )),
            },
        }
    } else {
        ServerStatus::default()
    }
}

fn require_launchable_artifact(
    label: &str,
    path: &Path,
) -> Result<artifact::ArtifactInspection, String> {
    require_regular_non_reparse_file(label, path)?;
    let inspection = artifact::inspect_artifact(path, &[], false)?;
    if inspection.complete && inspection.header_consistent && inspection.problems.is_empty() {
        return Ok(inspection);
    }
    let details = inspection
        .problems
        .iter()
        .map(|problem| problem.message.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    Err(format!(
        "{label} cannot be launched because its artifact is incomplete or inconsistent{}{}",
        if details.is_empty() { "" } else { ": " },
        details
    ))
}

fn validate_profile_artifacts(
    profile: &LaunchProfile,
) -> Result<Vec<artifact::ArtifactInspection>, String> {
    let mut artifacts = vec![require_launchable_artifact(
        "Model",
        Path::new(&profile.model),
    )?];
    if let Some(path) = profile.draft_model.as_ref().filter(|path| !path.is_empty()) {
        let draft = require_launchable_artifact("Draft model", Path::new(path))?;
        for key in ["tokenizer.ggml.model", "tokenizer.ggml.pre"] {
            if let (Some(model_value), Some(draft_value)) = (
                artifact_metadata_string(&artifacts[0], key),
                artifact_metadata_string(&draft, key),
            ) {
                if model_value != draft_value {
                    return Err(format!(
                        "Draft model tokenizer identity {key} conflicts with the target model"
                    ));
                }
            }
        }
        let model_vocab = artifacts[0]
            .summary
            .as_ref()
            .and_then(|summary| summary.vocab_size);
        let draft_vocab = draft
            .summary
            .as_ref()
            .and_then(|summary| summary.vocab_size);
        if model_vocab
            .zip(draft_vocab)
            .is_some_and(|(model, draft)| model != draft)
        {
            return Err("Draft model vocabulary size conflicts with the target model".into());
        }
        artifacts.push(draft);
    }
    if let Some(path) = profile.mmproj.as_ref().filter(|path| !path.is_empty()) {
        let projector = require_launchable_artifact("Vision projector", Path::new(path))?;
        let architecture = projector
            .summary
            .as_ref()
            .map(|summary| summary.architecture.as_str())
            .unwrap_or_default();
        if architecture != "clip" {
            return Err(format!(
                "Vision projector GGUF must use the clip architecture; found {architecture}"
            ));
        }
        let model_embedding = artifacts[0]
            .summary
            .as_ref()
            .and_then(|summary| summary.embedding_length);
        let projector_embedding = artifact_metadata_u64(&projector, "clip.vision.projection_dim");
        if let Some((model_embedding, projector_embedding)) =
            model_embedding.zip(projector_embedding)
        {
            if model_embedding != projector_embedding {
                return Err(format!(
                    "Vision projector embedding size {projector_embedding} does not match model embedding size {model_embedding}"
                ));
            }
        }
        artifacts.push(projector);
    }
    let main_architecture = artifacts
        .first()
        .and_then(|artifact| artifact.summary.as_ref())
        .map(|summary| summary.architecture.as_str())
        .unwrap_or_default()
        .to_string();
    for (label, path) in lora_references(profile)? {
        let lora = require_launchable_artifact(&label, &path)?;
        let lora_type = artifact_metadata_string(&lora, "general.type").unwrap_or_default();
        let adapter_type = artifact_metadata_string(&lora, "adapter.type").unwrap_or_default();
        let lora_architecture = lora
            .summary
            .as_ref()
            .map(|summary| summary.architecture.as_str())
            .unwrap_or_default();
        if !lora_type.eq_ignore_ascii_case("adapter") || !adapter_type.eq_ignore_ascii_case("lora")
        {
            return Err(format!(
                "{label} must declare general.type=adapter and adapter.type=lora"
            ));
        }
        if main_architecture.is_empty()
            || lora_architecture.is_empty()
            || lora_architecture != main_architecture
        {
            return Err(format!(
                "{label} architecture {lora_architecture} does not match model architecture {main_architecture}"
            ));
        }
        artifacts.push(lora);
    }
    Ok(artifacts)
}

fn artifact_metadata_string<'a>(
    artifact: &'a artifact::ArtifactInspection,
    key: &str,
) -> Option<&'a str> {
    artifact
        .summary
        .as_ref()?
        .metadata_facts
        .iter()
        .find(|fact| fact.key == key)
        .and_then(|fact| match &fact.value {
            gguf::MetadataValue::Scalar {
                value: gguf::MetadataScalar::String(value),
            } => Some(value.as_str()),
            _ => None,
        })
}

fn artifact_metadata_u64(artifact: &artifact::ArtifactInspection, key: &str) -> Option<u64> {
    artifact
        .summary
        .as_ref()?
        .metadata_facts
        .iter()
        .find(|fact| fact.key == key)
        .and_then(|fact| match fact.value {
            gguf::MetadataValue::Scalar {
                value: gguf::MetadataScalar::Unsigned(value),
            } => Some(value),
            gguf::MetadataValue::Scalar {
                value: gguf::MetadataScalar::Signed(value),
            } => u64::try_from(value).ok(),
            _ => None,
        })
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchValidation {
    runtime: RuntimeCapabilities,
    artifacts: Vec<artifact::ArtifactInspection>,
    arguments: core::LaunchArgumentValidation,
    effective_context: evidence::Evidence<u32>,
    unverified_requirements: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchFailureEvidence {
    schema: u16,
    phase: String,
    exit_code: Option<i32>,
    timed_out: bool,
    cancelled: bool,
    message: String,
    log_tail: String,
}

const LAUNCH_FAILURE_SERIALIZATION_FALLBACK: &str = r#"{"schema":1,"phase":"serialization","exitCode":null,"timedOut":false,"cancelled":false,"message":"Could not serialize launch failure","logTail":""}"#;

fn unverified_companion_requirements(profile: &LaunchProfile) -> Vec<String> {
    let mut requirements = Vec::new();
    if profile
        .draft_model
        .as_ref()
        .is_some_and(|path| !path.is_empty())
    {
        requirements
            .push("Draft model token-sequence compatibility requires runtime validation.".into());
    }
    if profile.mmproj.as_ref().is_some_and(|path| !path.is_empty()) {
        requirements.push("Projector-to-model compatibility requires runtime validation.".into());
    }
    if !profile.lora.trim().is_empty() || !profile.lora_scaled.trim().is_empty() {
        requirements.push("LoRA base-model compatibility requires runtime validation.".into());
    }
    requirements
}

fn prepare_launch(profile: &LaunchProfile) -> Result<LaunchValidation, String> {
    validate_profile_paths(profile)?;
    runtime::verify_managed_runtime_for_launch(Path::new(&profile.runtime))?;
    let artifacts = validate_profile_artifacts(profile)?;
    let runtime = core::inspect_runtime(Path::new(&profile.runtime))?;
    let arguments = core::validate_launch_arguments(profile, &runtime)?;
    let observed_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "System time is outside the supported range")?;
    Ok(LaunchValidation {
        runtime,
        artifacts,
        arguments,
        effective_context: unobserved_effective_context(observed_at_ms),
        unverified_requirements: unverified_companion_requirements(profile),
    })
}

fn execution_path_for(
    profile: &LaunchProfile,
    selected_adapter_ids: &[String],
) -> evidence::ExecutionPath {
    if profile.gpu_layers.trim() == "0" {
        return evidence::ExecutionPath::Cpu;
    }
    if !profile.tensor_split.trim().is_empty() {
        return if selected_adapter_ids.len() > 1 {
            evidence::ExecutionPath::MultiGpu
        } else {
            evidence::ExecutionPath::Unknown
        };
    }
    if profile.gpu_layers.trim() == "auto" {
        return evidence::ExecutionPath::Unknown;
    }
    match selected_adapter_ids.len() {
        0 => evidence::ExecutionPath::Unknown,
        1 if profile.gpu_layers.eq_ignore_ascii_case("all") => evidence::ExecutionPath::FullGpu,
        1 => evidence::ExecutionPath::LayerOffload,
        _ => evidence::ExecutionPath::MultiGpu,
    }
}

/// Validate every path the profile references, then spawn llama-server with
/// output redirected to a per-port log file. Shared by the Start button and
/// the tuner so both launch exactly the same way.
fn require_regular_non_reparse_file(label: &str, path: &Path) -> Result<(), String> {
    artifact::validate_regular_non_reparse_file(label, path)
}

fn lora_references(profile: &LaunchProfile) -> Result<Vec<(String, PathBuf)>, String> {
    let mut references = Vec::new();
    if !profile.lora.trim().is_empty() {
        for (index, value) in profile.lora.split(',').enumerate() {
            let path = value.trim();
            if path.is_empty() {
                return Err(format!("LoRA entry {} has an empty path", index + 1));
            }
            references.push((format!("LoRA entry {}", index + 1), PathBuf::from(path)));
        }
    }
    if !profile.lora_scaled.trim().is_empty() {
        for (index, value) in profile.lora_scaled.split(',').enumerate() {
            let entry = index + 1;
            let (path, scale) = value
                .trim()
                .rsplit_once(':')
                .ok_or_else(|| format!("Scaled LoRA entry {entry} must use path:scale syntax"))?;
            scale
                .trim()
                .parse::<f32>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or_else(|| format!("Scaled LoRA entry {entry} has an invalid scale"))?;
            if path.trim().is_empty() {
                return Err(format!("Scaled LoRA entry {entry} has an empty path"));
            }
            references.push((
                format!("Scaled LoRA entry {entry}"),
                PathBuf::from(path.trim()),
            ));
        }
    }
    Ok(references)
}

fn validate_lora_paths(profile: &LaunchProfile) -> Result<(), String> {
    for (label, path) in lora_references(profile)? {
        require_regular_non_reparse_file(&label, &path)?;
    }
    Ok(())
}

fn validate_profile_paths(profile: &LaunchProfile) -> Result<(), String> {
    require_regular_non_reparse_file("Runtime", Path::new(&profile.runtime))?;
    require_regular_non_reparse_file("Model", Path::new(&profile.model))?;
    if let Some(path) = profile.draft_model.as_ref().filter(|path| !path.is_empty()) {
        require_regular_non_reparse_file("Draft model", Path::new(path))?;
    }
    if let Some(path) = profile.mmproj.as_ref().filter(|path| !path.is_empty()) {
        require_regular_non_reparse_file("Vision projector", Path::new(path))?;
    }
    for (label, path) in [
        ("API key file", profile.api_key_file.as_str()),
        ("SSL private key", profile.ssl_key_file.as_str()),
        ("SSL certificate", profile.ssl_cert_file.as_str()),
        ("Chat template", profile.chat_template_file.as_str()),
    ] {
        if !path.is_empty() {
            require_regular_non_reparse_file(label, Path::new(path))?;
        }
    }
    validate_lora_paths(profile)?;
    Ok(())
}

fn spawn_server(
    profile: &LaunchProfile,
    log_name: &str,
) -> Result<
    (
        proc::ContainedProcess,
        LaunchValidation,
        String,
        Option<runtime::ManagedExecutionLease>,
    ),
    String,
> {
    let validation = prepare_launch(profile)
        .map_err(|message| launch_failure("validation", message, "", None, false))?;
    // Acquire the execution-identity lease after verification and before the
    // process starts: for a managed runtime the approved bytes stay pinned by
    // read-shared handles for as long as the returned lease lives, so the
    // content cannot change between the trust check and image/DLL loading
    // (audit RT-04). An external runtime deliberately keeps no lease.
    let execution_lease =
        runtime::authorize_managed_execution_lease(Path::new(&profile.runtime))
            .map_err(|message| launch_failure("validation", message, "", None, false))?;

    let log_dir = std::env::temp_dir().join("localmotive");
    fs::create_dir_all(&log_dir).map_err(|error| {
        launch_failure(
            "log_setup",
            format!("Could not create the launch log directory: {error}"),
            "",
            None,
            false,
        )
    })?;
    let log_path = log_dir.join(format!("{log_name}.log"));
    let log_path_text = log_path.to_string_lossy().to_string();
    core::probe_port_available(&profile.host, profile.port)
        .map_err(|error| launch_failure("port_probe", error, &log_path_text, None, false))?;
    let stdout = File::create(&log_path).map_err(|error| {
        launch_failure(
            "log_setup",
            format!("Could not create the launch log: {error}"),
            &log_path_text,
            None,
            false,
        )
    })?;
    let stderr = stdout.try_clone().map_err(|error| {
        launch_failure(
            "log_setup",
            format!("Could not prepare the launch log: {error}"),
            &log_path_text,
            None,
            false,
        )
    })?;
    let mut command = proc::hidden_command(&profile.runtime);
    command
        .args(&validation.arguments.effective_args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    let child = proc::spawn_contained_process(&mut command).map_err(|error| {
        launch_failure(
            "spawn",
            format!("Could not start llama-server: {error}"),
            &log_path_text,
            None,
            false,
        )
    })?;
    Ok((child, validation, log_path_text, execution_lease))
}

fn connect_host(host: &str) -> &str {
    let host = host.trim();
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    match host {
        "0.0.0.0" => "127.0.0.1",
        "::" => "::1",
        value => value,
    }
}

fn health_socket_addresses(host: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
    let addresses = (connect_host(host), port)
        .to_socket_addrs()
        .map_err(|error| format!("Could not resolve health endpoint {host}:{port}: {error}"))?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(format!(
            "Health endpoint {host}:{port} did not resolve to an address"
        ));
    }
    Ok(addresses)
}

#[cfg(windows)]
fn listener_owners_for_family(
    port: u16,
    address_family: u32,
) -> Result<Vec<(std::net::IpAddr, u32)>, String> {
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID,
        TCP_TABLE_OWNER_PID_LISTENER,
    };
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};

    let mut byte_count = 0_u32;
    // SAFETY: The first call supplies a null table only to obtain the required size.
    let first_status = unsafe {
        GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut byte_count,
            0,
            address_family,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        )
    };
    if first_status != ERROR_INSUFFICIENT_BUFFER && first_status != NO_ERROR {
        return Err(format!(
            "Could not size the Windows TCP listener table: {}",
            std::io::Error::from_raw_os_error(first_status as i32)
        ));
    }
    if byte_count == 0 {
        return Ok(Vec::new());
    }

    let requested_bytes = usize::try_from(byte_count)
        .map_err(|_| "Windows TCP listener table size does not fit this process")?;
    let word_count = requested_bytes
        .checked_add(size_of::<u32>() - 1)
        .ok_or("Windows TCP listener table size overflowed")?
        / size_of::<u32>();
    let mut table = vec![0_u32; word_count];
    let mut actual_bytes = byte_count;
    // SAFETY: `table` is aligned and writable for at least `actual_bytes` bytes.
    let status = unsafe {
        GetExtendedTcpTable(
            table.as_mut_ptr().cast(),
            &mut actual_bytes,
            0,
            address_family,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        )
    };
    if status != NO_ERROR {
        return Err(format!(
            "Could not read the Windows TCP listener table: {}",
            std::io::Error::from_raw_os_error(status as i32)
        ));
    }
    let actual_bytes = usize::try_from(actual_bytes)
        .map_err(|_| "Windows TCP listener table result does not fit this process")?;
    if actual_bytes < size_of::<u32>() || actual_bytes > table.len() * size_of::<u32>() {
        return Err("Windows returned an invalid TCP listener table size".into());
    }

    let row_count = table[0] as usize;
    let rows = unsafe { table.as_ptr().cast::<u8>().add(size_of::<u32>()) };
    let mut owners = Vec::new();
    if address_family == u32::from(AF_INET) {
        let required = row_count
            .checked_mul(size_of::<MIB_TCPROW_OWNER_PID>())
            .and_then(|bytes| bytes.checked_add(size_of::<u32>()))
            .ok_or("Windows IPv4 TCP listener table length overflowed")?;
        if required > actual_bytes {
            return Err("Windows returned a truncated IPv4 TCP listener table".into());
        }
        for index in 0..row_count {
            // SAFETY: The validated table contains `row_count` complete IPv4 rows.
            let row = unsafe {
                std::ptr::read_unaligned(
                    rows.add(index * size_of::<MIB_TCPROW_OWNER_PID>())
                        .cast::<MIB_TCPROW_OWNER_PID>(),
                )
            };
            if u16::from_be(row.dwLocalPort as u16) == port {
                owners.push((
                    std::net::Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes()).into(),
                    row.dwOwningPid,
                ));
            }
        }
    } else if address_family == u32::from(AF_INET6) {
        let required = row_count
            .checked_mul(size_of::<MIB_TCP6ROW_OWNER_PID>())
            .and_then(|bytes| bytes.checked_add(size_of::<u32>()))
            .ok_or("Windows IPv6 TCP listener table length overflowed")?;
        if required > actual_bytes {
            return Err("Windows returned a truncated IPv6 TCP listener table".into());
        }
        for index in 0..row_count {
            // SAFETY: The validated table contains `row_count` complete IPv6 rows.
            let row = unsafe {
                std::ptr::read_unaligned(
                    rows.add(index * size_of::<MIB_TCP6ROW_OWNER_PID>())
                        .cast::<MIB_TCP6ROW_OWNER_PID>(),
                )
            };
            if u16::from_be(row.dwLocalPort as u16) == port {
                owners.push((
                    std::net::Ipv6Addr::from(row.ucLocalAddr).into(),
                    row.dwOwningPid,
                ));
            }
        }
    } else {
        return Err(format!(
            "Unsupported TCP listener address family {address_family}"
        ));
    }
    Ok(owners)
}

#[cfg(windows)]
fn listener_is_owned_by_at(address: SocketAddr, expected_pid: u32) -> Result<bool, String> {
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};

    let address_family = if address.is_ipv4() {
        u32::from(AF_INET)
    } else {
        u32::from(AF_INET6)
    };
    let owners = listener_owners_for_family(address.port(), address_family)?;
    Ok(owners.iter().any(|(listener_address, pid)| {
        *pid == expected_pid
            && (*listener_address == address.ip() || listener_address.is_unspecified())
    }))
}

#[cfg(not(windows))]
fn listener_is_owned_by_at(_address: SocketAddr, _expected_pid: u32) -> Result<bool, String> {
    Err("TCP listener ownership verification is available only on Windows".into())
}

#[cfg(test)]
fn is_healthy_response(response: &[u8]) -> bool {
    response.starts_with(b"HTTP/1.1 200 ") || response.starts_with(b"HTTP/1.0 200 ")
}

fn parse_server_effective_context(status: u16, body: &[u8]) -> Result<u32, String> {
    if status != 200 {
        return Err(format!("llama-server /props returned HTTP {status}"));
    }
    let payload = std::str::from_utf8(body)
        .map_err(|_| "llama-server /props returned invalid UTF-8".to_string())?;
    let value: serde_json::Value = serde_json::from_str(payload)
        .map_err(|error| format!("Could not parse llama-server /props: {error}"))?;
    value
        .get("default_generation_settings")
        .and_then(|settings| settings.get("n_ctx"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| "llama-server /props did not contain a positive effective context".into())
}

#[cfg(test)]
fn read_health_response(reader: &mut impl std::io::Read) -> std::io::Result<Vec<u8>> {
    use std::io::Read;

    const MAX_HEALTH_RESPONSE_BYTES: usize = 16 * 1024;
    let mut response = Vec::with_capacity(MAX_HEALTH_RESPONSE_BYTES);
    reader
        .take((MAX_HEALTH_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut response)?;
    if response.len() > MAX_HEALTH_RESPONSE_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Health response exceeded 16384 bytes",
        ));
    }
    Ok(response)
}

#[cfg(test)]
fn read_props_response(reader: &mut impl std::io::Read) -> std::io::Result<Vec<u8>> {
    const MAX_PROPS_RESPONSE_BYTES: usize = 1_048_576;
    let mut response = Vec::with_capacity(16 * 1024);
    let mut buffer = [0_u8; 4_096];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        if response.len().saturating_add(read) > MAX_PROPS_RESPONSE_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "llama-server /props response exceeded the 1 MiB limit",
            ));
        }
        response.extend_from_slice(&buffer[..read]);
    }
    Ok(response)
}

fn query_server_effective_context(
    client: &crate::local_client::LocalHttpClient,
    timeout: Duration,
) -> Result<u32, String> {
    // The effective-context probe goes through the centralized local client
    // (audit MT-06), so a TLS/API-key-configured server answers it.
    let (status, body) = client
        .get_bytes("/props", timeout)
        .map_err(|error| format!("Could not observe llama-server effective context: {error}"))?;
    parse_server_effective_context(status, &body)
        .map_err(|error| format!("Could not observe llama-server effective context: {error}"))
}

fn observe_server_effective_context(
    client: &crate::local_client::LocalHttpClient,
    timeout: Duration,
    observed_at_ms: u64,
) -> evidence::Evidence<u32> {
    match query_server_effective_context(client, timeout) {
        Ok(context) => evidence::Evidence {
            value: Some(context),
            level: evidence::EvidenceLevel::Observed,
            source: evidence::EvidenceSource {
                kind: evidence::EvidenceSourceKind::Runtime,
                detail: "llama-server GET /props default_generation_settings.n_ctx".into(),
            },
            observed_at_ms,
            notes: vec![
                "The value is the effective per-slot context reported after health validation."
                    .into(),
            ],
        },
        Err(error) => evidence::Evidence {
            value: None,
            level: evidence::EvidenceLevel::Unknown,
            source: evidence::EvidenceSource {
                kind: evidence::EvidenceSourceKind::Runtime,
                detail: "llama-server GET /props default_generation_settings.n_ctx".into(),
            },
            observed_at_ms,
            notes: vec![error],
        },
    }
}

fn unobserved_effective_context(observed_at_ms: u64) -> evidence::Evidence<u32> {
    evidence::Evidence {
        value: None,
        level: evidence::EvidenceLevel::Unknown,
        source: evidence::EvidenceSource {
            kind: evidence::EvidenceSourceKind::Runtime,
            detail: "llama-server GET /props default_generation_settings.n_ctx".into(),
        },
        observed_at_ms,
        notes: vec![
            "Effective context is unknown until llama-server passes health validation.".into(),
        ],
    }
}

fn update_launch_effective_context(
    validation: &mut LaunchValidation,
    host: &str,
    port: u16,
    timeout: Duration,
    observed_at_ms: u64,
) {
    validation.effective_context = observe_server_effective_context(
        &crate::local_client::LocalHttpClient::plain(host, port).unwrap_or_else(|_| {
            crate::local_client::LocalHttpClient::plain("127.0.0.1", port).unwrap()
        }),
        timeout,
        observed_at_ms,
    );
}

fn bounded_log_tail(log_path: &str) -> String {
    use std::io::{Read, Seek, SeekFrom};

    const MAX_TAIL_BYTES: u64 = 16 * 1024;
    const MAX_TAIL_LINES: usize = 12;

    let Ok(mut file) = File::open(log_path) else {
        return String::new();
    };
    let Ok(length) = file.metadata().map(|metadata| metadata.len()) else {
        return String::new();
    };
    let start = length.saturating_sub(MAX_TAIL_BYTES);
    if file.seek(SeekFrom::Start(start)).is_err() {
        return String::new();
    }
    let mut bytes = Vec::with_capacity((length - start) as usize);
    if file.take(MAX_TAIL_BYTES).read_to_end(&mut bytes).is_err() {
        return String::new();
    }
    let text = String::from_utf8_lossy(&bytes);
    text.lines()
        .rev()
        .take(MAX_TAIL_LINES)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

fn launch_failure(
    phase: &str,
    message: String,
    log_path: &str,
    exit_code: Option<i32>,
    timed_out: bool,
) -> String {
    let evidence = launch_failure_evidence(phase, message, log_path, exit_code, timed_out);
    serde_json::to_string(&evidence)
        .unwrap_or_else(|_| LAUNCH_FAILURE_SERIALIZATION_FALLBACK.into())
}

fn cancelled_launch_failure(message: String, log_path: &str) -> String {
    let mut evidence = launch_failure_evidence("health_cancelled", message, log_path, None, false);
    evidence.cancelled = true;
    serde_json::to_string(&evidence)
        .unwrap_or_else(|_| LAUNCH_FAILURE_SERIALIZATION_FALLBACK.into())
}

fn launch_failure_evidence(
    phase: &str,
    message: String,
    log_path: &str,
    exit_code: Option<i32>,
    timed_out: bool,
) -> LaunchFailureEvidence {
    LaunchFailureEvidence {
        schema: 1,
        phase: phase.into(),
        exit_code,
        timed_out,
        cancelled: false,
        message,
        log_tail: bounded_log_tail(log_path),
    }
}

/// Block until `/health` answers 200, the child exits, or the deadline passes.
/// The production startup path uses `wait_until_healthy_cancellable`
/// (audit IPC-01); this non-cancellable form remains for tests.
#[cfg(test)]
fn wait_until_healthy(
    child: &mut impl HealthProcess,
    client: &crate::local_client::LocalHttpClient,
    log_path: &str,
    timeout: Duration,
) -> Result<(), String> {
    wait_until_healthy_inner(child, client, log_path, timeout, None)
}

fn wait_until_healthy_cancellable(
    child: &mut impl HealthProcess,
    client: &crate::local_client::LocalHttpClient,
    log_path: &str,
    timeout: Duration,
    cancelled: &AtomicBool,
) -> Result<(), String> {
    wait_until_healthy_inner(child, client, log_path, timeout, Some(cancelled))
}

fn wait_until_healthy_inner(
    child: &mut impl HealthProcess,
    client: &crate::local_client::LocalHttpClient,
    log_path: &str,
    timeout: Duration,
    cancelled: Option<&AtomicBool>,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let mut last_health_error = None;
    let addresses = health_socket_addresses(client.host(), client.port())
        .map_err(|message| launch_failure("health_connect", message, log_path, None, false))?;
    loop {
        if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err(cancelled_launch_failure(
                "Benchmark cancelled while waiting for a fresh runtime".into(),
                log_path,
            ));
        }
        if let Ok(Some(code)) = child.try_wait() {
            return Err(launch_failure(
                "health_exit",
                format!(
                    "llama-server exited with {} before becoming healthy",
                    code.code()
                        .map(|c| format!("code {c}"))
                        .unwrap_or_else(|| "a signal".into())
                ),
                log_path,
                code.code(),
                false,
            ));
        }
        {
            // The startup health probe goes through the centralized local
            // client (audit MT-06): a TLS/API-key-configured server answers
            // it, framing and bounds apply, and the profile's transport is
            // honored instead of a plaintext TCP guess.
            // Bound each probe so the loop still observes process exit and
            // cancellation between attempts (a refused connect can retry for
            // seconds inside one call).
            let probe_budget = deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_secs(1));
            if let Ok((status, _)) = client.get_bytes("/health", probe_budget) {
                if status == 200 {
                    // Any of the host's candidate loopback addresses must be
                    // owned by the child; an explicit check error surfaces.
                    let owner_checks = addresses
                        .iter()
                        .map(|address| listener_is_owned_by_at(*address, child.id()))
                        .collect::<Vec<_>>();
                    let owned = if owner_checks.iter().any(|check| matches!(check, Ok(true))) {
                        Ok(true)
                    } else if let Some(Err(error)) =
                        owner_checks.iter().find(|check| check.is_err())
                    {
                        Err(error.clone())
                    } else {
                        Ok(false)
                    };
                    match owned {
                        Ok(true) => match child.try_wait() {
                            Ok(None) => return Ok(()),
                            Ok(Some(code)) => {
                                return Err(launch_failure(
                                    "health_exit",
                                    format!(
                                        "llama-server exited with {} while health was being validated",
                                        code.code()
                                            .map(|value| format!("code {value}"))
                                            .unwrap_or_else(|| "a signal".into())
                                    ),
                                    log_path,
                                    code.code(),
                                    false,
                                ));
                            }
                            Err(error) => {
                                return Err(launch_failure(
                                    "health_owner",
                                    format!(
                                        "Could not confirm llama-server process state after health validation: {error}"
                                    ),
                                    log_path,
                                    None,
                                    false,
                                ));
                            }
                        },
                        Ok(false) => {
                            last_health_error = Some(format!(
                                "llama-server process {} does not own TCP port {}",
                                child.id(),
                                client.port()
                            ));
                        }
                        Err(error) => {
                            return Err(launch_failure(
                                "health_owner",
                                error,
                                log_path,
                                None,
                                false,
                            ));
                        }
                    }
                }
            }
        }
        // Exit wins over the timeout verdict: a child that exited during the
        // final probe must still produce structured health_exit evidence.
        if let Ok(Some(code)) = child.try_wait() {
            return Err(launch_failure(
                "health_exit",
                format!(
                    "llama-server exited with {} before becoming healthy",
                    code.code()
                        .map(|c| format!("code {c}"))
                        .unwrap_or_else(|| "a signal".into())
                ),
                log_path,
                code.code(),
                false,
            ));
        }
        if Instant::now() >= deadline {
            let detail = last_health_error
                .as_deref()
                .map(|error| format!("; last validation error: {error}"))
                .unwrap_or_default();
            return Err(launch_failure(
                "health_timeout",
                format!(
                    "llama-server did not become healthy within {} ms{detail}",
                    timeout.as_millis()
                ),
                log_path,
                None,
                true,
            ));
        }
        std::thread::sleep(Duration::from_millis(400));
    }
}

#[tauri::command]
async fn scan_models(root: String) -> Result<Vec<LogicalModel>, String> {
    tauri::async_runtime::spawn_blocking(move || core::scan_models(Path::new(&root)))
        .await
        .map_err(|error| format!("Model scan task failed: {error}"))?
}

#[tauri::command]
async fn inspect_runtime(path: String) -> Result<RuntimeCapabilities, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&path);
        runtime::verify_managed_runtime_for_launch(path)?;
        core::inspect_runtime(path)
    })
    .await
    .map_err(|error| format!("Runtime inspection task failed: {error}"))?
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeHealthRequest {
    path: String,
    #[serde(default)]
    expected_adapters: Vec<String>,
    expected_backend: String,
    expected_model: String,
}

#[tauri::command]
async fn check_runtime_health(
    request: RuntimeHealthRequest,
) -> Result<core::RuntimeDeviceHealth, String> {
    tauri::async_runtime::spawn_blocking(move || {
        core::check_runtime_health(
            Path::new(&request.path),
            &request.expected_adapters,
            &request.expected_backend,
            &request.expected_model,
        )
    })
    .await
    .map_err(|error| format!("Runtime health task failed: {error}"))?
}

#[tauri::command]
fn describe_runtime(path: String) -> runtime::RuntimeIdentity {
    let path = Path::new(&path);
    let mut identity = runtime::describe_runtime(path);
    identity.managed_verified = runtime::managed_runtime_verified(path).unwrap_or(false);
    identity
}

#[tauri::command]
fn list_managed_runtimes() -> Result<Vec<runtime::ManagedRuntimeRecord>, String> {
    runtime::list_managed_runtimes()
}

#[tauri::command]
async fn read_gguf_summary(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<gguf::GgufSummary, String> {
    // Parsing runs in bounded background work: the flag lets a large or
    // malformed header be cancelled without holding the UI (audit DC-08 I4).
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut slot = state
            .gguf_read
            .lock()
            .map_err(|_| "GGUF read state is unavailable".to_string())?;
        *slot = Some(cancel.clone());
    }
    let result = tauri::async_runtime::spawn_blocking(move || {
        gguf::read_summary_cancellable(Path::new(&path), Some(&cancel))
    })
    .await
    .map_err(|error| format!("GGUF read task failed: {error}"))?;
    if let Ok(mut slot) = state.gguf_read.lock() {
        *slot = None;
    }
    result
}

#[tauri::command]
fn cancel_gguf_read(state: tauri::State<'_, AppState>) -> bool {
    state
        .gguf_read
        .lock()
        .ok()
        .and_then(|slot| {
            slot.as_ref()
                .map(|flag| flag.store(true, std::sync::atomic::Ordering::Relaxed))
        })
        .is_some()
}

#[tauri::command]
async fn inspect_model_artifact(
    first_shard: String,
    companions: Vec<String>,
    hash_files: bool,
) -> Result<artifact::ArtifactInspection, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let companion_paths = companions
            .into_iter()
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>();
        artifact::inspect_artifact(Path::new(&first_shard), &companion_paths, hash_files)
    })
    .await
    .map_err(|error| format!("Artifact inspection task failed: {error}"))?
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreflightRequest {
    profile: LaunchProfile,
    #[serde(default)]
    selected_adapter_ids: Vec<String>,
    #[serde(default)]
    manual_overrides: Vec<runtime::HardwareOverride>,
    reserve_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreflightResult {
    report: preflight::PreflightReport,
    device_plan: Vec<preflight::DeviceAllocationPlan>,
    launch: LaunchValidation,
    hardware: runtime::HardwareInfo,
    selected_adapter_ids: Vec<String>,
}

fn unknown_memory(detail: &str, observed_at_ms: u64) -> evidence::Evidence<u64> {
    evidence::Evidence {
        value: None,
        level: evidence::EvidenceLevel::Unknown,
        source: evidence::EvidenceSource {
            kind: evidence::EvidenceSourceKind::Policy,
            detail: detail.into(),
        },
        observed_at_ms,
        notes: vec![detail.into()],
    }
}

#[tauri::command]
async fn preflight_model(request: PreflightRequest) -> Result<PreflightResult, String> {
    // Preflight hashes files and probes hardware: run it on a blocking
    // worker so neither the Tauri main thread nor the async executor stalls
    // (audit IPC-01 I4).
    tauri::async_runtime::spawn_blocking(move || {
        let launch = prepare_launch(&request.profile)?;
        let mut hardware = runtime::detect_hardware();
        hardware.manual_overrides = request.manual_overrides.clone();
        let observed_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis()
            .try_into()
            .map_err(|_| "System time is outside the supported range")?;
        let execution_path = execution_path_for(&request.profile, &request.selected_adapter_ids);
        let manual_capacity = runtime::manual_override_capacity(
            &request.manual_overrides,
            request
                .selected_adapter_ids
                .first()
                .map(String::as_str)
                .unwrap_or(""),
            observed_at_ms,
        )?;
        let device_capacities = if matches!(execution_path, evidence::ExecutionPath::Cpu) {
            vec![(
                "system-memory".into(),
                hardware.system_memory.available_physical_bytes.clone(),
            )]
        } else {
            request
                .selected_adapter_ids
                .iter()
                .enumerate()
                .map(|(index, adapter_id)| {
                    let manual = if index == 0 {
                        manual_capacity.clone()
                    } else {
                        runtime::manual_override_capacity(
                            &request.manual_overrides,
                            adapter_id,
                            observed_at_ms,
                        )?
                    };
                    let capacity = manual
                        .or_else(|| {
                            hardware
                                .adapters
                                .iter()
                                .find(|adapter| adapter.adapter_id == *adapter_id)
                                .map(|adapter| adapter.available_budget_bytes.clone())
                        })
                        .unwrap_or_else(|| {
                            unknown_memory(
                                "The selected adapter was not present in the current hardware observation.",
                                observed_at_ms,
                            )
                        });
                    Ok((adapter_id.clone(), capacity))
                })
                .collect::<Result<Vec<_>, String>>()?
        };
        let available_memory = match execution_path {
            evidence::ExecutionPath::Cpu => hardware.system_memory.available_physical_bytes.clone(),
            evidence::ExecutionPath::FullGpu | evidence::ExecutionPath::LayerOffload => {
                if device_capacities.len() != 1 {
                    unknown_memory(
                        "Select exactly one adapter for this execution path.",
                        observed_at_ms,
                    )
                } else {
                    device_capacities[0].1.clone()
                }
            }
            evidence::ExecutionPath::MultiGpu => unknown_memory(
                "Per-device placement is required; Localmotive does not aggregate adapter memory.",
                observed_at_ms,
            ),
            _ => unknown_memory(
                "Select an explicit CPU, single-adapter, or multi-adapter execution path.",
                observed_at_ms,
            ),
        };
        let main_artifact = launch
            .artifacts
            .first()
            .ok_or("Launch validation did not return a model artifact")?;
        let summary = main_artifact
            .summary
            .as_ref()
            .ok_or("The model GGUF summary is unavailable")?;
        let companion_bytes = launch.artifacts.iter().enumerate().try_fold(
            main_artifact.companion_bytes,
            |total, (index, artifact)| {
                if index == 0 {
                    Ok(total)
                } else {
                    total
                        .checked_add(artifact.shard_bytes)
                        .and_then(|value| value.checked_add(artifact.companion_bytes))
                        .ok_or("Companion storage size overflowed")
                }
            },
        )?;
        let recurrent_or_hybrid = summary.metadata_facts.iter().any(|fact| {
            fact.key.contains(".recurrent.")
                || fact.key.contains(".ssm.")
                || fact.key.contains(".state_space.")
        });
        let mut assumptions = vec![
            "Artifact file bytes are a weight-allocation proxy, not observed device memory.".into(),
            "The named reserve covers runtime allocations that GGUF metadata cannot describe.".into(),
        ];
        assumptions.extend(
            launch
                .arguments
                .rejected
                .iter()
                .map(|rejected| format!("{}: {}", rejected.flag, rejected.reason)),
        );
        let storage_files = launch
            .artifacts
            .iter()
            .flat_map(|artifact| artifact.shards.iter().chain(&artifact.companions))
            .map(|file| (PathBuf::from(&file.path), file.size_bytes))
            .collect::<Vec<_>>();
        let report = preflight::build_preflight_report(preflight::PreflightFacts {
            execution_path,
            runtime_topology_known: matches!(execution_path, evidence::ExecutionPath::Cpu),
            unverified_requirements: launch.unverified_requirements.clone(),
            requested_context: u64::from(request.profile.context),
            native_context: summary.context_length,
            runtime_fit_enabled: request.profile.fit && launch.runtime.fit,
            weight_bytes: main_artifact.shard_bytes,
            companion_bytes,
            kv: preflight::KvCacheInputs {
                architecture: summary.architecture.clone(),
                block_count: summary.block_count,
                head_count_kv: summary.head_count_kv,
                key_length: summary.key_length,
                value_length: summary.value_length,
                context: u64::from(request.profile.context),
                cache_type_k: request.profile.cache_type_k.clone(),
                cache_type_v: request.profile.cache_type_v.clone(),
                recurrent_or_hybrid,
                observed_at_ms,
            },
            available_memory,
            available_disk: preflight::available_disk_bytes(
                Path::new(&request.profile.model),
                observed_at_ms,
            ),
            storage_volumes: preflight::storage_volume_evidence(&storage_files, observed_at_ms),
            reserve_bytes: request.reserve_bytes.unwrap_or(1_073_741_824),
            offload_possible: !matches!(execution_path, evidence::ExecutionPath::Cpu),
            assumptions,
        });
        let device_weight_bytes = report
            .weight_bytes
            .value
            .ok_or("Loaded artifact size overflowed before device planning")?;
        let device_plan = preflight::build_device_plan(
            &device_capacities,
            device_weight_bytes,
            &report.kv_cache_bytes,
            report.requested_context.observed_at_ms,
        );
        Ok(PreflightResult {
            report,
            device_plan,
            launch,
            hardware,
            selected_adapter_ids: request.selected_adapter_ids,
        })

    })
    .await
    .map_err(|error| format!("Preflight task failed: {error}"))?
}

#[tauri::command]
fn preview_command(profile: LaunchProfile) -> Result<String, String> {
    // Cheap provisional composition only: no runtime probes, no artifact
    // hashing, no trust checks (audit FE-04 I2). Every authoritative check
    // still runs at validation and launch.
    core::compose_provisional_command(&profile)
}

#[tauri::command]
fn validate_launch_profile(profile: LaunchProfile) -> Result<LaunchValidation, String> {
    prepare_launch(&profile)
}

/// Stop-to-exit budget for a startup that is still waiting for readiness
/// (audit IPC-01). The worker polls its cancellation signal every few
/// hundred milliseconds, then terminates and reaps the contained tree; this
/// deadline bounds how long Stop waits for that terminal state.
pub(crate) const STARTUP_STOP_DEADLINE_SECS: u64 = 10;

#[tauri::command]

/// The lifecycle phase observable while a start is pending (audit IPC-01).
fn server_phase(state: &AppState) -> &'static str {
    if state
        .starting
        .lock()
        .map(|starting| starting.is_some())
        .unwrap_or(false)
    {
        return "starting";
    }
    "idle"
}

#[tauri::command]
async fn start_server(
    app: tauri::AppHandle,
    profile: LaunchProfile,
    state: tauri::State<'_, AppState>,
) -> Result<ServerStatus, String> {
    // One owner at a time: a running server, benchmark, tuning session, or
    // quality suite all hold the reservation (audit MT-05).
    let reservation = reserve_operation(&state.operations, OperationOwner::Server)?;
    let operation_id = reservation.generation;
    let cancel = Arc::new(AtomicBool::new(false));
    {
        // One short lock claims the operation and publishes the starting
        // state; the readiness wait runs on a blocking worker without any
        // server lock held (audit IPC-01 I1/I2).
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable".to_string())?;
        if status_from(&mut slot).running {
            return Err("Stop the running server before starting another profile".into());
        }
        let mut starting = state
            .starting
            .lock()
            .map_err(|_| "Server startup state is unavailable".to_string())?;
        if starting.is_some() {
            return Err("A server start is already in progress; Stop it first".into());
        }
        *starting = Some(StartingServer {
            operation_id,
            cancel: cancel.clone(),
        });
    }
    let worker_app = app.clone();
    let worker_cancel = cancel.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        start_server_worker(&worker_app, profile, operation_id, worker_cancel)
    })
    .await
    .map_err(|error| format!("Server startup task failed: {error}"))?;
    outcome
}

/// True when the given startup operation may still publish: the server
/// reservation is still held, no cancellation arrived, and the starting slot
/// still names this operation (audit IPC-01 I3). A late completion from an
/// older operation can never publish or clear a newer operation's state.
fn startup_is_current(state: &AppState, operation_id: u64, cancel: &AtomicBool) -> bool {
    !cancel.load(Ordering::Relaxed)
        && active_operation_owner(&state.operations) == Some(OperationOwner::Server)
        && state
            .starting
            .lock()
            .ok()
            .and_then(|starting| starting.as_ref().map(|entry| entry.operation_id))
            == Some(operation_id)
}

/// The blocking half of `start_server`: launch, wait for readiness without
/// holding any lock, and commit the server slot only while this operation
/// still owns it; otherwise terminate and reap before returning
/// (audit IPC-01 I2/I3).
fn start_server_worker(
    app: &tauri::AppHandle,
    profile: LaunchProfile,
    operation_id: u64,
    cancel: Arc<AtomicBool>,
) -> Result<ServerStatus, String> {
    let state = app.state::<AppState>();
    let clear_starting = |state: &AppState| {
        if let Ok(mut starting) = state.starting.lock() {
            if starting
                .as_ref()
                .is_some_and(|entry| entry.operation_id == operation_id)
            {
                *starting = None;
            }
        }
    };
    let (mut child, mut validation, log_path, execution_lease) =
        match spawn_server(&profile, &format!("server-{}", profile.port)) {
            Ok(spawned) => spawned,
            Err(error) => {
                clear_starting(&state);
                return Err(error);
            }
        };
    let health = wait_until_healthy_cancellable(
        &mut child,
        &local_client(&profile)?,
        &log_path,
        Duration::from_secs(600),
        &cancel,
    );
    if let Err(error) = health {
        let _ = child.terminate_and_wait();
        clear_starting(&state);
        return Err(error);
    }
    let observed_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| duration.as_millis().try_into().ok())
        .unwrap_or(validation.effective_context.observed_at_ms);
    update_launch_effective_context(
        &mut validation,
        &profile.host,
        profile.port,
        Duration::from_secs(5),
        observed_at_ms,
    );
    let command = validation.arguments.command.clone();
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let mut slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable".to_string())?;
    let owns = startup_is_current(&state, operation_id, &cancel);
    if !owns {
        drop(slot);
        let _ = child.terminate_and_wait();
        clear_starting(&state);
        return Err(
            "Server startup was cancelled or replaced; the contained process tree was stopped."
                .into(),
        );
    }
    *slot = Some(ManagedServer {
        child,
        profile,
        command,
        validation,
        log_path,
        started_at,
        runtime_lease: execution_lease,
    });
    clear_starting(&state);
    Ok(status_from(&mut slot))
}

#[tauri::command]
async fn stop_server(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ServerStatus, String> {
    if let Some(conflict) = stop_owner_conflict(active_operation_owner(&state.operations)) {
        return Err(conflict);
    }
    // A pending start is cancelled through its signal; its own worker
    // terminates and reaps the tree before clearing the starting slot, and
    // Stop waits a bounded time for that terminal state (audit IPC-01 I2/I3).
    let pending = state
        .starting
        .lock()
        .map_err(|_| "Server startup state is unavailable".to_string())?
        .as_ref()
        .map(|entry| entry.cancel.clone());
    let worker_app = app.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let state = worker_app.state::<AppState>();
        if let Some(cancel) = pending {
            cancel.store(true, Ordering::Relaxed);
            let deadline = Instant::now() + Duration::from_secs(STARTUP_STOP_DEADLINE_SECS);
            loop {
                let cleared = state
                    .starting
                    .lock()
                    .map(|starting| starting.is_none())
                    .unwrap_or(false);
                if cleared {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(format!(
                        "The starting server did not stop within {STARTUP_STOP_DEADLINE_SECS} seconds; its contained process tree is still being reaped."
                    ));
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        }
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable".to_string())?;
        if let Some(server) = slot.as_mut() {
            if !server.child.terminate_and_wait() {
                return Err("The contained llama-server process tree did not stop".into());
            }
        }
        *slot = None;
        Ok(status_from(&mut slot))
    })
    .await
    .map_err(|error| format!("Server stop task failed: {error}"))?;
    outcome
}

#[tauri::command]
fn server_status(state: tauri::State<AppState>) -> Result<ServerStatus, String> {
    let mut slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable")?;
    let mut status = status_from(&mut slot);
    if !status.running && server_phase(&state) == "starting" {
        status.phase = "starting".to_string();
    }
    Ok(status)
}

#[derive(Clone)]
struct ValidatedServerSnapshot {
    pid: u32,
    profile: LaunchProfile,
    validation: LaunchValidation,
}

fn validated_server_snapshot(
    slot: &mut Option<ManagedServer>,
    action: &str,
) -> Result<ValidatedServerSnapshot, String> {
    let status = status_from(slot);
    if !status.running {
        return Err(format!(
            "Start and validate a Localmotive server before {action}"
        ));
    }
    if status.result_class != evidence::FitClass::LaunchValidated {
        return Err(format!(
            "{action} requires a successful launch health proof"
        ));
    }
    let server = slot
        .as_ref()
        .ok_or("Validated server state is unavailable")?;
    Ok(ValidatedServerSnapshot {
        pid: server.child.id(),
        profile: server.profile.clone(),
        validation: server.validation.clone(),
    })
}

#[tauri::command]
fn read_server_log(state: tauri::State<AppState>) -> Result<String, String> {
    let slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable")?;
    let Some(server) = slot.as_ref() else {
        return Ok("No server is running.".into());
    };
    Ok(bounded_log_tail(&server.log_path))
}

#[tauri::command]
async fn benchmark_server(
    tokens: u32,
    repeats: u16,
    state: tauri::State<'_, AppState>,
) -> Result<BenchmarkSummary, String> {
    let server = {
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable".to_string())?;
        validated_server_snapshot(&mut slot, "benchmarking")?
    };
    // The legacy warm measurement blocks on loopback requests: it runs on a
    // blocking worker instead of the main thread or async executor
    // (audit IPC-01 I4).
    tauri::async_runtime::spawn_blocking(move || {
        let client = local_client(&server.profile)?;
        core::benchmark_server(&client, tokens, repeats)
    })
    .await
    .map_err(|error| format!("Benchmark task failed: {error}"))?
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkRunResult {
    manifest: evidence::BenchmarkManifest,
    summary: Option<measurement::BenchmarkSummaryV2>,
    manifest_path: String,
    compatibility_key: String,
    result_class: evidence::FitClass,
    failure: Option<String>,
}

fn benchmark_file_fact(file: &artifact::ArtifactFileFact) -> evidence::FileFact {
    evidence::FileFact {
        path: file.path.clone(),
        bytes: file.size_bytes,
        sha256: file.sha256.clone(),
    }
}

fn benchmark_compatibility_key_from_snapshot(
    profile: &LaunchProfile,
    validation: &LaunchValidation,
    artifacts: &[artifact::ArtifactInspection],
    runtime_identity: &runtime::RuntimeIdentity,
    executable_sha256: &str,
    hardware: &runtime::HardwareInfo,
    workload: &evidence::Workload,
) -> Result<String, String> {
    let main = artifacts
        .first()
        .ok_or("Benchmark snapshot did not contain a model artifact")?;
    let model_architecture = main
        .summary
        .as_ref()
        .map(|summary| summary.architecture.clone())
        .ok_or("Benchmark model architecture is unavailable")?;
    let content_ids = artifacts
        .iter()
        .map(|artifact| {
            artifact
                .content_id
                .clone()
                .ok_or("Benchmark selected artifact content digest is unavailable")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let model_content_sha256 = calibration::selected_artifact_set_sha256(&content_ids)?;
    let mut adapters = hardware
        .adapters
        .iter()
        .map(|adapter| {
            (
                adapter.adapter_id.clone(),
                adapter
                    .driver
                    .value
                    .clone()
                    .unwrap_or_else(|| "unknown".into()),
            )
        })
        .collect::<Vec<_>>();
    adapters.sort_by(|left, right| left.0.cmp(&right.0));
    let (adapter_ids, driver_versions) = adapters.into_iter().unzip();
    calibration::compatibility_key(&calibration::CompatibilityIdentity {
        model_content_sha256,
        model_architecture,
        runtime_sha256: executable_sha256.into(),
        runtime_help_sha256: validation.runtime.help_sha256.clone(),
        runtime_backend: runtime_identity.backend.clone(),
        runtime_version: validation.runtime.version.clone(),
        runtime_build: validation.runtime.build.clone(),
        adapter_ids,
        driver_versions,
        context: profile.context,
        parallel: profile.parallel,
        batch: profile.batch,
        ubatch: profile.ubatch,
        main_gpu: profile.main_gpu,
        cache_type_k: profile.cache_type_k.clone(),
        cache_type_v: profile.cache_type_v.clone(),
        gpu_layers: profile.gpu_layers.clone(),
        split_mode: profile.split_mode.clone(),
        tensor_split: profile.tensor_split.clone(),
        workload_sha256: calibration::workload_sha256(workload)?,
        harness_version: env!("CARGO_PKG_VERSION").into(),
    })
}

fn run_benchmark_snapshot(
    server_pid: u32,
    profile: LaunchProfile,
    validation: LaunchValidation,
    workload: evidence::Workload,
    prepared_prompt_tokens: Option<Vec<i32>>,
    cancelled: &AtomicBool,
    directory: &Path,
) -> Result<BenchmarkRunResult, String> {
    let mut artifacts = Vec::new();
    for artifact in &validation.artifacts {
        artifacts.push(artifact::inspect_artifact(
            Path::new(&artifact.first_shard),
            &[],
            true,
        )?);
    }
    let main = artifacts
        .first()
        .ok_or("Benchmark snapshot did not contain a model artifact")?;
    let mut companion_files = main
        .companions
        .iter()
        .map(benchmark_file_fact)
        .collect::<Vec<_>>();
    for artifact in artifacts.iter().skip(1) {
        companion_files.extend(artifact.shards.iter().map(benchmark_file_fact));
        companion_files.extend(artifact.companions.iter().map(benchmark_file_fact));
    }
    let header_sha256 = main
        .shards
        .first()
        .and_then(|file| file.header_sha256.clone())
        .ok_or("Benchmark model header digest is unavailable")?;
    let runtime_identity = runtime::describe_runtime(Path::new(&profile.runtime));
    let executable_sha256 = artifact::sha256_path(Path::new(&profile.runtime))?;
    let help_sha256 = validation.runtime.help_sha256.clone();
    let model_architecture = main
        .summary
        .as_ref()
        .map(|summary| summary.architecture.clone())
        .ok_or("Benchmark model architecture is unavailable")?;
    let harness_version = env!("CARGO_PKG_VERSION").to_string();
    let hardware = runtime::detect_hardware();
    let hardware_facts = hardware
        .adapters
        .iter()
        .map(|adapter| evidence::HardwareFact {
            adapter_id: adapter.adapter_id.clone(),
            name: adapter.name.clone(),
            vendor: adapter.vendor.clone(),
            driver: adapter.driver.value.clone(),
            backend: adapter.backend.value.clone(),
            dedicated_bytes: adapter.dedicated_bytes.clone(),
            shared_bytes: adapter.shared_bytes.clone(),
            budget_bytes: adapter.budget_bytes.clone(),
            current_usage_bytes: adapter.current_usage_bytes.clone(),
        })
        .collect();
    let compatibility_key = benchmark_compatibility_key_from_snapshot(
        &profile,
        &validation,
        &artifacts,
        &runtime_identity,
        &executable_sha256,
        &hardware,
        &workload,
    )?;
    let launch_fact = evidence::LaunchFact {
        requested_context: profile.context,
        effective_context: validation.effective_context.clone(),
        parallel: profile.parallel,
        gpu_layers: profile.gpu_layers.clone(),
        batch: profile.batch,
        ubatch: profile.ubatch,
        cache_type_k: profile.cache_type_k.clone(),
        cache_type_v: profile.cache_type_v.clone(),
        split_mode: profile.split_mode.clone(),
        tensor_split: profile.tensor_split.clone(),
        main_gpu: profile.main_gpu,
        command_args: core::manifest_safe_args(&validation.arguments.effective_args),
        rejected_flags: validation
            .arguments
            .rejected
            .iter()
            .map(|item| item.flag.clone())
            .collect(),
    };
    launch_fact.validate().map_err(|error| error.to_string())?;
    let prompt_tokens = match (workload.cache_mode, prepared_prompt_tokens) {
        (evidence::CacheMode::Cold, None) => {
            return Err("Cold-cache benchmarking requires prepared prompt tokens".into());
        }
        (_, Some(prompt_tokens)) => prompt_tokens,
        (evidence::CacheMode::Warm, None) => measurement::prepare_exact_prompt_tokens_cancellable(
            &local_client(&profile)?,
            &workload,
            cancelled,
        )?,
    };
    let workload_run = if workload.cache_mode == evidence::CacheMode::Cold {
        measurement::run_cold_workload_with(&workload, cancelled, || {
            if cancelled.load(Ordering::Relaxed) {
                return Err("Benchmark cancelled before fresh runtime launch".into());
            }
            // The lease binding pins the verified runtime content for the whole
            // cold attempt (audit RT-04); it drops with this scope.
            let (mut child, _, log_path, _lease) = spawn_server(&profile, "benchmark-cold")?;
            let attempt = wait_until_healthy_cancellable(
                &mut child,
                &local_client(&profile)?,
                &log_path,
                Duration::from_secs(120),
                cancelled,
            )
            .and_then(|_| {
                let mut timing = measurement::completion_request_with_prompt_tokens_cancellable(
                    &local_client(&profile)?,
                    &workload,
                    &prompt_tokens,
                    cancelled,
                )?;
                timing.peak_process_rss_bytes = runtime::process_peak_working_set(child.id());
                Ok(timing)
            });
            let cleanup = child.terminate_and_wait();
            match (attempt, cleanup) {
                (Ok(timing), true) => Ok(timing),
                (Err(error), true) => Err(error),
                (Ok(_), false) => {
                    Err("Could not stop the contained fresh benchmark runtime tree".into())
                }
                (Err(error), false) => Err(format!(
                    "{error}; contained fresh runtime tree cleanup also failed"
                )),
            }
        })?
    } else {
        measurement::run_workload_with(&workload, cancelled, || {
            let mut timing = measurement::completion_request_with_prompt_tokens_cancellable(
                &local_client(&profile)?,
                &workload,
                &prompt_tokens,
                cancelled,
            )?;
            timing.peak_process_rss_bytes = runtime::process_peak_working_set(server_pid);
            Ok(timing)
        })?
    };
    let mut manifest = evidence::BenchmarkManifest {
        schema: evidence::BENCHMARK_SCHEMA_VERSION,
        harness_version,
        compatibility_key: Some(compatibility_key.clone()),
        runtime: Some(evidence::RuntimeFact {
            path: profile.runtime.clone(),
            version: validation.runtime.version.clone(),
            build: validation.runtime.build.clone(),
            executable_sha256: Some(executable_sha256),
            help_sha256,
            backend: runtime_identity.backend,
        }),
        hardware: hardware_facts,
        model: Some(evidence::ModelFact {
            logical_id: main.logical_id.clone(),
            architecture: model_architecture,
            shards: main.shards.iter().map(benchmark_file_fact).collect(),
            companions: companion_files,
            gguf_header_sha256: header_sha256,
        }),
        launch: Some(launch_fact),
        workload,
        warmups: workload_run.warmups,
        observations: workload_run.observations,
        terminal_outcome: workload_run.terminal_outcome,
    };
    manifest
        .validate_complete()
        .map_err(|error| error.to_string())?;
    let summary_result = measurement::summarize_observations(&manifest.observations);
    let (summary, result_class, failure) = match (summary_result, manifest.terminal_outcome) {
        (Ok(summary), None) => (Some(summary), evidence::FitClass::Measured, None),
        (Ok(summary), Some(outcome)) => (
            Some(summary),
            evidence::FitClass::Failed,
            Some(format!("Benchmark terminated with {outcome:?}")),
        ),
        (Err(error), _) => (None, evidence::FitClass::Failed, Some(error)),
    };
    let manifest_path = measurement::persist_manifest(directory, &manifest)?;
    Ok(BenchmarkRunResult {
        manifest: std::mem::take(&mut manifest),
        summary,
        manifest_path: manifest_path.to_string_lossy().to_string(),
        compatibility_key,
        result_class,
        failure,
    })
}

#[tauri::command]
async fn benchmark_v2(
    workload: evidence::Workload,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<BenchmarkRunResult, String> {
    workload.validate().map_err(|error| error.to_string())?;
    // One machine owner: a benchmark cannot start while a server, tuning
    // session, or quality suite owns the operations slot, and a cold attempt
    // is its own owner kind (audit MT-05).
    let owner = if workload.cache_mode == evidence::CacheMode::Cold {
        OperationOwner::ColdBenchmark
    } else {
        OperationOwner::Benchmark
    };
    let reservation = reserve_operation(&state.operations, owner)?;
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("benchmarks");
    let server = {
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable")?;
        validated_server_snapshot(&mut slot, "benchmarking")?
    };
    let cancelled = Arc::new(AtomicBool::new(false));
    {
        let mut active = state
            .benchmark
            .lock()
            .map_err(|_| "Benchmark state is unavailable")?;
        if active.is_some() {
            return Err("A benchmark is already running".into());
        }
        *active = Some(cancelled.clone());
    }
    let benchmark_result: Result<BenchmarkRunResult, String> = async {
        let prepared_prompt_tokens = if workload.cache_mode == evidence::CacheMode::Cold {
            let profile = server.profile.clone();
            let workload = workload.clone();
            let preparation_cancelled = cancelled.clone();
            Some(
                tauri::async_runtime::spawn_blocking(move || {
                    let client = crate::local_client::LocalHttpClient::from_profile(&profile)?;
                    measurement::prepare_exact_prompt_tokens_cancellable(
                        &client,
                        &workload,
                        preparation_cancelled.as_ref(),
                    )
                })
                .await
                .map_err(|error| format!("Benchmark prompt preparation task failed: {error}"))??,
            )
        } else {
            None
        };
        if workload.cache_mode == evidence::CacheMode::Cold {
            let mut managed = {
                let mut slot = state
                    .server
                    .lock()
                    .map_err(|_| "Server state is unavailable")?;
                let current = slot
                    .as_ref()
                    .ok_or("Validated server stopped during cold benchmark preparation")?;
                if current.child.id() != server.pid {
                    return Err("Validated server changed during cold benchmark preparation".into());
                }
                slot.take().expect("validated server was present")
            };
            if !managed.child.terminate_and_wait() {
                let mut slot = state
                    .server
                    .lock()
                    .map_err(|_| "Server state is unavailable")?;
                *slot = Some(managed);
                return Err(
                    "Could not stop the contained server tree for cold benchmarking".into(),
                );
            }
        }
        let task_cancelled = cancelled.clone();
        tauri::async_runtime::spawn_blocking(move || {
            run_benchmark_snapshot(
                server.pid,
                server.profile,
                server.validation,
                workload,
                prepared_prompt_tokens,
                task_cancelled.as_ref(),
                &directory,
            )
        })
        .await
        .map_err(|error| format!("Benchmark task failed: {error}"))?
    }
    .await;
    let mut active = state
        .benchmark
        .lock()
        .map_err(|_| "Benchmark state is unavailable")?;
    *active = None;
    let result = benchmark_result?;
    // A replaced or stopped server invalidates the whole record: results must
    // never be finalized under an identity that no longer exists
    // (audit MT-05 I3).
    if !reservation.is_current() {
        return Err(
            "The managed server was stopped or replaced during the benchmark; the record was discarded."
                .into(),
        );
    }
    Ok(result)
}

#[tauri::command]
fn cancel_benchmark(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let active = state
        .benchmark
        .lock()
        .map_err(|_| "Benchmark state is unavailable")?;
    let cancel = active.as_ref().ok_or("No benchmark is running")?;
    cancel.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn replay_benchmark_manifest(
    manifest: evidence::BenchmarkManifest,
    state: tauri::State<'_, AppState>,
) -> Result<evidence::Workload, String> {
    manifest
        .validate_complete()
        .map_err(|error| error.to_string())?;
    let server = {
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable".to_string())?;
        validated_server_snapshot(&mut slot, "replaying a benchmark manifest")?
    };
    // Replay re-hashes artifacts and the runtime executable: blocking worker
    // (audit IPC-01 I4).
    tauri::async_runtime::spawn_blocking(move || replay_benchmark_manifest_worker(manifest, server))
        .await
        .map_err(|error| format!("Replay task failed: {error}"))?
}

fn replay_benchmark_manifest_worker(
    manifest: evidence::BenchmarkManifest,
    server: ValidatedServerSnapshot,
) -> Result<evidence::Workload, String> {
    let logical_id = server
        .validation
        .artifacts
        .first()
        .map(|artifact| artifact.logical_id.as_str())
        .ok_or("Running server model identity is unavailable")?;
    let artifacts = server
        .validation
        .artifacts
        .iter()
        .map(|artifact| artifact::inspect_artifact(Path::new(&artifact.first_shard), &[], true))
        .collect::<Result<Vec<_>, _>>()?;
    let runtime_identity = runtime::describe_runtime(Path::new(&server.profile.runtime));
    let executable_sha256 = artifact::sha256_path(Path::new(&server.profile.runtime))?;
    let hardware = runtime::detect_hardware();
    let current_compatibility_key = benchmark_compatibility_key_from_snapshot(
        &server.profile,
        &server.validation,
        &artifacts,
        &runtime_identity,
        &executable_sha256,
        &hardware,
        &manifest.workload,
    )?;
    measurement::validate_replay_compatibility(
        &manifest,
        logical_id,
        &core::manifest_safe_args(&server.validation.arguments.effective_args),
        &current_compatibility_key,
    )?;
    Ok(manifest.workload)
}

#[tauri::command]
async fn run_quality_suite(
    state: tauri::State<'_, AppState>,
) -> Result<recommend::QualitySuiteResult, String> {
    // Quality checks drive the running server: they take the single
    // operations reservation and must not finalize against a replacement
    // (audit MT-05).
    let reservation = reserve_operation(&state.operations, OperationOwner::Quality)?;
    let (profile, runtime_path, model_logical_id) = {
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable")?;
        let server = validated_server_snapshot(&mut slot, "quality checks")?;
        let model_logical_id = server
            .validation
            .artifacts
            .first()
            .map(|artifact| artifact.logical_id.clone())
            .ok_or("Validated model identity is unavailable")?;
        (
            server.profile.clone(),
            server.profile.runtime.clone(),
            model_logical_id,
        )
    };
    let joined = tauri::async_runtime::spawn_blocking(
        move || -> Result<recommend::QualitySuiteResult, String> {
            let client = crate::local_client::LocalHttpClient::from_profile(&profile)?;
            let mut result = recommend::run_quality_suite_with(|_, prompt| {
                measurement::quality_completion_request(&client, prompt)
            });
            result.observed_at_ms = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|error| error.to_string())?
                    .as_millis()
                    .try_into()
                    .map_err(|_| "System time is outside the supported range")?,
            );
            result.model_logical_id = Some(model_logical_id);
            result.runtime_sha256 = Some(artifact::sha256_path(Path::new(&runtime_path))?);
            Ok(result)
        },
    )
    .await
    .map_err(|error| format!("Quality task failed: {error}"))?;
    let result = joined?;
    if !reservation.is_current() {
        return Err(
            "The managed server was stopped or replaced during the quality suite; the results were discarded."
                .into(),
        );
    }
    Ok(result)
}

#[tauri::command]
fn rank_candidates(
    candidates: Vec<recommend::CandidateEvidence>,
    constraints: recommend::RecommendationConstraints,
    weights: recommend::ObjectiveWeights,
) -> Result<Vec<recommend::RankedCandidate>, String> {
    recommend::rank_candidates(&candidates, &constraints, &weights)
}

#[tauri::command]
fn build_compatibility_key(identity: calibration::CompatibilityIdentity) -> Result<String, String> {
    calibration::compatibility_key(&identity)
}

#[tauri::command]
fn build_calibration_model(
    anchors: Vec<calibration::CalibrationAnchor>,
    created_at_ms: u64,
    ttl_ms: u64,
) -> Result<calibration::CalibrationModel, String> {
    calibration::build_calibration(&anchors, created_at_ms, ttl_ms)
}

#[tauri::command]
fn apply_calibration_model(
    model: calibration::CalibrationModel,
    compatibility_key: String,
    estimated_value: f64,
    now_ms: u64,
) -> Result<calibration::CalibratedEstimate, String> {
    calibration::apply_calibration(&model, &compatibility_key, estimated_value, now_ms)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CalibrationRecords {
    anchors: Vec<calibration::CalibrationAnchor>,
    models: Vec<calibration::CalibrationModel>,
}

fn calibration_storage_root(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("calibration"))
}

#[tauri::command]
fn store_calibration_anchor(
    app: tauri::AppHandle,
    anchor: calibration::CalibrationAnchor,
) -> Result<CalibrationRecords, String> {
    let root = calibration_storage_root(&app)?;
    calibration::persist_calibration_anchor(&root, &anchor)?;
    load_calibration_records(app, anchor.compatibility_key)
}

#[tauri::command]
fn store_calibration_model(
    app: tauri::AppHandle,
    model: calibration::CalibrationModel,
) -> Result<CalibrationRecords, String> {
    let root = calibration_storage_root(&app)?;
    calibration::persist_calibration_model(&root, &model)?;
    load_calibration_records(app, model.compatibility_key)
}

#[tauri::command]
fn load_calibration_records(
    app: tauri::AppHandle,
    compatibility_key: String,
) -> Result<CalibrationRecords, String> {
    let root = calibration_storage_root(&app)?;
    Ok(CalibrationRecords {
        anchors: calibration::load_calibration_anchors(&root, &compatibility_key)?,
        models: calibration::load_calibration_models(&root, &compatibility_key)?,
    })
}

#[tauri::command]
fn import_external_evidence(
    bundle: calibration::ExternalEvidenceBundle,
) -> Result<calibration::ExternalEvidenceBundle, String> {
    calibration::validate_external_evidence(bundle)
}

#[tauri::command]
fn review_external_evidence(
    bundle: calibration::ExternalEvidenceBundle,
    state: calibration::ExternalEvidenceState,
    confirmed: bool,
) -> Result<calibration::ExternalEvidenceBundle, String> {
    calibration::review_external_evidence(bundle, state, confirmed)
}

#[tauri::command]
fn build_share_export(
    manifest: evidence::BenchmarkManifest,
    summary: Option<measurement::BenchmarkSummaryV2>,
    quality: Option<recommend::QualitySuiteResult>,
    compatibility_key: String,
    created_at_ms: u64,
    confirmed: bool,
) -> Result<sharing::ShareBundle, String> {
    sharing::require_export_confirmation(confirmed)?;
    sharing::build_share_bundle(
        &manifest,
        summary.as_ref(),
        quality.as_ref(),
        compatibility_key,
        created_at_ms,
    )
}

#[tauri::command]
fn write_share_export(
    path: String,
    bundle: sharing::ShareBundle,
    confirmed: bool,
) -> Result<String, String> {
    sharing::require_export_confirmation(confirmed)?;
    let written = sharing::persist_share_bundle(Path::new(&path), &bundle)?;
    Ok(written.to_string_lossy().into_owned())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeSetupResponse {
    hardware: runtime::HardwareInfo,
    catalog: Option<runtime::RuntimeCatalog>,
    catalog_error: Option<runtime::RuntimeCatalogError>,
    runtime_root: String,
    managed_runtimes: Vec<runtime::ManagedRuntimeRecord>,
}

#[tauri::command]
async fn load_runtime_setup(
    state: tauri::State<'_, AppState>,
    adapter_id: Option<String>,
) -> Result<RuntimeSetupResponse, String> {
    let (hardware, runtime_root, managed_runtimes) = tauri::async_runtime::spawn_blocking(|| {
        Ok::<_, String>((
            runtime::detect_hardware(),
            runtime::managed_runtime_install_root()
                .to_string_lossy()
                .into_owned(),
            runtime::list_managed_runtimes()?,
        ))
    })
    .await
    .map_err(|error| error.to_string())??;
    if let Some(adapter_id) = adapter_id.as_deref() {
        if adapter_id.len() > 256
            || !hardware
                .adapters
                .iter()
                .any(|adapter| adapter.adapter_id == adapter_id)
        {
            return Err("Selected adapter is not in the current hardware snapshot".into());
        }
    }
    let catalog_result =
        match ExclusiveOperation::acquire(&state.runtime_catalog, "runtime catalog") {
            Ok(_active) => runtime::fetch_catalog(&hardware).await.map(|mut catalog| {
                runtime::recommend_catalog_for_adapter(
                    &mut catalog,
                    &hardware,
                    adapter_id.as_deref(),
                );
                catalog
            }),
            Err(message) => Err(runtime::RuntimeCatalogError {
                kind: runtime::RuntimeCatalogErrorKind::Busy,
                message,
                retry_after_seconds: None,
            }),
        };
    let (catalog, catalog_error) = match catalog_result {
        Ok(catalog) => (Some(catalog), None),
        Err(error) => (None, Some(error)),
    };
    Ok(RuntimeSetupResponse {
        hardware,
        catalog,
        catalog_error,
        runtime_root,
        managed_runtimes,
    })
}

#[tauri::command]
fn detect_hardware() -> runtime::HardwareInfo {
    runtime::detect_hardware()
}

/// Verification instrumentation for the runtime setup screens: jobs run,
/// requests coalesced, bytes hashed, jobs cancelled (audit RT-06 I4).
#[tauri::command]
fn runtime_verification_stats() -> runtime::RuntimeVerificationStats {
    runtime::verification_stats()
}

#[tauri::command]
async fn fetch_runtime_catalog(
    state: tauri::State<'_, AppState>,
    adapter_id: Option<String>,
) -> Result<runtime::RuntimeCatalog, runtime::RuntimeCatalogError> {
    let _active = ExclusiveOperation::acquire(&state.runtime_catalog, "runtime catalog").map_err(
        |message| runtime::RuntimeCatalogError {
            kind: runtime::RuntimeCatalogErrorKind::Busy,
            message,
            retry_after_seconds: None,
        },
    )?;
    let hardware = tauri::async_runtime::spawn_blocking(runtime::detect_hardware)
        .await
        .map_err(|error| runtime::RuntimeCatalogError {
            kind: runtime::RuntimeCatalogErrorKind::InvalidResponse,
            message: format!("Hardware detection task failed: {error}"),
            retry_after_seconds: None,
        })?;
    if let Some(adapter_id) = adapter_id.as_deref() {
        if adapter_id.len() > 256
            || !hardware
                .adapters
                .iter()
                .any(|adapter| adapter.adapter_id == adapter_id)
        {
            return Err(runtime::RuntimeCatalogError {
                kind: runtime::RuntimeCatalogErrorKind::InvalidResponse,
                message: "Selected adapter is not in the current hardware snapshot".into(),
                retry_after_seconds: None,
            });
        }
    }
    let mut catalog = runtime::fetch_catalog(&hardware).await?;
    runtime::recommend_catalog_for_adapter(&mut catalog, &hardware, adapter_id.as_deref());
    Ok(catalog)
}

#[tauri::command]
fn managed_runtime_root() -> Result<String, String> {
    Ok(runtime::managed_runtime_install_root()
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
async fn install_managed_runtime(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: runtime::RuntimeInstallRequest,
) -> Result<runtime::InstalledRuntime, String> {
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut active = state.runtime_install.lock().unwrap();
        if active.is_some() {
            return Err("A managed runtime installation is already active".into());
        }
        *active = Some(Arc::clone(&cancel));
    }
    let event_app = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        runtime::install_runtime(request, cancel, move |progress| {
            let _ = event_app.emit("runtime-install-progress", progress);
        })
    })
    .await;
    *state.runtime_install.lock().unwrap() = None;
    result.map_err(|error| error.to_string())?
}

#[tauri::command]
fn cancel_managed_runtime_install(state: tauri::State<'_, AppState>) -> bool {
    let active = state.runtime_install.lock().unwrap();
    if let Some(cancel) = active.as_ref() {
        cancel.store(true, Ordering::Relaxed);
        true
    } else {
        false
    }
}

#[tauri::command]
async fn check_managed_runtime_health(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: health::ManagedHealthRequest,
) -> Result<health::HealthRunResult, String> {
    let runtime_id = request.install_key.clone();
    let progress_runtime_id = request.install_key.clone();
    let adapter_id = request.adapter_id.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut active = state.runtime_health.lock().unwrap();
        if active.is_some() {
            return Err("A managed runtime health run is already active".into());
        }
        *active = Some(Arc::clone(&cancel));
    }
    let result = tauri::async_runtime::spawn_blocking(move || {
        let context = match runtime::managed_health_context(&request) {
            Ok(context) => context,
            Err(error) => {
                return health::trust_failure_result(runtime_id, adapter_id, error);
            }
        };
        let model_cancel = Arc::clone(&cancel);
        if let Err(error) =
            runtime::ensure_pinned_health_model(model_cancel, |downloaded, total| {
                let _ = app.emit(
                    "health-model-progress",
                    health::HealthModelProgress {
                        install_key: progress_runtime_id.clone(),
                        downloaded,
                        total,
                    },
                );
            })
        {
            let reason = if cancel.load(Ordering::Relaxed) {
                health::HealthFailureReason::Cancelled
            } else {
                health::HealthFailureReason::TrustFailure
            };
            return health::model_setup_failure_result(&context, reason, error);
        }
        health::run_managed_health(context, cancel.as_ref())
    })
    .await;
    *state.runtime_health.lock().unwrap() = None;
    result.map_err(|error| error.to_string())
}

#[tauri::command]
fn cancel_managed_runtime_health(state: tauri::State<'_, AppState>) -> bool {
    let active = state.runtime_health.lock().unwrap();
    if let Some(cancel) = active.as_ref() {
        cancel.store(true, Ordering::Relaxed);
        true
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Cloud credentials
// ---------------------------------------------------------------------------

#[tauri::command]
fn cloud_providers() -> Vec<cloud::Provider> {
    cloud::PROVIDERS.to_vec()
}

#[tauri::command]
fn cloud_credential_status(provider: String) -> Result<cloud::CredentialStatus, String> {
    cloud::credential_status(&cloud::KeyringStore, &provider)
}

#[tauri::command]
fn cloud_save_credential(
    provider: String,
    secret: String,
) -> Result<cloud::CredentialStatus, String> {
    cloud::save_credential(&cloud::KeyringStore, &provider, &secret)
}

#[tauri::command]
fn cloud_clear_credential(provider: String) -> Result<cloud::CredentialStatus, String> {
    cloud::clear_credential(&cloud::KeyringStore, &provider)
}

#[tauri::command]
async fn cloud_list_models(provider: String) -> Result<Vec<cloud::CloudModel>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        cloud::list_models(&cloud::KeyringStore, &provider)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn cloud_probe(provider: String, model: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        cloud::probe(&cloud::KeyringStore, &provider, &model)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Full OpenRouter PKCE sign-in: bind a loopback callback, open the browser,
/// wait for the code, exchange it, store the resulting key. The key never
/// crosses the IPC boundary to the frontend.
#[tauri::command]
async fn cloud_openrouter_login(app: tauri::AppHandle) -> Result<cloud::CredentialStatus, String> {
    use tauri_plugin_opener::OpenerExt;
    let (listener, callback_url) = cloud::bind_callback()?;
    let pkce = cloud::pkce_challenge();
    let url = cloud::openrouter_auth_url(&callback_url, &pkce.challenge);
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| format!("Could not open the browser: {error}"))?;
    tauri::async_runtime::spawn_blocking(move || {
        let code = cloud::wait_for_code(&listener, Duration::from_secs(300))?;
        let key = cloud::exchange_code_for_key(&code, &pkce.verifier)?;
        cloud::save_credential(&cloud::KeyringStore, "openrouter", &key)
    })
    .await
    .map_err(|error| error.to_string())?
}

// ---------------------------------------------------------------------------
// AI tuning
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TuningProgress {
    phase: String,
    message: String,
    trial: Option<tune::TuningTrial>,
}

struct LiveBench<'a> {
    app: &'a tauri::AppHandle,
    cancel: Arc<AtomicBool>,
    tokens: u32,
    repeats: u16,
}

impl tune::Bench for LiveBench<'_> {
    fn measure(&mut self, profile: &LaunchProfile) -> Result<(BenchmarkSummary, String), String> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let _ = self.app.emit(
            "tuning-progress",
            TuningProgress {
                phase: "launch".into(),
                message: "Starting llama-server with the candidate configuration…".into(),
                trial: None,
            },
        );
        // The lease binding pins the verified runtime content for the whole
        // tuning session (audit RT-04); it drops when the session ends.
        let (mut child, mut validation, log_path, _lease) = spawn_server(profile, "tuning")?;
        let command = validation.arguments.command.clone();
        let result = (|| {
            // The cancellable health wait converts Stop into a prompt failure
            // instead of holding the session for the full 600-second bound
            // (audit MT-04).
            wait_until_healthy_cancellable(
                &mut child,
                &local_client(profile)?,
                &log_path,
                Duration::from_secs(600),
                &self.cancel,
            )?;
            let observed_at_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .and_then(|duration| duration.as_millis().try_into().ok())
                .unwrap_or(validation.effective_context.observed_at_ms);
            update_launch_effective_context(
                &mut validation,
                &profile.host,
                profile.port,
                Duration::from_secs(5),
                observed_at_ms,
            );
            let _ = self.app.emit(
                "tuning-progress",
                TuningProgress {
                    phase: "measure".into(),
                    message: format!(
                        "Healthy. Measuring {} × {} tokens…",
                        self.repeats, self.tokens
                    ),
                    trial: None,
                },
            );
            if self.cancel.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            // The cancellable completion path replaces the legacy benchmark so
            // Stop cannot be ignored while a generation request is pending
            // (audit MT-04).
            let client = crate::local_client::LocalHttpClient::from_profile(profile)?;
            core::benchmark_server_cancellable(&client, self.tokens, self.repeats, &self.cancel)
        })();
        // Cleanup failures must be visible: a measured result may not be
        // reported as a clean success when the trial server could not be
        // stopped (audit MT-04).
        let cleanup = child.terminate_and_wait();
        // Give the OS a moment to release the port before the next launch.
        std::thread::sleep(Duration::from_millis(600));
        match (result, cleanup) {
            (Ok(summary), true) => Ok((summary, command)),
            (Ok(_), false) => Err(
                "The measured configuration succeeded, but its trial server could not be stopped cleanly; the result is withheld for the next session to re-check."
                    .into(),
            ),
            (Err(error), true) => Err(error),
            (Err(error), false) => {
                Err(format!("{error} (cleanup also failed: the trial server did not exit)"))
            }
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TuningRequest {
    profile: LaunchProfile,
    provider: String,
    model: String,
    target_context: u32,
    max_trials: u32,
    tokens: u32,
    repeats: u16,
    companions: Vec<String>,
}

#[tauri::command]
async fn start_tuning(
    app: tauri::AppHandle,
    request: TuningRequest,
    state: tauri::State<'_, AppState>,
) -> Result<tune::TuningReport, String> {
    {
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable")?;
        if status_from(&mut slot).running {
            return Err("Stop the running server before tuning; the tuner launches its own".into());
        }
    }
    // One machine owner: a benchmark, quality suite, or another server may
    // not be replaced silently by a tuning session (audit MT-05).
    let _reservation = reserve_operation(&state.operations, OperationOwner::Tuning)?;
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut tuning = state
            .tuning
            .lock()
            .map_err(|_| "Tuning state is unavailable")?;
        if tuning.is_some() {
            return Err("A tuning session is already running".into());
        }
        *tuning = Some(cancel.clone());
    }
    if !(1..=12).contains(&request.max_trials) {
        clear_tuning(&state);
        return Err("Trials must be between 1 and 12".into());
    }
    if request.target_context < 512 {
        clear_tuning(&state);
        return Err("Target context must be at least 512 tokens".into());
    }

    let handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let emit = |phase: &str, message: String| {
            let _ = handle.emit(
                "tuning-progress",
                TuningProgress {
                    phase: phase.into(),
                    message,
                    trial: None,
                },
            );
        };
        emit("prepare", "Reading hardware, GGUF metadata, and runtime capabilities…".into());
        let hardware = runtime::detect_hardware();
        let capabilities = core::inspect_runtime(Path::new(&request.profile.runtime))?;
        let gguf = gguf::read_summary(Path::new(&request.profile.model)).ok();
        let system_ram_bytes = system_ram_bytes();
        let inputs = tune::TuningInputs {
            objective: "Maximise measured generation tokens/second at the target context while the server starts and answers.",
            target_context: request.target_context,
            hardware: &hardware,
            system_ram_bytes,
            gguf: gguf.as_ref(),
            capabilities: &capabilities,
            companions: &request.companions,
            max_trials: request.max_trials,
            budgets: tune::TuningBudgets {
                // Independent of measured trials: no-op and duplicate replies
                // each consume one of these calls, so the session cannot stay
                // alive on talkative advisors (audit MT-03).
                max_advisor_calls: request.max_trials.saturating_mul(2).saturating_add(6),
                max_consecutive_rejections: 3,
                deadline: Some(
                    std::time::Instant::now()
                        + Duration::from_secs(tune::TUNING_DEADLINE_SECS),
                ),
            },
            cancel: Some(cancel.as_ref()),
        };
        let mut bench = LiveBench {
            app: &handle,
            cancel: cancel.clone(),
            tokens: request.tokens.clamp(64, 2048),
            repeats: request.repeats.clamp(1, 5),
        };
        let store = cloud::KeyringStore;
        let mut advisor = cloud::CloudAdvisor {
            store: &store,
            provider_id: request.provider.clone(),
            model: request.model.clone(),
            last_raw_reply: String::new(),
        };
        let progress_handle = handle.clone();
        tune::run_tuning(&request.profile, &inputs, &mut bench, &mut advisor, |trial| {
            let _ = progress_handle.emit(
                "tuning-progress",
                TuningProgress {
                    phase: "trial".into(),
                    message: match (&trial.mean_tps, &trial.error) {
                        (Some(tps), _) => format!("Trial {}: {tps:.2} tok/s", trial.index),
                        (None, Some(error)) => format!("Trial {} failed: {}", trial.index, error.lines().next().unwrap_or("")),
                        _ => format!("Trial {}", trial.index),
                    },
                    trial: Some(trial.clone()),
                },
            );
        })
    })
    .await
    .map_err(|error| error.to_string());
    clear_tuning(&state);
    result?
}

fn clear_tuning(state: &tauri::State<'_, AppState>) {
    if let Ok(mut tuning) = state.tuning.lock() {
        *tuning = None;
    }
}

#[tauri::command]
fn cancel_tuning(state: tauri::State<AppState>) -> Result<bool, String> {
    let tuning = state
        .tuning
        .lock()
        .map_err(|_| "Tuning state is unavailable")?;
    match tuning.as_ref() {
        Some(flag) => {
            flag.store(true, Ordering::Relaxed);
            Ok(true)
        }
        None => Ok(false),
    }
}

fn system_ram_bytes() -> Option<u64> {
    let output = proc::hidden_command("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

#[tauri::command]
fn suggest_port(host: String, preferred: u16) -> Result<u16, String> {
    core::pick_free_port(&host, preferred)
}

// ---------------------------------------------------------------------------
// HF catalog and downloads
// ---------------------------------------------------------------------------

/// Where the catalog cache and any in-flight download bookkeeping live.
/// The centralized local-server client for one validated profile (audit
/// MT-06): honors TLS/API-key configuration and applies bounded,
/// deadline-governed requests.
fn local_client(
    profile: &core::LaunchProfile,
) -> Result<crate::local_client::LocalHttpClient, String> {
    crate::local_client::LocalHttpClient::from_profile(profile)
}

fn catalog_cache_root(app: &tauri::AppHandle) -> std::path::PathBuf {
    use tauri::Manager;
    // The packaged verifier runs inside an isolated profile whose root the
    // process names explicitly (audit GH-05): Tauri's known-folder cache
    // path ignores a redirected LOCALAPPDATA, so the override keeps the
    // controlled matrix off the real user cache.
    if let Some(root) = catalog::verify_catalog_root() {
        return root;
    }
    app.path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("localmotive"))
}

/// Load the local catalog without any network request: the signature-verified
/// cache, else the bundled snapshot. Populates the authoritative in-memory
/// catalog so browsing, filtering, and download authorization survive a
/// restart inside the refresh cooldown and offline use (audit DC-01).
#[tauri::command]
async fn load_model_catalog(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<catalog::CatalogSnapshot, String> {
    let root = catalog_cache_root(&app);
    let snapshot = tauri::async_runtime::spawn_blocking(move || {
        catalog::load_catalog_snapshot(&root, &catalog::effective_catalog_url())
    })
    .await
    .map_err(|error| format!("Catalog load failed: {error}"))??;
    publish_loaded_catalog(&state.catalog, &snapshot);
    Ok(snapshot)
}

/// Publish a resolved local snapshot as the authoritative catalog state, so
/// browsing, facets, and download authorization work inside the refresh
/// cooldown and offline (audit DC-01).
fn publish_loaded_catalog(
    slot: &std::sync::Mutex<Option<catalog::Catalog>>,
    snapshot: &catalog::CatalogSnapshot,
) {
    *slot.lock().unwrap() = Some(snapshot.catalog.clone());
}

/// Fetch the catalog for the HF Catalog tab. Holds the in-flight guard for
/// the whole refresh so two Refresh clicks cannot start two network fetches.
/// Returns the remaining cooldown in minutes instead of hitting the network
/// when the last success is younger than CATALOG_REFRESH_COOLDOWN_MINUTES.
/// First start (no stamp) always fills.
#[tauri::command]
async fn fetch_model_catalog(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<catalog::CatalogSnapshot, String> {
    let root = catalog_cache_root(&app);
    let _in_flight = catalog::CatalogRefreshGuard::try_acquire().ok_or_else(|| {
        let stamp = catalog::read_refresh_stamp(&root);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        match catalog::refresh_cooldown_remaining_minutes(
            stamp.as_deref(),
            now,
            catalog::CATALOG_REFRESH_COOLDOWN_MINUTES,
        ) {
            Some(left) => {
                format!("A catalog refresh is already running. Try again in about {left} min.")
            }
            None => "A catalog refresh is already running. Try again shortly.".to_string(),
        }
    })?;
    let stamp = catalog::read_refresh_stamp(&root);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Some(left) = catalog::refresh_cooldown_remaining_minutes(
        stamp.as_deref(),
        now,
        catalog::CATALOG_REFRESH_COOLDOWN_MINUTES,
    ) {
        return Err(format!(
            "The catalog was refreshed recently. Try again in about {left} min."
        ));
    }
    let url = catalog::effective_catalog_url();
    let fetch_root = root.clone();
    let snapshot =
        tauri::async_runtime::spawn_blocking(move || catalog::fetch_catalog(&url, &fetch_root))
            .await
            .map_err(|error| format!("Catalog fetch failed: {error}"))??;
    let mut snapshot = snapshot;
    // Mirror verified rows locally for fast browse/filter/sort. The mirror
    // never authorizes anything: downloads still resolve against the signed
    // snapshot in state. A healthy database is updated transactionally,
    // preserving user rows; a confirmed migration or corruption failure goes
    // through controlled recovery that quarantines the old file and salvages
    // readable user rows, and every persistence failure stays visible
    // (audit DC-03).
    let mirror_models = snapshot.catalog.models.clone();
    let mirror_root = root.clone();
    let persistence = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let mut connection = catalog_db::open_catalog_db(&mirror_root)?;
        match catalog_db::migrate_catalog_db(&connection) {
            Ok(()) => catalog_db::mirror_verified_catalog(&mut connection, &mirror_models)
                .map(|()| String::new()),
            Err(migration_error) => {
                // Close the connection before the file is quarantined.
                drop(connection);
                catalog_db::recover_catalog_db_from_verified(
                    &mirror_root,
                    &catalog::Catalog {
                        schema_version: catalog::SUPPORTED_SCHEMA,
                        updated: String::new(),
                        source: String::new(),
                        note: String::new(),
                        models: mirror_models,
                    },
                    &migration_error,
                )
            }
        }
    })
    .await
    .unwrap_or_else(|error| Err(format!("Catalog persistence task failed: {error}")));
    match persistence {
        Ok(message) if !message.is_empty() => {
            snapshot.persistence_notice = Some(message);
        }
        Ok(_) => {}
        Err(error) => {
            snapshot.persistence_notice = Some(format!(
                "The verified catalog is available in memory, but local persistence failed: {error}"
            ));
        }
    }
    *state.catalog.lock().unwrap() = Some(snapshot.catalog.clone());
    Ok(snapshot)
}

/// Read the local SQLite mirror: verified rows plus marked user rows.
/// Falls back to the in-memory snapshot when the mirror is unavailable, so
/// the tab never goes empty because of a local database problem.
#[tauri::command]
fn catalog_local_models(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<catalog::CatalogModel>, String> {
    let root = catalog_cache_root(&app);
    let authoritative = state.catalog.lock().unwrap().clone();
    read_local_catalog_rows(&root, authoritative.as_ref())
}

/// Read the local SQLite mirror: verified rows plus marked user rows. Any
/// database-open, migration, or read failure selects the verified in-memory
/// or bundled rows instead, so the tab never goes empty because of a local
/// database problem (audit DC-07).
fn read_local_catalog_rows(
    root: &std::path::Path,
    authoritative: Option<&catalog::Catalog>,
) -> Result<Vec<catalog::CatalogModel>, String> {
    let connection = match catalog_db::open_catalog_db(root) {
        Ok(connection) => connection,
        Err(_) => return fallback_rows(authoritative),
    };
    if catalog_db::migrate_catalog_db(&connection).is_err() {
        return fallback_rows(authoritative);
    }
    match catalog_db::read_catalog_db_models(&connection) {
        Ok(models) if !models.is_empty() => Ok(models),
        _ => fallback_rows(authoritative),
    }
}

fn fallback_rows(
    authoritative: Option<&catalog::Catalog>,
) -> Result<Vec<catalog::CatalogModel>, String> {
    match authoritative {
        Some(catalog) => Ok(catalog.models.clone()),
        None => Ok(catalog::bundled_catalog()?.models),
    }
}

/// Save one user-added catalog row locally. The row is marked user-sourced,
/// never touches the signed artifact, and never passes signature checks.
#[tauri::command]
fn save_user_catalog_override(
    app: tauri::AppHandle,
    model: catalog::CatalogModel,
) -> Result<Vec<catalog::CatalogModel>, String> {
    let root = catalog_cache_root(&app);
    let mut connection = catalog_db::open_catalog_db(&root)?;
    catalog_db::migrate_catalog_db(&connection)?;
    catalog_db::save_user_catalog_override(&mut connection, &model)?;
    catalog_db::read_catalog_db_models(&connection)
}

/// Remove one user-added catalog row. Curated rows are never deleted here.
#[tauri::command]
fn remove_user_catalog_override(
    app: tauri::AppHandle,
    id: String,
) -> Result<Vec<catalog::CatalogModel>, String> {
    let root = catalog_cache_root(&app);
    let mut connection = catalog_db::open_catalog_db(&root)?;
    catalog_db::migrate_catalog_db(&connection)?;
    catalog_db::remove_user_catalog_override(&mut connection, &id)?;
    catalog_db::read_catalog_db_models(&connection)
}

#[tauri::command]
fn filter_catalog(
    models: Vec<catalog::CatalogModel>,
    query: catalog::CatalogQuery,
) -> Result<Vec<catalog::CatalogModel>, String> {
    catalog::validate_catalog_query(&query, models.len())?;
    Ok(catalog::filter_models(&models, &query))
}

#[tauri::command]
fn catalog_facets(
    models: Vec<catalog::CatalogModel>,
) -> Result<(Vec<String>, Vec<String>), String> {
    catalog::validate_facet_models(models.len())?;
    Ok(catalog::facets(&models))
}

#[tauri::command]
fn catalog_rich_facets(
    models: Vec<catalog::CatalogModel>,
) -> Result<catalog::CatalogFacets, String> {
    catalog::validate_facet_models(models.len())?;
    Ok(catalog::rich_facets(&models))
}

#[tauri::command]
fn catalog_fit_budget(
    dedicated_bytes: Vec<u64>,
    shared_bytes: Vec<u64>,
    system_bytes: u64,
) -> Result<catalog::FitBudget, String> {
    catalog::validate_budget_inputs(dedicated_bytes.len(), shared_bytes.len())?;
    let (budget, source) =
        catalog::hardware_fit_budget(&dedicated_bytes, &shared_bytes, system_bytes);
    Ok(catalog::FitBudget {
        budget_bytes: budget,
        source: source.into(),
    })
}

#[tauri::command]
fn hf_token_status() -> catalog::TokenStatus {
    catalog::hf_token_status()
}

#[tauri::command]
fn save_hf_token(token: String) -> Result<catalog::TokenStatus, String> {
    catalog::save_hf_token(&token)
}

#[tauri::command]
fn clear_hf_token() -> Result<catalog::TokenStatus, String> {
    catalog::clear_hf_token()
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadEvent {
    key: String,
    downloaded: u64,
    total: u64,
    bytes_per_second: u64,
    state: String,
    message: String,
    path: String,
}

fn download_target(
    destination: &str,
    filename: &str,
) -> Result<(std::path::PathBuf, String), String> {
    let root = std::path::Path::new(destination);
    if destination.trim().is_empty() || !root.is_absolute() {
        return Err("Choose an absolute model folder before downloading.".into());
    }
    for ancestor in root.ancestors() {
        let metadata = std::fs::symlink_metadata(ancestor)
            .map_err(|error| format!("Could not inspect {}: {error}", ancestor.display()))?;
        #[cfg(windows)]
        let is_reparse_point = {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0
        };
        #[cfg(not(windows))]
        let is_reparse_point = false;
        if metadata.file_type().is_symlink() || is_reparse_point {
            return Err(format!(
                "The selected model folder cannot pass through a symbolic link or reparse point: {}",
                ancestor.display()
            ));
        }
    }
    let metadata = std::fs::symlink_metadata(root)
        .map_err(|error| format!("Could not inspect {}: {error}", root.display()))?;
    #[cfg(windows)]
    let is_reparse_point = {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let is_reparse_point = false;
    if metadata.file_type().is_symlink() || is_reparse_point {
        return Err("The selected model folder cannot be a symbolic link or reparse point.".into());
    }
    let root = root
        .canonicalize()
        .map_err(|error| format!("Could not resolve {}: {error}", root.display()))?;
    let target = root.join(filename);
    if target
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err("The destination path is unsafe.".into());
    }
    let key = target
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    Ok((target, key))
}

/// One resolved download authorization, tagged with its authority.
struct AuthorizedDownload {
    file: catalog::CatalogFile,
    /// `curated` for signature-verified snapshot rows, `user` for validated
    /// local overrides carrying their own exact digest (audit DC-04).
    authority: &'static str,
}

/// Resolve the authorization for one download with an explicit
/// curated-versus-user split. Curated rows come only from the loaded
/// signature-verified snapshot or the bundled copy. A row missing there may
/// still match a validated local override record — marked `user_sourced` on
/// both the model and the file — whose exact stored SHA-256 authorizes the
/// transfer. No other mutable row can authorize anything (audit DC-04,
/// DC-07).
fn resolve_catalog_download(
    active: Option<&catalog::Catalog>,
    cache_root: &std::path::Path,
    repo: &str,
    filename: &str,
    revision: &str,
) -> Result<AuthorizedDownload, String> {
    let bundled;
    let catalog = match active {
        Some(catalog) => catalog,
        None => {
            bundled = catalog::bundled_catalog()?;
            &bundled
        }
    };
    if let Some(file) = catalog::catalog_file(catalog, repo, filename, revision).cloned() {
        return Ok(AuthorizedDownload {
            file,
            authority: "curated",
        });
    }
    let connection = catalog_db::open_catalog_db(cache_root).map_err(|error| {
        format!(
            "That repository, file, or revision is not in the validated catalog, and the local override store is unavailable: {error}"
        )
    })?;
    match catalog_db::user_override_file(&connection, repo, filename, revision)? {
        Some(file) => Ok(AuthorizedDownload {
            file,
            authority: "user",
        }),
        None => Err(
            "That repository, file, or revision is not in the validated catalog or in your local overrides."
                .to_string(),
        ),
    }
}

/// Start a download. Progress is emitted as `download:progress` events so a
/// multi-gigabyte transfer never blocks the interface.
#[tauri::command]
async fn download_catalog_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    repo: String,
    filename: String,
    revision: Option<String>,
    destination: String,
    connections: Option<usize>,
) -> Result<String, String> {
    use tauri::Emitter;

    catalog::validate_download_target(&repo, &filename)?;
    let revision = revision.unwrap_or_else(|| "main".into());
    let root = catalog_cache_root(&app);
    let authorized = {
        let active = state.catalog.lock().unwrap();
        resolve_catalog_download(active.as_ref(), &root, &repo, &filename, &revision)?
    };
    let expected_size = authorized.file.size_bytes;
    let expected_sha256 = authorized.file.sha256;
    // Which authority admitted this transfer stays visible to the user
    // (audit DC-04: provenance travels through the whole workflow).
    let completion_message = if authorized.authority == "user" {
        "Download complete: checksum verified against your local override digest."
    } else {
        "Download complete and checksum verified."
    };
    let (target, lock_key) = download_target(&destination, &filename)?;
    let event_key = format!("{repo}/{filename}");
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        let mut running = state.downloads.lock().unwrap();
        if running.contains_key(&lock_key) {
            return Err("That file is already downloading.".into());
        }
        running.insert(lock_key.clone(), std::sync::Arc::clone(&cancel));
    }

    let url = download::resolve_url(&repo, &filename, &revision);
    let token = catalog::hf_token();
    let connections = connections.unwrap_or(4);
    let downloaded = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let emit_key = event_key.clone();
    let emit_app = app.clone();
    let emit_target = target.to_string_lossy().to_string();

    let worker_cancel = std::sync::Arc::clone(&cancel);
    let worker_downloaded = std::sync::Arc::clone(&downloaded);
    let worker_repo = repo.clone();
    let task_result = tauri::async_runtime::spawn_blocking(move || {
        let mut baseline: Option<(u64, std::time::Instant)> = None;
        download::download_file(
            &url,
            &target,
            &worker_repo,
            expected_size,
            &expected_sha256,
            token.as_deref(),
            connections,
            worker_cancel,
            worker_downloaded,
            move |done, total| {
                let (start_bytes, started) =
                    baseline.get_or_insert_with(|| (done, std::time::Instant::now()));
                let elapsed = started.elapsed().as_secs_f64().max(0.001);
                let transferred = done.saturating_sub(*start_bytes);
                let _ = emit_app.emit(
                    "download:progress",
                    DownloadEvent {
                        key: emit_key.clone(),
                        downloaded: done,
                        total,
                        bytes_per_second: (transferred as f64 / elapsed) as u64,
                        state: if done >= total && total > 0 {
                            "verifying".into()
                        } else {
                            "downloading".into()
                        },
                        message: String::new(),
                        path: emit_target.clone(),
                    },
                );
            },
        )
    })
    .await;

    state.downloads.lock().unwrap().remove(&lock_key);
    let result = task_result.map_err(|error| format!("Download task failed: {error}"))?;

    match result {
        Ok(path) => {
            let path = path.to_string_lossy().to_string();
            let _ = app.emit(
                "download:progress",
                DownloadEvent {
                    key: event_key.clone(),
                    downloaded: downloaded.load(std::sync::atomic::Ordering::Relaxed),
                    total: expected_size,
                    bytes_per_second: 0,
                    state: "done".into(),
                    message: completion_message.into(),
                    path: path.clone(),
                },
            );
            Ok(path)
        }
        Err(error) => {
            let _ = app.emit(
                "download:progress",
                DownloadEvent {
                    key: event_key,
                    downloaded: downloaded.load(std::sync::atomic::Ordering::Relaxed),
                    total: expected_size,
                    bytes_per_second: 0,
                    state: "error".into(),
                    message: error.clone(),
                    path: String::new(),
                },
            );
            Err(error)
        }
    }
}

#[tauri::command]
fn cancel_download(
    state: tauri::State<'_, AppState>,
    destination: String,
    filename: String,
) -> bool {
    let Ok((_, lock_key)) = download_target(&destination, &filename) else {
        return false;
    };
    match state.downloads.lock().unwrap().get(&lock_key) {
        Some(flag) => {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
            true
        }
        None => false,
    }
}

#[tauri::command]
fn format_bytes(bytes: u64) -> String {
    download::human_bytes(bytes)
}

/// Seconds remaining at the current rate, or null when it cannot be known.
#[tauri::command]
fn download_eta(downloaded: u64, total: u64, bytes_per_second: u64) -> Option<u64> {
    download::eta_seconds(downloaded, total, bytes_per_second)
}

#[tauri::command]
fn about_info(app: tauri::AppHandle) -> AboutInfo {
    let package = app.package_info();
    AboutInfo {
        name: package.name.clone(),
        version: package.version.to_string(),
        tauri_version: tauri::VERSION.to_string(),
        identifier: app.config().identifier.clone(),
        os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        runtime_root: runtime::managed_runtime_install_root()
            .to_string_lossy()
            .to_string(),
        log_dir: std::env::temp_dir()
            .join("localmotive")
            .to_string_lossy()
            .to_string(),
        repository: env!("CARGO_PKG_REPOSITORY").to_string(),
        license: env!("CARGO_PKG_LICENSE").to_string(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AboutInfo {
    name: String,
    version: String,
    tauri_version: String,
    identifier: String,
    os: String,
    runtime_root: String,
    log_dir: String,
    repository: String,
    license: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Record verifier-only catalog source overrides before any catalog work
    // starts (audit GH-05); normal runs keep the shipped defaults.
    catalog::apply_env_verify_source();
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_models,
            inspect_runtime,
            check_runtime_health,
            describe_runtime,
            list_managed_runtimes,
            runtime_verification_stats,
            read_gguf_summary,
            cancel_gguf_read,
            inspect_model_artifact,
            preflight_model,
            preview_command,
            validate_launch_profile,
            start_server,
            stop_server,
            server_status,
            read_server_log,
            benchmark_server,
            benchmark_v2,
            cancel_benchmark,
            replay_benchmark_manifest,
            run_quality_suite,
            rank_candidates,
            build_compatibility_key,
            build_calibration_model,
            apply_calibration_model,
            store_calibration_anchor,
            store_calibration_model,
            load_calibration_records,
            import_external_evidence,
            review_external_evidence,
            build_share_export,
            write_share_export,
            load_runtime_setup,
            detect_hardware,
            fetch_runtime_catalog,
            managed_runtime_root,
            install_managed_runtime,
            cancel_managed_runtime_install,
            check_managed_runtime_health,
            cancel_managed_runtime_health,
            cloud_providers,
            cloud_credential_status,
            cloud_save_credential,
            cloud_clear_credential,
            cloud_list_models,
            cloud_probe,
            cloud_openrouter_login,
            start_tuning,
            cancel_tuning,
            suggest_port,
            about_info,
            load_model_catalog,
            fetch_model_catalog,
            catalog_local_models,
            save_user_catalog_override,
            remove_user_catalog_override,
            filter_catalog,
            catalog_facets,
            catalog_rich_facets,
            catalog_fit_budget,
            hf_token_status,
            save_hf_token,
            clear_hf_token,
            download_catalog_file,
            cancel_download,
            format_bytes,
            download_eta,
        ])
        .build(tauri::generate_context!())
        .expect("Localmotive failed to build its Tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Window close during a pending start or a running server:
                // signal the startup worker and stop the contained child.
                // Containment (a job object) guarantees the tree dies with
                // this process even if the orderly stop is interrupted
                // (audit IPC-01 I3).
                let state = app.state::<AppState>();
                if let Ok(starting) = state.starting.lock() {
                    if let Some(entry) = starting.as_ref() {
                        entry.cancel.store(true, Ordering::Relaxed);
                    }
                }
                let mut slot = match state.server.lock() {
                    Ok(slot) => slot,
                    Err(_) => return,
                };
                if let Some(server) = slot.as_mut() {
                    let _ = server.child.terminate_and_wait();
                }
                *slot = None;
            }
        });
}

#[cfg(test)]
mod release_security_tests {
    use super::*;

    #[test]
    fn inspect_runtime_command_verifies_managed_trust_before_probe() {
        // A forged runtime.json inside the managed root must not cause the
        // inspect command to execute attacker-controlled bytes.
        let source = include_str!("lib.rs");
        let command = source
            .split("fn inspect_runtime(path: String)")
            .nth(1)
            .unwrap()
            .split("struct RuntimeHealthRequest")
            .next()
            .unwrap();
        let trust_check = command
            .find("runtime::verify_managed_runtime_for_launch")
            .expect("inspect_runtime must enforce managed-runtime trust");
        let probe = command
            .find("core::inspect_runtime")
            .expect("inspect_runtime must call the runtime probe");
        assert!(
            trust_check < probe,
            "managed trust must be checked before execution"
        );
    }

    #[test]
    fn catalog_request_gate_permits_only_one_active_request() {
        let active = AtomicBool::new(false);
        let first = ExclusiveOperation::acquire(&active, "runtime catalog").unwrap();
        assert!(ExclusiveOperation::acquire(&active, "runtime catalog").is_err());
        drop(first);
        assert!(ExclusiveOperation::acquire(&active, "runtime catalog").is_ok());
    }

    fn put_test_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend((value.len() as u64).to_le_bytes());
        bytes.extend(value.as_bytes());
    }

    fn put_test_string_fact(bytes: &mut Vec<u8>, key: &str, value: &str) {
        put_test_string(bytes, key);
        bytes.extend(8_u32.to_le_bytes());
        put_test_string(bytes, value);
    }

    fn put_test_u32_fact(bytes: &mut Vec<u8>, key: &str, value: u32) {
        put_test_string(bytes, key);
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(value.to_le_bytes());
    }

    fn write_test_gguf_with_typed_identity(
        path: &Path,
        architecture: &str,
        extra_string_facts: &[(&str, &str)],
        extra_u32_facts: &[(&str, u32)],
    ) {
        let mut bytes = Vec::new();
        bytes.extend(b"GGUF");
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend(0_u64.to_le_bytes());
        bytes.extend(
            (6_u64 + extra_string_facts.len() as u64 + extra_u32_facts.len() as u64).to_le_bytes(),
        );
        put_test_string_fact(&mut bytes, "general.architecture", architecture);
        put_test_string_fact(&mut bytes, "general.name", "Fixture");
        put_test_u32_fact(&mut bytes, "general.file_type", 7);
        put_test_u32_fact(&mut bytes, &format!("{architecture}.block_count"), 2);
        put_test_u32_fact(&mut bytes, "split.no", 0);
        put_test_u32_fact(&mut bytes, "split.count", 1);
        for (key, value) in extra_string_facts {
            put_test_string_fact(&mut bytes, key, value);
        }
        for (key, value) in extra_u32_facts {
            put_test_u32_fact(&mut bytes, key, *value);
        }
        std::fs::write(path, bytes).unwrap();
    }

    fn write_test_gguf_with_identity(
        path: &Path,
        architecture: &str,
        extra_string_facts: &[(&str, &str)],
    ) {
        write_test_gguf_with_typed_identity(path, architecture, extra_string_facts, &[]);
    }

    fn write_test_gguf(path: &Path) {
        write_test_gguf_with_identity(path, "llama", &[]);
    }

    fn launch_validation_fixture() -> LaunchValidation {
        LaunchValidation {
            runtime: RuntimeCapabilities {
                path: "llama-server.exe".into(),
                version: "test".into(),
                build: "1".into(),
                commit: "abc".into(),
                help_sha256: "a".repeat(64),
                spec_types: Vec::new(),
                supported_flags: Vec::new(),
                metrics: false,
                multimodal: false,
                fit: true,
            },
            artifacts: Vec::new(),
            arguments: core::LaunchArgumentValidation {
                effective_args: Vec::new(),
                rejected: Vec::new(),
                command: "llama-server.exe".into(),
            },
            effective_context: unobserved_effective_context(42),
            unverified_requirements: Vec::new(),
        }
    }

    #[test]
    fn idle_server_status_does_not_claim_launch_validation() {
        let status = ServerStatus::default();

        assert_eq!(status.result_class, evidence::FitClass::Unknown);
        assert!(status.validation.is_none());
    }

    #[test]
    fn health_validation_accepts_only_an_http_success_status() {
        assert!(is_healthy_response(b"HTTP/1.1 200 OK\r\n\r\n"));
        assert!(!is_healthy_response(
            b"HTTP/1.1 503 Service Unavailable\r\n\r\n"
        ));
        assert!(!is_healthy_response(b"HTTP/1.1 2000 Invalid\r\n\r\n"));
    }

    #[cfg(windows)]
    #[test]
    fn listener_ownership_proves_the_process_that_bound_the_port() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();

        assert!(listener_is_owned_by_at(address, std::process::id()).unwrap());
        assert!(!listener_is_owned_by_at(address, std::process::id().wrapping_add(1)).unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn listener_ownership_is_bound_to_the_connected_address() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let other_address = SocketAddr::from(([127, 0, 0, 2], address.port()));

        assert!(listener_is_owned_by_at(address, std::process::id()).unwrap());
        assert!(!listener_is_owned_by_at(other_address, std::process::id()).unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn health_from_an_unrelated_process_cannot_validate_the_spawned_child() {
        use std::io::{Read, Write};
        use std::process::Stdio;

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1_024];
            let _ = stream.read(&mut request);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
        });
        let mut child = proc::hidden_command("ping.exe")
            .args(["127.0.0.1", "-n", "10"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let error = wait_until_healthy(
            &mut child,
            &crate::local_client::LocalHttpClient::plain("127.0.0.1", port).unwrap(),
            "",
            Duration::from_millis(500),
        )
        .unwrap_err();

        let _ = child.kill();
        let _ = child.wait();
        server.join().unwrap();
        assert!(error.contains("does not own"), "unexpected error: {error}");
    }

    #[test]
    fn server_props_parser_reads_the_effective_slot_context() {
        let body = b"{\"default_generation_settings\":{\"n_ctx\":4096}}";

        assert_eq!(parse_server_effective_context(200, body).unwrap(), 4_096);
        // A non-200 status is surfaced, never parsed as a body.
        assert!(parse_server_effective_context(500, body)
            .unwrap_err()
            .contains("HTTP 500"));
    }

    #[test]
    fn server_props_probe_records_effective_context_evidence() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            use std::io::{Read, Write};

            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1_024];
            let read = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..read]).starts_with("GET /props "));
            let body = r#"{"default_generation_settings":{"n_ctx":8192}}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });

        let context = observe_server_effective_context(
            &crate::local_client::LocalHttpClient::plain("127.0.0.1", port).unwrap(),
            Duration::from_secs(1),
            42,
        );

        assert_eq!(context.value, Some(8_192));
        assert_eq!(context.level, evidence::EvidenceLevel::Observed);
        assert_eq!(context.observed_at_ms, 42);
        server.join().unwrap();
    }

    #[test]
    fn post_health_probe_updates_launch_effective_context() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            use std::io::{Read, Write};

            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1_024];
            let _ = stream.read(&mut request).unwrap();
            let body = r#"{"default_generation_settings":{"n_ctx":16384}}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        let mut validation = launch_validation_fixture();

        update_launch_effective_context(
            &mut validation,
            "127.0.0.1",
            port,
            Duration::from_secs(1),
            42,
        );

        assert_eq!(validation.effective_context.value, Some(16_384));
        assert_eq!(
            validation.effective_context.level,
            evidence::EvidenceLevel::Observed
        );
        server.join().unwrap();
    }

    #[test]
    fn unavailable_server_props_produces_unknown_context_evidence() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let context = observe_server_effective_context(
            &crate::local_client::LocalHttpClient::plain("127.0.0.1", port).unwrap(),
            Duration::from_millis(100),
            42,
        );

        assert_eq!(context.value, None);
        assert_eq!(context.level, evidence::EvidenceLevel::Unknown);
        assert!(context.notes[0].contains("Could not observe"));
    }

    #[test]
    fn effective_context_is_unknown_before_runtime_health() {
        let context = unobserved_effective_context(42);

        assert_eq!(context.value, None);
        assert_eq!(context.level, evidence::EvidenceLevel::Unknown);
        assert_eq!(context.observed_at_ms, 42);
    }

    #[test]
    fn launch_validation_serializes_effective_context_evidence() {
        let validation = launch_validation_fixture();

        let serialized = serde_json::to_value(validation).unwrap();
        assert_eq!(
            serialized["effectiveContext"]["value"],
            serde_json::Value::Null
        );
        assert_eq!(serialized["effectiveContext"]["level"], "unknown");
    }

    #[test]
    fn server_props_response_read_is_bounded() {
        let mut response = std::io::Cursor::new(vec![b'x'; 2 * 1_048_576]);

        let error = read_props_response(&mut response).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(response.position() <= 1_048_576 + 4_096);
    }

    #[cfg(windows)]
    #[test]
    fn health_address_resolution_failure_is_structured() {
        let mut child = proc::hidden_command("cmd.exe")
            .args(["/C", "ping -n 5 127.0.0.1 >NUL"])
            .spawn()
            .expect("spawn test process");

        // A syntactically plain host that cannot resolve keeps the covered
        // behavior: the health wait fails through the structured
        // health_connect evidence path.
        let error = wait_until_healthy(
            &mut child,
            &crate::local_client::LocalHttpClient::plain("host-name-with-no-dns.invalid", 30_144)
                .unwrap(),
            "",
            Duration::from_millis(10),
        )
        .expect_err("invalid host should fail resolution");
        let _ = child.kill();
        let _ = child.wait();
        let evidence: LaunchFailureEvidence =
            serde_json::from_str(&error).expect("structured launch evidence");

        assert_eq!(evidence.phase, "health_connect");
        assert!(evidence.message.contains("resolve health endpoint"));
    }

    #[test]
    fn health_response_read_is_bounded() {
        let mut response = std::io::Cursor::new(vec![b'x'; 32 * 1024]);

        let error = read_health_response(&mut response).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(response.position() <= (16 * 1024 + 1));
    }

    #[test]
    fn health_endpoint_resolution_supports_ipv6_loopback() {
        let addresses = health_socket_addresses("::1", 8080).unwrap();

        assert!(addresses.iter().any(SocketAddr::is_ipv6));
    }

    #[cfg(windows)]
    #[test]
    fn health_timeout_preserves_a_bounded_log_tail() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let log_path = std::env::temp_dir().join(format!(
            "localmotive-health-timeout-{}.log",
            std::process::id()
        ));
        std::fs::write(&log_path, "initial line\nuseful timeout detail\n").unwrap();
        let mut child = crate::proc::hidden_command("cmd")
            .args(["/C", "ping", "-n", "6", "127.0.0.1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let error = wait_until_healthy(
            &mut child,
            &crate::local_client::LocalHttpClient::plain("127.0.0.1", port).unwrap(),
            log_path.to_string_lossy().as_ref(),
            Duration::from_millis(1),
        )
        .unwrap_err();

        let _ = child.kill();
        let _ = child.wait();
        std::fs::remove_file(log_path).unwrap();
        let evidence: LaunchFailureEvidence =
            serde_json::from_str(&error).expect("launch failure must be structured JSON");
        assert_eq!(evidence.phase, "health_timeout");
        assert!(evidence.timed_out);
        assert_eq!(evidence.exit_code, None);
        assert!(evidence.log_tail.contains("useful timeout detail"));
    }

    #[cfg(windows)]
    #[test]
    fn cancelled_health_wait_returns_before_the_launch_timeout() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let log_path = std::env::temp_dir().join(format!(
            "localmotive-health-cancelled-{}.log",
            std::process::id()
        ));
        std::fs::write(&log_path, "cancelled health detail\n").unwrap();
        let mut child = proc::hidden_command("ping.exe")
            .args(["127.0.0.1", "-n", "30"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let cancelled = AtomicBool::new(true);
        let started = Instant::now();

        let error = wait_until_healthy_cancellable(
            &mut child,
            &crate::local_client::LocalHttpClient::plain("127.0.0.1", port).unwrap(),
            &log_path.to_string_lossy(),
            Duration::from_secs(120),
            &cancelled,
        )
        .unwrap_err();

        assert!(started.elapsed() < Duration::from_secs(2));
        let _ = child.kill();
        let _ = child.wait();
        std::fs::remove_file(log_path).unwrap();
        let evidence: LaunchFailureEvidence =
            serde_json::from_str(&error).expect("launch failure must be structured JSON");
        assert_eq!(evidence.phase, "health_cancelled");
        assert!(!evidence.timed_out);
        assert!(evidence.cancelled);
    }

    #[cfg(windows)]
    #[test]
    fn health_exit_preserves_structured_exit_evidence() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let log_path = std::env::temp_dir().join(format!(
            "localmotive-health-exit-{}.log",
            std::process::id()
        ));
        std::fs::write(&log_path, "useful exit detail\n").unwrap();
        let mut child = crate::proc::hidden_command("cmd")
            .args(["/C", "exit", "7"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let error = wait_until_healthy(
            &mut child,
            &crate::local_client::LocalHttpClient::plain("127.0.0.1", port).unwrap(),
            log_path.to_string_lossy().as_ref(),
            Duration::from_secs(2),
        )
        .unwrap_err();

        let _ = child.wait();
        std::fs::remove_file(log_path).unwrap();
        let evidence: LaunchFailureEvidence =
            serde_json::from_str(&error).expect("launch failure must be structured JSON");
        assert_eq!(evidence.phase, "health_exit");
        assert_eq!(evidence.exit_code, Some(7));
        assert!(!evidence.timed_out);
        assert!(evidence.log_tail.contains("useful exit detail"));
    }

    #[test]
    fn post_health_exit_evidence_preserves_status_with_a_byte_bounded_tail() {
        let log_path = std::env::temp_dir().join(format!(
            "localmotive-runtime-exit-{}.log",
            std::process::id()
        ));
        let mut log = vec![b'x'; 32 * 1024];
        log.extend_from_slice(b"\nuseful runtime exit detail\n");
        std::fs::write(&log_path, log).unwrap();

        let evidence = launch_failure_evidence(
            "runtime_exit",
            "llama-server exited after health validation".into(),
            log_path.to_string_lossy().as_ref(),
            Some(7),
            false,
        );

        std::fs::remove_file(log_path).unwrap();
        assert_eq!(evidence.exit_code, Some(7));
        assert!(evidence.log_tail.ends_with("useful runtime exit detail"));
        assert!(evidence.log_tail.len() <= 16 * 1024);
    }

    #[test]
    fn spawn_does_not_claim_port_availability_from_a_released_probe() {
        let source = include_str!("lib.rs");
        let spawn_start = source.find("fn spawn_server(").unwrap();
        let spawn_end = source[spawn_start..]
            .find("fn connect_host(")
            .map(|offset| spawn_start + offset)
            .unwrap();
        let spawn_source = &source[spawn_start..spawn_end];

        assert!(!spawn_source.contains("TcpListener::bind"));
        assert!(spawn_source.contains("proc::spawn_contained_process"));
        assert!(!spawn_source.contains(".spawn()"));
    }

    #[test]
    fn spawn_failures_use_structured_launch_evidence() {
        let source = include_str!("lib.rs");
        let spawn_source = source
            .split_once("fn spawn_server(")
            .and_then(|(_, rest)| rest.split_once("fn connect_host(").map(|(body, _)| body))
            .unwrap();

        assert!(spawn_source.contains("launch_failure(\"validation\""));
        assert!(spawn_source.contains("\"log_setup\""));
        assert!(spawn_source.contains("\"spawn\""));
        assert!(spawn_source.matches("launch_failure(").count() >= 3);
    }

    #[test]
    fn spawn_holds_the_execution_lease_across_process_creation() {
        let source = include_str!("lib.rs");
        let body = source
            .split_once("fn spawn_server(")
            .and_then(|(_, rest)| rest.split_once("fn connect_host(").map(|(body, _)| body))
            .unwrap();

        // The managed execution lease must be acquired after verification and
        // before the process starts, so the approved content cannot change
        // between the trust check and image/DLL loading (audit RT-04).
        let lease = body
            .find("authorize_managed_execution_lease")
            .expect("spawn_server must acquire the managed execution lease");
        let spawn = body
            .find("spawn_contained_process")
            .expect("spawn_server must use the contained process runner");
        assert!(
            lease < spawn,
            "the execution lease must be held before the process starts"
        );
        assert!(
            body.contains("execution_lease"),
            "spawn_server must return the lease to its caller"
        );
    }

    #[test]
    fn every_measurement_entry_point_requires_a_validated_server_snapshot() {
        let source = include_str!("lib.rs");
        for (start, end) in [
            ("fn benchmark_server(", "struct BenchmarkRunResult"),
            ("async fn benchmark_v2(", "fn cancel_benchmark("),
            (
                "fn replay_benchmark_manifest(",
                "async fn run_quality_suite(",
            ),
            ("async fn run_quality_suite(", "fn rank_candidates("),
        ] {
            let body = source
                .split_once(start)
                .and_then(|(_, rest)| rest.split_once(end).map(|(body, _)| body))
                .unwrap();
            assert!(
                body.contains("validated_server_snapshot"),
                "{start} bypasses the shared validated-server gate"
            );
        }
    }

    #[test]
    fn cold_benchmark_uses_a_fresh_runtime_for_each_attempt() {
        let source = include_str!("lib.rs");
        let body = source
            .split_once("fn run_benchmark_snapshot(")
            .and_then(|(_, rest)| {
                rest.split_once("async fn benchmark_v2(")
                    .map(|(body, _)| body)
            })
            .unwrap();

        assert!(body.contains("CacheMode::Cold"));
        assert!(body.contains("run_cold_workload_with"));
        assert!(body.contains("spawn_server("));
    }

    #[test]
    fn benchmark_memory_samples_the_process_that_served_each_request() {
        let source = include_str!("lib.rs");
        let start = source.find("fn run_benchmark_snapshot(").unwrap();
        let end = source[start..]
            .find("async fn benchmark_v2(")
            .map(|offset| start + offset)
            .unwrap();
        let body = &source[start..end];

        assert!(body.contains("process_peak_working_set(server_pid)"));
        assert!(body.contains("process_peak_working_set(child.id())"));
    }

    #[test]
    fn multi_gpu_path_requires_explicit_adapter_selection() {
        let profile = LaunchProfile {
            tensor_split: "1,1".into(),
            ..LaunchProfile::default()
        };

        assert_eq!(
            execution_path_for(&profile, &["gpu-a".into(), "gpu-b".into()]),
            evidence::ExecutionPath::MultiGpu
        );
        assert_eq!(
            execution_path_for(&profile, &[]),
            evidence::ExecutionPath::Unknown
        );
    }

    #[test]
    fn automatic_gpu_placement_remains_unknown_before_launch() {
        let profile = LaunchProfile {
            gpu_layers: "auto".into(),
            ..LaunchProfile::default()
        };

        let path = execution_path_for(&profile, &["gpu-0".into()]);

        assert_eq!(path, evidence::ExecutionPath::Unknown);
    }

    #[test]
    fn a_download_destination_must_already_exist() {
        // Folder creation before reparse-point validation could change an
        // attacker-controlled location. The picker supplies an existing folder.
        let root = std::env::temp_dir().join(format!(
            "localmotive-missing-download-root-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let result = download_target(root.to_string_lossy().as_ref(), "model.gguf");
        assert!(result.is_err());
        assert!(!root.exists(), "validation created the missing destination");
    }

    #[test]
    fn launch_validation_rejects_an_invalid_model_artifact() {
        let path = std::env::temp_dir().join(format!(
            "localmotive-invalid-launch-artifact-{}.gguf",
            std::process::id()
        ));
        std::fs::write(&path, b"not a GGUF file").unwrap();

        let error = require_launchable_artifact("Model", &path).unwrap_err();

        assert!(error.contains("cannot be launched"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn profile_artifact_validation_uses_the_main_model_gate() {
        let path = std::env::temp_dir().join(format!(
            "localmotive-invalid-profile-artifact-{}.gguf",
            std::process::id()
        ));
        std::fs::write(&path, b"not a GGUF file").unwrap();
        let profile = LaunchProfile {
            model: path.to_string_lossy().to_string(),
            ..LaunchProfile::default()
        };

        let error = validate_profile_artifacts(&profile).unwrap_err();

        assert!(error.contains("Model cannot be launched"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn shared_launch_path_validation_checks_lora_files() {
        let root =
            std::env::temp_dir().join(format!("localmotive-missing-lora-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let model = root.join("fixture.gguf");
        write_test_gguf(&model);
        let profile = LaunchProfile {
            runtime: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            model: model.to_string_lossy().to_string(),
            lora: root.join("missing-lora.gguf").to_string_lossy().to_string(),
            ..LaunchProfile::default()
        };

        let error = validate_profile_paths(&profile).unwrap_err();

        assert!(error.contains("LoRA"), "unexpected error: {error}");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launch_artifact_validation_includes_lora_identity_and_bytes() {
        let root =
            std::env::temp_dir().join(format!("localmotive-lora-artifact-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let model = root.join("model.gguf");
        let lora = root.join("adapter.gguf");
        write_test_gguf(&model);
        write_test_gguf_with_identity(
            &lora,
            "llama",
            &[("general.type", "adapter"), ("adapter.type", "lora")],
        );
        let profile = LaunchProfile {
            model: model.to_string_lossy().to_string(),
            lora: lora.to_string_lossy().to_string(),
            ..LaunchProfile::default()
        };

        let artifacts = validate_profile_artifacts(&profile).unwrap();

        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[1].first_shard, lora.to_string_lossy());
        assert!(artifacts[1].shard_bytes > 0);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launch_artifact_validation_rejects_a_model_in_the_lora_slot() {
        let root =
            std::env::temp_dir().join(format!("localmotive-lora-role-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let model = root.join("model.gguf");
        let unrelated = root.join("not-an-adapter.gguf");
        write_test_gguf(&model);
        write_test_gguf(&unrelated);
        let profile = LaunchProfile {
            model: model.to_string_lossy().into_owned(),
            lora: unrelated.to_string_lossy().into_owned(),
            ..LaunchProfile::default()
        };

        let error = validate_profile_artifacts(&profile).unwrap_err();

        assert!(error.contains("LoRA"), "unexpected error: {error}");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launch_artifact_validation_rejects_a_text_model_as_projector() {
        let root =
            std::env::temp_dir().join(format!("localmotive-projector-role-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let model = root.join("model.gguf");
        let unrelated = root.join("mmproj-unrelated.gguf");
        write_test_gguf(&model);
        write_test_gguf(&unrelated);
        let profile = LaunchProfile {
            model: model.to_string_lossy().into_owned(),
            mmproj: Some(unrelated.to_string_lossy().into_owned()),
            ..LaunchProfile::default()
        };

        let error = validate_profile_artifacts(&profile).unwrap_err();

        assert!(error.contains("projector"), "unexpected error: {error}");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launch_artifact_validation_accepts_a_clip_projector_role() {
        let root = std::env::temp_dir().join(format!(
            "localmotive-projector-valid-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let model = root.join("model.gguf");
        let projector = root.join("mmproj.gguf");
        write_test_gguf(&model);
        write_test_gguf_with_identity(&projector, "clip", &[]);
        let profile = LaunchProfile {
            model: model.to_string_lossy().into_owned(),
            mmproj: Some(projector.to_string_lossy().into_owned()),
            ..LaunchProfile::default()
        };

        let artifacts = validate_profile_artifacts(&profile).unwrap();

        assert_eq!(artifacts.len(), 2);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launch_artifact_validation_rejects_a_projector_embedding_mismatch() {
        let root = std::env::temp_dir().join(format!(
            "localmotive-projector-dimension-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let model = root.join("model.gguf");
        let projector = root.join("mmproj.gguf");
        write_test_gguf_with_typed_identity(
            &model,
            "llama",
            &[],
            &[("llama.embedding_length", 4_096)],
        );
        write_test_gguf_with_typed_identity(
            &projector,
            "clip",
            &[],
            &[("clip.vision.projection_dim", 2_048)],
        );
        let profile = LaunchProfile {
            model: model.to_string_lossy().into_owned(),
            mmproj: Some(projector.to_string_lossy().into_owned()),
            ..LaunchProfile::default()
        };

        let error = validate_profile_artifacts(&profile).unwrap_err();

        assert!(error.contains("embedding"), "unexpected error: {error}");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn companion_requirements_are_visible_without_raw_paths() {
        let profile = LaunchProfile {
            draft_model: Some(r"C:\private\draft.gguf".into()),
            mmproj: Some(r"C:\private\projector.gguf".into()),
            lora: r"C:\private\adapter.gguf".into(),
            ..LaunchProfile::default()
        };

        let requirements = unverified_companion_requirements(&profile);

        assert_eq!(requirements.len(), 3);
        assert!(!requirements.join(" ").contains("C:\\private"));
    }

    #[test]
    fn launch_artifact_validation_rejects_conflicting_draft_tokenizer_identity() {
        let root =
            std::env::temp_dir().join(format!("localmotive-draft-identity-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let model = root.join("model.gguf");
        let draft = root.join("draft.gguf");
        write_test_gguf_with_identity(&model, "llama", &[("tokenizer.ggml.model", "bpe")]);
        write_test_gguf_with_identity(
            &draft,
            "llama",
            &[("tokenizer.ggml.model", "sentencepiece")],
        );
        let profile = LaunchProfile {
            model: model.to_string_lossy().into_owned(),
            draft_model: Some(draft.to_string_lossy().into_owned()),
            ..LaunchProfile::default()
        };

        let error = validate_profile_artifacts(&profile).unwrap_err();

        assert!(error.contains("tokenizer"), "unexpected error: {error}");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shared_launch_path_validation_rejects_malformed_scaled_lora_entries() {
        let profile = LaunchProfile {
            runtime: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            model: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            lora_scaled: "missing-scale.gguf:not-a-number".into(),
            ..LaunchProfile::default()
        };

        let error = validate_profile_paths(&profile).unwrap_err();

        assert!(error.contains("scale"), "unexpected error: {error}");
    }

    #[cfg(windows)]
    #[test]
    fn scaled_lora_validation_preserves_the_windows_drive_prefix() {
        let root =
            std::env::temp_dir().join(format!("localmotive-scaled-lora-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let lora = root.join("adapter with spaces.gguf");
        std::fs::write(&lora, b"fixture").unwrap();
        let profile = LaunchProfile {
            runtime: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            model: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            lora_scaled: format!("{}:0.5", lora.display()),
            ..LaunchProfile::default()
        };

        validate_profile_paths(&profile).unwrap();

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shared_launch_validation_rejects_an_invalid_artifact_before_spawn() {
        let path = std::env::temp_dir().join(format!(
            "localmotive-invalid-shared-{}.gguf",
            std::process::id()
        ));
        std::fs::write(&path, b"not a GGUF").unwrap();
        let profile = LaunchProfile {
            alias: "fixture".into(),
            runtime: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            model: path.to_string_lossy().to_string(),
            ..LaunchProfile::default()
        };

        let error = prepare_launch(&profile).unwrap_err();

        assert!(error.contains("incomplete or inconsistent"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn preflight_command_rejects_an_invalid_artifact() {
        let path = std::env::temp_dir().join(format!(
            "localmotive-invalid-preflight-{}.gguf",
            std::process::id()
        ));
        std::fs::write(&path, b"not a GGUF").unwrap();
        let profile = LaunchProfile {
            alias: "fixture".into(),
            runtime: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            model: path.to_string_lossy().to_string(),
            ..LaunchProfile::default()
        };

        let result = tauri::async_runtime::block_on(preflight_model(PreflightRequest {
            profile,
            selected_adapter_ids: Vec::new(),
            manual_overrides: Vec::new(),
            reserve_bytes: Some(536_870_912),
        }));

        assert!(result.unwrap_err().contains("incomplete or inconsistent"));
        let _ = std::fs::remove_file(path);
    }

    #[cfg(windows)]
    #[test]
    fn executable_validation_rejects_a_parent_directory_reparse_point() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("localmotive-launch-root-{nonce}"));
        let outside = std::env::temp_dir().join(format!("localmotive-launch-outside-{nonce}"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("llama-server.exe"), b"outside").unwrap();
        let junction = root.join("runtime");
        let status = crate::proc::hidden_command("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&outside)
            .status()
            .unwrap();
        assert!(status.success());

        let error = require_regular_non_reparse_file("Runtime", &junction.join("llama-server.exe"))
            .unwrap_err();

        assert!(error.contains("reparse-point"), "unexpected error: {error}");
        std::fs::remove_dir(junction).unwrap();
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(outside).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn shared_launch_preparation_rejects_a_runtime_parent_reparse_before_execution() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("localmotive-prepare-root-{nonce}"));
        std::fs::create_dir_all(&root).unwrap();
        let model = root.join("fixture.gguf");
        write_test_gguf(&model);
        let current_exe = std::env::current_exe().unwrap();
        let runtime_parent = current_exe.parent().unwrap();
        let junction = root.join("runtime");
        let status = crate::proc::hidden_command("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(runtime_parent)
            .status()
            .unwrap();
        assert!(status.success());
        let profile = LaunchProfile {
            alias: "fixture".into(),
            runtime: junction
                .join(current_exe.file_name().unwrap())
                .to_string_lossy()
                .to_string(),
            model: model.to_string_lossy().to_string(),
            ..LaunchProfile::default()
        };

        let error = prepare_launch(&profile).unwrap_err();

        assert!(error.contains("reparse-point"), "unexpected error: {error}");
        std::fs::remove_dir(junction).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod ipc01_startup_tests {
    use super::*;

    #[test]
    fn ipc01_startup_commit_requires_the_operation_to_still_own_the_slot() {
        // A late completion from an older operation can never publish
        // (audit IPC-01 I3): mismatched IDs, superseded reservations, or a
        // cancelled start must all be refused.
        let state = AppState::default();
        let cancel = AtomicBool::new(false);
        assert!(
            !startup_is_current(&state, 1, &cancel),
            "no reservation must not publish"
        );
        let reservation = reserve_operation(&state.operations, OperationOwner::Server).unwrap();
        let operation_id = reservation.generation;
        assert!(
            !startup_is_current(&state, operation_id, &cancel),
            "no starting slot must not publish"
        );
        *state.starting.lock().unwrap() = Some(StartingServer {
            operation_id,
            cancel: Arc::new(AtomicBool::new(false)),
        });
        assert!(startup_is_current(&state, operation_id, &cancel));
        assert!(
            !startup_is_current(&state, operation_id + 1, &cancel),
            "a newer operation's ID must not be satisfied by the old worker"
        );
        cancel.store(true, Ordering::Relaxed);
        assert!(
            !startup_is_current(&state, operation_id, &cancel),
            "a cancelled start must never publish"
        );
    }

    #[test]
    fn ipc01_startup_never_holds_the_server_lock_across_the_readiness_wait() {
        // The audited defect: the server mutex was held for the whole 600 s
        // health wait, freezing every other command (audit IPC-01). The
        // worker must reach the cancellable wait before any server lock,
        // and the commit must check ownership before publishing.
        let source = include_str!("lib.rs");
        let worker = source
            .split("fn start_server_worker(")
            .nth(1)
            .unwrap()
            .split("#[tauri::command]")
            .next()
            .unwrap();
        let wait = worker
            .find("wait_until_healthy_cancellable")
            .expect("the worker must wait cancellably");
        assert_eq!(
            worker.matches(".server").count(),
            1,
            "the worker must touch the server slot exactly once, at commit"
        );
        let lock = worker
            .find(".server")
            .expect("the worker must lock the server slot at commit");
        assert!(
            lock > wait,
            "the server lock must not be taken before the readiness wait"
        );
        assert!(
            worker.contains("let owns = startup_is_current("),
            "the commit must check operation ownership"
        );
        assert!(
            worker.contains("child.terminate_and_wait();"),
            "a refused commit must reap the process tree"
        );
        let command = source
            .split("async fn start_server(")
            .nth(1)
            .unwrap()
            .split("fn start_server_worker(")
            .next()
            .unwrap();
        assert!(
            command.contains("spawn_blocking"),
            "start_server must run the launch on a blocking worker"
        );
        assert!(
            command.contains("OperationOwner::Server"),
            "start_server must reserve the server owner"
        );
    }

    #[test]
    fn ipc01_expensive_commands_run_on_blocking_workers() {
        // Every command the audit named as expensive must run its blocking
        // half on a worker, never on the Tauri main thread or the async
        // executor (audit IPC-01 I4).
        let source = include_str!("lib.rs");
        for (start, end) in [
            (
                "async fn scan_models(root: String)",
                "async fn inspect_runtime(path: String)",
            ),
            (
                "async fn inspect_runtime(path: String)",
                "async fn check_runtime_health(",
            ),
            ("async fn check_runtime_health(", "fn describe_runtime"),
            (
                "async fn preflight_model(request: PreflightRequest)",
                "fn preview_command",
            ),
            ("async fn benchmark_server(", "struct BenchmarkRunResult"),
            (
                "async fn replay_benchmark_manifest(",
                "fn replay_benchmark_manifest_worker",
            ),
            ("async fn read_gguf_summary(", "fn cancel_gguf_read"),
        ] {
            let body = source
                .split(start)
                .nth(1)
                .and_then(|rest| rest.split(end).next())
                .unwrap_or_else(|| panic!("{start} not found"));
            assert!(
                body.contains("spawn_blocking"),
                "{start} must run its expensive half on a blocking worker"
            );
        }
    }
}

#[cfg(test)]
mod catalog_command_tests {
    use super::*;

    pub(crate) fn unique_dir(prefix: &str) -> std::path::PathBuf {
        for _ in 0..16 {
            let candidate = std::env::temp_dir().join(format!(
                "{prefix}-{}-{:016x}",
                std::process::id(),
                rand::random::<u64>()
            ));
            if std::fs::create_dir_all(&candidate).is_ok() {
                return candidate;
            }
        }
        panic!("could not create a unique test directory");
    }

    fn shipped_signed_pair() -> (String, String) {
        let catalog_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("catalog");
        (
            std::fs::read_to_string(catalog_dir.join("catalog.json")).unwrap(),
            std::fs::read_to_string(catalog_dir.join("catalog.json.sig")).unwrap(),
        )
    }

    #[test]
    fn dc01_local_load_publishes_state_inside_the_cooldown_and_authorizes_downloads() {
        // A restart with a new app state, a signed cache, and a fresh stamp
        // must browse and authorize downloads without any network request
        // (audit DC-01 I2/V1/V2). This path contains no HTTP client, so no
        // network request is structurally possible.
        let root = unique_dir("localmotive-lib-load");
        let (body, signature) = shipped_signed_pair();
        catalog::save_cache_record(&root, &body, Some("etag"), &signature).unwrap();
        catalog::write_refresh_stamp(&root);

        let slot = std::sync::Mutex::new(None);
        let snapshot = catalog::load_catalog_snapshot(&root, catalog::DEFAULT_CATALOG_URL).unwrap();
        publish_loaded_catalog(&slot, &snapshot);

        assert_eq!(snapshot.origin, "cache");
        assert!(!snapshot.catalog.models.is_empty());
        assert!(
            snapshot.cooldown_remaining_minutes.is_some(),
            "a fresh stamp must report the remaining cooldown"
        );
        assert!(snapshot.refresh_error.is_none());

        // The published state authorizes a download of one of its own rows...
        let active = slot.lock().unwrap();
        let state_catalog = active.as_ref().expect("local load must publish state");
        let (model, file) = state_catalog
            .models
            .iter()
            .find_map(|model| model.files.first().map(|file| (model, file)))
            .expect("the shipped catalog must carry at least one file");
        let authorized = resolve_catalog_download(
            active.as_ref(),
            &root,
            &model.repo,
            &file.filename,
            &file.revision,
        )
        .expect("a published row must authorize its own file");
        assert_eq!(authorized.file.sha256, file.sha256);
        assert_eq!(authorized.authority, "curated");
        // ...and nothing else: an unlisted file stays unauthorized.
        assert!(resolve_catalog_download(
            active.as_ref(),
            &root,
            &model.repo,
            "not-listed.gguf",
            "main"
        )
        .is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dc01_local_load_without_a_cache_publishes_the_bundled_catalog() {
        // First start with no cache: the bundled snapshot is authoritative
        // immediately, so facets and authorization work before any refresh.
        let root = unique_dir("localmotive-lib-bundled");
        let slot = std::sync::Mutex::new(None);
        let snapshot = catalog::load_catalog_snapshot(&root, catalog::DEFAULT_CATALOG_URL).unwrap();
        publish_loaded_catalog(&slot, &snapshot);

        assert_eq!(snapshot.origin, "bundled");
        let active = slot.lock().unwrap();
        assert!(active
            .as_ref()
            .is_some_and(|catalog| !catalog.models.is_empty()));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dc07_database_open_and_migration_failures_select_verified_rows() {
        // A mirror that cannot be opened or migrated must not empty the tab
        // or fail the command: verified memory wins, else the bundled rows
        // (audit DC-07 I4/V3).
        let root = unique_dir("localmotive-lib-db");

        // (a) The database path is a directory: open fails.
        std::fs::create_dir_all(catalog_db::catalog_db_path(&root)).unwrap();
        let rows = read_local_catalog_rows(&root, None).expect("open failure must fall back");
        assert!(!rows.is_empty());
        std::fs::remove_dir_all(catalog_db::catalog_db_path(&root)).unwrap();

        // (b) Garbage bytes: open succeeds, migration fails.
        std::fs::write(catalog_db::catalog_db_path(&root), "not a database").unwrap();
        let authorized = crate::catalog::bundled_catalog().unwrap();
        let rows = read_local_catalog_rows(&root, Some(&authorized))
            .expect("migration failure must fall back");
        assert_eq!(rows.len(), authorized.models.len());
        let _ = std::fs::remove_dir_all(root);
    }
}

#[cfg(test)]
mod catalog_persistence_source_tests {
    #[test]
    fn dc03_fetch_mirrors_healthy_databases_and_only_recovers_after_migration_failure() {
        // Source guard for the audited inverted branch: a healthy database
        // must be updated transactionally through mirror_verified_catalog,
        // and only a confirmed migration failure may enter quarantine-based
        // recovery (audit DC-03). Swapping these branches must fail here.
        let source = include_str!("lib.rs");
        let block = source
            .split("let persistence = tauri::async_runtime::spawn_blocking")
            .nth(1)
            .expect("fetch_model_catalog must run its persistence work in the shared block")
            .split("match persistence {")
            .next()
            .unwrap();
        let healthy = block.split("Err(migration_error) =>").next().unwrap();
        assert!(
            healthy.contains("Ok(()) =>")
                && healthy.contains("catalog_db::mirror_verified_catalog"),
            "the healthy migration branch must mirror transactionally: {healthy}"
        );
        assert!(
            !healthy.contains("recover_catalog_db_from_verified"),
            "the healthy branch must not rebuild or quarantine: {healthy}"
        );
        let failure = block.split("Err(migration_error) =>").nth(1).unwrap();
        assert!(
            failure.contains("recover_catalog_db_from_verified"),
            "a failed migration must enter controlled recovery: {failure}"
        );
    }
}

#[cfg(test)]
mod override_authority_tests {
    use super::*;

    fn seed_user_override(root: &std::path::Path) -> (String, String, String) {
        let model = catalog::CatalogModel {
            id: "mine".into(),
            repo: "local/Handmade-GGUF".into(),
            family: "Handmade".into(),
            parameters: "1B".into(),
            publisher: "local".into(),
            author: "local".into(),
            summary: String::new(),
            tags: Vec::new(),
            gated: false,
            downloads: 0,
            likes: 0,
            license: String::new(),
            pipeline_tag: String::new(),
            library_name: String::new(),
            architecture: String::new(),
            last_modified: String::new(),
            created_at: String::new(),
            files: vec![catalog::CatalogFile {
                quant: "Q4_K_M".into(),
                filename: "mine-Q4_K_M.gguf".into(),
                size_bytes: 1024,
                sha256: "d".repeat(64),
                revision: "main".into(),
                last_modified: String::new(),
                created_at: String::new(),
                user_sourced: true,
            }],
            user_sourced: true,
        };
        let mut connection = crate::catalog_db::open_catalog_db(root).unwrap();
        crate::catalog_db::migrate_catalog_db(&connection).unwrap();
        crate::catalog_db::save_user_catalog_override(&mut connection, &model).unwrap();
        (
            model.repo.clone(),
            "mine-Q4_K_M.gguf".to_string(),
            "d".repeat(64),
        )
    }

    #[test]
    fn dc04_user_overrides_authorize_with_their_own_digest_and_only_when_marked() {
        // The explicit curated-versus-user split (audit DC-04): a saved,
        // validated override authorizes a download with its exact stored
        // digest; removal and provenance flags change that immediately, and
        // nothing here ever acquires signed curated status.
        let root = crate::catalog_command_tests::unique_dir("localmotive-lib-dc04");
        let (repo, filename, sha256) = seed_user_override(&root);

        let authorized = resolve_catalog_download(None, &root, &repo, &filename, "main").unwrap();
        assert_eq!(authorized.authority, "user");
        assert_eq!(authorized.file.sha256, sha256);
        assert!(authorized.file.user_sourced);

        // A directly modified row (provenance flag cleared) loses authority.
        {
            let connection = crate::catalog_db::open_catalog_db(&root).unwrap();
            connection
                .execute(
                    "UPDATE catalog_file SET user_sourced = 0 WHERE filename_lower = ?1",
                    [filename.to_lowercase()],
                )
                .unwrap();
        }
        assert!(resolve_catalog_download(None, &root, &repo, &filename, "main").is_err());
        {
            let connection = crate::catalog_db::open_catalog_db(&root).unwrap();
            connection
                .execute(
                    "UPDATE catalog_file SET user_sourced = 1 WHERE filename_lower = ?1",
                    [filename.to_lowercase()],
                )
                .unwrap();
            connection
                .execute(
                    "UPDATE catalog_model SET user_sourced = 0 WHERE id = 'mine'",
                    [],
                )
                .unwrap();
        }
        assert!(resolve_catalog_download(None, &root, &repo, &filename, "main").is_err());

        // Restore provenance, then removal ends authorization.
        {
            let connection = crate::catalog_db::open_catalog_db(&root).unwrap();
            connection
                .execute(
                    "UPDATE catalog_model SET user_sourced = 1 WHERE id = 'mine'",
                    [],
                )
                .unwrap();
        }
        assert!(resolve_catalog_download(None, &root, &repo, &filename, "main").is_ok());
        {
            let mut connection = crate::catalog_db::open_catalog_db(&root).unwrap();
            crate::catalog_db::remove_user_catalog_override(&mut connection, "mine").unwrap();
        }
        assert!(resolve_catalog_download(None, &root, &repo, &filename, "main").is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dc04_curated_authority_wins_and_unknown_rows_never_authorize() {
        // A curated row resolves from the signed snapshot even when a user
        // store also exists; an unknown filename resolves nowhere (DC-04).
        let root = crate::catalog_command_tests::unique_dir("localmotive-lib-dc04-curated");
        let (repo, _filename, _) = seed_user_override(&root);
        let snapshot = catalog::load_catalog_snapshot(&root, catalog::DEFAULT_CATALOG_URL).unwrap();
        let (model, file) = snapshot
            .catalog
            .models
            .iter()
            .find_map(|model| model.files.first().map(|file| (model, file)))
            .expect("the bundled catalog must carry a file");
        let authorized = resolve_catalog_download(
            Some(&snapshot.catalog),
            &root,
            &model.repo,
            &file.filename,
            &file.revision,
        )
        .unwrap();
        assert_eq!(authorized.authority, "curated");
        assert_eq!(authorized.file.sha256, file.sha256);
        assert!(resolve_catalog_download(
            Some(&snapshot.catalog),
            &root,
            &repo,
            "unknown.gguf",
            "main"
        )
        .is_err());
        let _ = std::fs::remove_dir_all(root);
    }
}

#[cfg(test)]
mod tuning_lifecycle_source_tests {
    #[test]
    fn tuning_lifecycle_uses_cancellable_paths_and_reports_cleanup_failures() {
        // Audit MT-04: the live tuner must use the cancellable health wait and
        // the cancellable benchmark, and a failed trial-server cleanup must be
        // surfaced instead of discarded. Reverting any of these fails here.
        let source = include_str!("lib.rs");
        let bench_block = source
            .split("impl tune::Bench for LiveBench<'_> {")
            .nth(1)
            .expect("LiveBench must implement tune::Bench")
            .split("// Give the OS a moment to release the port")
            .next()
            .unwrap();
        assert!(
            bench_block.contains("wait_until_healthy_cancellable("),
            "the live bench must use the cancellable health wait"
        );
        assert!(
            !bench_block.contains("wait_until_healthy(\n"),
            "the live bench must not use the non-cancellable health wait"
        );
        assert!(
            bench_block.contains("benchmark_server_cancellable("),
            "the live bench must use the cancellable benchmark"
        );
        assert!(
            !bench_block.contains("core::benchmark_server("),
            "the live bench must not use the legacy benchmark"
        );
        let tail = source
            .split("impl tune::Bench for LiveBench<'_> {")
            .nth(1)
            .unwrap()
            .split("struct TuningRequest")
            .next()
            .unwrap();
        assert!(
            tail.contains("let cleanup = child.terminate_and_wait();"),
            "the cleanup result must be captured"
        );
        assert!(
            tail.contains("could not be stopped cleanly"),
            "a failed cleanup must be surfaced to the caller"
        );
    }
}

#[cfg(test)]
mod operation_coordinator_tests {
    use super::*;

    fn coordinator() -> Mutex<OperationCoordinator> {
        Mutex::new(OperationCoordinator::default())
    }

    #[test]
    fn mt05_one_operation_owns_the_machine_and_generations_advance() {
        // The audited interleavings all relied on two subsystems passing
        // their own "nothing running" checks; the reservation closes that
        // window with one atomic slot (audit MT-05 V1).
        let coordinator = coordinator();
        let server = reserve_operation(&coordinator, OperationOwner::Server).unwrap();
        assert_eq!(
            active_operation_owner(&coordinator),
            Some(OperationOwner::Server)
        );
        let error = reserve_operation(&coordinator, OperationOwner::Tuning).unwrap_err();
        assert!(
            error.contains("a running server is already active"),
            "{error}"
        );
        let error = reserve_operation(&coordinator, OperationOwner::Benchmark).unwrap_err();
        assert!(error.contains("already active"), "{error}");
        let server_generation = server.generation;
        drop(server);
        let tuning = reserve_operation(&coordinator, OperationOwner::Tuning).unwrap();
        assert!(
            tuning.generation > server_generation,
            "every successful reservation must advance the generation"
        );
        assert_eq!(
            active_operation_owner(&coordinator),
            Some(OperationOwner::Tuning)
        );
        drop(tuning);
        assert_eq!(active_operation_owner(&coordinator), None);
    }

    #[test]
    fn mt05_a_stale_reservation_cannot_release_or_finalize_a_replacement() {
        // A stopped server's work must not (a) look current after a
        // replacement acquired the slot or (b) release the replacement when
        // its own cleanup finally runs (audit MT-05 V2).
        let coordinator = coordinator();
        let first = reserve_operation(&coordinator, OperationOwner::Server).unwrap();
        let stale = OperationReservation {
            coordinator: &coordinator,
            owner: OperationOwner::Server,
            generation: first.generation,
        };
        drop(first); // the server was stopped
        let replacement = reserve_operation(&coordinator, OperationOwner::Quality).unwrap();
        assert!(
            !stale.is_current(),
            "a stale reservation must not look current"
        );
        drop(stale);
        assert!(
            replacement.is_current(),
            "dropping a stale reservation must not release the current owner"
        );
        assert_eq!(
            active_operation_owner(&coordinator),
            Some(OperationOwner::Quality)
        );
    }

    #[test]
    fn mt05_stop_rejects_while_another_owner_holds_the_machine() {
        assert!(stop_owner_conflict(None).is_none());
        assert!(stop_owner_conflict(Some(OperationOwner::Server)).is_none());
        for owner in [
            OperationOwner::Benchmark,
            OperationOwner::ColdBenchmark,
            OperationOwner::Tuning,
            OperationOwner::Quality,
        ] {
            let message = stop_owner_conflict(Some(owner)).expect("foreign owners must block Stop");
            assert!(message.contains(owner.label()), "{message}");
            assert!(message.contains("cancel it before stopping"), "{message}");
        }
    }

    #[test]
    fn mt05_every_launching_command_reserves_the_operation_slot() {
        // Source guard: each managed-inference entry point reserves its owner
        // kind, Stop consults the ownership gate, and both benchmark paths
        // discard results whose server identity was replaced (audit MT-05).
        let source = include_str!("lib.rs");
        for (start, end, expected) in [
            (
                "fn start_server(",
                "fn stop_server(",
                "reserve_operation(&state.operations, OperationOwner::Server)",
            ),
            (
                "async fn benchmark_v2(",
                "fn cancel_benchmark(",
                "OperationOwner::ColdBenchmark",
            ),
            (
                "async fn run_quality_suite(",
                "fn rank_candidates(",
                "reserve_operation(&state.operations, OperationOwner::Quality)",
            ),
            (
                "async fn start_tuning(",
                "fn cancel_tuning(",
                "reserve_operation(&state.operations, OperationOwner::Tuning)",
            ),
        ] {
            let body = source
                .split(start)
                .nth(1)
                .and_then(|rest| rest.split(end).next())
                .unwrap_or_else(|| panic!("{start} must exist before {end}"));
            assert!(
                body.contains("reserve_operation(&state.operations"),
                "{start} must reserve an operation slot"
            );
            assert!(
                body.contains(expected),
                "{start} must reserve its expected owner: {expected}"
            );
        }
        let stop = source
            .split("fn stop_server(")
            .nth(1)
            .unwrap()
            .split("#[tauri::command]")
            .next()
            .unwrap();
        assert!(
            stop.contains("stop_owner_conflict(active_operation_owner(&state.operations))"),
            "Stop must consult the ownership gate"
        );
        for (start, end) in [
            ("async fn benchmark_v2(", "fn cancel_benchmark("),
            ("async fn run_quality_suite(", "fn rank_candidates("),
        ] {
            let body = source
                .split(start)
                .nth(1)
                .unwrap()
                .split(end)
                .next()
                .unwrap();
            assert!(
                body.contains("reservation.is_current()"),
                "{start} must discard results when its server identity was replaced"
            );
        }
    }
}
