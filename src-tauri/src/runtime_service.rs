//! Managed-runtime setup, install and health command family (audit S-27 I1).
//!
//! Extracted from `lib.rs`: these are the same Tauri commands, and the
//! runtime/health ownership (download verification, execution authorization,
//! supervised health runs) stays in `runtime.rs` and `health.rs` with their
//! tests — this module is the command surface over that authority.
use crate::{health, runtime, AppState, ExclusiveOperation, RuntimeSetupResponse};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;

#[tauri::command]
pub(crate) async fn load_runtime_setup(
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
pub(crate) fn detect_hardware() -> runtime::HardwareInfo {
    runtime::detect_hardware()
}

/// Verification instrumentation for the runtime setup screens: jobs run,
/// requests coalesced, bytes hashed, jobs cancelled (audit RT-06 I4).
#[tauri::command]
pub(crate) fn runtime_verification_stats() -> runtime::RuntimeVerificationStats {
    runtime::verification_stats()
}

#[tauri::command]
pub(crate) async fn fetch_runtime_catalog(
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
pub(crate) fn managed_runtime_root() -> Result<String, String> {
    Ok(runtime::managed_runtime_install_root()
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
pub(crate) async fn install_managed_runtime(
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
pub(crate) fn cancel_managed_runtime_install(state: tauri::State<'_, AppState>) -> bool {
    let active = state.runtime_install.lock().unwrap();
    if let Some(cancel) = active.as_ref() {
        cancel.store(true, Ordering::Relaxed);
        true
    } else {
        false
    }
}

#[tauri::command]
pub(crate) async fn check_managed_runtime_health(
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
pub(crate) async fn repair_health_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let cancel = {
        let mut slot = state
            .health_repair
            .lock()
            .map_err(|_| "The health model repair lock is poisoned".to_string())?;
        if slot.is_some() {
            return Err("A health model repair is already running.".into());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        *slot = Some(Arc::clone(&cancel));
        cancel
    };
    let result = tauri::async_runtime::spawn_blocking(move || {
        runtime::repair_pinned_health_model(Arc::clone(&cancel), |downloaded, total| {
            let _ = app.emit(
                "health-model-progress",
                health::HealthModelProgress {
                    install_key: "repair".into(),
                    downloaded,
                    total,
                },
            );
        })
    })
    .await
    .map_err(|error| format!("The health model repair task failed: {error}"))?;
    if let Ok(mut slot) = state.health_repair.lock() {
        *slot = None;
    }
    result.map(|_| ())
}

#[tauri::command]
pub(crate) fn cancel_health_model_repair(state: tauri::State<'_, AppState>) -> bool {
    let Ok(slot) = state.health_repair.lock() else {
        return false;
    };
    match slot.as_ref() {
        Some(flag) => {
            flag.store(true, Ordering::Relaxed);
            true
        }
        None => false,
    }
}

#[tauri::command]
pub(crate) fn cancel_managed_runtime_health(state: tauri::State<'_, AppState>) -> bool {
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
