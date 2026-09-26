//! Measurement command family.
//! operation coordinator and the measurement/evidence authorities stay in
//! their modules and are consumed here.
use crate::core::LaunchProfile;
use crate::local_client::LocalHttpClient;
use crate::server_service::{start_server, validated_server_snapshot, ValidatedServerSnapshot};
use crate::LaunchValidation;
use crate::{
    artifact, calibration, core, evidence, local_client, measurement, reserve_operation, runtime,
    AppState, OperationOwner,
};
use crate::{spawn_server, wait_until_healthy_cancellable};

/// Slack above the workload's own request deadline before a worker drain is
/// considered abnormal. A cancellable worker can never outlive
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
/// it. The ceiling derives from the workload's own request deadline
/// plus teardown slack; a worker owned past it is anomalous, so the wait is
/// bounded there and the outcome is reported as [`DrainOutcome::Unresolved`]
/// instead of holding the slot permanently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrainOutcome {
    /// Every owned worker exited within the ceiling.
    Drained,
    /// A worker was still owned past the ceiling. The caller records the
    /// unresolved state so replacement work is informed.
    Unresolved,
}

pub(crate) fn drain_owned_workers(client: &LocalHttpClient, ceiling: Duration) -> DrainOutcome {
    if client.wait_for_worker_drain(ceiling) {
        DrainOutcome::Drained
    } else {
        DrainOutcome::Unresolved
    }
}

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BenchmarkRunResult {
    manifest: evidence::BenchmarkManifest,
    summary: Option<measurement::BenchmarkSummaryV2>,
    manifest_path: String,
    compatibility_key: String,
    result_class: evidence::FitClass,
    failure: Option<String>,
    /// Cold runs take the user's server for a quiet machine.
    /// True when the same profile was relaunched after the run; false when
    /// this run never took a server or the relaunch failed (see
    /// `server_restore_error`).
    #[serde(default)]
    server_restored: bool,
    /// The relaunch failure, when the benchmark record survived but the
    /// server did not come back. Never replaces the benchmark result.
    #[serde(default)]
    server_restore_error: Option<String>,
}

/// Record a cold-run server relaunch without touching the measurements:
/// success marks the server restored, failure keeps the benchmark record
/// and names the restore error instead of discarding the run.
fn cold_restore_outcome(outcome: Result<(), String>) -> (bool, Option<String>) {
    match outcome {
        Ok(()) => (true, None),
        Err(error) => (false, Some(error)),
    }
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

fn lora_identity_sha256(profile: &LaunchProfile) -> Result<String, String> {
    let references = crate::lora_references(profile)?;
    if references.is_empty() {
        return Ok(String::new());
    }
    let legacy_single = references.len() == 1 && profile.lora_scaled.trim().is_empty();
    let mut hasher = Sha256::new();
    hasher.update(b"localmotive.lora-set.v1\0");
    hasher.update((references.len() as u64).to_le_bytes());
    for (label, path, scale) in references {
        artifact::validate_regular_non_reparse_file(&label, &path)?;
        let digest = artifact::sha256_path(&path)?;
        if legacy_single {
            // Preserve the identity of existing single ordinary-adapter records.
            return Ok(digest);
        }
        hasher.update(digest.as_bytes());
        hasher.update(scale.to_bits().to_le_bytes());
    }
    Ok(hex::encode(hasher.finalize()))
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
    // the same name must change the key.
    let file_sha = |path: &str| -> Result<String, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Ok(String::new());
        }
        artifact::sha256_path(Path::new(trimmed))
    };
    let lora_sha256 = lora_identity_sha256(profile)?;
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
    // supported and are named explicitly.
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
    // workload digest and identify a launch configuration, while
    // `launch+workload` snapshots identify a measured run.
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

/// One tuning-trial measurement through the v2 evidence contract: a warmup
/// plus `repeats` measured completions against the trial server, with prompt
/// accounting and generated-token checks on every response. This replaces the
/// legacy `benchmark_server_cancellable`, which accepted a positive
/// server-reported rate without checking the generated token count. The trial
/// workload keeps the established v2 default shape (deterministic
/// server-prepared prompt, seed 42, 600 s request bound); only the generation
/// length and trial count follow the tuning request.
pub(crate) fn measure_trial_summary(
    client: &LocalHttpClient,
    tokens: u32,
    repeats: u16,
    cancelled: &AtomicBool,
) -> Result<measurement::BenchmarkSummaryV2, String> {
    let workload = evidence::Workload {
        id: "tune-trial-v1".into(),
        generation_tokens: tokens,
        trials: repeats,
        ..Default::default()
    };
    let run = measurement::run_workload_with(&workload, cancelled, || {
        measurement::completion_request_cancellable(client, &workload, cancelled)
    })?;
    if run.terminal_outcome == Some(evidence::AttemptOutcome::Cancelled) {
        return Err("The local request was cancelled".into());
    }
    measurement::summarize_observations(&run.observations)
}

/// Everything one measured attempt needs, grouped so the run signature stays
/// readable as the ownership requirements grow.
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
    // The whole run shares one client - the
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
            // cold attempt; it drops with this scope.
            let (mut child, _, log_path, _lease, drains) =
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
            // The log drains are part of cleanup: join them on the same
            // bounded policy as managed-server stop, so a run never reports
            // finished while its launch evidence is still unwritten.
            let drains_settled = crate::server_service::join_log_drains(drains);
            match (attempt, cleanup && drains_settled) {
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
    // A cancelled attempt abandons its worker, which keeps its
    // request until the operation deadline. The run holds ownership until
    // the ceiling derives from THIS workload's request deadline; a worker
    // still owned past it is reported Unresolved instead of holding
    // the slot forever.
    let drain = drain_owned_workers(&client, benchmark_drain_ceiling(&workload));
    let mut manifest = evidence::BenchmarkManifest {
        schema: evidence::BENCHMARK_SCHEMA_VERSION,
        harness_version,
        scope_note: evidence::WORKLOAD_SCOPE_NOTE.into(),
        compatibility_key: Some(compatibility_key.clone()),
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
    // an unresolved drain means a worker may still be inferring. The
    // record keeps its observations, but the run is Failed so unsettled work
    // never feeds fit as Measured, and the note tells replacement work why.
    let (result_class, failure) = match drain {
        DrainOutcome::Drained => (result_class, failure),
        DrainOutcome::Unresolved => (
            evidence::FitClass::Failed,
            Some(match failure {
                Some(note) => {
                    format!("Benchmark workers did not drain within the workload deadline; {note}")
                }
                None => "Benchmark workers did not drain within the workload deadline; \
                    a worker may still be inferring"
                    .into(),
            }),
        ),
    };
    let manifest_path = measurement::persist_manifest(directory, &manifest)?;
    Ok(BenchmarkRunResult {
        manifest: std::mem::take(&mut manifest),
        summary,
        manifest_path: manifest_path.to_string_lossy().to_string(),
        compatibility_key,
        result_class,
        failure,
        server_restored: false,
        server_restore_error: None,
    })
}

/// with construction-window cancel (review deleg_16c0e72a): the slot is
/// published before the fallible client is constructed, so a Cancel during
/// construction lands on this run's flag instead of reporting "No benchmark
/// is running". A construction failure clears only our own flag, so the slot
/// stays free without clearing a replacement; an occupied slot is never
/// overwritten, so an older owner's state can never be cleared by a newer
/// caller.
fn publish_benchmark_slot<T>(
    slot: &std::sync::Mutex<Option<Arc<AtomicBool>>>,
    cancelled: Arc<AtomicBool>,
    construct: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    // Early publication: the frontend shows Cancel as soon as
    // benchmark_v2 is dispatched, so the slot must exist during
    // construction for the cancel to land on this run's own flag (review
    // deleg_16c0e72a). Occupancy is checked first so a refused caller never
    // overwrites the occupant; a construction failure clears only our own
    // flag, so the slot stays free without clearing a replacement.
    {
        let mut active = slot.lock().map_err(|_| "Benchmark state is unavailable")?;
        if active.is_some() {
            return Err("A benchmark is already running".into());
        }
        *active = Some(cancelled.clone());
    }
    let product = match construct() {
        Ok(product) => product,
        Err(error) => {
            let mut active = slot.lock().map_err(|_| "Benchmark state is unavailable")?;
            if active
                .as_ref()
                .map(|held| Arc::ptr_eq(held, &cancelled))
                .unwrap_or(false)
            {
                *active = None;
            }
            return Err(error);
        }
    };
    Ok(product)
}

#[tauri::command]
pub(crate) async fn benchmark_v2(
    workload: evidence::Workload,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<BenchmarkRunResult, String> {
    workload.validate().map_err(|error| error.to_string())?;
    // One machine owner: a benchmark cannot start while a server or tuning
    // session owns the operations slot, and a cold attempt is its own owner
    // kind.
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
    // a cold run takes the user's server for a quiet machine. Keep
    // the profile aside so the same server is relaunched after the finalized
    // record below.
    let cold_profile = if workload.cache_mode == evidence::CacheMode::Cold {
        Some(server.profile.clone())
    } else {
        None
    };
    let cancelled = Arc::new(AtomicBool::new(false));
    // the fallible client is built BEFORE the active slot is published.
    // One client owns every request of this run: prompt preparation and the
    // measured attempts share it, so a cancellation during preparation is
    // covered by the same worker drain as the measured attempts. A
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
    // Ownership is released only after every owned request and worker
    // has actually terminated. This covers the boundaries the run's own drain
    // cannot - a cancellation during prompt preparation, an ordinary error
    // before the run, a cleanup failure, and a worker that outlived even its
    // own request deadline. The slot and the operations reservation stay held
    // for the whole wait, so replacement work is refused while the abandoned
    // request can still be inferring; a discarded result never releases
    // ownership early. The ceiling derives from this workload's deadline; a
    // worker still owned past it is reported Unresolved.
    let command_drain = drain_owned_workers(&client, drain_ceiling);
    {
        let mut active = state
            .benchmark
            .lock()
            .map_err(|_| "Benchmark state is unavailable")?;
        *active = None;
    }
    let mut result = benchmark_result?;
    // A replaced or stopped server invalidates the whole record: results must
    // never be finalized under an identity that no longer exists
    //.
    if !reservation.is_current() {
        return Err(
            "The managed server was stopped or replaced during the benchmark; the record was discarded."
                .into(),
        );
    }
    // an unresolved command drain means a worker may still be
    // inferring. Mark the run Failed so unsettled work never feeds fit as
    // Measured, and skip the server relaunch: starting replacement work over
    // a still-inferring worker is the overlap the drain guards against.
    if command_drain == DrainOutcome::Unresolved {
        result.result_class = evidence::FitClass::Failed;
        let note = "Benchmark workers did not drain within the workload deadline; \
            a worker may still be inferring";
        result.failure = Some(match result.failure.take() {
            Some(previous) => format!("{note}; {previous}"),
            None => note.into(),
        });
        if cold_profile.is_some() {
            result.server_restored = false;
            result.server_restore_error = Some(
                "Server restore skipped: benchmark workers did not drain, so the \
                relaunched server could overlap a still-inferring worker"
                    .into(),
            );
        }
    }
    // the record above is finalized; now give the user their server
    // back. The benchmark reservation is released first so the relaunch can
    // own the machine through the normal start path. A relaunch failure is
    // recorded on the result, never substituted for the benchmark outcome.
    if let Some(profile) = cold_profile.filter(|_| command_drain == DrainOutcome::Drained) {
        drop(reservation);
        let outcome = start_server(app.clone(), profile, state).await.map(|_| ());
        let (restored, error) = cold_restore_outcome(outcome);
        result.server_restored = restored;
        result.server_restore_error = error;
    }
    Ok(result)
}

#[tauri::command]
pub(crate) fn cancel_benchmark(state: tauri::State<'_, AppState>) -> Result<(), String> {
    request_benchmark_cancellation(&state)
}

fn request_benchmark_cancellation(state: &AppState) -> Result<(), String> {
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
    //.
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
    let current_snapshot = benchmark_execution_snapshot_from_profile(
        &server.profile,
        &server.validation,
        &artifacts,
        &runtime_identity,
        &executable_sha256,
        &hardware,
        Some(&manifest.workload),
    )?;
    measurement::validate_replay_compatibility(
        &manifest,
        logical_id,
        &server.validation.arguments.effective_args,
        &current_snapshot.compatibility_key,
        &current_snapshot.unknown_identities,
    )?;
    Ok(manifest.workload)
}

#[cfg(test)]
mod lora_identity_tests {
    use super::*;
    use std::{fs, path::PathBuf};

    struct Fixture {
        root: PathBuf,
        profile: LaunchProfile,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "localmotive-lora-identity-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            fs::create_dir(&root).unwrap();
            let model = root.join("model.gguf");
            crate::release_security_tests::write_test_gguf_with_identity(&model, "llama", &[]);
            fs::write(root.join("adapter a.gguf"), b"adapter-a fixture").unwrap();
            fs::write(root.join("adapter b.gguf"), b"adapter-b fixture").unwrap();
            Self {
                root,
                profile: LaunchProfile {
                    alias: "fixture".into(),
                    runtime: "fixture-runtime".into(),
                    model: model.to_string_lossy().into_owned(),
                    ..LaunchProfile::default()
                },
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }

    fn snapshot_key(profile: &LaunchProfile) -> Result<String, String> {
        // Isolate snapshot assembly from runtime execution and hardware probes.
        // Model headers and adapter bytes are real temporary files.
        let artifacts = vec![artifact::inspect_artifact(
            Path::new(&profile.model),
            &[],
            true,
        )?];
        let source = evidence::EvidenceSource {
            kind: evidence::EvidenceSourceKind::Unknown,
            detail: "isolated identity fixture".into(),
        };
        let validation = LaunchValidation {
            runtime: core::parse_capabilities("fixture", "--lora FNAME\n--lora-scaled FNAME:SCALE"),
            artifacts: artifacts.clone(),
            arguments: core::LaunchArgumentValidation {
                effective_args: profile.build_args()?,
                rejected: vec![],
                command: String::new(),
            },
            effective_context: evidence::Evidence::unknown(source.clone(), 1, "not measured"),
            unverified_requirements: vec![],
        };
        let runtime = runtime::RuntimeIdentity {
            path: profile.runtime.clone(),
            backend: "cpu".into(),
            cuda_major: None,
            tag: None,
            install_key: None,
            source: "none".into(),
            managed_verified: false,
        };
        let hardware = runtime::HardwareInfo {
            architecture: "fixture".into(),
            gpu_names: vec![],
            vendor: "unknown".into(),
            cuda_major: None,
            driver_version: String::new(),
            detection_status: "fixture".into(),
            recommendation: String::new(),
            system_memory: runtime::SystemMemoryInfo {
                total_physical_bytes: evidence::Evidence::unknown(
                    source.clone(),
                    1,
                    "not measured",
                ),
                available_physical_bytes: evidence::Evidence::unknown(
                    source.clone(),
                    1,
                    "not measured",
                ),
                memory_load_percent: evidence::Evidence::unknown(source, 1, "not measured"),
            },
            adapters: vec![],
            manual_overrides: vec![],
            unassigned_nvidia: vec![],
        };
        benchmark_execution_snapshot_from_profile(
            profile,
            &validation,
            &artifacts,
            &runtime,
            &"a".repeat(64),
            &hardware,
            None,
        )
        .map(|outcome| outcome.compatibility_key)
    }

    #[test]
    fn cold_restore_outcome_lands_in_the_record() {
        // the cold benchmark relaunches the profile it took,
        // and the relaunch outcome is recorded without touching the
        // measurements. Success marks the server restored; a relaunch
        // failure keeps the benchmark record and names the restore error
        // instead of discarding the run.
        let (restored, error) = super::cold_restore_outcome(Ok(()));
        assert!(restored);
        assert_eq!(error, None);
        let (restored, error) = super::cold_restore_outcome(Err("the port was taken".to_string()));
        assert!(!restored);
        assert_eq!(error.as_deref(), Some("the port was taken"));
    }

    #[test]
    fn scaled_lora_snapshot_reads_the_adapter_and_binds_its_scale() {
        let mut fixture = Fixture::new();
        fixture.profile.lora_scaled =
            format!("{}:0.5", fixture.root.join("adapter a.gguf").display());
        // Independent hashlib/struct vectors pin the byte format across hosts.
        assert_eq!(
            lora_identity_sha256(&fixture.profile).unwrap(),
            "a300e08a80f79c2fb59214716c7c218cefbf1495f647dbb24c2e83539229fc26"
        );
        let first = snapshot_key(&fixture.profile)
            .unwrap_or_else(|error| panic!("valid scaled adapter rejected: {error}"));
        fixture.profile.lora_scaled =
            format!("{}:0.25", fixture.root.join("adapter a.gguf").display());
        assert_ne!(first, snapshot_key(&fixture.profile).unwrap());
    }

    #[test]
    fn lora_identity_preserves_empty_and_single_ordinary_digests() {
        let mut fixture = Fixture::new();
        assert_eq!(lora_identity_sha256(&fixture.profile).unwrap(), "");
        let adapter = fixture.root.join("adapter a.gguf");
        fixture.profile.lora = adapter.to_string_lossy().into_owned();
        assert_eq!(
            lora_identity_sha256(&fixture.profile).unwrap(),
            artifact::sha256_path(&adapter).unwrap()
        );
    }

    #[test]
    fn lora_identity_binds_order_duplicates_and_later_file_bytes() {
        let mut fixture = Fixture::new();
        let a = fixture.root.join("adapter a.gguf");
        let b = fixture.root.join("adapter b.gguf");
        fixture.profile.lora = format!("{},{}", a.display(), b.display());
        let baseline = lora_identity_sha256(&fixture.profile).unwrap();
        assert_eq!(
            baseline,
            "522a47b9345706c82bef43cbe962f7d6d4c9d00f7329ee6d41291a1b3f32167d"
        );
        fixture.profile.lora = format!("{},{}", b.display(), a.display());
        assert_ne!(baseline, lora_identity_sha256(&fixture.profile).unwrap());
        fixture.profile.lora = format!("{},{}", a.display(), a.display());
        let duplicates = lora_identity_sha256(&fixture.profile).unwrap();
        assert_ne!(baseline, duplicates);
        fixture.profile.lora = a.to_string_lossy().into_owned();
        assert_ne!(duplicates, lora_identity_sha256(&fixture.profile).unwrap());
        fixture.profile.lora = format!("{},{}", a.display(), b.display());
        fs::write(b, b"changed adapter-b fixture").unwrap();
        assert_ne!(baseline, lora_identity_sha256(&fixture.profile).unwrap());
    }

    #[test]
    fn lora_identity_includes_scaled_entries_beside_ordinary_entries() {
        let mut fixture = Fixture::new();
        let a = fixture.root.join("adapter a.gguf");
        let b = fixture.root.join("adapter b.gguf");
        fixture.profile.lora = a.to_string_lossy().into_owned();
        fixture.profile.lora_scaled = format!("{}:0.5", b.display());
        assert_eq!(
            lora_identity_sha256(&fixture.profile).unwrap(),
            "2f973e052881609cd61f2b885105749312d6f36b3d54e3f1314fbdf5f1c4c64d"
        );
        let baseline = snapshot_key(&fixture.profile).unwrap();
        fixture.profile.lora_scaled = format!("{}:0.25", b.display());
        assert_ne!(baseline, snapshot_key(&fixture.profile).unwrap());
        fixture.profile.lora_scaled = format!("{}:0.50", b.display());
        assert_eq!(baseline, snapshot_key(&fixture.profile).unwrap());
        fs::write(b, b"changed scaled adapter").unwrap();
        assert_ne!(baseline, snapshot_key(&fixture.profile).unwrap());
    }

    #[test]
    fn lora_identity_rejects_malformed_lists_and_unreadable_files() {
        let mut fixture = Fixture::new();
        let a = fixture.root.join("adapter a.gguf");
        for scale in ["", "NaN", "inf", "-inf", "1e100", "not-a-number"] {
            fixture.profile.lora_scaled = format!("{}:{scale}", a.display());
            assert!(
                lora_identity_sha256(&fixture.profile).is_err(),
                "scale {scale}"
            );
        }
        fixture.profile.lora_scaled = ":0.5".into();
        assert!(lora_identity_sha256(&fixture.profile)
            .unwrap_err()
            .contains("empty path"));
        fixture.profile.lora_scaled.clear();
        fixture.profile.lora = format!("{},", a.display());
        assert!(lora_identity_sha256(&fixture.profile)
            .unwrap_err()
            .contains("empty path"));
        fixture.profile.lora = fixture.root.to_string_lossy().into_owned();
        assert!(lora_identity_sha256(&fixture.profile).is_err());
        let missing = fixture.root.join("missing.gguf");
        fixture.profile.lora = missing.to_string_lossy().into_owned();
        assert!(lora_identity_sha256(&fixture.profile).is_err());
        assert!(!missing.exists());
    }

    #[cfg(windows)]
    #[test]
    fn lora_identity_refuses_a_junction_ancestor() {
        let mut fixture = Fixture::new();
        let target = fixture.root.join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("adapter.gguf"), b"fixture").unwrap();
        let junction = fixture.root.join("junction");
        let output = crate::proc::hidden_command("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&target)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "could not create the junction fixture"
        );
        fixture.profile.lora = junction.join("adapter.gguf").to_string_lossy().into_owned();
        let result = lora_identity_sha256(&fixture.profile);
        fs::remove_dir(junction).unwrap();
        assert!(result.unwrap_err().contains("reparse"));
    }
}

#[cfg(test)]
mod ownership_tests {
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
    fn a_client_construction_failure_leaves_the_benchmark_slot_free() {
        // the previous order published the active slot and THEN called
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
    fn a_refused_caller_never_replaces_the_occupants_state() {
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
    fn cancel_is_observable_while_the_client_is_constructed() {
        // Review deleg_16c0e72a: the frontend shows Cancel as soon as
        // the command is dispatched, but the backend published the slot
        // only after local_client construction succeeded. A cancel in that
        // window received "No benchmark is running" while construction
        // continued and the benchmark ran anyway. The slot must exist during
        // construction so the cancel lands on the run's own flag.
        let slot = std::sync::Arc::new(std::sync::Mutex::new(None));
        let cancelled = Arc::new(AtomicBool::new(false));
        let probe = cancelled.clone();
        let worker_slot = slot.clone();
        let worker = std::thread::spawn(move || {
            publish_benchmark_slot(&worker_slot, cancelled, || {
                std::thread::sleep(Duration::from_millis(400));
                Ok::<u32, String>(42)
            })
        });
        // The constructor is still running here (it sleeps 400 ms). The slot
        // must already hold this run's flag so request_benchmark_cancellation
        // can observe it. Check once at 100 ms: late publication still has
        // an empty slot at that point, early publication already holds it.
        std::thread::sleep(Duration::from_millis(100));
        let observed = slot.lock().unwrap().is_some();
        let flag_matches = slot
            .lock()
            .unwrap()
            .as_ref()
            .map(|held| Arc::ptr_eq(held, &probe))
            .unwrap_or(false);
        // Cancel now: with early publication this succeeds and the run must
        // observe it. With late publication the slot is still empty here.
        if observed {
            slot.lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .store(true, Ordering::Relaxed);
        }
        let product = worker.join().unwrap();
        assert!(
            observed,
            "the slot must be published before construction finishes"
        );
        assert!(
            flag_matches,
            "the published flag must be this run's own cancellation flag"
        );
        assert_eq!(product.unwrap(), 42);
        assert!(
            probe.load(Ordering::Relaxed),
            "a cancel during construction must survive construction"
        );
    }

    #[test]
    fn the_drain_bound_follows_the_workload_request_deadline() {
        // The previous fixed 300 s ceiling sat BELOW the default 600 s
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
    fn bounded_drain_reports_an_unresolved_worker() {
        // the ownership drain is bounded by its ceiling. A
        // worker still owned past the ceiling is reported Unresolved instead
        // of holding the benchmark slot forever; a settled client drains.
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
        let outcome = drain_owned_workers(&client, Duration::from_millis(50));
        assert_eq!(
            outcome,
            DrainOutcome::Unresolved,
            "a worker owned past the ceiling must not hold the slot forever"
        );
        assert!(
            settle_until(
                || client.active_cancellable_workers() == 0,
                Duration::from_secs(30)
            ),
            "the abandoned worker winds down on its own"
        );
        assert_eq!(
            drain_owned_workers(&client, Duration::from_millis(50)),
            DrainOutcome::Drained
        );
    }

    #[test]
    fn cancellation_during_preparation_leaves_an_owned_worker_the_drain_waits_for() {
        // supersedes the infinite wait this test once pinned: the
        // abandoned-worker scenario now lives in
        // bounded_drain_reports_an_unresolved_worker, which asserts the
        // ceiling bounds the drain and the straggler is reported Unresolved.
        // This test keeps the derivation pin: the bound follows the
        // workload's own request deadline.
        let short = evidence::Workload {
            timeout_ms: 5_000,
            ..evidence::Workload::default()
        };
        assert_eq!(
            benchmark_drain_ceiling(&short),
            Duration::from_millis(35_000)
        );
    }
}
