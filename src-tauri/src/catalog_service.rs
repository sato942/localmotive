//! Catalog store/cache and browse command family (audit S-27 I1).
//!
//! Extracted from `lib.rs` so the catalog's load/fetch/mirror/browse
//! ownership lives behind one tested boundary: these are the same Tauri
//! commands, and `lib.rs` keeps only the `generate_handler!` registration.
//! Rust stays authoritative — every command here delegates to `catalog` /
//! `catalog_db` functions, which carry the behavior tests.
use crate::{catalog, catalog_db, AppState};

pub(crate) fn catalog_cache_root(app: &tauri::AppHandle) -> std::path::PathBuf {
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
pub(crate) async fn load_model_catalog(
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
pub(crate) fn publish_loaded_catalog(
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
pub(crate) async fn fetch_model_catalog(
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
                        sequence: None,
                        expires: None,
                        source: String::new(),
                        note: String::new(),
                        models: mirror_models,
                        dropped: Vec::new(),
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
pub(crate) fn catalog_local_models(
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
pub(crate) fn read_local_catalog_rows(
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

pub(crate) fn fallback_rows(
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
pub(crate) fn save_user_catalog_override(
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
pub(crate) fn remove_user_catalog_override(
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
pub(crate) fn filter_catalog(
    models: catalog::IpcCatalogModels,
    query: catalog::CatalogQuery,
) -> Result<Vec<catalog::CatalogModel>, String> {
    let models = models.0;
    catalog::validate_catalog_payload(&models)?;
    catalog::validate_catalog_query(&query, models.len())?;
    Ok(catalog::filter_models(&models, &query))
}

#[tauri::command]
pub(crate) fn catalog_facets(
    models: catalog::IpcCatalogModels,
) -> Result<(Vec<String>, Vec<String>), String> {
    let models = models.0;
    catalog::validate_catalog_payload(&models)?;
    catalog::validate_facet_models(models.len())?;
    Ok(catalog::facets(&models))
}

#[tauri::command]
pub(crate) fn catalog_rich_facets(
    models: catalog::IpcCatalogModels,
) -> Result<catalog::CatalogFacets, String> {
    let models = models.0;
    catalog::validate_catalog_payload(&models)?;
    catalog::validate_facet_models(models.len())?;
    Ok(catalog::rich_facets(&models))
}

#[tauri::command]
pub(crate) fn catalog_fit_budget(
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
pub(crate) fn hf_token_status() -> catalog::TokenStatus {
    catalog::hf_token_status()
}

#[tauri::command]
pub(crate) fn save_hf_token(token: String) -> Result<catalog::TokenStatus, String> {
    catalog::save_hf_token(&token)
}

#[tauri::command]
pub(crate) fn clear_hf_token() -> Result<catalog::TokenStatus, String> {
    catalog::clear_hf_token()
}
