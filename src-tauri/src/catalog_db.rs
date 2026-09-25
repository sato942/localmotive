//! Local SQLite mirror of the verified catalog, plus local user overrides.
//!
//! The signed JSON stays the network contract: every byte mirrored here comes
//! from a document that already passed Ed25519 verification and
//! [`super::catalog::parse_catalog`]. SQLite is a local query cache only. It
//! never authorizes a download, never replaces signature checks, and never
//! leaves this PC. No server is involved.
//!
//! Layout: one row per catalog file, keyed by lowercase filename (the client
//! downloads into one shared folder, so filenames are unique across the whole
//! catalog). `user_sourced` marks rows the user added locally; network rows
//! always carry `user_sourced = 0`. A corrupt or migrated database rebuilds
//! from verified bytes instead of failing the catalog tab.

use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

use hmac::Mac;

use super::catalog::{Catalog, CatalogFile, CatalogModel};

/// Current local database schema. Bumped only with a migration in
/// [`migrate_catalog_db`]: version 1 gains the override integrity column in
/// place, older files rebuild.
pub const CATALOG_DB_SCHEMA_VERSION: u32 = 2;

/// Local user-added catalog entries live in the same SQLite mirror with
/// `user_sourced = 1`. They are never written to the signed artifact, never
/// sent to the network, and never pass signature verification: downloads of
/// user rows still require the exact SHA-256 the user supplied.
pub const MAX_USER_OVERRIDE_MODELS: usize = 200;
/// Upper bound for rows read from the mirror database: beyond
/// this the file is treated as corrupt and the recovery path rebuilds it.
pub const MAX_CATALOG_MIRROR_ROWS: usize = 5000;
pub const MAX_USER_OVERRIDE_TEXT_LEN: usize = 512;
pub const MAX_USER_OVERRIDE_TAGS: usize = 32;
pub const MAX_USER_OVERRIDE_TAG_TEXT_LEN: usize = 256;
pub const MAX_USER_OVERRIDE_DATE_LEN: usize = 64;
pub const MAX_USER_OVERRIDE_QUANT_LEN: usize = 64;
pub const MAX_USER_OVERRIDE_REVISION_LEN: usize = 128;
/// Bound on the whole serialized override payload, independent of field and
/// row caps: one paste cannot inflate the database or browse responses
///.
pub const MAX_USER_OVERRIDE_SERIALIZED_BYTES: usize = 64 * 1024;

/// Where the local mirror lives, beside the JSON cache record.
pub fn catalog_db_path(root: &Path) -> PathBuf {
    root.join("catalog-mirror.sqlite")
}

/// Open the mirror, creating parent directories. Callers run
/// [`migrate_catalog_db`] next; a database that cannot migrate rebuilds from
/// verified bytes via [`recover_catalog_db_from_verified`].
pub fn open_catalog_db(root: &Path) -> Result<Connection, String> {
    std::fs::create_dir_all(root)
        .map_err(|error| format!("Could not create {}: {error}", root.display()))?;
    Connection::open(catalog_db_path(root))
        .map_err(|error| format!("Could not open the local catalog database: {error}"))
}

/// Create tables and stamp the schema version. Runs inside one transaction so
/// a crash cannot leave half a schema behind. Unknown or older versions fall
/// through to a full rebuild by the caller.
pub fn migrate_catalog_db(connection: &Connection) -> Result<(), String> {
    let version: u32 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| format!("Could not read the catalog database version: {error}"))?;
    if version == CATALOG_DB_SCHEMA_VERSION {
        return Ok(());
    }
    if version > CATALOG_DB_SCHEMA_VERSION {
        return Err(format!(
            "The local catalog database is newer (version {version}) than this build understands (version {CATALOG_DB_SCHEMA_VERSION}). Delete it to rebuild."
        ));
    }
    if version == 1 {
        // Version 2 adds integrity tags on user override file rows (audit
        // Existing rows keep their bytes with an empty tag, so reads
        // refuse them until they are re-saved; tags are never backfilled,
        // because backfilling would launder a swapped digest.
        connection
            .execute_batch(
                "BEGIN;
                 ALTER TABLE catalog_file ADD COLUMN row_mac TEXT NOT NULL DEFAULT '';
                 PRAGMA user_version = 2;
                 COMMIT;",
            )
            .map_err(|error| format!("Could not migrate the local catalog database: {error}"))?;
        return Ok(());
    }
    connection
        .execute_batch(
            "BEGIN;
             DROP TABLE IF EXISTS catalog_file;
             DROP TABLE IF EXISTS catalog_model;
             CREATE TABLE catalog_model (
                 id TEXT PRIMARY KEY,
                 repo TEXT NOT NULL,
                 family TEXT NOT NULL DEFAULT '',
                 parameters TEXT NOT NULL DEFAULT '',
                 publisher TEXT NOT NULL DEFAULT '',
                 author TEXT NOT NULL DEFAULT '',
                 summary TEXT NOT NULL DEFAULT '',
                 tags_json TEXT NOT NULL DEFAULT '[]',
                 gated INTEGER NOT NULL DEFAULT 0,
                 downloads INTEGER NOT NULL DEFAULT 0,
                 likes INTEGER NOT NULL DEFAULT 0,
                 license TEXT NOT NULL DEFAULT '',
                 pipeline_tag TEXT NOT NULL DEFAULT '',
                 library_name TEXT NOT NULL DEFAULT '',
                 architecture TEXT NOT NULL DEFAULT '',
                 last_modified TEXT NOT NULL DEFAULT '',
                 created_at TEXT NOT NULL DEFAULT '',
                 user_sourced INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE catalog_file (
                 filename_lower TEXT PRIMARY KEY,
                 model_id TEXT NOT NULL REFERENCES catalog_model(id) ON DELETE CASCADE,
                 quant TEXT NOT NULL DEFAULT '',
                 filename TEXT NOT NULL,
                 size_bytes INTEGER NOT NULL,
                 sha256 TEXT NOT NULL,
                 revision TEXT NOT NULL DEFAULT 'main',
                 last_modified TEXT NOT NULL DEFAULT '',
                 created_at TEXT NOT NULL DEFAULT '',
                 user_sourced INTEGER NOT NULL DEFAULT 0,
                 row_mac TEXT NOT NULL DEFAULT ''
             );
             CREATE INDEX catalog_file_model_idx ON catalog_file(model_id);
             PRAGMA user_version = 2;
             COMMIT;",
        )
        .map_err(|error| format!("Could not migrate the local catalog database: {error}"))?;
    Ok(())
}

/// Replace network rows with the verified catalog, preserving user rows. The
/// caller passes only signature-verified, parsed models: this function never
/// fetches, never verifies, never invents.
pub fn mirror_verified_catalog(
    connection: &mut Connection,
    models: &[CatalogModel],
) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Could not start the catalog mirror transaction: {error}"))?;
    transaction
        .execute("DELETE FROM catalog_file WHERE user_sourced = 0", [])
        .map_err(|error| format!("Could not clear the catalog mirror: {error}"))?;
    transaction
        .execute("DELETE FROM catalog_model WHERE user_sourced = 0", [])
        .map_err(|error| format!("Could not clear the catalog mirror: {error}"))?;
    for model in models {
        // Curator inserts never overwrite rows the user owns: a filename the
        // user added stays theirs.
        insert_model_guarded(&transaction, model, false)?;
    }
    transaction
        .commit()
        .map_err(|error| format!("Could not publish the catalog mirror: {error}"))?;
    Ok(())
}

/// Insert one model row (and its files) as network or user content. Used for
/// user writes, which must replace the user's own previous row verbatim.
fn insert_model(
    connection: &Connection,
    model: &CatalogModel,
    user_sourced: bool,
) -> Result<(), String> {
    insert_model_with_ownership_guard(connection, model, user_sourced, false)
}

/// Mirror-side insert for curated content: the ON CONFLICT update is skipped
/// when the existing row belongs to the user, so a refresh can never relabel
/// user rows as curator data or steal a user-owned filename.
fn insert_model_guarded(
    connection: &Connection,
    model: &CatalogModel,
    user_sourced: bool,
) -> Result<(), String> {
    insert_model_with_ownership_guard(connection, model, user_sourced, true)
}

fn insert_model_with_ownership_guard(
    connection: &Connection,
    model: &CatalogModel,
    user_sourced: bool,
    preserve_user_owned: bool,
) -> Result<(), String> {
    let flag: u32 = u32::from(user_sourced);
    let model_guard = if preserve_user_owned {
        " WHERE catalog_model.user_sourced = 0"
    } else {
        ""
    };
    let file_guard = if preserve_user_owned {
        " WHERE catalog_file.user_sourced = 0"
    } else {
        ""
    };
    connection
        .execute(
            &format!(
                "INSERT INTO catalog_model
             (id, repo, family, parameters, publisher, author, summary, tags_json,
              gated, downloads, likes, license, pipeline_tag, library_name,
              architecture, last_modified, created_at, user_sourced)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(id) DO UPDATE SET
              repo = excluded.repo, family = excluded.family,
              parameters = excluded.parameters, publisher = excluded.publisher,
              author = excluded.author, summary = excluded.summary,
              tags_json = excluded.tags_json, gated = excluded.gated,
              downloads = excluded.downloads, likes = excluded.likes,
              license = excluded.license, pipeline_tag = excluded.pipeline_tag,
              library_name = excluded.library_name,
              architecture = excluded.architecture,
              last_modified = excluded.last_modified,
              created_at = excluded.created_at,
              user_sourced = excluded.user_sourced{model_guard}",
            ),
            params![
                model.id,
                model.repo,
                model.family,
                model.parameters,
                model.publisher,
                model.author,
                model.summary,
                serde_json::to_string(&model.tags)
                    .map_err(|error| format!("Could not encode catalog tags: {error}"))?,
                u32::from(model.gated),
                i64::try_from(model.downloads)
                    .map_err(|_| format!("Catalog model {} downloads exceed the supported range.", model.id))?,
                i64::try_from(model.likes)
                    .map_err(|_| format!("Catalog model {} likes exceed the supported range.", model.id))?,
                model.license,
                model.pipeline_tag,
                model.library_name,
                model.architecture,
                model.last_modified,
                model.created_at,
                flag,
            ],
        )
        .map_err(|error| format!("Could not mirror catalog model {}: {error}", model.id))?;
    for file in &model.files {
        let size_bytes = i64::try_from(file.size_bytes).map_err(|_| {
            format!(
                "Catalog file {} is larger than the local database supports.",
                file.filename
            )
        })?;
        // user rows carry an integrity tag over every authorization
        // field so a later database edit cannot swap the digest; curated
        // rows carry none (their authority is the signed snapshot).
        let row_mac = if user_sourced {
            let key = override_integrity_key()?;
            mac_override_file(&key, &model.id, &model.repo, file)?
        } else {
            String::new()
        };
        connection
            .execute(
                &format!(
                    "INSERT INTO catalog_file
                 (filename_lower, model_id, quant, filename, size_bytes, sha256,
                  revision, last_modified, created_at, user_sourced, row_mac)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(filename_lower) DO UPDATE SET
                  model_id = excluded.model_id, quant = excluded.quant,
                  filename = excluded.filename, size_bytes = excluded.size_bytes,
                  sha256 = excluded.sha256, revision = excluded.revision,
                  last_modified = excluded.last_modified,
                  created_at = excluded.created_at,
                  user_sourced = excluded.user_sourced,
                  row_mac = excluded.row_mac{file_guard}",
                ),
                params![
                    file.filename.to_ascii_lowercase(),
                    model.id,
                    file.quant,
                    file.filename,
                    size_bytes,
                    file.sha256,
                    file.revision,
                    file.last_modified,
                    file.created_at,
                    flag,
                    row_mac,
                ],
            )
            .map_err(|error| format!("Could not mirror catalog file {}: {error}", file.filename))?;
    }
    Ok(())
}

/// Read the whole mirror back: network rows plus marked user rows. Browse,
/// filter, and sort run over this in memory via the existing
/// [`super::catalog`] functions, so SQLite is storage, not a second filter
/// implementation.
pub fn read_catalog_db_models(connection: &Connection) -> Result<Vec<CatalogModel>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, repo, family, parameters, publisher, author, summary,
                    tags_json, gated, downloads, likes, license, pipeline_tag,
                    library_name, architecture, last_modified, created_at,
                    user_sourced
             FROM catalog_model ORDER BY id",
        )
        .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i64>(8)?,
                u64::try_from(row.get::<_, i64>(9)?).ok(),
                u64::try_from(row.get::<_, i64>(10)?).ok(),
                row.get::<_, String>(11)?,
                row.get::<_, String>(12)?,
                row.get::<_, String>(13)?,
                row.get::<_, String>(14)?,
                row.get::<_, String>(15)?,
                row.get::<_, String>(16)?,
                row.get::<_, i64>(17)?,
            ))
        })
        .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
    let mut models = Vec::new();
    for row in rows {
        let (
            id,
            repo,
            family,
            parameters,
            publisher,
            author,
            summary,
            tags_json,
            gated,
            downloads,
            likes,
            license,
            pipeline_tag,
            library_name,
            architecture,
            last_modified,
            created_at,
            user_sourced,
        ) = row.map_err(|error| format!("Could not read the local catalog database: {error}"))?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let mut file_statement = connection
            .prepare(
                "SELECT quant, filename, size_bytes, sha256, revision,
                        last_modified, created_at, user_sourced
                 FROM catalog_file WHERE model_id = ?1 ORDER BY filename",
            )
            .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
        let files = file_statement
            .query_map([&id], |row| {
                Ok((
                    CatalogFile {
                        quant: row.get(0)?,
                        filename: row.get(1)?,
                        size_bytes: row.get::<_, i64>(2)? as u64,
                        sha256: row.get(3)?,
                        revision: row.get(4)?,
                        last_modified: row.get(5)?,
                        created_at: row.get(6)?,
                        user_sourced: row.get::<_, i64>(7)? != 0,
                    },
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|error| format!("Could not read the local catalog database: {error}"))?
            .map(|row| {
                row.map_err(|error| format!("Could not read the local catalog database: {error}"))
            })
            .map(|row| {
                row.and_then(|(file, raw_size)| {
                    // Faithful round-trip: a negative or absent size is a
                    // database integrity problem, not a silent wrap.
                    let size = u64::try_from(raw_size).map_err(|_| {
                        format!(
                            "Catalog file {} has an out-of-range size in the local database.",
                            file.filename
                        )
                    })?;
                    Ok(CatalogFile {
                        size_bytes: size,
                        ..file
                    })
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        models.push(CatalogModel {
            id,
            repo,
            family,
            parameters,
            publisher,
            author,
            summary,
            tags,
            gated: gated != 0,
            downloads: downloads.unwrap_or(0),
            likes: likes.unwrap_or(0),
            license,
            pipeline_tag,
            library_name,
            architecture,
            last_modified,
            created_at,
            files,
            user_sourced: user_sourced != 0,
        });
    }
    // Bound the mirror read: a tampered or corrupted database
    // must not feed an unbounded row set into the app. Exceeding the bound is
    // a corruption signal, so the caller's recovery path rebuilds the mirror.
    if models.len() > MAX_CATALOG_MIRROR_ROWS {
        return Err(format!(
            "The local catalog database contains too many rows ({}).",
            models.len()
        ));
    }
    Ok(models)
}

/// Read only the locally added rows, for salvage before a recovery rebuild.
pub fn read_user_catalog_overrides(connection: &Connection) -> Result<Vec<CatalogModel>, String> {
    let mut models = read_catalog_db_models(connection)?;
    models.retain(|model| model.user_sourced);
    Ok(models)
}

/// Quarantine an unreadable or unsupported mirror and rebuild it from
/// verified bytes, preserving every salvageable local override.
///
/// The previous database is renamed, never deleted, so corruption or a
/// downgrade cannot silently destroy user data. Returns a
/// human-readable outcome for the persistence notice; an `Err` means no
/// change happened and the previous state remains on disk.
pub fn recover_catalog_db_from_verified(
    root: &Path,
    verified: &Catalog,
    reason: &str,
) -> Result<String, String> {
    // Salvage readable user rows while the old file is still in place; a
    // damaged database often still answers this query. Failure to read is
    // reported in the outcome instead of being hidden.
    let salvage = match open_catalog_db(root) {
        Ok(connection) => read_user_catalog_overrides(&connection).unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let quarantine = quarantine_catalog_db(root)?;
    let mut connection = open_catalog_db(root)?;
    migrate_catalog_db(&connection)?;
    mirror_verified_catalog(&mut connection, &verified.models)?;
    let mut preserved = 0_usize;
    for model in &salvage {
        if save_user_catalog_override(&mut connection, model).is_ok() {
            preserved += 1;
        }
    }
    let dropped = salvage.len().saturating_sub(preserved);
    let quarantine_note = quarantine
        .map(|previous| format!(" The previous database was kept at {}.", previous.display()))
        .unwrap_or_default();
    let loss_note = if dropped > 0 {
        format!("; {dropped} unreadable or invalid override row(s) could not be preserved")
    } else {
        String::new()
    };
    Ok(format!(
        "The local catalog database was rebuilt because {reason}.{quarantine_note} {preserved} local override(s) preserved{loss_note}."
    ))
}

fn quarantine_catalog_db(root: &Path) -> Result<Option<PathBuf>, String> {
    let path = catalog_db_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let destination = root.join(format!("catalog-mirror.sqlite.quarantine-{seconds}"));
    std::fs::rename(&path, &destination).map_err(|error| {
        format!(
            "The local catalog database needs a rebuild, but the previous file could not be quarantined: {error}"
        )
    })?;
    Ok(Some(destination))
}

/// Credential Manager slot for the override integrity key. The
/// MAC key lives outside the database, so a direct database edit cannot
/// recompute row tags after swapping a digest.
pub const OVERRIDE_MAC_SERVICE: &str = "Localmotive";
const OVERRIDE_MAC_ACCOUNT: &str = "catalog-override-mac";

/// Load the override integrity key, creating and storing it on first use.
///
/// Tests set `LOCALMOTIVE_TEST_OVERRIDE_MAC` (64 hex chars) to avoid touching
/// Credential Manager; production builds never read that variable.
fn override_integrity_key() -> Result<Vec<u8>, String> {
    #[cfg(test)]
    if let Ok(hex_key) = std::env::var("LOCALMOTIVE_TEST_OVERRIDE_MAC") {
        return hex::decode(hex_key.trim())
            .map_err(|error| format!("The test override key is not hex: {error}"));
    }
    let entry = keyring::Entry::new(OVERRIDE_MAC_SERVICE, OVERRIDE_MAC_ACCOUNT)
        .map_err(|error| format!("Could not open Credential Manager: {error}"))?;
    match entry.get_password() {
        Ok(secret) if !secret.trim().is_empty() => hex::decode(secret.trim())
            .map_err(|error| format!("The stored override key is not valid: {error}")),
        _ => {
            let key: [u8; 32] = rand::random();
            entry
                .set_password(&hex::encode(key))
                .map_err(|error| format!("Could not store the override key: {error}"))?;
            Ok(key.to_vec())
        }
    }
}

/// Tag one user override file row: HMAC-SHA256 over a canonical encoding of
/// every authorization field. Length-prefixing keeps concatenations
/// unambiguous across field boundaries.
fn mac_override_file(
    key: &[u8],
    model_id: &str,
    repo: &str,
    file: &CatalogFile,
) -> Result<String, String> {
    use hmac::{Hmac, Mac};
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key)
        .map_err(|error| format!("Could not start the override tag: {error}"))?;
    feed_override_mac(&mut mac, model_id, repo, file);
    Ok(hex::encode(mac.finalize().into_bytes()))
}

fn feed_override_mac(
    mac: &mut hmac::Hmac<sha2::Sha256>,
    model_id: &str,
    repo: &str,
    file: &CatalogFile,
) {
    for field in [
        model_id,
        repo,
        &file.filename.to_ascii_lowercase(),
        &file.quant,
        &file.sha256,
        &file.revision,
    ] {
        mac.update(&(field.len() as u64).to_be_bytes());
        mac.update(field.as_bytes());
    }
    mac.update(&file.size_bytes.to_be_bytes());
}

/// Recompute the row tag and compare it in constant time. A missing tag means
/// the row predates integrity protection; a mismatch means the row changed
/// outside the application. Both refuse the row as download authorization.
fn verify_override_file(
    key: &[u8],
    model_id: &str,
    repo: &str,
    file: &CatalogFile,
    row_mac: &str,
) -> Result<(), String> {
    use hmac::{Hmac, Mac};
    if row_mac.is_empty() {
        return Err(format!(
            "Override file {} was saved before integrity protection and must be removed and re-added.",
            file.filename
        ));
    }
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key)
        .map_err(|error| format!("Could not check the override tag: {error}"))?;
    feed_override_mac(&mut mac, model_id, repo, file);
    let stored = hex::decode(row_mac.trim()).map_err(|_| {
        format!(
            "Override file {} has been modified outside the application.",
            file.filename
        )
    })?;
    mac.verify_slice(&stored).map_err(|_| {
        format!(
            "Override file {} has been modified outside the application.",
            file.filename
        )
    })
}

/// Validate one user-supplied override before it touches the mirror. The same
/// filename, repo, digest, and bound rules as network rows apply; the row is
/// additionally capped in count and text length so one paste cannot bloat the
/// local database.
pub fn validate_user_override(model: &CatalogModel) -> Result<(), String> {
    if model.id.trim().is_empty() || model.id.len() > MAX_USER_OVERRIDE_TEXT_LEN {
        return Err(format!(
            "The override id is missing or too long (maximum {MAX_USER_OVERRIDE_TEXT_LEN} bytes)."
        ));
    }
    if !super::catalog::is_valid_repo(&model.repo) {
        return Err("The Hugging Face repository must have the form owner/name.".into());
    }
    if model.downloads > i64::MAX as u64 || model.likes > i64::MAX as u64 {
        // Rejected before any mutation: the local database stores signed
        // integers, so values beyond i64::MAX have no faithful round-trip
        //.
        return Err("Override download or like counts exceed the supported range.".into());
    }
    if model.files.is_empty() || model.files.len() > 64 {
        return Err("The override must list 1 to 64 files.".into());
    }
    for text in [
        &model.family,
        &model.parameters,
        &model.publisher,
        &model.author,
        &model.summary,
        &model.license,
        &model.pipeline_tag,
        &model.library_name,
        &model.architecture,
    ] {
        if text.len() > MAX_USER_OVERRIDE_TEXT_LEN {
            return Err(format!(
                "An override field is too long (maximum {MAX_USER_OVERRIDE_TEXT_LEN} bytes)."
            ));
        }
    }
    if model.tags.len() > MAX_USER_OVERRIDE_TAGS {
        return Err(format!(
            "The override has too many tags (maximum {MAX_USER_OVERRIDE_TAGS})."
        ));
    }
    for tag in &model.tags {
        if tag.len() > MAX_USER_OVERRIDE_TAG_TEXT_LEN {
            return Err(format!(
                "A tag is too long (maximum {MAX_USER_OVERRIDE_TAG_TEXT_LEN} bytes)."
            ));
        }
    }
    for date in [&model.last_modified, &model.created_at] {
        if date.len() > MAX_USER_OVERRIDE_DATE_LEN {
            return Err(format!(
                "A date field is too long (maximum {MAX_USER_OVERRIDE_DATE_LEN} bytes)."
            ));
        }
    }
    let mut seen_files = std::collections::HashSet::new();
    for file in &model.files {
        super::catalog::validate_download_target(&model.repo, &file.filename)?;
        // The mirror keys files by lowercase name across the whole catalog,
        // so a payload with two case-variants would silently overwrite one
        // with the other.
        if !seen_files.insert(file.filename.to_ascii_lowercase()) {
            return Err(format!(
                "The override lists {} more than once (names are case-insensitive).",
                file.filename
            ));
        }
        if file.size_bytes == 0 {
            return Err(format!("Override file {} has no size.", file.filename));
        }
        if file.size_bytes > i64::MAX as u64 {
            return Err(format!(
                "Override file {} is larger than the local database supports.",
                file.filename
            ));
        }
        if file.quant.len() > MAX_USER_OVERRIDE_QUANT_LEN {
            return Err(format!(
                "A quantization label is too long (maximum {MAX_USER_OVERRIDE_QUANT_LEN} bytes)."
            ));
        }
        if file.revision.len() > MAX_USER_OVERRIDE_REVISION_LEN {
            return Err(format!(
                "A revision is too long (maximum {MAX_USER_OVERRIDE_REVISION_LEN} bytes)."
            ));
        }
        for date in [&file.last_modified, &file.created_at] {
            if date.len() > MAX_USER_OVERRIDE_DATE_LEN {
                return Err(format!(
                    "A file date field is too long (maximum {MAX_USER_OVERRIDE_DATE_LEN} bytes)."
                ));
            }
        }
        if file.sha256.len() != 64 || !file.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "Override file {} needs its exact 64-digit SHA-256.",
                file.filename
            ));
        }
    }
    let serialized = serde_json::to_vec(model)
        .map_err(|error| format!("Could not size the override payload: {error}"))?;
    if serialized.len() > MAX_USER_OVERRIDE_SERIALIZED_BYTES {
        return Err(format!(
            "The override payload is too large (maximum {} bytes).",
            MAX_USER_OVERRIDE_SERIALIZED_BYTES
        ));
    }
    Ok(())
}

/// Replace one user row and its complete file set, in one transaction, after
/// ownership and collision checks. `user_sourced = 1` marks the result so the
/// UI can say USER ADDED and network refreshes never mistake it for curator
/// data. Curator ids and curator filenames are rejected instead of being
/// overwritten, and files omitted by the new version are removed; any failure
/// rolls the whole replacement back.
pub fn save_user_catalog_override(
    connection: &mut Connection,
    model: &CatalogModel,
) -> Result<(), String> {
    validate_user_override(model)?;
    // IMMEDIATE: the row-count and collision checks must hold under
    // concurrent writers, so take the write lock before reading (DC-06).
    let transaction = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start the override transaction: {error}"))?;
    let existing: Option<i64> = transaction
        .query_row(
            "SELECT user_sourced FROM catalog_model WHERE id = ?1",
            [&model.id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
    match existing {
        Some(0) => {
            return Err(format!(
                "The id {} belongs to a curated catalog entry and cannot be replaced by a local override.",
                model.id
            ))
        }
        Some(_) => {}
        None => {
            let count: i64 = transaction
                .query_row(
                    "SELECT COUNT(*) FROM catalog_model WHERE user_sourced = 1",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
            if count as usize >= MAX_USER_OVERRIDE_MODELS {
                return Err(format!(
                    "Too many local overrides (maximum {MAX_USER_OVERRIDE_MODELS}). Remove one first."
                ));
            }
        }
    }
    for file in &model.files {
        let owner: Option<(String, i64)> = transaction
            .query_row(
                "SELECT model_id, user_sourced FROM catalog_file WHERE filename_lower = ?1",
                [file.filename.to_ascii_lowercase()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
        match owner {
            // A curated filename is curator-owned: the user must pick another
            // name instead of silently stealing the catalog row (DC-05).
            Some((_, 0)) => {
                return Err(format!(
                    "The filename {} belongs to the curated catalog and cannot be reused by a local override.",
                    file.filename
                ))
            }
            // A file of another local entry needs explicit resolution: remove
            // or rename it there first; never move it silently (DC-05).
            Some((owner_id, _)) if owner_id != model.id => {
                return Err(format!(
                    "The filename {} already belongs to the local override {}.",
                    file.filename, owner_id
                ))
            }
            _ => {}
        }
    }
    // Exact replacement: drop the previous version's files, then write the
    // complete new set. Files omitted by the new version are gone; a failure
    // above or below leaves the previous version intact.
    transaction
        .execute(
            "DELETE FROM catalog_file WHERE model_id = ?1 AND user_sourced = 1",
            [&model.id],
        )
        .map_err(|error| format!("Could not replace the local override files: {error}"))?;
    let mut marked = model.clone();
    marked.user_sourced = true;
    insert_model(&transaction, &marked, true)?;
    transaction
        .commit()
        .map_err(|error| format!("Could not publish the local override: {error}"))
}

/// Remove one user row and its files. Network rows are never deleted here: a
/// typo in the id reports missing instead of touching curator data.
pub fn remove_user_catalog_override(connection: &mut Connection, id: &str) -> Result<(), String> {
    if id.trim().is_empty() || id.len() > MAX_USER_OVERRIDE_TEXT_LEN {
        return Err("The override id is missing or too long.".into());
    }
    // One transaction: files and the model row disappear together or not at
    // all, so a crash between the statements cannot strand file rows
    //.
    let transaction = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start the override transaction: {error}"))?;
    let removed = transaction
        .execute(
            "DELETE FROM catalog_file WHERE model_id IN
             (SELECT id FROM catalog_model WHERE id = ?1 AND user_sourced = 1)",
            [&id],
        )
        .map_err(|error| format!("Could not remove the local override: {error}"))?;
    transaction
        .execute(
            "DELETE FROM catalog_model WHERE id = ?1 AND user_sourced = 1",
            [&id],
        )
        .map_err(|error| format!("Could not remove the local override: {error}"))?;
    if removed == 0 {
        let exists: Option<i64> = transaction
            .query_row("SELECT 1 FROM catalog_model WHERE id = ?1", [&id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
        if exists.is_some() {
            return Err(
                "That entry ships with the curated catalog and cannot be removed here.".into(),
            );
        }
        return Err("No local override with that id exists.".into());
    }
    transaction
        .commit()
        .map_err(|error| format!("Could not publish the local removal: {error}"))
}

/// One user override file row as read back for authorization: the model
/// identity, every authorization field, and the integrity tag.
type OverrideFileRow = (
    String,
    String,
    String,
    String,
    i64,
    String,
    String,
    String,
    String,
    String,
);

/// Resolve one user-owned file row for download authorization. Only rows
/// explicitly marked `user_sourced = 1` (on both the file and its model) are
/// returned; the exact stored SHA-256 and size are the authorization. Mutable
/// rows never become signed authority: this is the separate, validated
/// override route from the curated snapshot.
pub fn user_override_file(
    connection: &Connection,
    repo: &str,
    filename: &str,
    revision: &str,
) -> Result<Option<CatalogFile>, String> {
    let row: Option<OverrideFileRow> = connection
        .query_row(
            "SELECT m.id, m.repo, f.quant, f.filename, f.size_bytes, f.sha256, f.revision,
                    f.last_modified, f.created_at, f.row_mac
             FROM catalog_file f
             JOIN catalog_model m ON m.id = f.model_id
             WHERE m.repo = ?1 AND f.filename_lower = ?2 AND f.revision = ?3
               AND f.user_sourced = 1 AND m.user_sourced = 1",
            params![repo, filename.to_ascii_lowercase(), revision],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                ))
            },
        )
        .optional()
        .map_err(|error| format!("Could not read the local override store: {error}"))?;
    let Some((
        model_id,
        model_repo,
        quant,
        filename,
        size_bytes,
        sha256,
        revision,
        last_modified,
        created_at,
        row_mac,
    )) = row
    else {
        return Ok(None);
    };
    let size_bytes = u64::try_from(size_bytes).map_err(|_| {
        format!("Override file {filename} has an out-of-range size in the local database.")
    })?;
    let file = CatalogFile {
        quant,
        filename,
        size_bytes,
        sha256,
        revision,
        last_modified,
        created_at,
        user_sourced: true,
    };
    // the stored digest is authorization, so the row tag is
    // rechecked on every read. A swapped digest fails here even though it
    // would still pass write-time shape validation.
    let key = override_integrity_key()?;
    verify_override_file(&key, &model_id, &model_repo, &file, &row_mac)?;
    Ok(Some(file))
}

#[cfg(test)]
mod tests {
    use super::super::catalog;
    use super::*;

    fn sample_catalog() -> Catalog {
        catalog::parse_catalog(
            r#"{
              "schemaVersion": 2,
              "updated": "2026-09-02",
              "models": [
                {"id":"a","repo":"unsloth/Qwen3-Coder-30B-A3B-Instruct-GGUF","family":"Qwen3 Coder",
                 "publisher":"unsloth","parameters":"30B-A3B","tags":["code"],
                 "downloads":12706054,"likes":945,
                 "files":[{"quant":"Q4_K_M","filename":"a-Q4_K_M.gguf","sizeBytes":18556689568,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
                {"id":"b","repo":"bartowski/Llama-3.2-1B-Instruct-GGUF","family":"Llama 3.2",
                 "publisher":"bartowski","parameters":"1B","tags":["general"],
                 "downloads":147312,"likes":176,
                 "files":[{"quant":"Q4_K_M","filename":"b-Q4_K_M.gguf","sizeBytes":807694464,"sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]}
              ]
            }"#,
        )
        .unwrap()
    }

    fn sample_override(id: &str) -> CatalogModel {
        CatalogModel {
            id: id.into(),
            repo: "local/Handmade-GGUF".into(),
            family: "Handmade".into(),
            parameters: "1B".into(),
            publisher: "local".into(),
            author: "local".into(),
            summary: "Local test row.".into(),
            tags: vec!["general".into()],
            gated: false,
            downloads: 0,
            likes: 0,
            license: String::new(),
            pipeline_tag: String::new(),
            library_name: String::new(),
            architecture: String::new(),
            last_modified: String::new(),
            created_at: String::new(),
            files: vec![CatalogFile {
                quant: "Q4_K_M".into(),
                filename: format!("{id}.gguf"),
                size_bytes: 10,
                sha256: "c".repeat(64),
                revision: "main".into(),
                last_modified: String::new(),
                created_at: String::new(),
                user_sourced: false,
            }],
            user_sourced: true,
        }
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        for _ in 0..16 {
            let candidate = std::env::temp_dir().join(format!(
                "{prefix}-{}-{:016x}",
                std::process::id(),
                rand::random::<u64>()
            ));
            if std::fs::create_dir(&candidate).is_ok() {
                return candidate;
            }
        }
        panic!("could not allocate a unique temp dir for {prefix}");
    }

    #[test]
    fn mirror_stores_verified_models_and_reads_them_back() {
        // First start fills the mirror from verified bytes; browse, filter,
        // and sort read the same rows back.
        let root = unique_test_dir("localmotive-catalog-db");
        let verified = sample_catalog();
        let mirrored = seed_mirror(&root, &verified);
        assert_eq!(mirrored.len(), 2);
        let reread = read_catalog_db_models(&open_catalog_db(&root).unwrap()).unwrap();
        assert_eq!(reread.len(), 2);
        assert!(reread.iter().all(|m| !m.user_sourced));
        assert_eq!(reread[0].files.len(), 1);
        assert_eq!(reread[0].files[0].size_bytes, 18_556_689_568);
        let filtered = catalog::filter_models(
            &reread,
            &catalog::CatalogQuery {
                text: "qwen".into(),
                ..Default::default()
            },
        );
        assert_eq!(filtered.len(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn migrations_stamp_the_schema_version() {
        // Versioned migrations: a fresh database lands on
        // CATALOG_DB_SCHEMA_VERSION, and rerunning the migration is a no-op.
        let root = unique_test_dir("localmotive-catalog-db-migrate");
        let connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, CATALOG_DB_SCHEMA_VERSION);
        migrate_catalog_db(&connection).unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    /// Populate the mirror through the production functions, as a refresh
    /// does, and return the rows a browse would read.
    fn seed_mirror(root: &Path, verified: &Catalog) -> Vec<CatalogModel> {
        let mut connection = open_catalog_db(root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        mirror_verified_catalog(&mut connection, &verified.models).unwrap();
        read_catalog_db_models(&connection).unwrap()
    }

    #[test]
    fn corrupt_database_recovery_rebuilds_quarantines_and_reports() {
        // Disk corruption or an interrupted write must not leave the catalog
        // tab empty: garbage bytes go through the production recovery path,
        // the old file is quarantined (never deleted), and the outcome is
        // reported for the persistence notice.
        let root = unique_test_dir("localmotive-catalog-db-corrupt");
        std::fs::write(catalog_db_path(&root), "not a database").unwrap();
        let verified = sample_catalog();
        let outcome =
            recover_catalog_db_from_verified(&root, &verified, "the local file was corrupt")
                .unwrap();
        assert!(outcome.contains("was rebuilt because"), "{outcome}");
        assert!(
            outcome.contains("0 local override(s) preserved"),
            "{outcome}"
        );
        let quarantined = std::fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("catalog-mirror.sqlite.quarantine-")
            });
        assert!(quarantined, "the previous database must be quarantined");
        let rows = read_catalog_db_models(&open_catalog_db(&root).unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_salvages_user_rows_from_an_unsupported_newer_schema() {
        // A downgrade against a newer database must not silently destroy
        // user rows: readable overrides are exported before the rebuild and
        // re-saved afterwards.
        let root = unique_test_dir("localmotive-catalog-db-newer");
        let verified = sample_catalog();
        seed_mirror(&root, &verified);
        {
            let mut connection = open_catalog_db(&root).unwrap();
            save_user_catalog_override(&mut connection, &sample_override("mine")).unwrap();
            connection
                .execute_batch("PRAGMA user_version = 99")
                .unwrap();
        }
        let outcome =
            recover_catalog_db_from_verified(&root, &verified, "its schema is newer").unwrap();
        assert!(
            outcome.contains("1 local override(s) preserved"),
            "{outcome}"
        );
        let rows = read_catalog_db_models(&open_catalog_db(&root).unwrap()).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows
            .iter()
            .any(|model| model.id == "mine" && model.user_sourced));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn override_file_with_a_swapped_digest_is_refused_at_read() {
        // the override read trusts the stored digest, which a
        // direct database edit can swap to authorize malicious bytes. A
        // tampered row must be refused at read time, never returned.
        std::env::set_var(
            "LOCALMOTIVE_TEST_OVERRIDE_MAC",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        let root = unique_test_dir("localmotive-catalog-db-proc09");
        let verified = sample_catalog();
        seed_mirror(&root, &verified);
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        save_user_catalog_override(&mut connection, &sample_override("mine")).unwrap();
        connection
            .execute(
                "UPDATE catalog_file SET sha256 = ?1 WHERE user_sourced = 1",
                [&"f".repeat(64)],
            )
            .unwrap();
        let error = user_override_file(&connection, "local/Handmade-GGUF", "mine.gguf", "main")
            .unwrap_err();
        assert!(
            error.contains("modified") || error.contains("integrity"),
            "a swapped digest must be refused, got: {error}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn untampered_override_rows_still_authorize() {
        // an honestly saved override reads back with its
        // exact digest (the tag is transparent on the good path).
        std::env::set_var(
            "LOCALMOTIVE_TEST_OVERRIDE_MAC",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        let root = unique_test_dir("localmotive-catalog-db-proc09b");
        let verified = sample_catalog();
        seed_mirror(&root, &verified);
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        save_user_catalog_override(&mut connection, &sample_override("mine")).unwrap();
        let file = user_override_file(&connection, "local/Handmade-GGUF", "mine.gguf", "main")
            .unwrap()
            .expect("the saved override authorizes");
        assert_eq!(file.sha256, "c".repeat(64));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn legacy_override_rows_are_refused_until_resaved() {
        // rows saved before integrity protection carry no
        // tag and are refused (never backfilled: backfilling would launder a
        // swapped digest). Re-saving the same override authorizes again.
        std::env::set_var(
            "LOCALMOTIVE_TEST_OVERRIDE_MAC",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        let root = unique_test_dir("localmotive-catalog-db-proc09c");
        let verified = sample_catalog();
        seed_mirror(&root, &verified);
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        let model = sample_override("mine");
        connection
            .execute(
                "INSERT INTO catalog_model (id, repo, user_sourced) VALUES (?1, ?2, 1)",
                [&model.id, &model.repo],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO catalog_file
                 (filename_lower, model_id, quant, filename, size_bytes, sha256,
                  revision, user_sourced, row_mac)
                 VALUES (?1, ?2, '', ?3, 10, ?4, 'main', 1, '')",
                ["mine.gguf", "mine", "mine.gguf", &"c".repeat(64)],
            )
            .unwrap();
        let error = user_override_file(&connection, "local/Handmade-GGUF", "mine.gguf", "main")
            .unwrap_err();
        assert!(
            error.contains("before integrity protection"),
            "legacy rows must be refused, got: {error}"
        );
        save_user_catalog_override(&mut connection, &model).unwrap();
        user_override_file(&connection, "local/Handmade-GGUF", "mine.gguf", "main")
            .unwrap()
            .expect("the re-saved override authorizes");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn migration_adds_row_tags_without_touching_rows() {
        // a version-1 database gains the tag column in
        // place with its rows intact; old rows read as legacy (refused until
        // re-saved), and the version stamp advances.
        let root = unique_test_dir("localmotive-catalog-db-proc09d");
        let connection = open_catalog_db(&root).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE catalog_model (id TEXT PRIMARY KEY, repo TEXT NOT NULL, user_sourced INTEGER NOT NULL DEFAULT 0);
                 CREATE TABLE catalog_file (filename_lower TEXT PRIMARY KEY, model_id TEXT NOT NULL, quant TEXT NOT NULL DEFAULT '', filename TEXT NOT NULL, size_bytes INTEGER NOT NULL, sha256 TEXT NOT NULL, revision TEXT NOT NULL DEFAULT 'main', last_modified TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL DEFAULT '', user_sourced INTEGER NOT NULL DEFAULT 0);
                 INSERT INTO catalog_model (id, repo, user_sourced) VALUES ('mine', 'local/Handmade-GGUF', 1);
                 PRAGMA user_version = 1;",
            )
            .unwrap();
        migrate_catalog_db(&connection).unwrap();
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
        let kept: String = connection
            .query_row(
                "SELECT repo FROM catalog_model WHERE id = 'mine'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(kept, "local/Handmade-GGUF");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn user_override_is_marked_and_survives_network_refresh() {
        // User rows carry user_sourced so the UI says USER ADDED; mirroring a
        // fresh verified catalog preserves them instead of wiping local work.
        let root = unique_test_dir("localmotive-catalog-db-user");
        let verified = sample_catalog();
        seed_mirror(&root, &verified);
        let connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        let mut owned = connection;
        save_user_catalog_override(&mut owned, &sample_override("mine")).unwrap();
        mirror_verified_catalog(&mut owned, &verified.models).unwrap();
        let rows = read_catalog_db_models(&owned).unwrap();
        assert_eq!(rows.len(), 3);
        let mine = rows.iter().find(|m| m.id == "mine").unwrap();
        assert!(mine.user_sourced);
        assert!(rows
            .iter()
            .filter(|m| m.id != "mine")
            .all(|m| !m.user_sourced));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn user_override_rejects_bad_repos_missing_digests_and_oversize_text() {
        // Local rows never weaken verification: same repo, filename, digest,
        // and bound rules as network rows, plus count and length caps.
        let root = unique_test_dir("localmotive-catalog-db-limits");
        let connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        let mut connection = connection;
        let mut bad = sample_override("bad");
        bad.repo = "not a repo!!".into();
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.files[0].sha256 = "zz".into();
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.files[0].filename = "../evil.gguf".into();
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.summary = "x".repeat(MAX_USER_OVERRIDE_TEXT_LEN + 1);
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        assert!(remove_user_catalog_override(&mut connection, "nope").is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn user_override_remove_never_touches_network_rows() {
        // A wrong id against a network row reports curated, not missing, and
        // the network row stays in the mirror.
        let root = unique_test_dir("localmotive-catalog-db-remove");
        let verified = sample_catalog();
        seed_mirror(&root, &verified);
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        let error = remove_user_catalog_override(&mut connection, "a").unwrap_err();
        assert!(error.contains("curated"), "{error}");
        let rows = read_catalog_db_models(&connection).unwrap();
        assert_eq!(rows.len(), 2);
        save_user_catalog_override(&mut connection, &sample_override("mine")).unwrap();
        remove_user_catalog_override(&mut connection, "mine").unwrap();
        assert_eq!(read_catalog_db_models(&connection).unwrap().len(), 2);
        let _ = std::fs::remove_dir_all(root);
    }

    fn override_with_files(id: &str, files: &[&str]) -> CatalogModel {
        let mut model = sample_override(id);
        model.files = files
            .iter()
            .map(|filename| CatalogFile {
                quant: "Q4_K_M".into(),
                filename: (*filename).into(),
                size_bytes: 10,
                sha256: "d".repeat(64),
                revision: "main".into(),
                last_modified: String::new(),
                created_at: String::new(),
                user_sourced: false,
            })
            .collect();
        model
    }

    #[test]
    fn curator_id_collision_is_rejected_and_curated_rows_survive() {
        // A user row carrying a curator id must not replace the model's repo
        // or provenance, and must not leave the curated row half-owned
        //.
        let root = unique_test_dir("localmotive-dc05-id");
        let verified = sample_catalog();
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        mirror_verified_catalog(&mut connection, &verified.models).unwrap();

        let error = save_user_catalog_override(&mut connection, &sample_override("a")).unwrap_err();
        assert!(error.contains("curated"), "{error}");

        let rows = read_catalog_db_models(&connection).unwrap();
        let curated = rows.iter().find(|model| model.id == "a").unwrap();
        assert!(!curated.user_sourced);
        assert_eq!(curated.repo, "unsloth/Qwen3-Coder-30B-A3B-Instruct-GGUF");
        assert_eq!(curated.files.len(), 1);
        assert!(!curated.files[0].user_sourced);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn filename_collisions_with_curated_or_other_user_rows_are_rejected() {
        // The mirror keys files by lowercase name across the whole catalog: a
        // collision must be rejected with the owner named, never silently
        // move the file row.
        let root = unique_test_dir("localmotive-dc05-files");
        let verified = sample_catalog();
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        mirror_verified_catalog(&mut connection, &verified.models).unwrap();

        let error = save_user_catalog_override(
            &mut connection,
            &override_with_files("mine", &["a-Q4_K_M.gguf"]),
        )
        .unwrap_err();
        assert!(error.contains("curated catalog"), "{error}");
        // The curated model still owns its file.
        let rows = read_catalog_db_models(&connection).unwrap();
        let curated = rows.iter().find(|model| model.id == "a").unwrap();
        assert_eq!(curated.files.len(), 1);
        assert!(!curated.files[0].user_sourced);

        save_user_catalog_override(&mut connection, &override_with_files("one", &["F.gguf"]))
            .unwrap();
        let error =
            save_user_catalog_override(&mut connection, &override_with_files("two", &["f.gguf"]))
                .unwrap_err();
        assert!(error.contains("already belongs"), "{error}");
        // Entry one keeps its file; entry two was never created.
        let rows = read_catalog_db_models(&connection).unwrap();
        assert!(rows.iter().any(|model| model.id == "one"));
        assert!(!rows.iter().any(|model| model.id == "two"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn refresh_never_relabels_or_steals_user_owned_files() {
        // A verified refresh that contains the same lowercase filename must
        // skip the user-owned row: the user file stays user-sourced and the
        // curated model is mirrored without it.
        let root = unique_test_dir("localmotive-dc05-refresh");
        let verified = sample_catalog();
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        mirror_verified_catalog(&mut connection, &verified.models).unwrap();
        save_user_catalog_override(
            &mut connection,
            &override_with_files("mine", &["mine.gguf"]),
        )
        .unwrap();

        let colliding = catalog::parse_catalog(
            r#"{
              "schemaVersion": 2,
              "updated": "2026-09-02",
              "models": [
                {"id":"z","repo":"org/Z-GGUF","family":"Z","publisher":"org","parameters":"1B",
                 "tags":[],"downloads":1,"likes":1,
                 "files":[{"quant":"Q4_K_M","filename":"mine.gguf","sizeBytes":10,
                           "sha256":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"}]}
              ]
            }"#,
        )
        .unwrap();
        mirror_verified_catalog(&mut connection, &colliding.models).unwrap();

        let rows = read_catalog_db_models(&connection).unwrap();
        let mine = rows.iter().find(|model| model.id == "mine").unwrap();
        assert!(mine.user_sourced);
        assert_eq!(mine.files.len(), 1);
        assert!(mine.files[0].user_sourced);
        assert_eq!(mine.files[0].filename, "mine.gguf");
        let curated = rows.iter().find(|model| model.id == "z").unwrap();
        assert!(!curated.user_sourced);
        assert!(
            curated.files.is_empty(),
            "the guard must skip the user-owned filename"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn edit_replaces_the_complete_file_set() {
        // Editing to a smaller file set must remove the omitted filenames,
        // not return both versions.
        let root = unique_test_dir("localmotive-dc06-replace");
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        save_user_catalog_override(
            &mut connection,
            &override_with_files("mine", &["a0.gguf", "a1.gguf"]),
        )
        .unwrap();
        save_user_catalog_override(&mut connection, &override_with_files("mine", &["b9.gguf"]))
            .unwrap();

        let rows = read_catalog_db_models(&connection).unwrap();
        let mine = rows.iter().find(|model| model.id == "mine").unwrap();
        assert_eq!(mine.files.len(), 1);
        assert_eq!(mine.files[0].filename, "b9.gguf");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn failed_save_preserves_the_previous_complete_version() {
        // Validation failures and a locked database must leave the previous
        // record exactly as it was: no partial replacement.
        let root = unique_test_dir("localmotive-dc06-rollback");
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        save_user_catalog_override(
            &mut connection,
            &override_with_files("mine", &["a0.gguf", "a1.gguf"]),
        )
        .unwrap();

        // (a) Invalid second file: rejected before any mutation.
        let mut bad = override_with_files("mine", &["a0.gguf", "a1.gguf"]);
        bad.files[1].sha256 = "zz".into();
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());

        // (b) A held write lock fails the transaction itself.
        let holder = open_catalog_db(&root).unwrap();
        holder.execute_batch("BEGIN IMMEDIATE").unwrap();
        let locked =
            save_user_catalog_override(&mut connection, &override_with_files("mine", &["b9.gguf"]));
        assert!(locked.is_err(), "a locked database must fail the save");
        drop(holder);

        let rows = read_catalog_db_models(&connection).unwrap();
        let mine = rows.iter().find(|model| model.id == "mine").unwrap();
        let mut names: Vec<&str> = mine
            .files
            .iter()
            .map(|file| file.filename.as_str())
            .collect();
        names.sort_unstable();
        assert_eq!(names, vec!["a0.gguf", "a1.gguf"]);

        // The lock released: the same save now succeeds.
        save_user_catalog_override(&mut connection, &override_with_files("mine", &["b9.gguf"]))
            .unwrap();
        let rows = read_catalog_db_models(&connection).unwrap();
        assert_eq!(
            rows.iter()
                .find(|model| model.id == "mine")
                .unwrap()
                .files
                .len(),
            1
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn validation_rejects_long_and_duplicate_metadata_before_mutation() {
        let root = unique_test_dir("localmotive-dc06-bounds");
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();

        let mut bad = sample_override("bad");
        bad.tags = vec!["t".into(); MAX_USER_OVERRIDE_TAGS + 1];
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.tags = vec!["t".repeat(MAX_USER_OVERRIDE_TAG_TEXT_LEN + 1)];
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.last_modified = "2".repeat(MAX_USER_OVERRIDE_DATE_LEN + 1);
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.files[0].last_modified = "2".repeat(MAX_USER_OVERRIDE_DATE_LEN + 1);
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.files[0].quant = "Q".repeat(MAX_USER_OVERRIDE_QUANT_LEN + 1);
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.files[0].revision = "r".repeat(MAX_USER_OVERRIDE_REVISION_LEN + 1);
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = override_with_files("bad", &["F.gguf", "f.gguf"]);
        bad.files[1].quant = "Q8_0".into();
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.downloads = u64::MAX;
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.likes = u64::MAX;
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.summary = "x".repeat(MAX_USER_OVERRIDE_SERIALIZED_BYTES);
        assert!(save_user_catalog_override(&mut connection, &bad).is_err());

        // Nothing above mutated the mirror.
        assert!(read_catalog_db_models(&connection).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn user_cap_is_origin_aware_and_holds_for_concurrent_new_entries() {
        let root = unique_test_dir("localmotive-dc06-cap");
        let mut connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        // Fill the cap directly; the cap counts user rows only, and editing an
        // existing entry stays allowed at the cap.
        for index in 0..MAX_USER_OVERRIDE_MODELS {
            connection
                .execute(
                    "INSERT INTO catalog_model (id, repo, user_sourced) VALUES (?1, 'local/fill', 1)",
                    [format!("fill-{index}")],
                )
                .unwrap();
        }
        let error =
            save_user_catalog_override(&mut connection, &sample_override("overflow")).unwrap_err();
        assert!(error.contains("Too many local overrides"), "{error}");
        save_user_catalog_override(&mut connection, &sample_override("fill-0")).unwrap();
        drop(connection);

        // Near the cap, two concurrent new entries cannot both win.
        let root2 = unique_test_dir("localmotive-dc06-race");
        let seed = open_catalog_db(&root2).unwrap();
        migrate_catalog_db(&seed).unwrap();
        for index in 0..MAX_USER_OVERRIDE_MODELS - 1 {
            seed.execute(
                "INSERT INTO catalog_model (id, repo, user_sourced) VALUES (?1, 'local/fill', 1)",
                [format!("fill-{index}")],
            )
            .unwrap();
        }
        drop(seed);
        let path = catalog_db_path(&root2);
        let handles: Vec<_> = ["race-a", "race-b"]
            .into_iter()
            .map(|id| {
                let path = path.clone();
                std::thread::spawn(move || {
                    let model = CatalogModel {
                        id: id.into(),
                        repo: "local/Handmade-GGUF".into(),
                        family: "Handmade".into(),
                        parameters: "1B".into(),
                        publisher: "local".into(),
                        author: "local".into(),
                        summary: String::new(),
                        tags: vec![],
                        gated: false,
                        downloads: 0,
                        likes: 0,
                        license: String::new(),
                        pipeline_tag: String::new(),
                        library_name: String::new(),
                        architecture: String::new(),
                        last_modified: String::new(),
                        created_at: String::new(),
                        files: vec![CatalogFile {
                            quant: "Q4_K_M".into(),
                            filename: format!("{id}.gguf"),
                            size_bytes: 10,
                            sha256: "d".repeat(64),
                            revision: "main".into(),
                            last_modified: String::new(),
                            created_at: String::new(),
                            user_sourced: false,
                        }],
                        user_sourced: false,
                    };
                    let mut connection = Connection::open(&path).unwrap();
                    let result = save_user_catalog_override(&mut connection, &model);
                    drop(connection);
                    result.is_ok()
                })
            })
            .collect();
        let successes = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|joined| *joined)
            .count();
        assert_eq!(
            successes, 1,
            "exactly one concurrent new entry may win the last slot"
        );
        let connection = open_catalog_db(&root2).unwrap();
        migrate_catalog_db(&connection).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM catalog_model WHERE user_sourced = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count as usize, MAX_USER_OVERRIDE_MODELS);
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(root2);
    }
}
