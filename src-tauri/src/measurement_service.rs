//! Measurement, quality and calibration command family (audit S-27 I1,
//! slice 3b). Extracted from `lib.rs`: the same Tauri commands; the shared
//! operation coordinator and the measurement/evidence authorities stay in
//! their modules and are consumed here.
use crate::core::{BenchmarkSummary, LaunchProfile};
use crate::local_client::LocalHttpClient;
use crate::server_service::{validated_server_snapshot, ValidatedServerSnapshot};
use crate::LaunchValidation;
use crate::{
    artifact, calibration, core, evidence, local_client, measurement, recommend, reserve_operation,
    runtime, AppState, OperationOwner,
};
use crate::{spawn_server, wait_until_healthy_cancellable};

/// Slack above the workload's own request deadline before a worker drain is
/// considered abnormal (R16 follow-up). A cancellable worker can never outlive
/// its request deadline by more than process teardown, so the drain bound is
/// derived from the deadline the run actually used instead of a fixed value:
/// the previous fixed 300 s ceiling sat BELOW the default 600 s request
/// deadline, so a cancelled slow request could outlive the ceiling while the
/// ownership was already released.
const WORKER_TEARDOWN_SLACK: std::time::Duration = std::time::Duration::from_secs(30);

/// The drain bound for one workload: its request deadline plus teardown slack.
pub(crate) fn benchmark_drain_ceiling(workload: &evidence::Workload) -> Duration {
    Duration::from_millis(workload.timeout_ms).saturating_add(WORKER_TEARDOWN_SLACK)
}

/// Hold ownership until every owned cancellable worker has exited.
///
/// A cancelled benchmark abandons its worker, which keeps its request until
/// the request deadline. The run - and the command that owns it - must not
/// finalize a record, clear its slot, or release the operations reservation
/// while such a worker can still be inferring: replacement work would overlap
/// it (R04/R16). The ceiling is the expected bound; if it is exceeded, the
/// worker outlived its own request deadline - an unexpected state - and the
/// wait still continues rather than releasing ownership while work continues.
pub(crate) fn drain_owned_workers(client: &LocalHttpClient, ceiling: Duration) {
    if client.wait_for_worker_drain(ceiling) {
        return;
    }
    while !client.wait_for_worker_drain(Duration::from_secs(1)) {}
}

use serde::Serialize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[tauri::command]
pub(crate) async fn benchmark_server(
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
pub(crate) struct BenchmarkRunResult {
    manifest: evidence::BenchmarkManifest,
    summary: Option<measurement::BenchmarkSummaryV2>,
    manifest_path: String,
    compatibility_key: String,
    result_class: evidence::FitClass,
    failure: Option<String>,
}

pub(crate) fn benchmark_file_fact(file: &artifact::ArtifactFileFact) -> evidence::FileFact {
    evidence::FileFact {
        path: file.path.clone(),
        bytes: file.size_bytes,
        sha256: file.sha256.clone(),
    }
}

pub(crate) struct ExecutionSnapshotOutcome {
    compatibility_key: String,
    schema_version: String,
    unknown_identities: Vec<String>,
}

pub(crate) fn benchmark_execution_snapshot_from_profile(
    profile: &LaunchProfile,
    validation: &LaunchValidation,
    artifacts: &[artifact::ArtifactInspection],
    runtime_identity: &runtime::RuntimeIdentity,
    executable_sha256: &str,
    hardware: &runtime::HardwareInfo,
    workload: Option<&evidence::Workload>,
) -> Result<ExecutionSnapshotOutcome, String> {
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
    let (adapter_ids, driver_versions): (Vec<String>, Vec<String>) = adapters.into_iter().unzip();

    // Content identities for file-backed influences: a changed file under
    // the same name must change the key (audit MT-07 I2).
    let file_sha = |path: &str| -> Result<String, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Ok(String::new());
        }
        artifact::sha256_path(Path::new(trimmed))
    };
    let lora_path = profile.lora.split(',').next().unwrap_or("").trim();
    let lora_scaled_path = profile.lora_scaled.split(',').next().unwrap_or("").trim();
    let lora_sha256 = if !lora_path.is_empty() {
        file_sha(lora_path)?
    } else if !lora_scaled_path.is_empty() {
        file_sha(lora_scaled_path)?
    } else {
        String::new()
    };
    let draft_model_sha256 = match profile.draft_model.as_deref() {
        Some(path) => file_sha(path)?,
        None => String::new(),
    };
    let mmproj_sha256 = match profile.mmproj.as_deref() {
        Some(path) => file_sha(path)?,
        None => String::new(),
    };

    let host_memory_bytes = hardware
        .system_memory
        .total_physical_bytes
        .value
        .unwrap_or(0);
    let host_cpu_model = String::new();

    // Material facts this machine cannot observe make reuse insufficiently
    // supported and are named explicitly (audit MT-07 I3).
    let mut unknown_identities = Vec::new();
    for (adapter, driver) in adapter_ids.iter().zip(driver_versions.iter()) {
        let normalized = driver.trim().to_ascii_lowercase();
        if normalized.is_empty() || normalized == "unknown" {
            unknown_identities.push(format!("driverVersion:{adapter}"));
        }
    }
    if host_cpu_model.trim().is_empty() {
        // The platform probe does not collect a CPU model yet; recorded as
        // unknown so cross-machine reuse stays visibly unsupported.
        unknown_identities.push("hostCpuModel".into());
    }
    if host_memory_bytes == 0 {
        unknown_identities.push("hostMemoryBytes".into());
    }
    if validation.effective_context.value.is_none() {
        unknown_identities.push("contextEffective".into());
    }
    let backend = runtime_identity.backend.clone();
    if matches!(backend.as_str(), "cuda" | "vulkan" | "rocm" | "sycl") && adapter_ids.is_empty() {
        unknown_identities.push("adapterIds".into());
    }
    unknown_identities.sort();
    unknown_identities.dedup();

    // The scope is part of the identity: `launch` snapshots carry no
    // workload digest and identify a configuration for quality attachment,
    // `launch+workload` snapshots identify a measured run (audit MT-09).
    let (scope, workload_sha256) = match workload {
        Some(workload) => (
            calibration::SNAPSHOT_SCOPE_LAUNCH_WORKLOAD,
            calibration::workload_sha256(workload)?,
        ),
        None => (calibration::SNAPSHOT_SCOPE_LAUNCH, String::new()),
    };
    let snapshot = calibration::ExecutionSnapshotV2 {
        schema_version: calibration::EXECUTION_SNAPSHOT_SCHEMA.into(),
        scope: scope.into(),
        effective_args: calibration::sanitize_effective_args(&validation.arguments.effective_args),
        model_content_sha256,
        model_architecture,
        runtime_sha256: executable_sha256.into(),
        runtime_help_sha256: validation.runtime.help_sha256.clone(),
        runtime_backend: backend,
        runtime_version: validation.runtime.version.clone(),
        runtime_build: validation.runtime.build.clone(),
        adapter_ids,
        driver_versions,
        host_cpu_model,
        host_platform: std::env::consts::OS.into(),
        host_memory_bytes,
        context_requested: profile.context,
        context_effective: validation.effective_context.value,
        draft_model_sha256,
        mmproj_sha256,
        lora_sha256,
        workload_sha256,
        harness_version: env!("CARGO_PKG_VERSION").into(),
        estimator_version: calibration::ESTIMATOR_VERSION.into(),
        unknown_identities,
    };
    let compatibility_key = calibration::execution_snapshot_key(&snapshot)?;
    Ok(ExecutionSnapshotOutcome {
        compatibility_key,
        schema_version: snapshot.schema_version,
        unknown_identities: snapshot.unknown_identities,
    })
}

/// Everything one measured attempt needs, grouped so the run signature stays
/// readable as the ownership requirements grow (R16 added the shared client).
pub(crate) struct BenchmarkRunContext<'a> {
    pub server_pid: u32,
    pub profile: LaunchProfile,
    pub validation: LaunchValidation,
    pub workload: evidence::Workload,
    pub prepared_prompt_tokens: Option<Vec<i32>>,
    pub cancelled: &'a AtomicBool,
    pub directory: &'a Path,
}

pub(crate) fn run_benchmark_snapshot(
    client: LocalHttpClient,
    context: BenchmarkRunContext<'_>,
) -> Result<BenchmarkRunResult, String> {
    let BenchmarkRunContext {
        server_pid,
        profile,
        validation,
        workload,
        prepared_prompt_tokens,
        cancelled,
        directory,
    } = context;
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
    let snapshot_outcome = benchmark_execution_snapshot_from_profile(
        &profile,
        &validation,
        &artifacts,
        &runtime_identity,
        &executable_sha256,
        &hardware,
        Some(&workload),
    )?;
    let compatibility_key = snapshot_outcome.compatibility_key.clone();
    // The launch-scope identity of the same configuration lets quality
    // evidence attach to measured runs without a workload (audit MT-09).
    let launch_compatibility_key = benchmark_execution_snapshot_from_profile(
        &profile,
        &validation,
        &artifacts,
        &runtime_identity,
        &executable_sha256,
        &hardware,
        None,
    )?
    .compatibility_key;
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
    // R04 (follow-up review db548c8): the whole run shares one client - the
    // command creates it and passes it to preparation and to this function -
    // so every cancellable worker of this benchmark is observable on it; the
    // run drains those workers before it returns.
    let prompt_tokens = match (workload.cache_mode, prepared_prompt_tokens) {
        (evidence::CacheMode::Cold, None) => {
            return Err("Cold-cache benchmarking requires prepared prompt tokens".into());
        }
        (_, Some(prompt_tokens)) => prompt_tokens,
        (evidence::CacheMode::Warm, None) => {
            measurement::prepare_exact_prompt_tokens_cancellable(&client, &workload, cancelled)?
        }
    };
    let workload_run = if workload.cache_mode == evidence::CacheMode::Cold {
        measurement::run_cold_workload_with(&workload, cancelled, || {
            if cancelled.load(Ordering::Relaxed) {
                return Err("Benchmark cancelled before fresh runtime launch".into());
            }
            // The lease binding pins the verified runtime content for the whole
            // cold attempt (audit RT-04); it drops with this scope.
            let (mut child, _, log_path, _lease, _drains) =
                spawn_server(&profile, "benchmark-cold")?;
            let attempt = wait_until_healthy_cancellable(
                &mut child,
                &client,
                &log_path,
                Duration::from_secs(120),
                cancelled,
            )
            .and_then(|_| {
                let mut timing = measurement::completion_request_with_prompt_tokens_cancellable(
                    &client,
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
                &client,
                &workload,
                &prompt_tokens,
                cancelled,
            )?;
            timing.peak_process_rss_bytes = runtime::process_peak_working_set(server_pid);
            Ok(timing)
        })?
    };
    // R04/R16: a cancelled attempt abandons its worker, which keeps its
    // request until the operation deadline. The run holds ownership until
    // every such worker has exited: the ceiling derives from THIS workload's
    // request deadline, and a worker that outlives even that keeps being
    // waited for instead of being discarded under unknown ownership.
    drain_owned_workers(&client, benchmark_drain_ceiling(&workload));
    let mut manifest = evidence::BenchmarkManifest {
        schema: evidence::BENCHMARK_SCHEMA_VERSION,
        harness_version,
        scope_note: evidence::WORKLOAD_SCOPE_NOTE.into(),
        compatibility_key: Some(compatibility_key.clone()),
        launch_compatibility_key: Some(launch_compatibility_key),
        execution_snapshot_schema: snapshot_outcome.schema_version.clone(),
        execution_snapshot_unknowns: snapshot_outcome.unknown_identities.clone(),
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

/// F9-02: publish the active benchmark slot only after the fallible client has
/// been constructed. The previous order published the slot first and then
/// called `local_client(&server.profile)?`, so a certificate or API-key file
/// that became invalid after the server was launched returned an error while
/// the slot stayed occupied - no benchmark had started, yet every later
/// attempt was refused with "A benchmark is already running". The constructor
/// runs before the slot is touched, and an occupied slot is never overwritten,
/// so an older owner's state can never be cleared by a newer caller.
fn publish_benchmark_slot<T>(
    slot: &std::sync::Mutex<Option<Arc<AtomicBool>>>,
    cancelled: Arc<AtomicBool>,
    construct: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let product = construct()?;
    let mut active = slot.lock().map_err(|_| "Benchmark state is unavailable")?;
    if active.is_some() {
        return Err("A benchmark is already running".into());
    }
    *active = Some(cancelled);
    Ok(product)
}

#[tauri::command]
pub(crate) async fn benchmark_v2(
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
    // F9-02: the fallible client is built BEFORE the active slot is published.
    // One client owns every request of this run: prompt preparation and the
    // measured attempts share it, so a cancellation during preparation is
    // covered by the same worker drain as the measured attempts (R16). A
    // construction failure here reports truthfully and leaves no occupied
    // benchmark state behind.
    let client = publish_benchmark_slot(&state.benchmark, cancelled.clone(), || {
        local_client(&server.profile)
    })?;
    let run_client = client.clone();
    // The drain bound is derived from this workload's own request deadline
    // before the workload is moved into the run task.
    let drain_ceiling = benchmark_drain_ceiling(&workload);
    let benchmark_result: Result<BenchmarkRunResult, String> = async {
        let prepared_prompt_tokens = if workload.cache_mode == evidence::CacheMode::Cold {
            let workload = workload.clone();
            let preparation_cancelled = cancelled.clone();
            let preparation_client = run_client.clone();
            Some(
                tauri::async_runtime::spawn_blocking(move || {
                    measurement::prepare_exact_prompt_tokens_cancellable(
                        &preparation_client,
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
        let task_client = run_client.clone();
        tauri::async_runtime::spawn_blocking(move || {
            run_benchmark_snapshot(
                task_client,
                BenchmarkRunContext {
                    server_pid: server.pid,
                    profile: server.profile,
                    validation: server.validation,
                    workload,
                    prepared_prompt_tokens,
                    cancelled: task_cancelled.as_ref(),
                    directory: &directory,
                },
            )
        })
        .await
        .map_err(|error| format!("Benchmark task failed: {error}"))?
    }
    .await;
    // R16: ownership is released only after every owned request and worker
    // has actually terminated. This covers the boundaries the run's own drain
    // cannot - a cancellation during prompt preparation, an ordinary error
    // before the run, a cleanup failure, and a worker that outlived even its
    // own request deadline. The slot and the operations reservation stay held
    // for the whole wait, so replacement work is refused while the abandoned
    // request can still be inferring; a discarded result never releases
    // ownership early. The ceiling derives from this workload's deadline.
    drain_owned_workers(&client, drain_ceiling);
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
pub(crate) fn cancel_benchmark(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let active = state
        .benchmark
        .lock()
        .map_err(|_| "Benchmark state is unavailable")?;
    let cancel = active.as_ref().ok_or("No benchmark is running")?;
    cancel.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub(crate) async fn replay_benchmark_manifest(
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

pub(crate) fn replay_benchmark_manifest_worker(
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
    let current_compatibility_key = benchmark_execution_snapshot_from_profile(
        &server.profile,
        &server.validation,
        &artifacts,
        &runtime_identity,
        &executable_sha256,
        &hardware,
        Some(&manifest.workload),
    )?
    .compatibility_key;
    measurement::validate_replay_compatibility(
        &manifest,
        logical_id,
        &core::manifest_safe_args(&server.validation.arguments.effective_args),
        &current_compatibility_key,
    )?;
    Ok(manifest.workload)
}

#[tauri::command]
pub(crate) async fn run_quality_suite(
    state: tauri::State<'_, AppState>,
) -> Result<recommend::QualitySuiteResult, String> {
    // Quality checks drive the running server: they take the single
    // operations reservation and must not finalize against a replacement
    // (audit MT-05).
    let reservation = reserve_operation(&state.operations, OperationOwner::Quality)?;
    let (profile, runtime_path, model_logical_id, validation) = {
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
            server.validation.clone(),
        )
    };
    let joined = tauri::async_runtime::spawn_blocking(
        move || -> Result<recommend::QualitySuiteResult, String> {
            // Full identity is captured BEFORE the quality requests
            // (audit MT-09 I1): model content, runtime executable, and the
            // launch-scope execution snapshot of the serving configuration.
            let (model_content_sha256, launch_compatibility_key) = {
                let inspection = artifact::inspect_artifact(
                    Path::new(
                        validation
                            .artifacts
                            .first()
                            .map(|artifact| artifact.first_shard.as_str())
                            .ok_or("Validated model identity is unavailable")?,
                    ),
                    &[],
                    true,
                )?;
                let content = inspection
                    .shards
                    .first()
                    .and_then(|shard| shard.sha256.clone())
                    .ok_or("Model content identity is unavailable")?;
                let runtime_identity = runtime::describe_runtime(Path::new(&runtime_path));
                let executable_sha256 = artifact::sha256_path(Path::new(&runtime_path))?;
                let hardware = runtime::detect_hardware();
                let key = benchmark_execution_snapshot_from_profile(
                    &profile,
                    &validation,
                    &[inspection],
                    &runtime_identity,
                    &executable_sha256,
                    &hardware,
                    None,
                )?
                .compatibility_key;
                (content, key)
            };
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
            result.model_content_sha256 = Some(model_content_sha256);
            result.compatibility_key = Some(launch_compatibility_key);
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

/// Join quality evidence to a measured run in Rust: the compatibility and
/// content checks happen at this boundary, never in the frontend
/// (audit MT-09 I3).
#[tauri::command]
pub(crate) fn join_quality_candidate(
    id: String,
    manifest: evidence::BenchmarkManifest,
    result_class: evidence::FitClass,
    quality: Option<recommend::QualitySuiteResult>,
) -> Result<recommend::CandidateEvidence, String> {
    recommend::candidate_from_manifest(&id, &manifest, result_class, quality.as_ref())
}

#[tauri::command]
pub(crate) fn rank_candidates(
    candidates: Vec<recommend::CandidateEvidence>,
    constraints: recommend::RecommendationConstraints,
    weights: recommend::ObjectiveWeights,
) -> Result<Vec<recommend::RankedCandidate>, String> {
    recommend::rank_candidates(&candidates, &constraints, &weights)
}

#[tauri::command]
pub(crate) fn build_compatibility_key(
    identity: calibration::CompatibilityIdentity,
) -> Result<String, String> {
    calibration::compatibility_key(&identity)
}

#[tauri::command]
pub(crate) fn build_calibration_model(
    anchors: Vec<calibration::CalibrationAnchor>,
    created_at_ms: u64,
    ttl_ms: u64,
) -> Result<calibration::CalibrationModel, String> {
    calibration::build_calibration(&anchors, created_at_ms, ttl_ms)
}

#[tauri::command]
pub(crate) fn apply_calibration_model(
    model: calibration::CalibrationModel,
    compatibility_key: String,
    estimated_value: f64,
    now_ms: u64,
) -> Result<calibration::CalibratedEstimate, String> {
    calibration::apply_calibration(&model, &compatibility_key, estimated_value, now_ms)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRecords {
    pub(crate) anchors: Vec<calibration::CalibrationAnchor>,
    pub(crate) models: Vec<calibration::CalibrationModel>,
    /// Bounded diagnostics from the load (audit S-16): quarantined corrupt or
    /// invalid records are reported here while valid history keeps loading.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) problems: Vec<String>,
}

pub(crate) fn calibration_storage_root(
    app: &tauri::AppHandle,
) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("calibration"))
}

#[cfg(test)]
mod r16_ownership_tests {
    use super::*;
    use crate::local_client::tests::serve_slow_body;
    use std::sync::atomic::Ordering;
    use std::time::Instant;

    fn settle_until(mut predicate: impl FnMut() -> bool, timeout: Duration) -> bool {
        let started = Instant::now();
        while started.elapsed() < timeout {
            if predicate() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        predicate()
    }

    #[test]
    fn f9_02_a_client_construction_failure_leaves_the_benchmark_slot_free() {
        // F9-02: the previous order published the active slot and THEN called
        // the fallible `local_client(&server.profile)?`. A certificate or
        // API-key file that became invalid after the server launched returned
        // an error while the slot stayed occupied, so no benchmark had started
        // yet every later attempt was refused as "already running". The
        // constructor now runs before the slot is touched: the failure is
        // truthful, the slot is free, and repairing the input lets the next
        // attempt publish and run.
        let slot: std::sync::Mutex<Option<Arc<AtomicBool>>> = std::sync::Mutex::new(None);
        let failed = publish_benchmark_slot(&slot, Arc::new(AtomicBool::new(false)), || {
            Err::<u32, String>("The API key file is empty".into())
        });
        assert_eq!(failed.unwrap_err(), "The API key file is empty");
        assert!(
            slot.lock().unwrap().is_none(),
            "a construction failure must not occupy the benchmark slot"
        );

        // Repair the input: the subsequent attempt succeeds and owns the slot.
        let cancelled = Arc::new(AtomicBool::new(true));
        let mut published = 0;
        let product = publish_benchmark_slot(&slot, cancelled.clone(), || {
            published += 1;
            Ok::<u32, String>(42)
        })
        .unwrap();
        assert_eq!(product, 42);
        assert_eq!(published, 1);
        let held = slot.lock().unwrap();
        let held = held.as_ref().expect("the repaired attempt owns the slot");
        assert!(Arc::ptr_eq(held, &cancelled));
    }

    #[test]
    fn f9_02_a_refused_caller_never_replaces_the_occupants_state() {
        // The occupant's slot and its cancellation flag stay exactly as the
        // occupant left them. The fallible dependency is deliberately built
        // before occupancy is known (that ordering is what keeps a
        // construction failure from occupying the slot), so this test asserts
        // the consequences that matter: the refusal is truthful, the
        // occupant's Arc is still the slot's owner, and the refused caller's
        // own flag was never published - an older cleanup can therefore never
        // clear a replacement operation's state.
        let owner = Arc::new(AtomicBool::new(false));
        let slot: std::sync::Mutex<Option<Arc<AtomicBool>>> =
            std::sync::Mutex::new(Some(owner.clone()));
        let refused_flag = Arc::new(AtomicBool::new(true));
        let product = publish_benchmark_slot(&slot, refused_flag.clone(), || Ok::<u32, String>(7));
        assert_eq!(product.unwrap_err(), "A benchmark is already running");
        let held = slot.lock().unwrap();
        assert!(
            Arc::ptr_eq(held.as_ref().unwrap(), &owner),
            "the occupant must still own the slot"
        );
        assert!(
            !owner.load(Ordering::Relaxed),
            "the occupant's flag is untouched"
        );
        assert!(
            !Arc::ptr_eq(held.as_ref().unwrap(), &refused_flag),
            "a refused caller must never publish its own flag"
        );
    }

    #[test]
    fn r16_the_drain_bound_follows_the_workload_request_deadline() {
        // R16: the previous fixed 300 s ceiling sat BELOW the default 600 s
        // request deadline, so a cancelled slow request could outlive the
        // ceiling while ownership was already released. The bound is derived
        // from the deadline the run actually uses.
        let default = evidence::Workload::default();
        assert_eq!(default.timeout_ms, 600_000);
        assert_eq!(
            benchmark_drain_ceiling(&default),
            Duration::from_millis(630_000),
            "the derived bound must sit above the default request deadline"
        );
        let short = evidence::Workload {
            timeout_ms: 5_000,
            ..evidence::Workload::default()
        };
        assert_eq!(
            benchmark_drain_ceiling(&short),
            Duration::from_millis(35_000)
        );
    }

    #[test]
    fn r16_cancellation_during_preparation_leaves_an_owned_worker_the_drain_waits_for() {
        // The cold path prepares the exact prompt before any measured trial.
        // A cancellation during that preparation abandons the tokenize worker
        // exactly like a cancelled trial; the run shares one client, so the
        // command's drain must observe it and hold ownership until it exits.
        let fixture = serve_slow_body(Duration::from_millis(1_200), 12);
        let client = LocalHttpClient::plain("127.0.0.1", fixture.port).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let workload = evidence::Workload {
            prompt_tokens: 64,
            timeout_ms: 30_000,
            ..evidence::Workload::default()
        };
        let flag = Arc::clone(&cancelled);
        let caller = client.clone();
        let work = std::thread::spawn(move || {
            measurement::prepare_exact_prompt_tokens_cancellable(&caller, &workload, &flag)
        });
        assert!(
            settle_until(
                || client.active_cancellable_workers() > 0,
                Duration::from_secs(5)
            ),
            "the tokenize worker must be observable while it runs"
        );
        cancelled.store(true, Ordering::Relaxed);
        assert!(
            work.join().unwrap().is_err(),
            "the cancelled preparation reports the cancellation"
        );
        assert!(
            client.active_cancellable_workers() > 0,
            "the abandoned preparation worker is still in flight after the cancellation"
        );
        // A ceiling below the worker's own deadline must not report the
        // ownership resolved, and the drain must keep waiting past it instead
        // of releasing ownership while the request is still running.
        assert!(
            !client.wait_for_worker_drain(Duration::from_millis(50)),
            "the drain must report the worker still owned"
        );
        let started = Instant::now();
        drain_owned_workers(&client, Duration::from_millis(50));
        let waited = started.elapsed();
        assert_eq!(
            client.active_cancellable_workers(),
            0,
            "the drain returns only after the worker exited"
        );
        assert!(
            waited >= Duration::from_millis(500),
            "the drain kept waiting past the exceeded ceiling ({waited:?})"
        );
    }
}
