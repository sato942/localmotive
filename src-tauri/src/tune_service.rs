//! AI tuning command family (audit S-27 I1, slice 3c): the last supervised
//! job-lifecycle owner. Extracted from `lib.rs`; the tuning engine, cloud
//! advisor and the shared operation coordinator stay authoritative in their
//! modules and are consumed here.
use crate::core::LaunchProfile;
use crate::tune;
use crate::{
    cloud, core, gguf, local_client, proc, reserve_operation, runtime, spawn_server, status_from,
    update_launch_effective_context, wait_until_healthy_cancellable, AppState, OperationOwner,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::Emitter;
pub(crate) struct LiveBench<'a> {
    app: &'a tauri::AppHandle,
    cancel: Arc<AtomicBool>,
    tokens: u32,
    repeats: u16,
}

impl tune::Bench for LiveBench<'_> {
    fn measure(&mut self, profile: &LaunchProfile) -> Result<tune::TrialMeasurement, String> {
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
        let (mut child, mut validation, log_path, _lease, _drains) =
            spawn_server(profile, "tuning")?;
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
            // The observed effective per-slot context is the identity the
            // requested-capacity objective is checked against (audit MT-11).
            let effective_context = validation.effective_context.value;
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
                .map(|summary| tune::TrialMeasurement {
                    summary,
                    command: String::new(),
                    effective_context,
                })
        })();
        // Cleanup failures must be visible: a measured result may not be
        // reported as a clean success when the trial server could not be
        // stopped (audit MT-04).
        let cleanup = child.terminate_and_wait();
        // Give the OS a moment to release the port before the next launch.
        std::thread::sleep(Duration::from_millis(600));
        match (result, cleanup) {
            (Ok(mut measurement), true) => {
                measurement.command = command;
                Ok(measurement)
            }
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TuningRequest {
    profile: LaunchProfile,
    provider: String,
    model: String,
    target_context: u32,
    max_trials: u32,
    tokens: u32,
    repeats: u16,
    companions: Vec<String>,
    /// What the cloud brief carries (audit S-20.I2); absent means Full so
    /// older callers keep the previous behaviour.
    #[serde(default)]
    disclosure: tune::BriefDisclosure,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TuningProgress {
    phase: String,
    message: String,
    trial: Option<tune::TuningTrial>,
}

/// The data-sent disclosure list for cloud tuning (audit S-20.I1): Rust owns
/// the list and a test keeps it in sync with the brief's wire fields.
#[tauri::command]
pub(crate) fn tune_disclosure_list() -> Vec<tune::DisclosureSection> {
    tune::disclosure_sections()
}

#[tauri::command]
pub(crate) async fn start_tuning(
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
            objective: "Maximise measured short-prompt decode throughput at the target allocated context while the server starts and answers. Quality and latency are not measured.",
            target_context: request.target_context,
            hardware: &hardware,
            system_ram_bytes,
            gguf: gguf.as_ref(),
            capabilities: &capabilities,
            companions: &request.companions,
            max_trials: request.max_trials,
            measured_tokens: request.tokens,
            measured_repeats: request.repeats,
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
            disclosure: request.disclosure,
            home_dir: None,
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
            deadline: Some(std::time::Instant::now() + Duration::from_secs(tune::TUNING_DEADLINE_SECS)),
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

pub(crate) fn clear_tuning(state: &tauri::State<'_, AppState>) {
    if let Ok(mut tuning) = state.tuning.lock() {
        *tuning = None;
    }
}

#[tauri::command]
pub(crate) fn cancel_tuning(state: tauri::State<AppState>) -> Result<bool, String> {
    cancel_tuning_with(&state)
}

/// Shared body for the command and its regression test. Cancelling stores the
/// cancel flag the running tuning session polls; a request with no session
/// running reports `false` instead of pretending something was stopped.
pub(crate) fn cancel_tuning_with(state: &AppState) -> Result<bool, String> {
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

pub(crate) fn system_ram_bytes() -> Option<u64> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// QC2 regression (S-27 slice 3c): the mutation "cancel stores false
    /// instead of true" passed every test before this one existed. A cancel
    /// request must set the flag the session polls, and must report no-op
    /// when no session is running.
    #[test]
    fn cancel_tuning_sets_the_flag_the_session_polls() {
        let state = AppState::default();
        assert!(!cancel_tuning_with(&state).unwrap());

        let flag = Arc::new(AtomicBool::new(false));
        *state.tuning.lock().unwrap() = Some(flag.clone());
        assert!(cancel_tuning_with(&state).unwrap());
        assert!(flag.load(Ordering::Relaxed), "cancel must set the flag");
        assert!(cancel_tuning_with(&state).unwrap());
        assert!(flag.load(Ordering::Relaxed));
    }
}
