//! Supervised server lifecycle command family.
//!
//! Extracted from `lib.rs`: these are the same Tauri commands. The shared
//! operation coordinator (`reserve_operation`, `OperationOwner`) stays in
//! `lib.rs` because every job family uses it, and this module consumes it
//! rather than restating any ownership rule.
use crate::core::LaunchProfile;
use crate::{
    active_operation_owner, bounded_log_tail, local_client, reject_if_benchmark_active,
    reserve_operation, spawn_server, status_from, stop_owner_conflict,
    update_launch_effective_context, wait_until_healthy_cancellable, AppState, ManagedServer,
    OperationOwner, ServerStatus, StartingServer, STARTUP_STOP_DEADLINE_SECS,
};
use crate::{evidence, LaunchValidation};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[tauri::command]
pub(crate) async fn start_server(
    app: tauri::AppHandle,
    profile: LaunchProfile,
    state: tauri::State<'_, AppState>,
) -> Result<ServerStatus, String> {
    // One owner at a time: a running server, benchmark, or tuning session
    // holds the reservation.
    let reservation = reserve_operation(&state.operations, OperationOwner::Server)?;
    // Aborted-command shape (review deleg_16c0e72a): an abandoned benchmark
    // worker can still infer after its reservation released. The held slot
    // refuses replacement server work until the run settles.
    reject_if_benchmark_active(&state)?;
    let operation_id = reservation.generation;
    let cancel = Arc::new(AtomicBool::new(false));
    {
        // One short lock claims the operation and publishes the starting
        // state; the readiness wait runs on a blocking worker without any
        // server lock held.
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
/// still names this operation. A late completion from an
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
///.
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
    // the readiness client is built before the spawn, so a client
    // failure returns while there is still no child to reap and no slot to
    // clear. It must not be constructed between spawn and the health wait.
    let client = local_client(&profile)?;
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
        &client,
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
    let request = prepare_stop(&state)?;
    let worker_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        stop_server_worker(&worker_app.state::<AppState>(), request)
    })
    .await
    .map_err(|error| format!("Server stop task failed: {error}"))?
}

const STALE_STOP: &str =
    "The managed server operation changed before Stop. Check the current run before requesting Stop again.";

fn prepare_stop(state: &AppState) -> Result<u64, String> {
    let coordinator = state
        .operations
        .lock()
        .map_err(|_| "Operation state is unavailable")?;
    if let Some(conflict) = stop_owner_conflict(coordinator.active.map(|(owner, _)| owner)) {
        return Err(conflict);
    }
    Ok(coordinator.generation)
}

// Check the epoch after atomic acquisition, not only before waiting. An
// operation can start and finish between the last observation and this claim.
fn reserve_stop(
    state: &AppState,
    generation: u64,
) -> Result<crate::OperationReservation<'_>, String> {
    let reservation = reserve_operation(&state.operations, OperationOwner::Server)?;
    if reservation.generation != generation.wrapping_add(1) {
        return Err(STALE_STOP.into());
    }
    Ok(reservation)
}

fn stop_server_worker(state: &AppState, generation: u64) -> Result<ServerStatus, String> {
    let deadline = Instant::now() + Duration::from_secs(STARTUP_STOP_DEADLINE_SECS);
    loop {
        let active = {
            let coordinator = state
                .operations
                .lock()
                .map_err(|_| "Operation state is unavailable")?;
            if coordinator.generation != generation {
                return Err(STALE_STOP.into());
            }
            coordinator.active
        };
        if let Some(conflict) = stop_owner_conflict(active_operation_owner(&state.operations)) {
            return Err(conflict);
        }
        let pending = {
            let starting = state
                .starting
                .lock()
                .map_err(|_| "Server startup state is unavailable")?;
            if let Some(entry) = starting.as_ref() {
                if entry.operation_id != generation {
                    return Err(STALE_STOP.into());
                }
                entry.cancel.store(true, Ordering::Relaxed);
            }
            starting.is_some()
        };
        if active.is_none() && !pending {
            break;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "The server operation did not stop within {STARTUP_STOP_DEADLINE_SECS} seconds. Wait for its startup and ownership state to clear before retrying Stop."
            ));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let _reservation = reserve_stop(state, generation)?;
    // Take the server out of the slot under the lock, then terminate and
    // join outside it: status reads and competing lifecycle calls keep the
    // mutex while Stop resolves the child. The reservation stays held
    // across the transition, so no other operation can claim the slot.
    let taken = {
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable".to_string())?;
        slot.take()
    };
    if let Some(mut server) = taken {
        if !server.child.terminate_and_wait() {
            // The child is still alive: restore the slot so the running
            // server stays tracked, then report the failure.
            let mut slot = state
                .server
                .lock()
                .map_err(|_| "Server state is unavailable".to_string())?;
            *slot = Some(server);
            return Err("The contained llama-server process tree did not stop".into());
        }
        // The child holds no more output: join the bounded-log drains so
        // the retained file is complete. A drain that
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
    let mut slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable".to_string())?;
    Ok(status_from(&mut slot))
}

#[tauri::command]
pub(crate) fn server_status(state: tauri::State<AppState>) -> Result<ServerStatus, String> {
    let mut slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable")?;
    let mut status = status_from(&mut slot);
    if !status.running
        && state
            .starting
            .lock()
            .map(|starting| starting.is_some())
            .unwrap_or(false)
    {
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

#[cfg(test)]
mod stop_ownership_tests {
    use super::*;

    #[cfg(windows)]
    fn owned_server(state: &AppState) -> u32 {
        use std::process::Stdio;
        // A contained process stand-in is sufficient: Stop must preserve
        // ownership without reading model data or contacting inference.
        let mut command = crate::proc::hidden_command("powershell.exe");
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 600",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = crate::proc::spawn_contained_process(&mut command).unwrap();
        assert!(child.try_wait().unwrap().is_none());
        let pid = child.id();
        *state.server.lock().unwrap() = Some(ManagedServer {
            child,
            profile: LaunchProfile::default(),
            command: "contained Stop fixture".into(),
            validation: crate::release_security_tests::launch_validation_fixture(),
            log_path: String::new(),
            started_at: 0,
            runtime_lease: None,
            log_drains: Vec::new(),
        });
        pid
    }

    #[test]
    fn queued_stop_refuses_an_owner_that_arrived_after_preparation() {
        // The Stop precheck passed before this competing operation began.
        // Its queued worker must still refuse instead of reporting success.
        for owner in [
            OperationOwner::Benchmark,
            OperationOwner::ColdBenchmark,
            OperationOwner::Tuning,
        ] {
            let state = AppState::default();
            let request = prepare_stop(&state).unwrap();
            let competing = reserve_operation(&state.operations, owner).unwrap();
            let outcome = stop_server_worker(&state, request);
            assert!(outcome.is_err(), "queued Stop bypassed {owner:?} ownership");
            assert!(competing.is_current());
        }
    }

    #[test]
    #[cfg(windows)]
    fn queued_stop_preserves_a_completed_replacement_process() {
        // The replacement finished startup before the old Stop worker ran.
        // Checking only the current owner sees idle and kills the wrong child.
        let state = AppState::default();
        let request = prepare_stop(&state).unwrap();
        let replacement = reserve_operation(&state.operations, OperationOwner::Server).unwrap();
        let pid = owned_server(&state);
        drop(replacement);

        let outcome = stop_server_worker(&state, request);
        let mut slot = state.server.lock().unwrap();
        let preserved = slot.as_mut().is_some_and(|server| {
            server.child.id() == pid && server.child.try_wait().unwrap().is_none()
        });
        // Cleanup only the fixture that this test owns, even on RED.
        if let Some(mut server) = slot.take() {
            assert!(server.child.terminate_and_wait());
        }
        assert!(preserved, "queued Stop terminated the replacement process");
        assert!(outcome.is_err(), "stale Stop must report refusal");
    }

    #[test]
    fn stop_claim_rechecks_the_epoch_after_a_completed_interleaving() {
        // This is the interval after the worker's wait check and before its
        // atomic claim. An idle owner alone cannot detect completed work.
        let state = AppState::default();
        let request = prepare_stop(&state).unwrap();
        drop(reserve_operation(&state.operations, OperationOwner::Benchmark).unwrap());
        assert!(reserve_stop(&state, request).is_err());
        assert_eq!(active_operation_owner(&state.operations), None);
    }

    #[test]
    fn queued_stop_does_not_signal_a_replacement_startup() {
        let state = AppState::default();
        let request = prepare_stop(&state).unwrap();
        let replacement = reserve_operation(&state.operations, OperationOwner::Server).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        *state.starting.lock().unwrap() = Some(StartingServer {
            operation_id: replacement.generation,
            cancel: cancel.clone(),
        });
        assert!(stop_server_worker(&state, request).is_err());
        assert!(!cancel.load(Ordering::Relaxed));
        assert!(replacement.is_current());
        assert!(state.starting.lock().unwrap().is_some());
    }

    #[test]
    fn stop_rejects_a_starting_slot_from_a_different_epoch() {
        // Represent a replacement slot observed after an earlier coordinator
        // read. Never signal that slot merely because the owner was Server.
        let state = AppState::default();
        let request = prepare_stop(&state).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        *state.starting.lock().unwrap() = Some(StartingServer {
            operation_id: request.wrapping_add(1),
            cancel: cancel.clone(),
        });
        assert!(stop_server_worker(&state, request).is_err());
        assert!(!cancel.load(Ordering::Relaxed));
    }

    #[test]
    fn stop_cancels_the_original_startup_and_waits_for_its_lease() {
        for publish_before_request in [false, true] {
            let state = AppState::default();
            let startup = reserve_operation(&state.operations, OperationOwner::Server).unwrap();
            let cancel = Arc::new(AtomicBool::new(false));
            let publish = || {
                *state.starting.lock().unwrap() = Some(StartingServer {
                    operation_id: startup.generation,
                    cancel: cancel.clone(),
                });
            };
            if publish_before_request {
                publish();
            }
            let request = prepare_stop(&state).unwrap();
            if !publish_before_request {
                publish();
            }
            std::thread::scope(|scope| {
                let state_ref = &state;
                let cancel_ref = &cancel;
                let finishing = scope.spawn(move || {
                    let deadline = Instant::now() + Duration::from_secs(30);
                    while !cancel_ref.load(Ordering::Relaxed) && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    let signalled = cancel_ref.load(Ordering::Relaxed);
                    // Only the original startup's worker clears its state.
                    *state_ref.starting.lock().unwrap() = None;
                    drop(startup);
                    signalled
                });
                let outcome = stop_server_worker(&state, request);
                assert!(
                    finishing.join().unwrap(),
                    "original startup was not cancelled"
                );
                assert!(!outcome.unwrap().running);
            });
            assert_eq!(active_operation_owner(&state.operations), None);
            assert!(state.starting.lock().unwrap().is_none());
        }
    }

    #[test]
    fn stop_retains_its_reservation_while_waiting_for_the_server_slot() {
        let state = AppState::default();
        let request = prepare_stop(&state).unwrap();
        std::thread::scope(|scope| {
            let slot = state.server.lock().unwrap();
            let state_ref = &state;
            let (send, done) = std::sync::mpsc::channel();
            let stopping = scope.spawn(move || {
                let outcome = stop_server_worker(state_ref, request);
                send.send(outcome).unwrap();
            });
            let deadline = Instant::now() + Duration::from_secs(30);
            while state.operations.lock().unwrap().generation == request
                && Instant::now() < deadline
            {
                std::thread::sleep(Duration::from_millis(1));
            }
            // The server lock is still held by this test, so Stop cannot
            // finish. Its operation lease must remain held throughout.
            let early = done.recv_timeout(Duration::from_secs(2));
            let retained =
                active_operation_owner(&state.operations) == Some(OperationOwner::Server);
            let refused = reserve_operation(&state.operations, OperationOwner::Benchmark).is_err();
            drop(slot);
            stopping.join().unwrap();
            let blocked = matches!(early, Err(std::sync::mpsc::RecvTimeoutError::Timeout));
            let outcome =
                early.unwrap_or_else(|_| done.recv_timeout(Duration::from_secs(30)).unwrap());
            assert!(
                blocked && retained && refused,
                "Stop released its reservation before the server slot"
            );
            assert!(!outcome.unwrap().running);
        });
        assert_eq!(active_operation_owner(&state.operations), None);
    }

    #[test]
    fn idle_stop_supports_the_coordinator_wraparound() {
        let state = AppState::default();
        state.operations.lock().unwrap().generation = u64::MAX;
        let request = prepare_stop(&state).unwrap();
        assert!(!stop_server_worker(&state, request).unwrap().running);
        assert_eq!(state.operations.lock().unwrap().generation, 0);
        assert_eq!(active_operation_owner(&state.operations), None);
    }

    #[test]
    #[cfg(windows)]
    fn current_stop_reaps_its_owned_process() {
        let state = AppState::default();
        owned_server(&state);
        let request = prepare_stop(&state).unwrap();
        assert!(!stop_server_worker(&state, request).unwrap().running);
        assert!(state.server.lock().unwrap().is_none());
        assert_eq!(active_operation_owner(&state.operations), None);
    }
}
