//! Supervised server lifecycle command family (audit S-27 I1, slice 3a).
//!
//! Extracted from `lib.rs`: these are the same Tauri commands. The shared
//! operation coordinator (`reserve_operation`, `OperationOwner`) stays in
//! `lib.rs` because every job family uses it, and this module consumes it
//! rather than restating any ownership rule.
use crate::core::LaunchProfile;
use crate::{
    active_operation_owner, bounded_log_tail, local_client, reserve_operation, spawn_server,
    status_from, stop_owner_conflict, update_launch_effective_context,
    wait_until_healthy_cancellable, AppState, ManagedServer, OperationOwner, ServerStatus,
    StartingServer, STARTUP_STOP_DEADLINE_SECS,
};
use crate::{evidence, LaunchValidation};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::Manager;

/// The lifecycle phase observable while a start is pending (audit IPC-01).
pub(crate) fn server_phase(state: &AppState) -> &'static str {
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
pub(crate) async fn start_server(
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
pub(crate) fn startup_is_current(state: &AppState, operation_id: u64, cancel: &AtomicBool) -> bool {
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
pub(crate) fn start_server_worker(
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
    let (mut child, mut validation, log_path, execution_lease, mut log_drains) =
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
        log_drains: std::mem::take(&mut log_drains),
    });
    clear_starting(&state);
    Ok(status_from(&mut slot))
}

#[tauri::command]
pub(crate) async fn stop_server(
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
            // The child holds no more output: join the bounded-log drains so
            // the retained file is complete (audit OPS-01). A drain that
            // somehow lingers is detached rather than blocking Stop.
            let deadline = Instant::now() + Duration::from_secs(2);
            for drain in server.log_drains.drain(..) {
                while !drain.is_finished() && Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(10));
                }
                if drain.is_finished() {
                    let _ = drain.join();
                }
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
pub(crate) fn server_status(state: tauri::State<AppState>) -> Result<ServerStatus, String> {
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
pub(crate) struct ValidatedServerSnapshot {
    pub(crate) pid: u32,
    pub(crate) profile: LaunchProfile,
    pub(crate) validation: LaunchValidation,
}

pub(crate) fn validated_server_snapshot(
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
pub(crate) fn read_server_log(state: tauri::State<AppState>) -> Result<String, String> {
    let slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable")?;
    let Some(server) = slot.as_ref() else {
        return Ok("No server is running.".into());
    };
    Ok(bounded_log_tail(&server.log_path))
}
