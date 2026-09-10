//! The curated model catalog.
//!
//! There is deliberately no database and no server behind this. The catalog is
//! a single JSON file in the project repository, fetched over HTTPS from
//! `raw.githubusercontent.com`, which is CDN-served and honours
//! `ETag`/`If-None-Match`. That gives curation (only listed models appear),
//! versioning (every edit is a commit), zero hosting cost, and near-free
//! refreshes — while the model bytes come straight from Hugging Face.

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Where the shipped catalog lives. Forks can change this compile-time value;
/// the production command does not accept an untrusted runtime URL.
pub const DEFAULT_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/sato942/localmotive/main/catalog/catalog.json";
const BUNDLED_CATALOG: &str = include_str!("../../catalog/catalog.json");
const CATALOG_VERIFYING_KEY: [u8; 32] = [
    234, 194, 139, 46, 191, 202, 36, 78, 104, 245, 230, 170, 90, 67, 238, 61, 1, 162, 242, 207,
    116, 254, 217, 3, 74, 69, 78, 102, 199, 170, 8, 119,
];
const MAX_CATALOG_BODY_BYTES: usize = 4 * 1024 * 1024;
const MAX_CATALOG_SIGNATURE_BYTES: usize = 16 * 1024;
const MAX_CATALOG_CACHE_BYTES: u64 = 5 * 1024 * 1024;

fn read_bounded_catalog_body(mut reader: impl Read, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(limit.min(64 * 1024));
    reader
        .by_ref()
        .take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read the catalog response: {error}"))?;
    if bytes.len() > limit {
        return Err(format!("Catalog response exceeds the {limit} byte limit"));
    }
    Ok(bytes)
}

/// Highest schema version this build understands. Version 2 is the current
/// network contract with rich filter fields. Version 1 documents stay readable
/// because every v2 field except the file identity triple has a default, but a
/// v1 file fails closed: without rich fields the 0.5 filters cannot be honest
/// about what they hide, so the loader rejects it with an upgrade message
/// instead of silently showing an unfiltered list.
pub const SUPPORTED_SCHEMA: u32 = 2;
const MIN_SUPPORTED_SCHEMA: u32 = 2;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogFile {
    pub quant: String,
    pub filename: String,
    pub size_bytes: u64,
    pub sha256: String,
    #[serde(default = "default_revision")]
    pub revision: String,
    /// Hugging Face repo update time, ISO 8601. Empty when the builder could
    /// not observe it; recency filters treat empty as unknown, never as new.
    #[serde(default)]
    pub last_modified: String,
    /// Repo creation time, ISO 8601. Empty when unobserved.
    #[serde(default)]
    pub created_at: String,
}

fn default_revision() -> String {
    "main".into()
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CatalogModel {
    pub id: String,
    pub repo: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub parameters: String,
    #[serde(default)]
    pub publisher: String,
    /// Display author. Defaults to the repo owner when the builder omits it.
    #[serde(default)]
    pub author: String,
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
    /// SPDX or Hub license id, e.g. `apache-2.0`. Empty when unobserved.
    #[serde(default)]
    pub license: String,
    /// Hub pipeline tag, e.g. `text-generation`. Empty when unobserved.
    /// File key is `pipeline_tag` (signed v2 artifact); the builder emits the
    /// same key. `rename_all` is intentionally absent on this struct so file
    /// keys stay stable under signature.
    #[serde(default, alias = "pipeline_tag")]
    pub pipeline_tag: String,
    /// Hub library name, e.g. `transformers`. Empty when unobserved.
    /// File key is `library_name` for the same signed-artifact reason.
    #[serde(default, alias = "library_name")]
    pub library_name: String,
    /// GGUF architecture from Hub metadata, e.g. `qwen35`. Empty when unknown.
    #[serde(default)]
    pub architecture: String,
    /// Repo update time, ISO 8601. Empty when unobserved.
    #[serde(default)]
    pub last_modified: String,
    /// Repo creation time, ISO 8601. Empty when unobserved.
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub files: Vec<CatalogFile>,
}

impl CatalogModel {
    /// Smallest offered build, which is what a size-constrained user wants to
    /// see first.
    pub fn smallest_bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size_bytes).min().unwrap_or(0)
    }

    /// Test helper: the fit-rule inputs as a query pair. Proves the serde
    /// defaults exist without reaching into query construction.
    #[cfg(test)]
    fn fit_query(&self, per_mille: u64, budget: u64) -> (u64, u64) {
        let _ = self.smallest_bytes();
        (per_mille, budget)
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
    if catalog.schema_version < MIN_SUPPORTED_SCHEMA || catalog.schema_version > SUPPORTED_SCHEMA {
        return Err(format!(
            "Catalog schema {} is not supported by this build (needs schema {MIN_SUPPORTED_SCHEMA}). Update Localmotive.",
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
    // Ed25519 signs exact bytes, so a CRLF checkout would invalidate the
    // shipped signature. Normalize CRLF to LF before verifying: JSON treats
    // both as insignificant whitespace, and the LF policy in .gitattributes
    // keeps the canonical bytes stable.
    let mut normalized: Vec<u8>;
    let bytes = if body.windows(2).any(|pair| pair == b"\r\n") {
        normalized = body
            .split(|byte| *byte == b'\r')
            .flat_map(|segment| {
                let stripped = segment.strip_prefix(b"\n").unwrap_or(segment);
                stripped.iter().copied().chain(std::iter::once(b'\n'))
            })
            .collect::<Vec<u8>>();
        // Drop the trailing newline the fold above always appends.
        normalized.pop();
        normalized.as_slice()
    } else {
        body
    };
    verify_catalog_signature_with_key(bytes, encoded, &CATALOG_VERIFYING_KEY)
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

/// User-facing input bounds. Every string a user can type passes through these
/// limits in Rust at the Tauri boundary. UI controls are hints only: a
/// compromised webview can send any JSON, so oversize input must be rejected
/// here, never panic or hang. Limits are generous enough that no legitimate
/// use hits them; they exist to bound CPU, memory, and serialization work.
pub const MAX_QUERY_TEXT_LEN: usize = 512;
pub const MAX_QUERY_TERM_LEN: usize = 128;
pub const MAX_FILTER_VALUE_LEN: usize = 128;
pub const MAX_FILTER_MODELS: usize = 10_000;
pub const MAX_FACET_MODELS: usize = 10_000;
pub const MAX_BUDGET_ENTRIES: usize = 64;

/// Validate a catalog query from an untrusted frontend. Rejects oversize
/// text, oversize filter values, and absurd model lists before any filtering
/// work starts.
pub fn validate_catalog_query(query: &CatalogQuery, model_count: usize) -> Result<(), String> {
    if query.text.len() > MAX_QUERY_TEXT_LEN {
        return Err(format!(
            "Search text is too long ({} bytes, maximum {MAX_QUERY_TEXT_LEN}). Shorten the search.",
            query.text.len()
        ));
    }
    for term in query.text.split_whitespace() {
        if term.len() > MAX_QUERY_TERM_LEN {
            return Err(format!(
                "A search term is too long (maximum {MAX_QUERY_TERM_LEN} bytes). Split the search."
            ));
        }
    }
    for (name, value) in [
        ("tag", query.tag.as_str()),
        ("quant", query.quant.as_str()),
        ("author", query.author.as_str()),
        ("license", query.license.as_str()),
        ("pipeline tag", query.pipeline_tag.as_str()),
        ("architecture", query.architecture.as_str()),
    ] {
        if value.len() > MAX_FILTER_VALUE_LEN {
            return Err(format!(
                "The {name} filter is too long (maximum {MAX_FILTER_VALUE_LEN} bytes)."
            ));
        }
    }
    if model_count > MAX_FILTER_MODELS {
        return Err(format!(
            "Too many catalog models ({model_count}, maximum {MAX_FILTER_MODELS}). Refresh the catalog."
        ));
    }
    Ok(())
}

/// Validate facet and budget inputs before work starts.
pub fn validate_facet_models(model_count: usize) -> Result<(), String> {
    if model_count > MAX_FACET_MODELS {
        return Err(format!(
            "Too many catalog models ({model_count}, maximum {MAX_FACET_MODELS}). Refresh the catalog."
        ));
    }
    Ok(())
}

pub fn validate_budget_inputs(dedicated: usize, shared: usize) -> Result<(), String> {
    if dedicated > MAX_BUDGET_ENTRIES || shared > MAX_BUDGET_ENTRIES {
        return Err(format!(
            "Too many hardware budget entries (maximum {MAX_BUDGET_ENTRIES} per kind)."
        ));
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
    /// Exact author match; empty means any.
    pub author: String,
    /// Exact license match, e.g. `apache-2.0`; empty means any.
    pub license: String,
    /// Exact pipeline-tag match, e.g. `text-generation`; empty means any.
    pub pipeline_tag: String,
    /// Exact architecture match, e.g. `qwen35`; empty means any.
    pub architecture: String,
    /// Hide models whose smallest file exceeds this fraction of the hardware
    /// budget, scaled by 1000 (500 = half the budget). 0 disables the rule.
    /// The UI auto-filter sets this from detected VRAM or system memory.
    pub fit_per_mille: u64,
    /// Hardware budget in bytes for the fit rule. 0 disables the rule.
    pub budget_bytes: u64,
}

/// Apply a query. Case-insensitive throughout, because users type lowercase and
/// repository names are mixed case.
pub fn filter_models(models: &[CatalogModel], query: &CatalogQuery) -> Vec<CatalogModel> {
    let text = query.text.trim().to_ascii_lowercase();
    let tag = query.tag.trim().to_ascii_lowercase();
    let quant = query.quant.trim().to_ascii_lowercase();
    let author = query.author.trim().to_ascii_lowercase();
    let license = query.license.trim().to_ascii_lowercase();
    let pipeline = query.pipeline_tag.trim().to_ascii_lowercase();
    let architecture = query.architecture.trim().to_ascii_lowercase();

    let mut out: Vec<CatalogModel> = models
        .iter()
        .filter(|model| {
            if query.hide_gated && model.gated {
                return false;
            }
            if !author.is_empty()
                && model.author.to_ascii_lowercase() != author
                && model.publisher.to_ascii_lowercase() != author
            {
                return false;
            }
            if !license.is_empty() && model.license.to_ascii_lowercase() != license {
                return false;
            }
            if !pipeline.is_empty() && model.pipeline_tag.to_ascii_lowercase() != pipeline {
                return false;
            }
            if !architecture.is_empty() && model.architecture.to_ascii_lowercase() != architecture {
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
            if model_hidden_by_fit_rule(model, query) {
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

/// Whether the hardware-fit rule hides this model. The rule compares the
/// smallest offered file against a tunable fraction of the detected budget:
/// `smallest > budget * per_mille / 1000` hides the row. A zero budget or a
/// zero fraction disables the rule, and saturating math keeps huge budgets
/// from overflowing. Dedicated and shared budgets are never combined here;
/// the caller picks one budget before calling.
pub fn model_hidden_by_fit_rule(model: &CatalogModel, query: &CatalogQuery) -> bool {
    if query.fit_per_mille == 0 || query.budget_bytes == 0 {
        return false;
    }
    let per_mille = query.fit_per_mille.min(1000);
    let limit = (query.budget_bytes as u128)
        .saturating_mul(per_mille as u128)
        .saturating_div(1000);
    (model.smallest_bytes() as u128) > limit
}

/// Pick the single budget the fit rule uses. Dedicated VRAM wins when any
/// adapter reports it; otherwise the largest shared figure; otherwise system
/// memory. Budgets are never summed: combining dedicated and shared memory
/// overstates what one load can use.
pub fn hardware_fit_budget(
    dedicated: &[u64],
    shared: &[u64],
    system_bytes: u64,
) -> (u64, &'static str) {
    if let Some(best) = dedicated.iter().copied().filter(|b| *b > 0).max() {
        return (best, "dedicated");
    }
    if let Some(best) = shared.iter().copied().filter(|b| *b > 0).max() {
        return (best, "shared");
    }
    (system_bytes, "system")
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

/// The single budget the fit rule uses, plus which evidence it came from.
/// The frontend shows the source so the user knows why rows are hidden.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FitBudget {
    pub budget_bytes: u64,
    pub source: String,
}

/// Distinct authors, licenses, pipeline tags, and architectures, for the rich
/// filter controls. Sorted and deduplicated; empty values are omitted.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogFacets {
    pub tags: Vec<String>,
    pub quants: Vec<String>,
    pub authors: Vec<String>,
    pub licenses: Vec<String>,
    pub pipeline_tags: Vec<String>,
    pub architectures: Vec<String>,
}

pub fn rich_facets(models: &[CatalogModel]) -> CatalogFacets {
    let collect = |pick: fn(&CatalogModel) -> &str| {
        models
            .iter()
            .map(pick)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    };
    let authors: Vec<String> = models
        .iter()
        .map(|m| {
            if m.author.trim().is_empty() {
                m.publisher.clone()
            } else {
                m.author.clone()
            }
        })
        .filter(|value| !value.trim().is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let (tags, quants) = facets(models);
    CatalogFacets {
        tags,
        quants,
        authors,
        licenses: collect(|m| m.license.as_str()),
        pipeline_tags: collect(|m| m.pipeline_tag.as_str()),
        architectures: collect(|m| m.architecture.as_str()),
    }
}

/// A catalog plus how it was obtained, so the interface can be honest about
/// whether it is showing live or cached data. `last_success_secs` is the
/// wall-clock second of the last successful network fill (None when never):
/// the UI renders it as last success. `cooldown_remaining_minutes` is Some
/// only when the caller just hit the cooldown instead of the network.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSnapshot {
    pub catalog: Catalog,
    /// "network", "not-modified", "cache" or "bundled".
    pub origin: String,
    pub fetched_at: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_success_secs: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown_remaining_minutes: Option<u64>,
}

#[derive(Deserialize, Serialize)]
struct CacheRecord {
    body: String,
    etag: Option<String>,
    signature: String,
}

/// Refresh cooldown in minutes. The app refuses a network refresh until this
/// long after the last success, so a stuck retry loop cannot hammer the CDN.
/// The UI shows the remaining wait. Tunable; the default covers a daily
/// rhythm with headroom for manual refreshes.
pub const CATALOG_REFRESH_COOLDOWN_MINUTES: u64 = 1560;

/// In-flight refresh guard: only one catalog refresh may run at a time.
/// The guard lives for the refresh duration; a second caller gets a clear
/// error instead of a second network fetch.
pub struct CatalogRefreshGuard {
    _private: (),
}

static CATALOG_REFRESH_ACTIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

impl CatalogRefreshGuard {
    pub fn try_acquire() -> Option<Self> {
        CATALOG_REFRESH_ACTIVE
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .ok()
            .map(|_| CatalogRefreshGuard { _private: () })
    }

    #[cfg(test)]
    pub fn held_for_test() -> bool {
        CATALOG_REFRESH_ACTIVE.load(std::sync::atomic::Ordering::Acquire)
    }
}

impl Drop for CatalogRefreshGuard {
    fn drop(&mut self) {
        CATALOG_REFRESH_ACTIVE.store(false, std::sync::atomic::Ordering::Release);
    }
}

/// Whether a refresh may hit the network now. Returns the remaining wait in
/// whole minutes when the cooldown still applies. A missing or unparsable
/// timestamp never blocks: first start must always be able to fill.
pub fn refresh_cooldown_remaining_minutes(
    last_success_secs: Option<&str>,
    now_secs_value: u64,
    cooldown_minutes: u64,
) -> Option<u64> {
    let last: u64 = last_success_secs?.trim().parse().ok()?;
    let elapsed = now_secs_value.saturating_sub(last);
    let cooldown_secs = cooldown_minutes.saturating_mul(60);
    if elapsed >= cooldown_secs {
        None
    } else {
        Some(cooldown_secs.saturating_sub(elapsed).div_ceil(60))
    }
}

/// Last successful refresh, seconds since epoch. Stored beside the cache so
/// the UI can show last success plus the remaining cooldown.
pub fn refresh_stamp_path(root: &Path) -> PathBuf {
    root.join("catalog-refresh-stamp.txt")
}

pub fn read_refresh_stamp(root: &Path) -> Option<String> {
    std::fs::read_to_string(refresh_stamp_path(root))
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn write_refresh_stamp(root: &Path) {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();
    if secs.is_empty() {
        return;
    }
    let _ = std::fs::create_dir_all(root);
    let _ = std::fs::write(refresh_stamp_path(root), &secs);
}

static CATALOG_CACHE_WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn cache_path(root: &Path) -> PathBuf {
    root.join("catalog-cache.json")
}

fn load_cache_record(root: &Path) -> Option<CacheRecord> {
    let _guard = CATALOG_CACHE_WRITE_LOCK.lock().ok()?;
    let path = cache_path(root);
    let metadata = std::fs::metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_CATALOG_CACHE_BYTES {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
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
    if record.len() as u64 > MAX_CATALOG_CACHE_BYTES {
        return Err("The catalog cache exceeds its size limit".into());
    }
    let cache = cache_path(root);
    // Parallel tests share one process id, so the temp name needs a random
    // suffix too: two threads publishing different bodies must not share one
    // temp file, or a reader can observe a mixed record.
    let temp = {
        let candidate = (0..16)
            .map(|_| {
                root.join(format!(
                    "catalog-cache-{}-{:016x}.tmp",
                    std::process::id(),
                    rand::random::<u64>()
                ))
            })
            .find(|candidate| {
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(candidate)
                    .map(drop)
                    .is_ok()
            })
            .ok_or_else(|| "Could not allocate a unique catalog cache temp file".to_string())?;
        candidate
    };
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
        Err(error) => return fallback(cached_body.map(str::to_string), url, error, cache_root),
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
                    cache_root,
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
                write_refresh_stamp(cache_root);
                return Ok(CatalogSnapshot {
                    catalog: parse_catalog(body)?,
                    origin: "not-modified".into(),
                    fetched_at: now(),
                    url: url.to_string(),
                    last_success_secs: read_refresh_stamp(cache_root),
                    cooldown_remaining_minutes: None,
                });
            }
            if response
                .content_length()
                .is_some_and(|length| length > MAX_CATALOG_BODY_BYTES as u64)
            {
                return fallback(
                    cached_body.map(str::to_string),
                    url,
                    "the catalog response exceeds its size limit".into(),
                    cache_root,
                );
            }
            let body =
                String::from_utf8(read_bounded_catalog_body(response, MAX_CATALOG_BODY_BYTES)?)
                    .map_err(|_| "The catalog response is not UTF-8".to_string())?;
            let signature_url = format!("{url}.sig");
            let signature = match client.get(&signature_url).send() {
                Ok(signature_response) if signature_response.status().is_success() => {
                    if signature_response
                        .content_length()
                        .is_some_and(|length| length > MAX_CATALOG_SIGNATURE_BYTES as u64)
                    {
                        return fallback(
                            cached_body.map(str::to_string),
                            url,
                            "the catalog signature exceeds its size limit".into(),
                            cache_root,
                        );
                    }
                    String::from_utf8(read_bounded_catalog_body(
                        signature_response,
                        MAX_CATALOG_SIGNATURE_BYTES,
                    )?)
                    .map_err(|_| "The catalog signature is not UTF-8".to_string())?
                }
                Ok(signature_response) => {
                    return fallback(
                        cached_body.map(str::to_string),
                        url,
                        format!(
                            "the catalog signature server answered HTTP {}",
                            signature_response.status().as_u16()
                        ),
                        cache_root,
                    )
                }
                Err(error) => {
                    return fallback(
                        cached_body.map(str::to_string),
                        url,
                        format!("could not fetch the catalog signature: {error}"),
                        cache_root,
                    )
                }
            };
            if !verify_catalog_signature(body.as_bytes(), &signature) {
                return fallback(
                    cached_body.map(str::to_string),
                    url,
                    "the catalog signature is invalid".into(),
                    cache_root,
                );
            }
            let catalog = parse_catalog(&body)?;
            // Only cache a signed document that parsed successfully.
            let _ = save_cache_record(cache_root, &body, etag.as_deref(), &signature);
            write_refresh_stamp(cache_root);
            Ok(CatalogSnapshot {
                catalog,
                origin: "network".into(),
                fetched_at: now(),
                url: url.to_string(),
                last_success_secs: read_refresh_stamp(cache_root),
                cooldown_remaining_minutes: None,
            })
        }
        Err(error) => fallback(
            cached_body.map(str::to_string),
            url,
            error.to_string(),
            cache_root,
        ),
    }
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent(format!("Localmotive/{}", env!("CARGO_PKG_VERSION")))
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
    cache_root: &Path,
) -> Result<CatalogSnapshot, String> {
    if let Some(body) = cached_body {
        if let Ok(catalog) = parse_catalog(&body) {
            return Ok(CatalogSnapshot {
                catalog,
                origin: "cache".into(),
                fetched_at: now(),
                url: url.to_string(),
                last_success_secs: read_refresh_stamp(cache_root),
                cooldown_remaining_minutes: None,
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
        last_success_secs: read_refresh_stamp(cache_root),
        cooldown_remaining_minutes: None,
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
pub const HF_KEYRING_SERVICE: &str = "Localmotive HF";
/// Previous product name. Upgrades read the legacy entry once and move it to
/// the current service so an existing token keeps working.
pub const LEGACY_HF_KEYRING_SERVICE: &str = "GGUF Pilot HF";
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
    hf_entry_for(HF_KEYRING_SERVICE)
}

fn hf_entry_for(service: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(service, HF_ACCOUNT)
        .map_err(|error| format!("Could not open Credential Manager: {error}"))
}

fn read_hf_service(service: &str) -> Option<String> {
    let entry = hf_entry_for(service).ok()?;
    match entry.get_password() {
        Ok(secret) if !secret.trim().is_empty() => Some(secret),
        _ => None,
    }
}

/// Read the stored token. Returns `None` rather than an error when absent, so
/// anonymous downloading stays the normal path.
pub fn hf_token() -> Option<String> {
    if let Some(token) = read_hf_service(HF_KEYRING_SERVICE) {
        return Some(token);
    }
    // Legacy installs stored the token under the previous product name.
    // Migrate the value forward so the old entry does not linger.
    if let Some(token) = read_hf_service(LEGACY_HF_KEYRING_SERVICE) {
        let migrated = token.clone();
        if hf_entry()
            .and_then(|entry| {
                entry.set_password(&token).map_err(|error| {
                    format!("Could not save the token to Credential Manager: {error}")
                })
            })
            .is_ok()
        {
            let _ = hf_entry_for(LEGACY_HF_KEYRING_SERVICE).and_then(|entry| {
                match entry.delete_credential() {
                    Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                    Err(error) => Err(format!("Could not remove the token: {error}")),
                }
            });
        }
        return Some(migrated);
    }
    None
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
    // A migrated write replaces the legacy entry; never keep two copies.
    let _ =
        hf_entry_for(LEGACY_HF_KEYRING_SERVICE).and_then(|entry| match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!("Could not remove the token: {error}")),
        });
    Ok(TokenStatus {
        configured: true,
        masked: mask_token(&token),
    })
}

pub fn clear_hf_token() -> Result<TokenStatus, String> {
    let entry = hf_entry()?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => (),
        Err(error) => return Err(format!("Could not remove the token: {error}")),
    }
    // Legacy installs may still hold a token under the previous product name.
    let legacy = hf_entry_for(LEGACY_HF_KEYRING_SERVICE)?;
    match legacy.delete_credential() {
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

    /// Give each test its own temp directory: parallel `cargo test` threads
    /// share one process id, so a pid-keyed name lets two tests wipe each
    /// other's cache records. A random suffix keeps ownership private.
    #[cfg(test)]
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

    /// Phase 0B RED: the signature must verify over the checked-out bytes,
    /// whatever line endings the checkout produced. A CRLF-normalized
    /// `catalog.json` must verify exactly like the LF original; otherwise a
    /// clean Windows clone breaks the catalog gate. This test failed before
    /// the verifier normalized line endings.
    #[test]
    fn shipped_signature_verifies_after_crlf_checkout_normalization() {
        let catalog_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("catalog");
        let body = std::fs::read_to_string(catalog_dir.join("catalog.json")).unwrap();
        let signature = std::fs::read_to_string(catalog_dir.join("catalog.json.sig")).unwrap();
        let crlf = body.replace('\n', "\r\n");
        assert!(
            verify_catalog_signature(crlf.as_bytes(), &signature),
            "signature must survive CRLF checkout normalization"
        );
    }

    #[test]
    fn bounded_catalog_reader_rejects_limit_plus_one() {
        let bytes = vec![b'x'; 33];
        assert!(read_bounded_catalog_body(std::io::Cursor::new(bytes), 32).is_err());
    }

    fn sample() -> Catalog {
        parse_catalog(
            r#"{
              "schemaVersion": 2,
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
        assert!(error.contains("not supported"), "{error}");
        assert!(parse_catalog(r#"{"schemaVersion": 0, "models": []}"#).is_err());
    }

    #[test]
    fn parse_rejects_schema_1_without_rich_fields() {
        // A v1 file has no license, pipeline, architecture, or recency fields.
        // The 0.5 filters cannot be honest about what they hide from such a
        // file, so it fails closed with an upgrade message. Seen live: the
        // bundled v1 catalog kept parsing after SUPPORTED_SCHEMA moved to 2,
        // which would have shown an unfiltered list as if it were filtered.
        let error =
            parse_catalog(r#"{"schemaVersion": 1, "models": [{"id":"x","repo":"a/b","files":[{"quant":"Q4","filename":"x.gguf","sizeBytes":5,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}]}"#)
                .unwrap_err();
        assert!(error.contains("needs schema 2"), "{error}");
    }

    #[test]
    fn parse_rejects_duplicate_model_ids() {
        // React keys and per-model file selections depend on IDs being unique;
        // a bad curator edit must fall back to the last good catalog.
        let json = r#"{
          "schemaVersion": 2,
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
          "schemaVersion": 2,
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
          "schemaVersion": 2,
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
            r#"{"schemaVersion":2,"models":[{"id":"x","repo":"a/b","files":[
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
        let body = br#"{"schemaVersion":2}"#;
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
            br#"{"schemaVersion":3}"#,
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
            r#"{"schemaVersion":2,"models":[
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
    fn filter_applies_rich_author_license_pipeline_and_architecture_constraints() {
        // The 0.5 filter bar narrows by author, license, pipeline tag, and
        // architecture from live HF metadata. A model missing the field is
        // hidden when the filter is set, never treated as a match.
        let catalog = parse_catalog(
            r#"{"schemaVersion":2,"providers":{"source":"t","cutoffDays":90,"allowlist":["a"]},"models":[
              {"id":"a","repo":"unsloth/Alpha-GGUF","author":"unsloth","publisher":"unsloth","license":"apache-2.0","pipelineTag":"text-generation","architecture":"qwen35","downloads":10,"likes":1,"files":[{"quant":"Q4_K_M","filename":"a-Q4_K_M.gguf","sizeBytes":100,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]},
              {"id":"b","repo":"bartowski/Beta-GGUF","author":"bartowski","publisher":"bartowski","license":"llama3.2","pipelineTag":"","architecture":"llama","downloads":5,"likes":1,"files":[{"quant":"Q8_0","filename":"b-Q8_0.gguf","sizeBytes":200,"sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]}
            ]}"#,
        )
        .unwrap();
        let one = |query: CatalogQuery| filter_models(&catalog.models, &query).len();
        assert_eq!(
            one(CatalogQuery {
                author: "UNSLOTH".into(),
                ..Default::default()
            }),
            1,
            "author match is case-insensitive"
        );
        assert_eq!(
            one(CatalogQuery {
                license: "llama3.2".into(),
                ..Default::default()
            }),
            1
        );
        assert_eq!(
            one(CatalogQuery {
                pipeline_tag: "text-generation".into(),
                ..Default::default()
            }),
            1,
            "empty pipeline tag is not a match"
        );
        assert_eq!(
            one(CatalogQuery {
                architecture: "LLAMA".into(),
                ..Default::default()
            }),
            1
        );
        assert_eq!(
            one(CatalogQuery {
                author: "nobody".into(),
                ..Default::default()
            }),
            0
        );
    }

    #[test]
    fn hardware_fit_rule_hides_models_above_a_tunable_budget_fraction() {
        // Default auto-filter: hide files above half the detected budget so a
        // 32 GiB card does not offer a 27 GiB Q8 row first. The user can
        // disable the rule (per_mille 0) or widen it; the UI explains why.
        let model = CatalogModel {
            id: "big".into(),
            repo: "unsloth/Big-GGUF".into(),
            files: vec![CatalogFile {
                quant: "Q8_0".into(),
                filename: "big-Q8_0.gguf".into(),
                size_bytes: 20_000_000_000,
                sha256: "a".repeat(64),
                revision: "main".into(),
                last_modified: String::new(),
                created_at: String::new(),
            }],
            ..Default::default()
        };
        // Serde defaults must exist so older JSON without the new keys keeps
        // parsing; deserializing `{}`-shaped models proves the default path.
        let decoded: CatalogModel = serde_json::from_str(
            r#"{"id":"big","repo":"unsloth/Big-GGUF","files":[{"quant":"Q8_0","filename":"big-Q8_0.gguf","sizeBytes":20000000000,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}"#,
        )
        .unwrap();
        assert_eq!(decoded.fit_query(0, 0).0, 0);
        let fit = |per_mille, budget| CatalogQuery {
            fit_per_mille: per_mille,
            budget_bytes: budget,
            ..Default::default()
        };
        assert!(model_hidden_by_fit_rule(&model, &fit(500, 34_359_738_368)));
        assert!(!model_hidden_by_fit_rule(&model, &fit(0, 34_359_738_368)));
        assert!(!model_hidden_by_fit_rule(&model, &fit(500, 0)));
        assert!(!model_hidden_by_fit_rule(
            &model,
            &fit(1000, 20_000_000_000)
        ));
        assert!(model_hidden_by_fit_rule(&model, &fit(1000, 19_999_999_999)));
        assert!(!model_hidden_by_fit_rule(&model, &fit(9999, u64::MAX)));
    }

    #[test]
    fn hardware_fit_budget_never_combines_dedicated_and_shared() {
        // Dedicated VRAM wins outright; shared is a fallback, never an addend.
        // Seen live: manual overrides already refuse aggregation, and the
        // auto-filter must follow the same rule or it overstates capacity.
        assert_eq!(
            hardware_fit_budget(&[8_000_000_000], &[32_000_000_000], 64_000_000_000),
            (8_000_000_000, "dedicated")
        );
        assert_eq!(
            hardware_fit_budget(&[], &[32_000_000_000], 64_000_000_000),
            (32_000_000_000, "shared")
        );
        assert_eq!(
            hardware_fit_budget(&[], &[], 64_000_000_000),
            (64_000_000_000, "system")
        );
        assert_eq!(hardware_fit_budget(&[], &[], 0), (0, "system"));
    }

    #[test]
    fn rich_facets_list_authors_licenses_pipelines_and_architectures() {
        let catalog = sample();
        let facets = rich_facets(&catalog.models);
        // sample() omits author but carries publisher, so the facet falls back
        // to publisher rather than dropping the row from the author control.
        assert!(facets.authors.contains(&"unsloth".to_string()));
        assert!(facets.authors.contains(&"bartowski".to_string()));
        assert_eq!(facets.tags.len(), 4);
        assert_eq!(facets.quants.len(), 3);
        let authored = parse_catalog(
            r#"{"schemaVersion":2,"providers":{"source":"t","cutoffDays":90,"allowlist":["unsloth"]},"models":[{"id":"a","repo":"unsloth/A-GGUF","author":"unsloth","publisher":"unsloth","license":"apache-2.0","pipelineTag":"text-generation","architecture":"qwen35","files":[{"quant":"Q4_K_M","filename":"a.gguf","sizeBytes":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}]}"#,
        )
        .unwrap();
        let rich = rich_facets(&authored.models);
        assert_eq!(rich.authors, vec!["unsloth"]);
        assert_eq!(rich.licenses, vec!["apache-2.0"]);
        assert_eq!(rich.pipeline_tags, vec!["text-generation"]);
        assert_eq!(rich.architectures, vec!["qwen35"]);
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
    fn snake_file_keys_parse_to_real_values() {
        // The signed v2 artifact uses snake_case file keys. If these parse to
        // "" the pipeline/architecture filters silently hide nothing and the
        // UI lies about what it filters. Seen live: rename_all camelCase
        // ignored pipeline_tag, so this test pins both spellings.
        for key in ["pipeline_tag", "pipelineTag"] {
            let json = format!(
                r#"{{"schemaVersion":2,"providers":{{"source":"t","cutoffDays":90,"allowlist":["u"]}},"models":[{{"id":"a","repo":"u/A-GGUF","files":[{{"quant":"Q4","filename":"a.gguf","sizeBytes":1,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}],"{key}":"text-generation"}}]}}"#
            );
            let catalog = parse_catalog(&json).unwrap();
            assert_eq!(
                catalog.models[0].pipeline_tag, "text-generation",
                "file key {key} must parse"
            );
        }
    }

    #[test]
    fn catalog_query_limits_reject_oversize_text_filters_and_model_lists() {
        // A compromised webview can send any JSON to the filter command. The
        // Rust boundary must reject oversize input with a clear error, never
        // panic or hang. Seen live: filter_catalog took Vec + query with no
        // length checks, so a hostile frontend could submit megabytes of text.
        let big_text = "x".repeat(MAX_QUERY_TEXT_LEN + 1);
        let query = CatalogQuery {
            text: big_text,
            ..Default::default()
        };
        assert!(validate_catalog_query(&query, 0).is_err());
        let big_term = format!("{} {}", "y".repeat(MAX_QUERY_TERM_LEN + 1), "z");
        let query = CatalogQuery {
            text: big_term,
            ..Default::default()
        };
        assert!(validate_catalog_query(&query, 0).is_err());
        for field in [
            ("tag", "tag"),
            ("quant", "quant"),
            ("author", "author"),
            ("license", "license"),
            ("pipeline_tag", "pipeline_tag"),
            ("architecture", "architecture"),
        ] {
            let mut query = CatalogQuery::default();
            let big = "v".repeat(MAX_FILTER_VALUE_LEN + 1);
            match field.1 {
                "tag" => query.tag = big,
                "quant" => query.quant = big,
                "author" => query.author = big,
                "license" => query.license = big,
                "pipeline_tag" => query.pipeline_tag = big,
                _ => query.architecture = big,
            }
            assert!(
                validate_catalog_query(&query, 0).is_err(),
                "oversize {} was accepted",
                field.0
            );
        }
        assert!(validate_catalog_query(&CatalogQuery::default(), MAX_FILTER_MODELS + 1).is_err());
        assert!(validate_catalog_query(&CatalogQuery::default(), MAX_FILTER_MODELS).is_ok());
        assert!(validate_facet_models(MAX_FACET_MODELS + 1).is_err());
        assert!(validate_budget_inputs(MAX_BUDGET_ENTRIES + 1, 0).is_err());
        assert!(validate_budget_inputs(0, MAX_BUDGET_ENTRIES + 1).is_err());
        assert!(validate_budget_inputs(1, 1).is_ok());
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
        let root = unique_test_dir("localmotive-cache-record");
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
        let root = unique_test_dir("localmotive-cache-race");
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
        let root = unique_test_dir("localmotive-cache-reader");
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
    fn refresh_cooldown_blocks_until_1560_minutes_pass_then_releases() {
        // The cooldown keeps a stuck retry loop from hammering the CDN. Seen
        // live: fetch_catalog had no cooldown at all, so every Refresh click
        // hit the network. First start (no stamp) must always fill.
        assert_eq!(
            refresh_cooldown_remaining_minutes(None, 1_000_000, 1560),
            None
        );
        assert_eq!(
            refresh_cooldown_remaining_minutes(Some("not-a-number"), 1_000_000, 1560),
            None
        );
        // Exactly at the boundary the network is allowed again.
        assert_eq!(
            refresh_cooldown_remaining_minutes(Some("0"), 1560 * 60, 1560),
            None
        );
        // One second before the boundary reports the remaining wait.
        assert_eq!(
            refresh_cooldown_remaining_minutes(Some("0"), 1560 * 60 - 1, 1560),
            Some(1)
        );
        // Halfway through reports half the wait, rounded up.
        assert_eq!(
            refresh_cooldown_remaining_minutes(Some("0"), 780 * 60, 1560),
            Some(780)
        );
        // Clock skew into the past never produces a huge wait.
        assert_eq!(
            refresh_cooldown_remaining_minutes(Some("9999999"), 1_000_000, 1560),
            Some(1560)
        );
    }

    #[test]
    fn refresh_stamp_round_trips_through_the_cache_dir() {
        // The UI shows last success plus the remaining cooldown, so the stamp
        // must survive a write/read cycle. Missing stamp means never.
        let root = unique_test_dir("localmotive-refresh-stamp");
        assert_eq!(read_refresh_stamp(&root), None);
        write_refresh_stamp(&root);
        let stamp = read_refresh_stamp(&root).expect("stamp must exist after write");
        assert!(stamp.trim().parse::<u64>().is_ok());
        assert_eq!(
            refresh_cooldown_remaining_minutes(Some(&stamp), u64::MAX, 1560),
            None
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn only_one_catalog_refresh_runs_at_a_time() {
        // Two Refresh clicks must not start two network fetches. The guard
        // releases when dropped, so a finished refresh unblocks the next one.
        assert!(!CatalogRefreshGuard::held_for_test());
        let first = CatalogRefreshGuard::try_acquire().expect("first acquire wins");
        assert!(CatalogRefreshGuard::held_for_test());
        assert!(CatalogRefreshGuard::try_acquire().is_none());
        drop(first);
        assert!(!CatalogRefreshGuard::held_for_test());
        let _second = CatalogRefreshGuard::try_acquire().expect("release unblocks");
    }

    #[test]
    fn cached_catalog_is_used_when_the_network_fails() {
        let root = unique_test_dir("localmotive-cat");
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
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn corrupt_cache_falls_back_to_the_bundled_catalog() {
        // An interrupted external cache edit or disk corruption must not leave
        // the catalog tab empty while the network is also unavailable.
        let root = unique_test_dir("localmotive-corrupt-cat");
        std::fs::write(cache_path(&root), "not json").unwrap();

        let snapshot = fetch_catalog("http://127.0.0.1:9/catalog.json", &root).unwrap();
        assert_eq!(snapshot.origin, "bundled");
        assert!(!snapshot.catalog.models.is_empty());
        let _ = std::fs::remove_dir_all(root);
    }
}
