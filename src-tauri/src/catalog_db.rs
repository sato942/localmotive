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

use super::catalog::{Catalog, CatalogFile, CatalogModel};

/// Current local database schema. Bumped only with a migration in
/// [`migrate_catalog_db`]; old files rebuild rather than partially upgrade.
pub const CATALOG_DB_SCHEMA_VERSION: u32 = 1;

/// Local user-added catalog entries live in the same SQLite mirror with
/// `user_sourced = 1`. They are never written to the signed artifact, never
/// sent to the network, and never pass signature verification: downloads of
/// user rows still require the exact SHA-256 the user supplied.
pub const MAX_USER_OVERRIDE_MODELS: usize = 200;
pub const MAX_USER_OVERRIDE_TEXT_LEN: usize = 512;

/// Where the local mirror lives, beside the JSON cache record.
pub fn catalog_db_path(root: &Path) -> PathBuf {
    root.join("catalog-mirror.sqlite")
}

/// Open the mirror, creating parent directories. Callers run
/// [`migrate_catalog_db`] next; a database that cannot migrate rebuilds from
/// verified bytes via [`rebuild_catalog_db_from_verified`].
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
                 user_sourced INTEGER NOT NULL DEFAULT 0
             );
             CREATE INDEX catalog_file_model_idx ON catalog_file(model_id);
             PRAGMA user_version = 1;
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
        insert_model(&transaction, model, false)?;
    }
    transaction
        .commit()
        .map_err(|error| format!("Could not publish the catalog mirror: {error}"))?;
    Ok(())
}

fn insert_model(
    connection: &Connection,
    model: &CatalogModel,
    user_sourced: bool,
) -> Result<(), String> {
    let flag: u32 = u32::from(user_sourced);
    connection
        .execute(
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
              user_sourced = excluded.user_sourced",
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
                model.downloads as i64,
                model.likes as i64,
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
        connection
            .execute(
                "INSERT INTO catalog_file
                 (filename_lower, model_id, quant, filename, size_bytes, sha256,
                  revision, last_modified, created_at, user_sourced)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(filename_lower) DO UPDATE SET
                  model_id = excluded.model_id, quant = excluded.quant,
                  filename = excluded.filename, size_bytes = excluded.size_bytes,
                  sha256 = excluded.sha256, revision = excluded.revision,
                  last_modified = excluded.last_modified,
                  created_at = excluded.created_at,
                  user_sourced = excluded.user_sourced",
                params![
                    file.filename.to_ascii_lowercase(),
                    model.id,
                    file.quant,
                    file.filename,
                    file.size_bytes as i64,
                    file.sha256,
                    file.revision,
                    file.last_modified,
                    file.created_at,
                    flag,
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
                row.get::<_, i64>(9)?,
                row.get::<_, i64>(10)?,
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
                        last_modified, created_at
                 FROM catalog_file WHERE model_id = ?1 ORDER BY filename",
            )
            .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
        let files = file_statement
            .query_map([&id], |row| {
                Ok(CatalogFile {
                    quant: row.get(0)?,
                    filename: row.get(1)?,
                    size_bytes: row.get::<_, i64>(2)? as u64,
                    sha256: row.get(3)?,
                    revision: row.get(4)?,
                    last_modified: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|error| format!("Could not read the local catalog database: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
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
            downloads: downloads as u64,
            likes: likes as u64,
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
    Ok(models)
}

/// Delete the mirror file when it is corrupt or from a newer build, then
/// rebuild it from already-verified bytes. Returns the mirrored models so the
/// caller can serve first start without another parse.
pub fn rebuild_catalog_db_from_verified(
    root: &Path,
    verified: &Catalog,
) -> Result<Vec<CatalogModel>, String> {
    let path = catalog_db_path(root);
    let _ = std::fs::remove_file(&path);
    let mut connection = open_catalog_db(root)?;
    migrate_catalog_db(&connection)?;
    mirror_verified_catalog(&mut connection, &verified.models)?;
    read_catalog_db_models(&connection)
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
    for file in &model.files {
        super::catalog::validate_download_target(&model.repo, &file.filename)?;
        if file.size_bytes == 0 {
            return Err(format!("Override file {} has no size.", file.filename));
        }
        if file.sha256.len() != 64 || !file.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "Override file {} needs its exact 64-digit SHA-256.",
                file.filename
            ));
        }
    }
    Ok(())
}

/// Insert or replace one user row. Marks `user_sourced = 1` so the UI can say
/// USER ADDED and network refreshes never mistake it for curator data.
pub fn save_user_catalog_override(
    connection: &Connection,
    model: &CatalogModel,
) -> Result<(), String> {
    validate_user_override(model)?;
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM catalog_model WHERE user_sourced = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
    let already: Option<i64> = connection
        .query_row(
            "SELECT user_sourced FROM catalog_model WHERE id = ?1",
            [&model.id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not read the local catalog database: {error}"))?;
    if already.is_none() && count as usize >= MAX_USER_OVERRIDE_MODELS {
        return Err(format!(
            "Too many local overrides (maximum {MAX_USER_OVERRIDE_MODELS}). Remove one first."
        ));
    }
    let mut marked = model.clone();
    marked.user_sourced = true;
    insert_model(connection, &marked, true)?;
    Ok(())
}

/// Remove one user row and its files. Network rows are never deleted here: a
/// typo in the id reports missing instead of touching curator data.
pub fn remove_user_catalog_override(connection: &Connection, id: &str) -> Result<(), String> {
    if id.trim().is_empty() || id.len() > MAX_USER_OVERRIDE_TEXT_LEN {
        return Err("The override id is missing or too long.".into());
    }
    let removed = connection
        .execute(
            "DELETE FROM catalog_file WHERE model_id IN
             (SELECT id FROM catalog_model WHERE id = ?1 AND user_sourced = 1)",
            [&id],
        )
        .map_err(|error| format!("Could not remove the local override: {error}"))?;
    connection
        .execute(
            "DELETE FROM catalog_model WHERE id = ?1 AND user_sourced = 1",
            [&id],
        )
        .map_err(|error| format!("Could not remove the local override: {error}"))?;
    if removed == 0 {
        let exists: Option<i64> = connection
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
    Ok(())
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
        let mirrored = rebuild_catalog_db_from_verified(&root, &verified).unwrap();
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

    #[test]
    fn corrupt_database_rebuilds_from_verified_bytes() {
        // Disk corruption or an interrupted write must not leave the catalog
        // tab empty: garbage bytes rebuild from verified data.
        let root = unique_test_dir("localmotive-catalog-db-corrupt");
        std::fs::write(catalog_db_path(&root), "not a database").unwrap();
        let verified = sample_catalog();
        let rebuilt = rebuild_catalog_db_from_verified(&root, &verified).unwrap();
        assert_eq!(rebuilt.len(), 2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn user_override_is_marked_and_survives_network_refresh() {
        // User rows carry user_sourced so the UI says USER ADDED; mirroring a
        // fresh verified catalog preserves them instead of wiping local work.
        let root = unique_test_dir("localmotive-catalog-db-user");
        let verified = sample_catalog();
        rebuild_catalog_db_from_verified(&root, &verified).unwrap();
        let connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        let mut owned = connection;
        save_user_catalog_override(&owned, &sample_override("mine")).unwrap();
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
        let mut bad = sample_override("bad");
        bad.repo = "not a repo!!".into();
        assert!(save_user_catalog_override(&connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.files[0].sha256 = "zz".into();
        assert!(save_user_catalog_override(&connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.files[0].filename = "../evil.gguf".into();
        assert!(save_user_catalog_override(&connection, &bad).is_err());
        let mut bad = sample_override("bad");
        bad.summary = "x".repeat(MAX_USER_OVERRIDE_TEXT_LEN + 1);
        assert!(save_user_catalog_override(&connection, &bad).is_err());
        assert!(remove_user_catalog_override(&connection, "nope").is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn user_override_remove_never_touches_network_rows() {
        // A wrong id against a network row reports curated, not missing, and
        // the network row stays in the mirror.
        let root = unique_test_dir("localmotive-catalog-db-remove");
        let verified = sample_catalog();
        rebuild_catalog_db_from_verified(&root, &verified).unwrap();
        let connection = open_catalog_db(&root).unwrap();
        migrate_catalog_db(&connection).unwrap();
        let error = remove_user_catalog_override(&connection, "a").unwrap_err();
        assert!(error.contains("curated"), "{error}");
        let rows = read_catalog_db_models(&connection).unwrap();
        assert_eq!(rows.len(), 2);
        save_user_catalog_override(&connection, &sample_override("mine")).unwrap();
        remove_user_catalog_override(&connection, "mine").unwrap();
        assert_eq!(read_catalog_db_models(&connection).unwrap().len(), 2);
        let _ = std::fs::remove_dir_all(root);
    }
}
