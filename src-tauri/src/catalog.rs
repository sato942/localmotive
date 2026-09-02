//! The curated model catalog.
//!
//! There is deliberately no database and no server behind this. The catalog is
//! a single JSON file in the project repository, fetched over HTTPS from
//! `raw.githubusercontent.com`, which is CDN-served and honours
//! `ETag`/`If-None-Match`. That gives curation (only listed models appear),
//! versioning (every edit is a commit), zero hosting cost, and near-free
//! refreshes — while the model bytes come straight from Hugging Face.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where the shipped catalog lives. Forks can change this compile-time value;
/// the production command does not accept an untrusted runtime URL.
pub const DEFAULT_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/sato942/gguf-pilot/main/catalog/catalog.json";
const BUNDLED_CATALOG: &str = include_str!("../../catalog/catalog.json");
const CATALOG_VERIFYING_KEY: [u8; 32] = [
    234, 194, 139, 46, 191, 202, 36, 78, 104, 245, 230, 170, 90, 67, 238, 61, 1, 162, 242, 207,
    116, 254, 217, 3, 74, 69, 78, 102, 199, 170, 8, 119,
];

/// Highest schema version this build understands. The loader accepts anything
/// at or below it so an older app keeps working after the catalog moves on.
pub const SUPPORTED_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogFile {
    pub quant: String,
    pub filename: String,
    pub size_bytes: u64,
    pub sha256: String,
    #[serde(default = "default_revision")]
    pub revision: String,
}

fn default_revision() -> String {
    "main".into()
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogModel {
    pub id: String,
    pub repo: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub parameters: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub gated: bool,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub likes: u64,
    #[serde(default)]
    pub files: Vec<CatalogFile>,
}

impl CatalogModel {
    /// Smallest offered build, which is what a size-constrained user wants to
    /// see first.
    pub fn smallest_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size_bytes).min().unwrap_or(0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub schema_version: u32,
    #[serde(default)]
    pub updated: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub models: Vec<CatalogModel>,
}

/// Parse and validate a catalog document. Rejects a schema this build cannot
/// read, and drops entries that are structurally unusable rather than showing a
/// row that cannot be downloaded.
pub fn parse_catalog(text: &str) -> Result<Catalog, String> {
    let mut catalog: Catalog = serde_json::from_str(text)
        .map_err(|error| format!("Catalog is not valid JSON: {error}"))?;
    if catalog.schema_version == 0 || catalog.schema_version > SUPPORTED_SCHEMA {
        return Err(format!(
            "Catalog schema {} is newer than this build supports ({SUPPORTED_SCHEMA}). Update GGUF Pilot.",
            catalog.schema_version
        ));
    }
    catalog.models.retain(|model| {
        !model.id.is_empty()
            && is_valid_repo(&model.repo)
            && !model.files.is_empty()
            && model.files.iter().all(|file| {
                is_safe_filename(&file.filename)
                    && file.size_bytes > 0
                    && is_safe_revision(&file.revision)
                    && is_sha256(&file.sha256)
            })
    });
    let mut ids = std::collections::BTreeSet::new();
    let mut files = std::collections::BTreeSet::new();
    for model in &catalog.models {
        if !ids.insert(model.id.as_str()) {
            return Err(format!("Catalog contains duplicate model id: {}", model.id));
        }
        for file in &model.files {
            if !files.insert(file.filename.to_ascii_lowercase()) {
                return Err(format!(
                    "Catalog model {} contains duplicate file: {}",
                    model.id, file.filename
                ));
            }
        }
    }
    if catalog.models.is_empty() {
        return Err("Catalog contains no usable models.".into());
    }
    Ok(catalog)
}

pub fn bundled_catalog() -> Result<Catalog, String> {
    parse_catalog(BUNDLED_CATALOG)
}

pub fn catalog_file<'a>(
    catalog: &'a Catalog,
    repo: &str,
    filename: &str,
    revision: &str,
) -> Option<&'a CatalogFile> {
    catalog
        .models
        .iter()
        .find(|m| m.repo == repo)?
        .files
        .iter()
        .find(|f| f.filename == filename && f.revision == revision)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_safe_revision(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && !value.contains("..")
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
        && value.split('/').all(|segment| !segment.is_empty())
}

fn verify_catalog_signature_with_key(body: &[u8], encoded: &str, key: &[u8; 32]) -> bool {
    use base64::Engine;
    let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded.trim()) else {
        return false;
    };
    let Ok(bytes) = <[u8; 64]>::try_from(decoded.as_slice()) else {
        return false;
    };
    let Ok(verifying_key) = ed25519_dalek::VerifyingKey::from_bytes(key) else {
        return false;
    };
    verifying_key
        .verify_strict(body, &ed25519_dalek::Signature::from_bytes(&bytes))
        .is_ok()
}

fn verify_catalog_signature(body: &[u8], encoded: &str) -> bool {
    verify_catalog_signature_with_key(body, encoded, &CATALOG_VERIFYING_KEY)
}

/// `owner/name`, the only shape Hugging Face uses. Rejecting anything else
/// keeps a malformed or hostile catalog from producing surprising URLs.
fn is_valid_repo(repo: &str) -> bool {
    let mut parts = repo.split('/');
    let (Some(owner), Some(name), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let ok = |s: &str| {
        !s.is_empty()
            && s.len() <= 96
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            && s != "."
            && s != ".."
    };
    ok(owner) && ok(name)
}

/// A catalog filename becomes a path on disk, so it must not escape the target
/// directory or name a drive.
fn is_safe_filename(name: &str) -> bool {
    if name.is_empty()
        || name.len() > 512
        || !name.to_ascii_lowercase().ends_with(".gguf")
        || name.contains(':')
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
        || name.starts_with('/')
    {
        return false;
    }
    name != "." && name != ".." && name.len() <= 255
}

/// Validate the untrusted arguments received by the Tauri download command.
/// Catalog parsing uses the same rules, but the command remains defensive if a
/// compromised webview invokes it directly.
pub fn validate_download_target(repo: &str, filename: &str) -> Result<(), String> {
    if !is_valid_repo(repo) {
        return Err("The Hugging Face repository must have the form owner/name.".into());
    }
    if !is_safe_filename(filename) {
        return Err("The catalog filename is unsafe or is not a GGUF file.".into());
    }
    Ok(())
}

/// How the catalog list is ordered in the interface.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CatalogSort {
    #[default]
    Downloads,
    Likes,
    Name,
    Size,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CatalogQuery {
    /// Free text over repo, family, publisher, summary and tags.
    pub text: String,
    /// Exact tag match; empty means any.
    pub tag: String,
    /// Exact quantisation match against a model's files; empty means any.
    pub quant: String,
    /// Upper bound in bytes on the smallest offered file; 0 means no limit.
    pub max_bytes: u64,
    /// Hide models that require accepting a licence on Hugging Face.
    pub hide_gated: bool,
    pub sort: CatalogSort,
}

/// Apply a query. Case-insensitive throughout, because users type lowercase and
/// repository names are mixed case.
pub fn filter_models(models: &[CatalogModel], query: &CatalogQuery) -> Vec<CatalogModel> {
    let text = query.text.trim().to_ascii_lowercase();
    let tag = query.tag.trim().to_ascii_lowercase();
    let quant = query.quant.trim().to_ascii_lowercase();

    let mut out: Vec<CatalogModel> = models
        .iter()
        .filter(|model| {
            if query.hide_gated && model.gated {
                return false;
            }
            if !tag.is_empty() && !model.tags.iter().any(|t| t.to_ascii_lowercase() == tag) {
                return false;
            }
            if !quant.is_empty()
                && !model
                    .files
                    .iter()
                    .any(|f| f.quant.to_ascii_lowercase() == quant)
            {
                return false;
            }
            if query.max_bytes > 0 && model.smallest_bytes() > query.max_bytes {
                return false;
            }
            if text.is_empty() {
                return true;
            }
            // Every whitespace-separated term must appear somewhere, so
            // "qwen coder" narrows instead of widening.
            let haystack = format!(
                "{} {} {} {} {} {}",
                model.repo,
                model.family,
                model.publisher,
                model.parameters,
                model.summary,
                model.tags.join(" ")
            )
            .to_ascii_lowercase();
            text.split_whitespace().all(|term| haystack.contains(term))
        })
        .cloned()
        .collect();

    match query.sort {
        CatalogSort::Downloads => out.sort_by(|a, b| {
            b.downloads
                .cmp(&a.downloads)
                .then_with(|| a.repo.cmp(&b.repo))
        }),
        CatalogSort::Likes => {
            out.sort_by(|a, b| b.likes.cmp(&a.likes).then_with(|| a.repo.cmp(&b.repo)))
        }
        CatalogSort::Name => out.sort_by(|a, b| {
            a.repo
                .to_ascii_lowercase()
                .cmp(&b.repo.to_ascii_lowercase())
        }),
        CatalogSort::Size => out.sort_by(|a, b| {
            a.smallest_bytes()
                .cmp(&b.smallest_bytes())
                .then_with(|| a.repo.cmp(&b.repo))
        }),
    }
    out
}

/// Every distinct tag and quantisation in the catalog, for the filter controls.
pub fn facets(models: &[CatalogModel]) -> (Vec<String>, Vec<String>) {
    let mut tags: Vec<String> = models
        .iter()
        .flat_map(|m| m.tags.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    tags.sort();
    let quants: Vec<String> = models
        .iter()
        .flat_map(|m| m.files.iter().map(|f| f.quant.clone()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    (tags, quants)
}

/// A catalog plus how it was obtained, so the interface can be honest about
/// whether it is showing live or cached data.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSnapshot {
    pub catalog: Catalog,
    /// "network", "not-modified", "cache" or "bundled".
    pub origin: String,
    pub fetched_at: String,
    pub url: String,
}

#[derive(Deserialize, Serialize)]
struct CacheRecord {
    body: String,
    etag: Option<String>,
    signature: String,
}

static CATALOG_CACHE_WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn cache_path(root: &Path) -> PathBuf {
    root.join("catalog-cache.json")
}

fn load_cache_record(root: &Path) -> Option<CacheRecord> {
    let _guard = CATALOG_CACHE_WRITE_LOCK.lock().ok()?;
    let text = std::fs::read_to_string(cache_path(root)).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_cache_record(
    root: &Path,
    body: &str,
    etag: Option<&str>,
    signature: &str,
) -> Result<(), String> {
    let _guard = CATALOG_CACHE_WRITE_LOCK
        .lock()
        .map_err(|_| "The catalog cache lock is unavailable.".to_string())?;
    std::fs::create_dir_all(root)
        .map_err(|error| format!("Could not create {}: {error}", root.display()))?;
    let record = serde_json::to_vec(&CacheRecord {
        body: body.to_string(),
        etag: etag.map(str::to_string),
        signature: signature.to_string(),
    })
    .map_err(|error| format!("Could not serialize the catalog cache: {error}"))?;
    let cache = cache_path(root);
    let temp = root.join(format!("catalog-cache.{}.tmp", std::process::id()));
    {
        let mut file = std::fs::File::create(&temp)
            .map_err(|error| format!("Could not create {}: {error}", temp.display()))?;
        use std::io::Write;
        file.write_all(&record)
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("Could not write {}: {error}", temp.display()))?;
    }
    // rename replaces a file atomically. Never delete the old record first: if
    // publication fails, the previous validated cache must remain available.
    std::fs::rename(&temp, &cache)
        .map_err(|error| format!("Could not publish {}: {error}", cache.display()))
}

/// Fetch the catalog, using a stored ETag so an unchanged catalog costs one
/// conditional request and no body. Falls back to the cached copy when the
/// network is unavailable, because an offline user should still see the list.
pub fn fetch_catalog(url: &str, cache_root: &Path) -> Result<CatalogSnapshot, String> {
    let cached = load_cache_record(cache_root)
        .filter(|record| verify_catalog_signature(record.body.as_bytes(), &record.signature));
    let cached_etag = cached.as_ref().and_then(|record| record.etag.as_ref());
    let cached_body = cached.as_ref().map(|record| record.body.as_str());

    let client = match http_client() {
        Ok(client) => client,
        Err(error) => return fallback(cached_body.map(str::to_string), url, error),
    };
    let mut request = client.get(url);
    if let (Some(etag), Some(_)) = (cached_etag, cached_body) {
        request = request.header(reqwest::header::IF_NONE_MATCH, etag);
    }

    match request.send() {
        Ok(response) => {
            let status = response.status().as_u16();
            if status >= 400 {
                return fallback(
                    cached_body.map(str::to_string),
                    url,
                    format!("the catalog server answered HTTP {status}"),
                );
            }
            let etag = response
                .headers()
                .get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            if status == 304 {
                let body = cached_body
                    .ok_or_else(|| "Catalog unchanged but no cached copy exists".to_string())?;
                return Ok(CatalogSnapshot {
                    catalog: parse_catalog(body)?,
                    origin: "not-modified".into(),
                    fetched_at: now(),
                    url: url.to_string(),
                });
            }
            let body = response
                .text()
                .map_err(|error| format!("Could not read the catalog: {error}"))?;
            let signature_url = format!("{url}.sig");
            let signature = match client.get(&signature_url).send() {
                Ok(signature_response) if signature_response.status().is_success() => {
                    signature_response
                        .text()
                        .map_err(|error| format!("Could not read the catalog signature: {error}"))?
                }
                Ok(signature_response) => {
                    return fallback(
                        cached_body.map(str::to_string),
                        url,
                        format!(
                            "the catalog signature server answered HTTP {}",
                            signature_response.status().as_u16()
                        ),
                    )
                }
                Err(error) => {
                    return fallback(
                        cached_body.map(str::to_string),
                        url,
                        format!("could not fetch the catalog signature: {error}"),
                    )
                }
            };
            if !verify_catalog_signature(body.as_bytes(), &signature) {
                return fallback(
                    cached_body.map(str::to_string),
                    url,
                    "the catalog signature is invalid".into(),
                );
            }
            let catalog = parse_catalog(&body)?;
            // Only cache a signed document that parsed successfully.
            let _ = save_cache_record(cache_root, &body, etag.as_deref(), &signature);
            Ok(CatalogSnapshot {
                catalog,
                origin: "network".into(),
                fetched_at: now(),
                url: url.to_string(),
            })
        }
        Err(error) => fallback(cached_body.map(str::to_string), url, error.to_string()),
    }
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent(format!("GGUF-Pilot/{}", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("Could not create an HTTPS client: {error}"))
}

/// An unreachable catalog is not fatal if a previous copy is on disk: an
/// offline user should still be able to browse what they saw last time.
fn fallback(
    cached_body: Option<String>,
    url: &str,
    error: String,
) -> Result<CatalogSnapshot, String> {
    if let Some(body) = cached_body {
        if let Ok(catalog) = parse_catalog(&body) {
            return Ok(CatalogSnapshot {
                catalog,
                origin: "cache".into(),
                fetched_at: now(),
                url: url.to_string(),
            });
        }
    }

    Ok(CatalogSnapshot {
        catalog: parse_catalog(BUNDLED_CATALOG).map_err(|bundled_error| {
            format!(
                "Could not reach the catalog ({error}), and the built-in catalog is invalid: {bundled_error}"
            )
        })?,
        origin: "bundled".into(),
        fetched_at: now(),
        url: url.to_string(),
    })
}

fn now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

// ---------------------------------------------------------------------------
// Hugging Face token
// ---------------------------------------------------------------------------

/// Credential Manager service name for the Hugging Face token. Separate from
/// the cloud-advisor service so revoking one never disturbs the other.
pub const HF_KEYRING_SERVICE: &str = "GGUF Pilot HF";
const HF_ACCOUNT: &str = "huggingface";

/// What the interface may know about a stored token: that one exists, and
/// enough of a suffix to tell two tokens apart. Never the token itself.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenStatus {
    pub configured: bool,
    pub masked: String,
}

/// Show only the last four characters. A token that leaks into a screenshot,
/// a log, or a support request must not be reusable.
pub fn mask_token(token: &str) -> String {
    let token = token.trim();
    if token.is_empty() {
        return String::new();
    }
    let tail: String = token
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

/// Reject input that cannot be a Hugging Face token before it reaches the
/// network, so the user gets an immediate, specific error.
pub fn validate_hf_token(token: &str) -> Result<String, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("Paste a Hugging Face access token, or press Remove to clear it.".into());
    }
    if token.chars().any(|c| c.is_whitespace()) {
        return Err(
            "That token contains spaces or line breaks. Copy it again from Hugging Face.".into(),
        );
    }
    if !token.is_ascii() {
        return Err(
            "That token contains non-ASCII characters and cannot be a Hugging Face token.".into(),
        );
    }
    if !token.starts_with("hf_") {
        return Err("Hugging Face access tokens begin with `hf_`. Create one at huggingface.co/settings/tokens.".into());
    }
    if token.len() < 12 {
        return Err("That token is too short to be a Hugging Face access token.".into());
    }
    Ok(token.to_string())
}

fn hf_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(HF_KEYRING_SERVICE, HF_ACCOUNT)
        .map_err(|error| format!("Could not open Credential Manager: {error}"))
}

/// Read the stored token. Returns `None` rather than an error when absent, so
/// anonymous downloading stays the normal path.
pub fn hf_token() -> Option<String> {
    let entry = hf_entry().ok()?;
    match entry.get_password() {
        Ok(secret) if !secret.trim().is_empty() => Some(secret),
        _ => None,
    }
}

pub fn hf_token_status() -> TokenStatus {
    match hf_token() {
        Some(token) => TokenStatus {
            configured: true,
            masked: mask_token(&token),
        },
        None => TokenStatus {
            configured: false,
            masked: String::new(),
        },
    }
}

pub fn save_hf_token(token: &str) -> Result<TokenStatus, String> {
    let token = validate_hf_token(token)?;
    hf_entry()?
        .set_password(&token)
        .map_err(|error| format!("Could not save the token to Credential Manager: {error}"))?;
    Ok(TokenStatus {
        configured: true,
        masked: mask_token(&token),
    })
}

pub fn clear_hf_token() -> Result<TokenStatus, String> {
    let entry = hf_entry()?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(TokenStatus {
            configured: false,
            masked: String::new(),
        }),
        Err(error) => Err(format!("Could not remove the token: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Catalog {
        parse_catalog(
            r#"{
              "schemaVersion": 1,
              "updated": "2026-09-02",
              "models": [
                {"id":"a","repo":"unsloth/Qwen3-Coder-30B-A3B-Instruct-GGUF","family":"Qwen3 Coder",
                 "publisher":"unsloth","parameters":"30B-A3B","tags":["code","moe"],
                 "downloads":12706054,"likes":945,
                 "files":[{"quant":"Q4_K_M","filename":"a-Q4_K_M.gguf","sizeBytes":18556689568,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
                          {"quant":"Q6_K","filename":"a-Q6_K.gguf","sizeBytes":25092535456,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
                {"id":"b","repo":"bartowski/Llama-3.2-1B-Instruct-GGUF","family":"Llama 3.2",
                 "publisher":"bartowski","parameters":"1B","tags":["small","general"],
                 "downloads":147312,"likes":176,
                 "files":[{"quant":"Q4_K_M","filename":"b-Q4_K_M.gguf","sizeBytes":807694464,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
                {"id":"c","repo":"meta-llama/Gated-GGUF","family":"Gated","publisher":"meta-llama",
                 "tags":["general"],"gated":true,"downloads":500,"likes":9,
                 "files":[{"quant":"Q8_0","filename":"c-Q8_0.gguf","sizeBytes":1000,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}
              ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn a_masked_token_cannot_be_reconstructed() {
        // A token shown in a screenshot or support request must not be reusable.
        let token = format!("hf_{}012345", "abcdefghijklmnopqrstuvwxyz");
        let masked = mask_token(&token);
        assert_eq!(masked, "…2345");
        assert!(!masked.contains("hf_"), "the prefix is not shown");
        assert!(
            !token[..token.len() - 4].contains(masked.trim_start_matches('…')),
            "only the tail is revealed"
        );
        assert_eq!(mask_token(""), "");
        assert_eq!(mask_token("   "), "");
        assert_eq!(mask_token("ab"), "…ab", "a short value does not panic");
        // Multi-byte input must not split a character and panic.
        assert_eq!(mask_token("aé日本語"), "…é日本語");
    }

    #[test]
    fn token_validation_names_the_problem_and_the_fix() {
        assert!(validate_hf_token("hf_abcdefghijklmno").is_ok());
        assert_eq!(
            validate_hf_token("  hf_abcdefghijklmno  ").unwrap(),
            "hf_abcdefghijklmno",
            "surrounding whitespace from a paste is trimmed"
        );

        let empty = validate_hf_token("").unwrap_err();
        assert!(empty.contains("Remove"), "{empty}");

        let spaced = validate_hf_token("hf_abc def12345678").unwrap_err();
        assert!(spaced.contains("spaces"), "{spaced}");

        let wrong = validate_hf_token("sk-not-a-hf-token").unwrap_err();
        assert!(
            wrong.contains("hf_") && wrong.contains("settings/tokens"),
            "{wrong}"
        );

        let short = validate_hf_token("hf_abc").unwrap_err();
        assert!(short.contains("too short"), "{short}");

        assert!(validate_hf_token("hf_ünicodetoken123").is_err());
    }

    #[test]
    fn download_targets_cannot_escape_the_model_folder() {
        assert!(validate_download_target("owner/repo", "model.gguf").is_ok());
        for filename in [
            "sub/model Q4.gguf",
            "../evil.gguf",
            "sub/../../evil.gguf",
            "sub\\evil.gguf",
            "C:evil.gguf",
            "/absolute.gguf",
            "model.exe",
        ] {
            assert!(
                validate_download_target("owner/repo", filename).is_err(),
                "unsafe filename accepted: {filename}"
            );
        }
        assert!(validate_download_target("owner/repo/extra", "model.gguf").is_err());
    }

    #[test]
    fn parse_rejects_a_schema_from_the_future() {
        let error = parse_catalog(r#"{"schemaVersion": 99, "models": []}"#).unwrap_err();
        assert!(error.contains("newer than this build"), "{error}");
        assert!(parse_catalog(r#"{"schemaVersion": 0, "models": []}"#).is_err());
    }

    #[test]
    fn parse_rejects_duplicate_model_ids() {
        // React keys and per-model file selections depend on IDs being unique;
        // a bad curator edit must fall back to the last good catalog.
        let json = r#"{
          "schemaVersion": 1,
          "models": [
            {"id":"same","repo":"a/one","files":[{"quant":"Q4","filename":"one.gguf","sizeBytes":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
            {"id":"same","repo":"b/two","files":[{"quant":"Q4","filename":"two.gguf","sizeBytes":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}
          ]
        }"#;
        assert!(parse_catalog(json)
            .unwrap_err()
            .contains("duplicate model id"));
    }

    #[test]
    fn parse_rejects_duplicate_files_within_a_model() {
        let json = r#"{
          "schemaVersion": 1,
          "models": [{
            "id":"one","repo":"a/one","files":[
              {"quant":"Q4","filename":"same.gguf","sizeBytes":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
              {"quant":"Q5","filename":"same.gguf","sizeBytes":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
            ]
          }]
        }"#;
        assert!(parse_catalog(json).unwrap_err().contains("duplicate file"));
    }

    #[test]
    fn parse_rejects_target_filename_collisions_across_models() {
        // Catalog downloads share one destination folder. Two repositories
        // writing the same filename could race over one .part sidecar.
        let json = r#"{
          "schemaVersion": 1,
          "models": [
            {"id":"one","repo":"a/one","files":[{"quant":"Q4","filename":"same.gguf","sizeBytes":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
            {"id":"two","repo":"b/two","files":[{"quant":"Q8","filename":"SAME.gguf","sizeBytes":2,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}
          ]
        }"#;
        assert!(parse_catalog(json).unwrap_err().contains("duplicate file"));
    }

    #[test]
    fn authorization_uses_the_current_validated_catalog_not_only_the_bundled_copy() {
        // Maintainers can publish catalog additions without shipping a new app.
        // A fetched, validated entry must therefore be downloadable immediately.
        let catalog = sample();
        assert_eq!(
            catalog_file(
                &catalog,
                "unsloth/Qwen3-Coder-30B-A3B-Instruct-GGUF",
                "a-Q4_K_M.gguf",
                "main",
            )
            .map(|file| file.size_bytes),
            Some(18_556_689_568)
        );
        assert_eq!(
            catalog_file(&catalog, "unknown/repo", "a-Q4_K_M.gguf", "main"),
            None
        );
    }

    #[test]
    fn every_authorized_download_has_a_cryptographic_digest() {
        // Size alone cannot detect a same-length CDN substitution. Catalog
        // authorization must carry the Hugging Face LFS SHA-256 into download
        // verification.
        let catalog = parse_catalog(
            r#"{"schemaVersion":1,"models":[{"id":"x","repo":"a/b","files":[
              {"quant":"Q4","filename":"x.gguf","sizeBytes":5}
            ]}]}"#,
        );
        assert!(catalog.is_err(), "a file without sha256 was authorized");
        assert!(sample()
            .models
            .iter()
            .all(|model| model.files.iter().all(|file| {
                file.sha256.len() == 64 && file.sha256.chars().all(|c| c.is_ascii_hexdigit())
            })));
    }

    #[test]
    fn network_catalogs_require_a_valid_maintainer_signature() {
        use ed25519_dalek::{Signer, SigningKey};

        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let body = br#"{"schemaVersion":1}"#;
        let signature = signing.sign(body);
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            signature.to_bytes(),
        );
        assert!(verify_catalog_signature_with_key(
            body,
            &encoded,
            signing.verifying_key().as_bytes(),
        ));
        assert!(!verify_catalog_signature_with_key(
            br#"{"schemaVersion":2}"#,
            &encoded,
            signing.verifying_key().as_bytes(),
        ));
        assert!(!verify_catalog_signature_with_key(
            body,
            "not base64",
            signing.verifying_key().as_bytes(),
        ));
    }

    #[test]
    fn parse_reports_malformed_json_instead_of_panicking() {
        assert!(parse_catalog("not json")
            .unwrap_err()
            .contains("not valid JSON"));
        assert!(parse_catalog("").is_err());
        assert!(parse_catalog("{}").is_err(), "schemaVersion is required");
    }

    #[test]
    fn parse_drops_entries_that_could_not_be_downloaded_safely() {
        // A hostile or broken catalog must not produce a row that escapes the
        // download directory, names a drive, or has no file to fetch.
        let catalog = parse_catalog(
            r#"{"schemaVersion":1,"models":[
              {"id":"ok","repo":"a/b","files":[{"quant":"Q4","filename":"ok.gguf","sizeBytes":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"escape","repo":"a/b","files":[{"quant":"Q4","filename":"../../evil.gguf","sizeBytes":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"drive","repo":"a/b","files":[{"quant":"Q4","filename":"C:evil.gguf","sizeBytes":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"absolute","repo":"a/b","files":[{"quant":"Q4","filename":"/etc/evil.gguf","sizeBytes":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"notgguf","repo":"a/b","files":[{"quant":"Q4","filename":"payload.exe","sizeBytes":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"nofiles","repo":"a/b","files":[]},
              {"id":"zerosize","repo":"a/b","files":[{"quant":"Q4","filename":"z.gguf","sizeBytes":0,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"badrepo","repo":"a/b/c","files":[{"quant":"Q4","filename":"x.gguf","sizeBytes":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"dotrepo","repo":"../etc","files":[{"quant":"Q4","filename":"x.gguf","sizeBytes":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"","repo":"a/b","files":[{"quant":"Q4","filename":"x.gguf","sizeBytes":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}
            ]}"#,
        )
        .unwrap();
        let ids: Vec<&str> = catalog.models.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["ok"], "only the safe entry survives");
    }

    #[test]
    fn filter_requires_every_search_term_to_match() {
        let catalog = sample();
        let query = |text: &str| CatalogQuery {
            text: text.into(),
            ..Default::default()
        };
        assert_eq!(filter_models(&catalog.models, &query("qwen")).len(), 1);
        assert_eq!(
            filter_models(&catalog.models, &query("QWEN CODER")).len(),
            1
        );
        assert_eq!(
            filter_models(&catalog.models, &query("qwen llama")).len(),
            0,
            "terms are ANDed, not ORed"
        );
        assert_eq!(filter_models(&catalog.models, &query("")).len(), 3);
        assert_eq!(
            filter_models(&catalog.models, &query("   ")).len(),
            3,
            "whitespace is not a search"
        );
        assert_eq!(
            filter_models(&catalog.models, &query("unsloth")).len(),
            1,
            "publisher is searchable"
        );
    }

    #[test]
    fn filter_applies_tag_quant_size_and_gated_constraints() {
        let catalog = sample();
        let by_tag = filter_models(
            &catalog.models,
            &CatalogQuery {
                tag: "CODE".into(),
                ..Default::default()
            },
        );
        assert_eq!(by_tag.len(), 1, "tag match is case-insensitive");

        let by_quant = filter_models(
            &catalog.models,
            &CatalogQuery {
                quant: "q6_k".into(),
                ..Default::default()
            },
        );
        assert_eq!(by_quant.len(), 1);

        let by_size = filter_models(
            &catalog.models,
            &CatalogQuery {
                max_bytes: 1_000_000_000,
                ..Default::default()
            },
        );
        assert_eq!(
            by_size.len(),
            2,
            "size compares against the smallest offered file"
        );

        let hide_gated = filter_models(
            &catalog.models,
            &CatalogQuery {
                hide_gated: true,
                ..Default::default()
            },
        );
        assert!(hide_gated.iter().all(|m| !m.gated));
        assert_eq!(hide_gated.len(), 2);
    }

    #[test]
    fn filter_sorts_deterministically_on_every_key() {
        let catalog = sample();
        let sorted = |sort| {
            filter_models(
                &catalog.models,
                &CatalogQuery {
                    sort,
                    ..Default::default()
                },
            )
            .into_iter()
            .map(|m| m.id)
            .collect::<Vec<_>>()
        };
        assert_eq!(sorted(CatalogSort::Downloads), vec!["a", "b", "c"]);
        assert_eq!(sorted(CatalogSort::Likes), vec!["a", "b", "c"]);
        assert_eq!(sorted(CatalogSort::Size), vec!["c", "b", "a"]);
        assert_eq!(
            sorted(CatalogSort::Name),
            vec!["b", "c", "a"],
            "name sorts by repo, case-insensitively"
        );
        // Stable across repeated calls.
        assert_eq!(
            sorted(CatalogSort::Downloads),
            sorted(CatalogSort::Downloads)
        );
    }

    #[test]
    fn facets_are_sorted_and_deduplicated() {
        let (tags, quants) = facets(&sample().models);
        assert_eq!(tags, vec!["code", "general", "moe", "small"]);
        assert_eq!(quants, vec!["Q4_K_M", "Q6_K", "Q8_0"]);
    }

    #[test]
    fn shipped_catalog_file_is_valid() {
        // The catalog that ships with the repository must always parse and keep
        // every entry, so a bad edit fails CI instead of reaching users.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("catalog")
            .join("catalog.json");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let raw: serde_json::Value = serde_json::from_str(&text).unwrap();
        let declared = raw["models"].as_array().map(|a| a.len()).unwrap_or(0);
        let catalog = parse_catalog(&text).expect("shipped catalog must parse");
        assert_eq!(
            catalog.models.len(),
            declared,
            "no shipped entry may be dropped by validation"
        );
        assert!(!catalog.models.is_empty());
        for model in &catalog.models {
            assert!(is_valid_repo(&model.repo), "{}", model.repo);
            for file in &model.files {
                assert!(is_safe_filename(&file.filename), "{}", file.filename);
                assert!(file.size_bytes > 0, "{} has no size", file.filename);
            }
        }
        let signature = std::fs::read_to_string(path.with_extension("json.sig")).unwrap();
        assert!(
            verify_catalog_signature(text.as_bytes(), &signature),
            "shipped catalog signature must match the embedded maintainer key"
        );
    }

    #[test]
    fn cache_body_and_etag_are_published_as_one_consistent_record() {
        let root =
            std::env::temp_dir().join(format!("gguf-pilot-cache-record-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        save_cache_record(
            &root,
            "catalog body",
            Some("catalog etag"),
            "catalog signature",
        )
        .unwrap();
        let record = load_cache_record(&root).expect("cache record should reload");
        assert_eq!(record.body, "catalog body");
        assert_eq!(record.etag.as_deref(), Some("catalog etag"));
        assert_eq!(record.signature, "catalog signature");

        // The app refreshes this file repeatedly. On Windows the second
        // publication must replace the existing record rather than failing.
        save_cache_record(
            &root,
            "replacement body",
            Some("replacement etag"),
            "replacement signature",
        )
        .unwrap();
        let replacement = load_cache_record(&root).expect("replacement should reload");
        assert_eq!(replacement.body, "replacement body");
        assert_eq!(replacement.etag.as_deref(), Some("replacement etag"));
        assert_eq!(replacement.signature, "replacement signature");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_cache_refreshes_never_mix_a_body_with_another_etag() {
        let root =
            std::env::temp_dir().join(format!("gguf-pilot-cache-race-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let results = std::thread::scope(|scope| {
            let handles = (0..8)
                .map(|index| {
                    let root = root.clone();
                    let barrier = barrier.clone();
                    scope.spawn(move || {
                        barrier.wait();
                        save_cache_record(
                            &root,
                            &format!("body-{index}"),
                            Some(&format!("etag-{index}")),
                            &format!("signature-{index}"),
                        )
                    })
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert!(results.iter().all(Result::is_ok), "{results:?}");
        let record = load_cache_record(&root).unwrap();
        assert_eq!(
            record.body.strip_prefix("body-"),
            record
                .etag
                .as_deref()
                .and_then(|etag| etag.strip_prefix("etag-"))
        );
        assert_eq!(
            record.body.strip_prefix("body-"),
            record.signature.strip_prefix("signature-")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cache_readers_wait_until_a_cache_publication_finishes() {
        let root =
            std::env::temp_dir().join(format!("gguf-pilot-cache-reader-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let guard = CATALOG_CACHE_WRITE_LOCK.lock().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        let reader_root = root.clone();
        let reader = std::thread::spawn(move || {
            send.send(load_cache_record(&reader_root)).unwrap();
        });
        assert!(
            receive.recv_timeout(Duration::from_millis(50)).is_err(),
            "a reader observed the cache while publication held the lock"
        );
        drop(guard);
        receive.recv_timeout(Duration::from_secs(1)).unwrap();
        reader.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cached_catalog_is_used_when_the_network_fails() {
        let root = std::env::temp_dir().join(format!("gguf-pilot-cat-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let catalog_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("catalog");
        let body = std::fs::read_to_string(catalog_dir.join("catalog.json")).unwrap();
        let signature = std::fs::read_to_string(catalog_dir.join("catalog.json.sig")).unwrap();
        save_cache_record(&root, &body, Some("cache-etag"), &signature).unwrap();

        // Port 9 (discard) refuses HTTP, standing in for an offline machine.
        let snapshot = fetch_catalog("http://127.0.0.1:9/catalog.json", &root).unwrap();
        assert_eq!(snapshot.origin, "cache");
        assert!(!snapshot.catalog.models.is_empty());

        // With no cache at all, first run falls back to the catalog embedded in
        // this exact app build rather than presenting an empty tab offline.
        let empty = root.join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        let bundled = fetch_catalog("http://127.0.0.1:9/catalog.json", &empty).unwrap();
        assert_eq!(bundled.origin, "bundled");
        assert!(!bundled.catalog.models.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn corrupt_cache_falls_back_to_the_bundled_catalog() {
        // An interrupted external cache edit or disk corruption must not leave
        // the catalog tab empty while the network is also unavailable.
        let root =
            std::env::temp_dir().join(format!("gguf-pilot-corrupt-cat-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(cache_path(&root), "not json").unwrap();

        let snapshot = fetch_catalog("http://127.0.0.1:9/catalog.json", &root).unwrap();
        assert_eq!(snapshot.origin, "bundled");
        assert!(!snapshot.catalog.models.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
