use crate::artifact::is_reparse_point;
use crate::evidence::EvidenceLevel;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Current on-disk calibration record version (audit S-18.I2).
pub const RECORD_SCHEMA_VERSION: u32 = 1;

fn default_record_schema_version() -> u32 {
    RECORD_SCHEMA_VERSION
}

const MAX_CALIBRATION_RECORD_BYTES: u64 = 64 * 1024;
const MAX_CALIBRATION_RECORDS: usize = 10_000;
/// Retention bound per category (audit S-16.I1): persisting prunes the oldest
/// recognized records beyond this count, keeping the enumeration bound far
/// above it so a load never trips the hard cap. Retention applies to the
/// calibration record store only; exported bundles and run manifests carry
/// their own copies and are untouched.
const MAX_RETAINED_RECORDS_PER_CATEGORY: usize = 4_000;
/// Retained load diagnostics (audit S-16.I2).
const MAX_LOAD_PROBLEMS: usize = 32;
static CALIBRATION_TEMP_NONCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityIdentity {
    pub model_content_sha256: String,
    pub model_architecture: String,
    pub runtime_sha256: String,
    pub runtime_help_sha256: String,
    pub runtime_backend: String,
    pub runtime_version: String,
    pub runtime_build: String,
    pub adapter_ids: Vec<String>,
    pub driver_versions: Vec<String>,
    pub context: u32,
    pub parallel: u16,
    pub batch: u32,
    pub ubatch: u32,
    pub main_gpu: u16,
    pub cache_type_k: String,
    pub cache_type_v: String,
    pub gpu_layers: String,
    pub split_mode: String,
    pub tensor_split: String,
    pub workload_sha256: String,
    pub harness_version: String,
}

pub fn workload_sha256(workload: &crate::evidence::Workload) -> Result<String, String> {
    workload.validate().map_err(|error| error.to_string())?;
    let canonical = serde_json::to_vec(workload).map_err(|error| error.to_string())?;
    Ok(hex::encode(Sha256::digest(canonical)))
}

pub fn selected_artifact_set_sha256(content_ids: &[String]) -> Result<String, String> {
    if content_ids.is_empty() || content_ids.len() > 256 {
        return Err("Selected artifact identity requires between 1 and 256 content digests".into());
    }
    let mut hasher = Sha256::new();
    hasher.update((content_ids.len() as u64).to_le_bytes());
    for (index, content_id) in content_ids.iter().enumerate() {
        crate::evidence::validate_sha256(
            &format!("selectedArtifactContentIds[{index}]"),
            content_id,
        )
        .map_err(|error| error.to_string())?;
        hasher.update((index as u64).to_le_bytes());
        hasher.update(content_id.as_bytes());
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Schema tag for the canonical execution snapshot (audit MT-07). A
/// compatibility key carries this prefix plus the SHA-256 of the canonical
/// snapshot JSON, so legacy keys cannot silently masquerade as a current
/// complete identity.
pub const EXECUTION_SNAPSHOT_SCHEMA: &str = "localmotive.execution-snapshot.v2";

/// Snapshot scopes.
pub const SNAPSHOT_SCOPE_LAUNCH: &str = "launch";
pub const SNAPSHOT_SCOPE_LAUNCH_WORKLOAD: &str = "launch+workload";
pub const EXECUTION_KEY_PREFIX: &str = "v2:";
/// Estimator/metric identity: bump when the calibration arithmetic or the
/// measured statistic changes meaning.
pub const ESTIMATOR_VERSION: &str = "decode-tps-mean.v1";

/// The versioned canonical execution snapshot: every material influence on a
/// measurement, derived from the effective launch arguments plus observed
/// facts, never a hand-maintained subset of profile fields.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSnapshotV2 {
    pub schema_version: String,
    /// What the snapshot scope covers: `launch` for launch-only identities
    /// (quality evidence) or `launch+workload` for measured-run identities
    /// (benchmarks). The scope is part of the canonical key, so a launch
    /// identity and a workload identity can never collide (audit MT-09).
    pub scope: String,
    /// The effective launch arguments after capability filtering, with secret
    /// values and volatile paths replaced by identity tokens.
    pub effective_args: Vec<String>,
    pub model_content_sha256: String,
    pub model_architecture: String,
    pub runtime_sha256: String,
    pub runtime_help_sha256: String,
    pub runtime_backend: String,
    pub runtime_version: String,
    pub runtime_build: String,
    pub adapter_ids: Vec<String>,
    pub driver_versions: Vec<String>,
    pub host_cpu_model: String,
    pub host_platform: String,
    pub host_memory_bytes: u64,
    pub context_requested: u32,
    pub context_effective: Option<u32>,
    pub draft_model_sha256: String,
    pub mmproj_sha256: String,
    pub lora_sha256: String,
    pub workload_sha256: String,
    pub harness_version: String,
    pub estimator_version: String,
    /// Material facts this machine could not observe. A non-empty list makes
    /// the snapshot insufficient for calibration reuse.
    pub unknown_identities: Vec<String>,
}

impl ExecutionSnapshotV2 {
    pub fn reuse_supported(&self) -> bool {
        self.unknown_identities.is_empty()
    }
}

/// Compute the canonical key for a snapshot: `v2:` plus the SHA-256 of the
/// canonical JSON. Every field participates, so any material change yields a
/// different key.
pub fn execution_snapshot_key(snapshot: &ExecutionSnapshotV2) -> Result<String, String> {
    if snapshot.schema_version != EXECUTION_SNAPSHOT_SCHEMA {
        return Err(format!(
            "Unsupported execution snapshot schema: {}",
            snapshot.schema_version
        ));
    }
    if snapshot.scope != SNAPSHOT_SCOPE_LAUNCH && snapshot.scope != SNAPSHOT_SCOPE_LAUNCH_WORKLOAD {
        return Err(format!(
            "Unsupported execution snapshot scope: {}",
            snapshot.scope
        ));
    }
    crate::evidence::validate_sha256("modelContentSha256", &snapshot.model_content_sha256)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("runtimeSha256", &snapshot.runtime_sha256)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("runtimeHelpSha256", &snapshot.runtime_help_sha256)
        .map_err(|error| error.to_string())?;
    if snapshot.scope == SNAPSHOT_SCOPE_LAUNCH_WORKLOAD {
        crate::evidence::validate_sha256("workloadSha256", &snapshot.workload_sha256)
            .map_err(|error| error.to_string())?;
    } else if !snapshot.workload_sha256.is_empty() {
        return Err("A launch-scope execution snapshot must not carry a workload digest".into());
    }
    if snapshot.model_architecture.trim().is_empty()
        || snapshot.runtime_backend.trim().is_empty()
        || snapshot.runtime_version.trim().is_empty()
        || snapshot.runtime_build.trim().is_empty()
        || snapshot.harness_version.trim().is_empty()
        || snapshot.estimator_version.trim().is_empty()
    {
        return Err(
            "Execution snapshot requires model, runtime, harness and estimator identities".into(),
        );
    }
    if snapshot.adapter_ids.len() != snapshot.driver_versions.len() {
        return Err("Execution snapshot adapter and driver identities must align".into());
    }
    let canonical = serde_json::to_vec(snapshot).map_err(|error| error.to_string())?;
    let digest = Sha256::digest(canonical);
    Ok(format!("{EXECUTION_KEY_PREFIX}{}", hex::encode(digest)))
}

/// Compatibility keys must carry the current schema prefix; a legacy key is
/// explicitly insufficient evidence for reuse (audit MT-07 I4).
pub fn validate_compatibility_key(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    let Some(hex_digest) = trimmed.strip_prefix(EXECUTION_KEY_PREFIX) else {
        return Err(
            "This record uses a legacy compatibility identity; rebuild it from current measurements"
                .into(),
        );
    };
    if hex_digest.len() != 64
        || !hex_digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err("Compatibility key is not a v2 sha256 digest".into());
    }
    Ok(())
}

/// Replace secret values and volatile paths in the effective launch arguments
/// with stable identity tokens. The API-key and TLS values never enter the
/// snapshot; file-backed influences are represented by their content digests
/// carried elsewhere in the snapshot.
pub fn sanitize_effective_args(args: &[String]) -> Vec<String> {
    let mut sanitized = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        let argument = args[index].as_str();
        let next = args.get(index + 1).map(String::as_str);
        let token = match argument {
            "-m" | "--model" => Some("[model]"),
            "--mmproj" => Some("[mmproj]"),
            "--lora" | "--lora-scaled" => Some("[lora]"),
            // The short draft flags carry local draft paths exactly like the
            // long forms (audit S-14.I2).
            "-md" | "-mdl" | "--draft-model" | "--spec-draft-model" | "--model-draft" => {
                Some("[draft-model]")
            }
            // Any other path-bearing value the launch builder can emit.
            "--chat-template-file" => Some("[configured]"),
            "--api-key-file" | "--ssl-key-file" | "--ssl-cert-file" => Some("[configured]"),
            value if value.ends_with("draft-model") || value.ends_with("model-draft") => {
                Some("[draft-model]")
            }
            _ => None,
        };
        sanitized.push(argument.to_string());
        if let (Some(token), Some(_)) = (token, next) {
            sanitized.push(token.to_string());
            index += 2;
            continue;
        }
        index += 1;
    }
    sanitized
}

pub fn compatibility_key(identity: &CompatibilityIdentity) -> Result<String, String> {
    crate::evidence::validate_sha256("modelContentSha256", &identity.model_content_sha256)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("runtimeSha256", &identity.runtime_sha256)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("runtimeHelpSha256", &identity.runtime_help_sha256)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("workloadSha256", &identity.workload_sha256)
        .map_err(|error| error.to_string())?;
    if identity.model_architecture.trim().is_empty()
        || identity.runtime_backend.trim().is_empty()
        || identity.runtime_version.trim().is_empty()
        || identity.runtime_build.trim().is_empty()
        || identity.harness_version.trim().is_empty()
    {
        return Err(
            "Compatibility identity requires model, runtime, and harness identities".into(),
        );
    }
    if identity.context == 0
        || identity.parallel == 0
        || identity.batch == 0
        || identity.ubatch == 0
        || identity.ubatch > identity.batch
    {
        return Err("Compatibility identity contains an invalid launch shape".into());
    }
    if identity.adapter_ids.len() != identity.driver_versions.len()
        || identity
            .adapter_ids
            .iter()
            .any(|adapter| adapter.trim().is_empty())
    {
        return Err("Compatibility adapter and driver identities must align".into());
    }
    let canonical = serde_json::to_vec(identity).map_err(|error| error.to_string())?;
    Ok(hex::encode(Sha256::digest(canonical)))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationAnchor {
    /// Record-format version (audit S-18.I2). Defaulted so records written
    /// before the field existed load as version 1; a NEWER version is
    /// rejected with an actionable message instead of being misread.
    #[serde(default = "default_record_schema_version")]
    pub schema_version: u32,
    pub compatibility_key: String,
    pub estimated_value: f64,
    pub measured_value: f64,
    pub observed_at_ms: u64,
    /// The schema of the execution snapshot behind `compatibility_key`
    /// (empty for legacy records).
    #[serde(default)]
    pub snapshot_schema_version: String,
    /// Material facts the source snapshot could not observe.
    #[serde(default)]
    pub unknown_identities: Vec<String>,
    /// Identity of the persisted benchmark run this anchor came from: the
    /// SHA-256 of the manifest bytes. Repeated actions on one run share it,
    /// so one run can never manufacture independent samples (audit MT-08).
    #[serde(default)]
    pub source_run_id: String,
    /// Who produced the estimate (e.g. `manual-estimate.v1`); anchors from
    /// different estimators never mix in one model.
    #[serde(default)]
    pub estimator: String,
}

impl CalibrationAnchor {
    pub fn new(
        compatibility_key: &str,
        estimated_value: f64,
        measured_value: f64,
        observed_at_ms: u64,
    ) -> Self {
        Self::new_with_snapshot(
            compatibility_key,
            EXECUTION_SNAPSHOT_SCHEMA,
            &[],
            estimated_value,
            measured_value,
            observed_at_ms,
        )
    }

    pub fn new_with_snapshot(
        compatibility_key: &str,
        snapshot_schema_version: &str,
        unknown_identities: &[String],
        estimated_value: f64,
        measured_value: f64,
        observed_at_ms: u64,
    ) -> Self {
        Self {
            schema_version: RECORD_SCHEMA_VERSION,
            compatibility_key: compatibility_key.into(),
            estimated_value,
            measured_value,
            observed_at_ms,
            snapshot_schema_version: snapshot_schema_version.into(),
            unknown_identities: unknown_identities.to_vec(),
            source_run_id: String::new(),
            estimator: String::new(),
        }
    }

    /// The full anchor for a persisted run: the source-run identity and the
    /// estimator identity travel with the sample.
    #[allow(clippy::too_many_arguments)]
    pub fn new_for_run(
        compatibility_key: &str,
        snapshot_schema_version: &str,
        unknown_identities: &[String],
        source_run_id: &str,
        estimator: &str,
        estimated_value: f64,
        measured_value: f64,
        observed_at_ms: u64,
    ) -> Self {
        let mut anchor = Self::new_with_snapshot(
            compatibility_key,
            snapshot_schema_version,
            unknown_identities,
            estimated_value,
            measured_value,
            observed_at_ms,
        );
        anchor.source_run_id = source_run_id.into();
        anchor.estimator = estimator.into();
        anchor
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationModel {
    /// Record-format version; see [`CalibrationAnchor::schema_version`].
    #[serde(default = "default_record_schema_version")]
    pub schema_version: u32,
    pub compatibility_key: String,
    pub factor: f64,
    pub residual_standard_deviation: f64,
    pub anchor_count: usize,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    /// The newest source-run observation behind this model: freshness is
    /// measured from the evidence, not from the rebuild time (audit MT-14).
    #[serde(default)]
    pub source_evidence_at_ms: u64,
    /// The estimator identity all anchors shared (audit MT-14 I3).
    #[serde(default)]
    pub estimator: String,
    /// The distinct source-run identities behind the model.
    #[serde(default)]
    pub source_run_ids: Vec<String>,
}

/// The longest accepted calibration TTL and evidence age (audit MT-14).
pub const MAX_CALIBRATION_TTL_MS: u64 = 365 * 24 * 60 * 60 * 1_000;
pub const MAX_EVIDENCE_AGE_MS: u64 = 90 * 24 * 60 * 60 * 1_000;

/// Estimators whose output semantics this build understands.
pub const SUPPORTED_ESTIMATORS: &[&str] = &["manual-estimate.v1"];

/// How a stored calibration model relates to the current configuration and
/// time (audit MT-14 I4): one backend evaluation shared by apply and display.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CalibrationState {
    Compatible,
    Expired,
    Incompatible,
    Scheduled,
    StaleEvidence,
    Invalid,
}

pub fn calibration_model_state(
    model: &CalibrationModel,
    compatibility_key: &str,
    now_ms: u64,
) -> CalibrationState {
    if validate_calibration_model(model).is_err() {
        return CalibrationState::Invalid;
    }
    if model.compatibility_key != compatibility_key {
        return CalibrationState::Incompatible;
    }
    if now_ms < model.created_at_ms {
        return CalibrationState::Scheduled;
    }
    if now_ms >= model.expires_at_ms {
        return CalibrationState::Expired;
    }
    if model.source_evidence_at_ms > 0
        && now_ms.saturating_sub(model.source_evidence_at_ms) > MAX_EVIDENCE_AGE_MS
    {
        return CalibrationState::StaleEvidence;
    }
    CalibrationState::Compatible
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CalibratedEstimate {
    pub value: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub evidence_level: EvidenceLevel,
    pub compatibility_key: String,
    pub expires_at_ms: u64,
}

pub fn build_calibration(
    anchors: &[CalibrationAnchor],
    created_at_ms: u64,
    ttl_ms: u64,
) -> Result<CalibrationModel, String> {
    const MAX_ANCHORS: usize = 10_000;
    const MAX_CALIBRATION_VALUE: f64 = 1_000_000_000.0;
    const MAX_TTL_MS: u64 = 365 * 24 * 60 * 60 * 1_000;
    if anchors.len() < 3 {
        return Err("Calibration requires at least three measured anchors".into());
    }
    if anchors.len() > MAX_ANCHORS {
        return Err(format!("Calibration anchor count exceeds {MAX_ANCHORS}"));
    }
    let key = anchors[0].compatibility_key.trim();
    if key.is_empty() || anchors.iter().any(|anchor| anchor.compatibility_key != key) {
        return Err("Calibration anchors must share one non-empty compatibility key".into());
    }
    validate_compatibility_key(key)?;
    if anchors
        .iter()
        .any(|anchor| anchor.snapshot_schema_version != EXECUTION_SNAPSHOT_SCHEMA)
    {
        return Err(
            "Calibration anchors predate the current execution snapshot schema; rebuild them from current measurements"
                .into(),
        );
    }
    let mut unknowns = anchors
        .iter()
        .flat_map(|anchor| anchor.unknown_identities.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>();
    if !unknowns.is_empty() {
        let listed = unknowns.iter().cloned().collect::<Vec<_>>().join(", ");
        return Err(format!(
            "Calibration reuse needs a complete hardware identity; unknown: {listed}"
        ));
    }
    unknowns.clear();
    if created_at_ms == 0 || ttl_ms == 0 || ttl_ms > MAX_TTL_MS {
        return Err(format!(
            "Calibration timestamps require a positive creation time and TTL no greater than {MAX_TTL_MS} ms"
        ));
    }
    // Distinct source runs (audit MT-08): three clicks on one benchmark or a
    // reimported copy of the same run must never satisfy the count gate.
    let mut source_runs = std::collections::BTreeSet::new();
    for anchor in anchors {
        if anchor.source_run_id.is_empty() {
            return Err(
                "Calibration anchors require a persisted source run identity; re-create them from saved benchmarks"
                    .into(),
            );
        }
        source_runs.insert(anchor.source_run_id.as_str());
    }
    if source_runs.len() < 3 {
        return Err(format!(
            "Calibration requires at least three distinct measured runs; {} distinct run{} found",
            source_runs.len(),
            if source_runs.len() == 1 { "" } else { "s" }
        ));
    }
    let estimator = anchors[0].estimator.trim();
    if estimator.is_empty()
        || anchors
            .iter()
            .any(|anchor| anchor.estimator.trim() != estimator)
    {
        return Err("Calibration anchors must share one non-empty estimator identity".into());
    }
    if anchors.iter().any(|anchor| {
        !anchor.estimated_value.is_finite()
            || !anchor.measured_value.is_finite()
            || anchor.estimated_value <= 0.0
            || anchor.measured_value <= 0.0
            || anchor.estimated_value > MAX_CALIBRATION_VALUE
            || anchor.measured_value > MAX_CALIBRATION_VALUE
            || anchor.observed_at_ms == 0
            || anchor.observed_at_ms > created_at_ms
    }) {
        return Err("Calibration values and observation timestamps are invalid".into());
    }
    let ratios = anchors
        .iter()
        .map(|anchor| anchor.measured_value / anchor.estimated_value)
        .collect::<Vec<_>>();
    let factor = ratios.iter().sum::<f64>() / ratios.len() as f64;
    let variance = ratios
        .iter()
        .map(|ratio| {
            let residual = ratio - factor;
            residual * residual
        })
        .sum::<f64>()
        / ratios.len() as f64;
    if !factor.is_finite() || !variance.is_finite() {
        return Err("Calibration arithmetic exceeded the finite numeric range".into());
    }
    let model = CalibrationModel {
        schema_version: RECORD_SCHEMA_VERSION,
        compatibility_key: key.into(),
        factor,
        residual_standard_deviation: variance.sqrt(),
        anchor_count: anchors.len(),
        created_at_ms,
        expires_at_ms: created_at_ms
            .checked_add(ttl_ms)
            .ok_or("Calibration expiry overflowed")?,
        // Freshness comes from the source runs, not from the rebuild time
        // (audit MT-14 I2), and the estimator/provenance travel with it.
        source_evidence_at_ms: anchors
            .iter()
            .map(|anchor| anchor.observed_at_ms)
            .max()
            .unwrap_or(0),
        estimator: estimator.to_string(),
        source_run_ids: {
            let mut runs = anchors
                .iter()
                .map(|anchor| anchor.source_run_id.clone())
                .collect::<Vec<_>>();
            runs.sort();
            runs.dedup();
            runs
        },
    };
    validate_calibration_model(&model)?;
    Ok(model)
}

pub fn apply_calibration(
    model: &CalibrationModel,
    compatibility_key: &str,
    estimated_value: f64,
    now_ms: u64,
) -> Result<CalibratedEstimate, String> {
    validate_compatibility_key(compatibility_key)?;
    validate_calibration_model(model)?;
    if compatibility_key != model.compatibility_key {
        return Err("Calibration compatibility key does not match".into());
    }
    if now_ms == 0 {
        return Err("Calibration requires a nonzero current time".into());
    }
    if now_ms < model.created_at_ms {
        return Err(
            "Calibration model was created after the current time; check the system clock".into(),
        );
    }
    if now_ms >= model.expires_at_ms {
        return Err("Calibration has expired".into());
    }
    if model.source_evidence_at_ms > 0
        && now_ms.saturating_sub(model.source_evidence_at_ms) > MAX_EVIDENCE_AGE_MS
    {
        return Err(
            "Calibration evidence is older than the supported freshness window; rebuild it from new measurements"
                .into(),
        );
    }
    if !estimated_value.is_finite() || estimated_value <= 0.0 || estimated_value > 1_000_000_000.0 {
        return Err("Estimated value must be positive and finite".into());
    }
    let value = estimated_value * model.factor;
    let uncertainty = estimated_value * model.residual_standard_deviation * 1.96;
    let lower_bound = (value - uncertainty).max(0.0);
    let upper_bound = value + uncertainty;
    if !value.is_finite()
        || !uncertainty.is_finite()
        || !lower_bound.is_finite()
        || !upper_bound.is_finite()
    {
        return Err("Calibration model produced a non-finite result".into());
    }
    Ok(CalibratedEstimate {
        value,
        lower_bound,
        upper_bound,
        evidence_level: EvidenceLevel::Derived,
        compatibility_key: model.compatibility_key.clone(),
        expires_at_ms: model.expires_at_ms,
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExternalEvidenceState {
    Pending,
    Verified,
    Flagged,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalObservation {
    pub metric: String,
    pub value: f64,
    pub unit: String,
    pub observed_at_ms: u64,
}

/// The only provenance an external bundle can carry (audit S-17). It is a
/// fixed label, forced on every import: a reloaded or hand-edited file can
/// never claim to be a locally measured run, and the review state (a user's
/// judgment) is a separate field.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExternalProvenance {
    #[default]
    ImportedExternal,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEvidenceBundle {
    pub schema: u32,
    pub source: String,
    pub compatibility_key: String,
    /// Fixed on import; see [`ExternalProvenance`].
    #[serde(default)]
    pub provenance: ExternalProvenance,
    /// Review state. `Verified` records a user review of this file — not a
    /// local rerun, not an origin signature, and not cryptographic proof of
    /// measurement (audit S-17.I1).
    pub state: ExternalEvidenceState,
    pub records: Vec<ExternalObservation>,
}

pub fn validate_external_evidence(
    mut bundle: ExternalEvidenceBundle,
) -> Result<ExternalEvidenceBundle, String> {
    if bundle.schema != 1 {
        return Err(format!(
            "External evidence schema {} is unsupported; expected 1",
            bundle.schema
        ));
    }
    if bundle.source.trim().is_empty() || bundle.source.len() > 256 {
        return Err("External evidence source must contain 1 to 256 bytes".into());
    }
    crate::evidence::validate_sha256("compatibilityKey", &bundle.compatibility_key)
        .map_err(|error| error.to_string())?;
    if bundle.records.is_empty() || bundle.records.len() > 1_000 {
        return Err("External evidence must contain between 1 and 1000 records".into());
    }
    let serialized_bytes = serde_json::to_vec(&bundle)
        .map_err(|error| format!("Could not validate external evidence size: {error}"))?;
    if serialized_bytes.len() > 1024 * 1024 {
        return Err("External evidence exceeds the 1048576-byte limit".into());
    }
    for record in &bundle.records {
        if record.metric.trim().is_empty()
            || record.metric.len() > 64
            || record.unit.trim().is_empty()
            || record.unit.len() > 64
            || !record.value.is_finite()
            || record.observed_at_ms == 0
        {
            return Err(
                "External evidence records require bounded metrics, units, finite values, and times"
                    .into(),
            );
        }
        let valid_range = match (record.metric.as_str(), record.unit.as_str()) {
            ("decodeTps" | "prefillTps", "tokensPerSecond") => {
                record.value > 0.0 && record.value <= 1_000_000.0
            }
            ("firstTokenMs" | "derivedTtftMs" | "p95LatencyMs", "milliseconds") => {
                record.value > 0.0 && record.value <= 86_400_000.0
            }
            ("peakProcessRssBytes" | "storageBytes", "bytes") => {
                record.value >= 0.0 && record.value <= u64::MAX as f64
            }
            ("qualityPassRate", "ratio") => (0.0..=1.0).contains(&record.value),
            _ => {
                return Err(format!(
                    "External evidence metric '{}' with unit '{}' is unsupported",
                    record.metric, record.unit
                ));
            }
        };
        if !valid_range {
            return Err(format!(
                "External evidence metric '{}' is outside its supported range",
                record.metric
            ));
        }
    }
    // Import is always Pending and always labelled as imported (audit S-17):
    // neither the review state nor the provenance can be self-declared.
    bundle.state = ExternalEvidenceState::Pending;
    bundle.provenance = ExternalProvenance::ImportedExternal;
    Ok(bundle)
}

pub fn review_external_evidence(
    bundle: ExternalEvidenceBundle,
    state: ExternalEvidenceState,
    confirmed: bool,
) -> Result<ExternalEvidenceBundle, String> {
    if !confirmed {
        return Err("External evidence review requires explicit confirmation".into());
    }
    if state == ExternalEvidenceState::Pending {
        return Err("External evidence review requires a terminal review state".into());
    }
    let mut reviewed = validate_external_evidence(bundle)?;
    reviewed.state = state;
    Ok(reviewed)
}

/// Test-only access to the persisted-anchor validator (used by the S-18
/// version contract test).
#[cfg(test)]
pub fn validate_persisted_anchor_for_test(anchor: &CalibrationAnchor) -> Result<(), String> {
    validate_persisted_anchor(anchor)
}

fn validate_persisted_anchor(anchor: &CalibrationAnchor) -> Result<(), String> {
    if anchor.schema_version != RECORD_SCHEMA_VERSION {
        return Err(format!(
            "This calibration anchor uses record version {} but this build reads version {RECORD_SCHEMA_VERSION}; rebuild it from current measurements with a newer Localmotive.",
            anchor.schema_version
        ));
    }
    validate_compatibility_key(&anchor.compatibility_key)?;
    if anchor.observed_at_ms == 0 {
        return Err("Calibration anchor observedAtMs must be greater than zero".into());
    }
    for (label, value) in [
        ("estimatedValue", anchor.estimated_value),
        ("measuredValue", anchor.measured_value),
    ] {
        if !value.is_finite() || value <= 0.0 || value > 1_000_000_000.0 {
            return Err(format!(
                "Calibration anchor {label} is outside the supported range"
            ));
        }
    }
    Ok(())
}

/// The single calibration-model validator used by build, persist, load and
/// apply (audit MT-14 I1): anchor count, creation/expiry ordering, the TTL
/// bound, finite metrics, and provenance shapes.
pub fn validate_calibration_model(model: &CalibrationModel) -> Result<(), String> {
    if model.schema_version != RECORD_SCHEMA_VERSION {
        return Err(format!(
            "This calibration model uses record version {} but this build reads version {RECORD_SCHEMA_VERSION}; rebuild it from current measurements with a newer Localmotive.",
            model.schema_version
        ));
    }
    validate_compatibility_key(&model.compatibility_key)?;
    if model.created_at_ms == 0
        || model.expires_at_ms <= model.created_at_ms
        || model.expires_at_ms - model.created_at_ms > MAX_CALIBRATION_TTL_MS
    {
        return Err("Calibration model timestamps are outside the supported range".into());
    }
    if !(3..=MAX_CALIBRATION_RECORDS).contains(&model.anchor_count) {
        return Err("Calibration model anchor count is outside the supported range".into());
    }
    if !model.factor.is_finite()
        || model.factor <= 0.0
        || !model.residual_standard_deviation.is_finite()
        || model.residual_standard_deviation < 0.0
    {
        return Err("Calibration model metrics are invalid".into());
    }
    if !model.estimator.is_empty() && !SUPPORTED_ESTIMATORS.contains(&model.estimator.as_str()) {
        return Err(format!(
            "Calibration model estimator {} is not supported by this build",
            model.estimator
        ));
    }
    if !model.source_run_ids.is_empty() {
        let unique = model
            .source_run_ids
            .iter()
            .collect::<std::collections::BTreeSet<_>>();
        if unique.len() != model.source_run_ids.len()
            || model.source_run_ids.iter().any(|run| run.trim().is_empty())
        {
            return Err("Calibration model source-run identities are invalid".into());
        }
    }
    if model.source_evidence_at_ms != 0 && model.source_evidence_at_ms > model.created_at_ms {
        return Err("Calibration model evidence postdates its creation".into());
    }
    Ok(())
}

fn ensure_record_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|error| format!("Could not create calibration directory: {error}"))?;
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Could not inspect calibration directory: {error}"))?;
    if !metadata.is_dir() || is_reparse_point(&metadata) || metadata.file_type().is_symlink() {
        return Err("Calibration storage cannot use a reparse point or symbolic link".into());
    }
    Ok(())
}

fn read_bounded_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Could not inspect calibration record: {error}"))?;
    if !metadata.is_file() || is_reparse_point(&metadata) || metadata.file_type().is_symlink() {
        return Err("Calibration records must be regular non-reparse files".into());
    }
    if metadata.len() > MAX_CALIBRATION_RECORD_BYTES {
        return Err("Calibration record exceeds the 64 KiB limit".into());
    }
    let file =
        File::open(path).map_err(|error| format!("Could not open calibration record: {error}"))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_CALIBRATION_RECORD_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read calibration record: {error}"))?;
    if bytes.len() as u64 > MAX_CALIBRATION_RECORD_BYTES {
        return Err("Calibration record grew beyond the 64 KiB limit".into());
    }
    Ok(bytes)
}

fn read_bounded_record<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&read_bounded_bytes(path)?)
        .map_err(|error| format!("Could not parse calibration record: {error}"))
}

fn persist_record<T: Serialize>(
    root: &Path,
    category: &str,
    filename: &str,
    record: &T,
) -> Result<PathBuf, String> {
    let directory = root.join(category);
    ensure_record_directory(&directory)?;
    let bytes = serde_json::to_vec_pretty(record)
        .map_err(|error| format!("Could not serialize calibration record: {error}"))?;
    if bytes.len() as u64 > MAX_CALIBRATION_RECORD_BYTES {
        return Err("Calibration record exceeds the 64 KiB limit".into());
    }
    let target = directory.join(filename);
    if target.exists() {
        let existing = read_bounded_bytes(&target)?;
        if existing == bytes {
            return Ok(target);
        }
        return Err("A different calibration record already uses the target identity".into());
    }
    let nonce = CALIBRATION_TEMP_NONCE.fetch_add(1, Ordering::Relaxed);
    let temporary = directory.join(format!(".{filename}.{}.{}.tmp", std::process::id(), nonce));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| format!("Could not create temporary calibration record: {error}"))?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(format!("Could not write calibration record: {error}"));
    }
    drop(file);
    if let Err(error) = fs::rename(&temporary, &target) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("Could not publish calibration record: {error}"));
    }
    Ok(target)
}

fn record_paths(root: &Path, category: &str) -> Result<Vec<PathBuf>, String> {
    let directory = root.join(category);
    if !directory.exists() {
        return Ok(Vec::new());
    }
    ensure_record_directory(&directory)?;
    let mut paths = Vec::new();
    for entry in fs::read_dir(&directory)
        .map_err(|error| format!("Could not list calibration records: {error}"))?
    {
        let path = entry
            .map_err(|error| format!("Could not inspect calibration record: {error}"))?
            .path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        paths.push(path);
        if paths.len() > MAX_CALIBRATION_RECORDS {
            return Err("Calibration storage contains too many records".into());
        }
    }
    paths.sort();
    Ok(paths)
}

/// A filesystem-safe token for a compatibility key (the key itself carries
/// the `v2:` schema prefix, which is not a Windows filename character).
fn key_file_token(compatibility_key: &str) -> String {
    compatibility_key
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .take(16)
        .collect()
}

pub fn persist_calibration_anchor(
    root: &Path,
    anchor: &CalibrationAnchor,
) -> Result<PathBuf, String> {
    validate_persisted_anchor(anchor)?;
    // One source run contributes at most one anchor for a compatibility key
    // (audit MT-08 I2): the filename derives from the run identity, so
    // persist_record refuses a different anchor for the same run and treats a
    // byte-identical repeat as idempotent. Repeated Add anchor actions cannot
    // manufacture samples.
    let file_token = if anchor.source_run_id.is_empty() {
        format!(
            "{}-{}",
            anchor.observed_at_ms,
            key_file_token(&anchor.compatibility_key)
        )
    } else {
        format!(
            "{}-{}",
            key_file_token(&anchor.source_run_id),
            key_file_token(&anchor.compatibility_key)
        )
    };
    persist_record(root, "anchors", &format!("{file_token}.json"), anchor)
}

/// Records plus bounded diagnostics from one history load (audit S-16.I2).
/// A corrupt, oversized or invalid record is quarantined individually and the
/// load continues, so one bad file can never hide the valid history.
#[derive(Clone, Debug)]
pub struct LoadedRecords<T> {
    pub records: Vec<T>,
    pub problems: Vec<String>,
}

impl<T> Default for LoadedRecords<T> {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            problems: Vec::new(),
        }
    }
}

impl<T> LoadedRecords<T> {
    fn push_problem(&mut self, problem: String) {
        if self.problems.len() < MAX_LOAD_PROBLEMS {
            self.problems.push(problem);
        } else if let Some(last) = self.problems.last_mut() {
            *last = "Further record diagnostics were suppressed (limit reached).".into();
        }
    }
}

/// Move a record that failed to load into `<root>/quarantine/` so its bytes
/// are preserved for inspection instead of deleted or rewritten.
fn quarantine_record(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let directory = root.join("quarantine");
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create the quarantine directory: {error}"))?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("record.json");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let target = directory.join(format!("{name}.corrupt-{stamp}"));
    fs::rename(path, &target)
        .map_err(|error| format!("Could not quarantine the record: {error}"))?;
    Ok(target)
}

pub fn load_calibration_anchors(
    root: &Path,
    compatibility_key: &str,
) -> Result<LoadedRecords<CalibrationAnchor>, String> {
    validate_compatibility_key(compatibility_key)?;
    let mut loaded = LoadedRecords::default();
    for path in record_paths(root, "anchors")? {
        let record: CalibrationAnchor = match read_bounded_record(&path) {
            Ok(record) => record,
            Err(error) => {
                let name = path.display().to_string();
                let quarantined = quarantine_record(root, &path)
                    .map(|target| target.display().to_string())
                    .unwrap_or_else(|quarantine_error| {
                        format!("(quarantine failed: {quarantine_error})")
                    });
                loaded.push_problem(format!("{name}: {error}; moved to {quarantined}"));
                continue;
            }
        };
        if let Err(error) = validate_persisted_anchor(&record) {
            let name = path.display().to_string();
            let quarantined = quarantine_record(root, &path)
                .map(|target| target.display().to_string())
                .unwrap_or_else(|quarantine_error| {
                    format!("(quarantine failed: {quarantine_error})")
                });
            loaded.push_problem(format!("{name}: {error}; moved to {quarantined}"));
            continue;
        }
        if record.compatibility_key == compatibility_key {
            loaded.records.push(record);
        }
    }
    loaded.records.sort_by_key(|record| record.observed_at_ms);
    loaded.records.dedup();
    Ok(loaded)
}

pub fn persist_calibration_model(root: &Path, model: &CalibrationModel) -> Result<PathBuf, String> {
    validate_calibration_model(model)?;
    persist_record(
        root,
        "models",
        &format!(
            "{}-{}.json",
            model.created_at_ms,
            key_file_token(&model.compatibility_key)
        ),
        model,
    )
}

pub fn load_calibration_models(
    root: &Path,
    compatibility_key: &str,
) -> Result<LoadedRecords<CalibrationModel>, String> {
    validate_compatibility_key(compatibility_key)?;
    let mut loaded = LoadedRecords::default();
    for path in record_paths(root, "models")? {
        let record: CalibrationModel = match read_bounded_record(&path) {
            Ok(record) => record,
            Err(error) => {
                let name = path.display().to_string();
                let quarantined = quarantine_record(root, &path)
                    .map(|target| target.display().to_string())
                    .unwrap_or_else(|quarantine_error| {
                        format!("(quarantine failed: {quarantine_error})")
                    });
                loaded.push_problem(format!("{name}: {error}; moved to {quarantined}"));
                continue;
            }
        };
        if let Err(error) = validate_calibration_model(&record) {
            let name = path.display().to_string();
            let quarantined = quarantine_record(root, &path)
                .map(|target| target.display().to_string())
                .unwrap_or_else(|quarantine_error| {
                    format!("(quarantine failed: {quarantine_error})")
                });
            loaded.push_problem(format!("{name}: {error}; moved to {quarantined}"));
            continue;
        }
        if record.compatibility_key == compatibility_key {
            loaded.records.push(record);
        }
    }
    loaded.records.sort_by_key(|record| record.created_at_ms);
    loaded.records.dedup();
    Ok(loaded)
}

/// Prune the oldest recognized records beyond the retention bound (audit
/// S-16.I1). Returns the number of files removed. Only `.json` records in
/// the named category are candidates; anything else in the directory is
/// left alone.
pub fn prune_records(root: &Path, category: &str) -> Result<usize, String> {
    prune_records_with(root, category, MAX_RETAINED_RECORDS_PER_CATEGORY)
}

/// [`prune_records`] with an explicit bound, so tests exercise the same code
/// without creating thousands of files.
pub fn prune_records_with(root: &Path, category: &str, bound: usize) -> Result<usize, String> {
    let mut paths = record_paths(root, category)?;
    if paths.len() <= bound {
        return Ok(0);
    }
    paths.sort();
    let excess = paths.len() - bound;
    let mut removed = 0usize;
    for path in paths.into_iter().take(excess) {
        if fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

/// Explicit user cleanup (audit S-16.I1): remove every calibration record in
/// both categories and report how many files were removed. Quarantined files
/// are kept: they are the diagnostic evidence of earlier failures.
pub fn clear_calibration_history(root: &Path) -> Result<usize, String> {
    let mut removed = 0usize;
    for category in ["anchors", "models"] {
        for path in record_paths(root, category)? {
            if fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_key_changes_when_a_material_identity_changes() {
        let identity = CompatibilityIdentity {
            model_content_sha256: "a".repeat(64),
            model_architecture: "llama".into(),
            runtime_sha256: "b".repeat(64),
            runtime_help_sha256: "c".repeat(64),
            runtime_backend: "cuda".into(),
            runtime_version: "0.0.1".into(),
            runtime_build: "1".into(),
            adapter_ids: vec!["gpu-0".into()],
            driver_versions: vec!["1.2.3".into()],
            context: 4_096,
            parallel: 1,
            batch: 512,
            ubatch: 128,
            main_gpu: 0,
            cache_type_k: "f16".into(),
            cache_type_v: "f16".into(),
            gpu_layers: "all".into(),
            split_mode: "layer".into(),
            tensor_split: String::new(),
            workload_sha256: "d".repeat(64),
            harness_version: "benchmark-v2".into(),
        };
        let mut changed = identity.clone();
        changed.context = 8_192;

        assert_eq!(compatibility_key(&identity).unwrap().len(), 64);
        assert_ne!(
            compatibility_key(&identity).unwrap(),
            compatibility_key(&changed).unwrap()
        );

        changed = identity.clone();
        changed.batch = 1_024;
        assert_ne!(
            compatibility_key(&identity).unwrap(),
            compatibility_key(&changed).unwrap()
        );
    }

    #[test]
    fn workload_digest_changes_when_a_material_workload_input_changes() {
        let workload = crate::evidence::Workload::default();
        let mut changed = workload.clone();
        changed.prompt_tokens += 1;

        assert_ne!(
            workload_sha256(&workload).unwrap(),
            workload_sha256(&changed).unwrap()
        );
    }

    #[test]
    fn external_evidence_import_stays_pending_after_schema_validation() {
        let bundle = ExternalEvidenceBundle {
            schema: 1,
            provenance: ExternalProvenance::ImportedExternal,
            source: "community-fixture".into(),
            compatibility_key: "c".repeat(64),
            state: ExternalEvidenceState::Verified,
            records: vec![ExternalObservation {
                metric: "decodeTps".into(),
                value: 42.0,
                unit: "tokensPerSecond".into(),
                observed_at_ms: 42,
            }],
        };

        let validated = validate_external_evidence(bundle).unwrap();

        assert_eq!(validated.state, ExternalEvidenceState::Pending);
        assert_eq!(validated.records.len(), 1);
    }

    #[test]
    fn external_evidence_enforces_metric_ranges_and_string_limits() {
        let mut bundle = ExternalEvidenceBundle {
            schema: 1,
            provenance: ExternalProvenance::ImportedExternal,
            source: "community-fixture".into(),
            compatibility_key: "c".repeat(64),
            state: ExternalEvidenceState::Pending,
            records: vec![ExternalObservation {
                metric: "qualityPassRate".into(),
                value: 1.1,
                unit: "ratio".into(),
                observed_at_ms: 42,
            }],
        };

        assert!(validate_external_evidence(bundle.clone())
            .unwrap_err()
            .contains("qualityPassRate"));
        bundle.records[0].value = 0.8;
        bundle.source = "x".repeat(257);
        assert!(validate_external_evidence(bundle)
            .unwrap_err()
            .contains("source"));
    }

    #[test]
    fn external_evidence_review_requires_confirmation_and_a_terminal_state() {
        let bundle = ExternalEvidenceBundle {
            schema: 1,
            provenance: ExternalProvenance::ImportedExternal,
            source: "community-fixture".into(),
            compatibility_key: "c".repeat(64),
            state: ExternalEvidenceState::Pending,
            records: vec![ExternalObservation {
                metric: "decodeTps".into(),
                value: 42.0,
                unit: "tokensPerSecond".into(),
                observed_at_ms: 42,
            }],
        };

        assert!(
            review_external_evidence(bundle.clone(), ExternalEvidenceState::Verified, false,)
                .is_err()
        );
        assert!(
            review_external_evidence(bundle.clone(), ExternalEvidenceState::Pending, true,)
                .is_err()
        );
        assert_eq!(
            review_external_evidence(bundle, ExternalEvidenceState::Flagged, true)
                .unwrap()
                .state,
            ExternalEvidenceState::Flagged
        );
    }

    fn test_key(seed: char) -> String {
        format!("v2:{}", seed.to_string().repeat(64))
    }

    /// An anchor as the run-derived flow creates it (audit MT-08): a distinct
    /// source-run identity and one estimator identity.
    fn test_anchor(
        key: &str,
        source_run: &str,
        estimated: f64,
        measured: f64,
        at: u64,
    ) -> CalibrationAnchor {
        CalibrationAnchor::new_for_run(
            key,
            EXECUTION_SNAPSHOT_SCHEMA,
            &[],
            source_run,
            "manual-estimate.v1",
            estimated,
            measured,
            at,
        )
    }

    #[test]
    fn mt14_apply_enforces_creation_expiry_and_evidence_freshness() {
        let key = test_key('a');
        let anchors = vec![
            test_anchor(&key, "run-1", 100.0, 110.0, 10),
            test_anchor(&key, "run-2", 200.0, 220.0, 20),
            test_anchor(&key, "run-3", 300.0, 330.0, 30),
        ];
        // The TTL outlives the evidence-freshness window so the freshness
        // rule itself is exercised, not the earlier expiry rule.
        let model = build_calibration(&anchors, 1_000, MAX_CALIBRATION_TTL_MS).unwrap();
        assert_eq!(model.source_evidence_at_ms, 30);
        assert_eq!(model.estimator, "manual-estimate.v1");
        assert_eq!(model.source_run_ids.len(), 3);

        // A clock behind the creation time refuses application.
        let error = apply_calibration(&model, &key, 400.0, 500).unwrap_err();
        assert!(error.contains("created after the current time"), "{error}");

        // Exact expiry: at the expiry instant the model is expired.
        let error = apply_calibration(&model, &key, 400.0, model.expires_at_ms).unwrap_err();
        assert!(error.contains("expired"), "{error}");

        // Freshness is measured from the source runs, not the rebuild time:
        // long after the runs, the model stops being applicable even inside
        // its nominal TTL.
        let long_after = model.created_at_ms + MAX_EVIDENCE_AGE_MS + 1;
        assert!(
            long_after < model.expires_at_ms,
            "the fixture must outlive the freshness window"
        );
        let error = apply_calibration(&model, &key, 400.0, long_after).unwrap_err();
        assert!(error.contains("freshness"), "{error}");

        // A fresh application still works.
        let applied = apply_calibration(&model, &key, 400.0, 5_000).unwrap();
        assert!(applied.lower_bound.is_finite() && applied.upper_bound.is_finite());
    }

    #[test]
    fn mt14_one_validator_rejects_bad_models_at_every_entry_point() {
        let key = test_key('a');
        let valid = build_calibration(
            &[
                test_anchor(&key, "run-1", 100.0, 110.0, 10),
                test_anchor(&key, "run-2", 200.0, 220.0, 20),
                test_anchor(&key, "run-3", 300.0, 330.0, 30),
            ],
            40,
            1_000,
        )
        .unwrap();
        validate_calibration_model(&valid).unwrap();

        // Inverted timestamps.
        let mut inverted = valid.clone();
        inverted.expires_at_ms = inverted.created_at_ms;
        assert!(apply_calibration(&inverted, &key, 400.0, 100).is_err());
        assert!(validate_calibration_model(&inverted).is_err());

        // Overlong TTL.
        let mut overlong = valid.clone();
        overlong.expires_at_ms = overlong.created_at_ms + MAX_CALIBRATION_TTL_MS + 1;
        assert!(validate_calibration_model(&overlong).is_err());

        // Insufficient anchor count.
        let mut few = valid.clone();
        few.anchor_count = 2;
        assert!(validate_calibration_model(&few).is_err());

        // An estimator this build does not understand.
        let mut foreign = valid.clone();
        foreign.estimator = "mystery-estimator.v9".into();
        assert!(validate_calibration_model(&foreign)
            .unwrap_err()
            .contains("not supported"));

        // Duplicate source-run provenance.
        let mut duplicated = valid.clone();
        duplicated.source_run_ids = vec!["run-1".into(), "run-1".into(), "run-3".into()];
        assert!(validate_calibration_model(&duplicated).is_err());

        // Evidence that postdates creation.
        let mut paradoxical = valid.clone();
        paradoxical.source_evidence_at_ms = paradoxical.created_at_ms + 1;
        assert!(validate_calibration_model(&paradoxical).is_err());

        // Persist and load both reject an invalid model: the validator is
        // shared, not a load-only epilogue.
        let root = std::env::temp_dir().join(format!(
            "localmotive-mt14-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(persist_calibration_model(&root, &overlong).is_err());
        persist_calibration_model(&root, &valid).unwrap();
        // A record tampered on disk fails the same validator at load. Since
        // audit S-16 the failure is quarantined and reported instead of
        // failing the whole load: one bad file must not hide valid history.
        let path = record_paths(&root, "models").unwrap().pop().unwrap();
        let mut tampered = valid.clone();
        tampered.factor = f64::INFINITY;
        std::fs::write(&path, serde_json::to_vec_pretty(&tampered).unwrap()).unwrap();
        let loaded = load_calibration_models(&root, &key).unwrap();
        assert!(loaded.records.is_empty());
        assert_eq!(loaded.problems.len(), 1, "{:?}", loaded.problems);
        assert!(
            loaded.problems[0].contains("quarantine"),
            "{:?}",
            loaded.problems
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn mt14_state_evaluation_matches_apply_semantics() {
        let key = test_key('a');
        let model = build_calibration(
            &[
                test_anchor(&key, "run-1", 100.0, 110.0, 10),
                test_anchor(&key, "run-2", 200.0, 220.0, 20),
                test_anchor(&key, "run-3", 300.0, 330.0, 30),
            ],
            1_000,
            MAX_CALIBRATION_TTL_MS,
        )
        .unwrap();
        assert_eq!(
            calibration_model_state(&model, &key, 5_000),
            CalibrationState::Compatible
        );
        assert_eq!(
            calibration_model_state(&model, &key, 500),
            CalibrationState::Scheduled
        );
        assert_eq!(
            calibration_model_state(&model, &key, model.expires_at_ms),
            CalibrationState::Expired
        );
        assert_eq!(
            calibration_model_state(&model, &test_key('b'), 5_000),
            CalibrationState::Incompatible
        );
        assert_eq!(
            calibration_model_state(&model, &key, model.created_at_ms + MAX_EVIDENCE_AGE_MS + 1),
            CalibrationState::StaleEvidence
        );
        let mut foreign = model.clone();
        foreign.estimator = "mystery-estimator.v9".into();
        assert_eq!(
            calibration_model_state(&foreign, &key, 5_000),
            CalibrationState::Invalid
        );

        // Every non-Compatible state is refused by apply with a matching
        // message category.
        assert!(apply_calibration(&model, &key, 400.0, 500).is_err());
        assert!(apply_calibration(&model, &key, 400.0, model.expires_at_ms).is_err());
        assert!(apply_calibration(&model, &test_key('b'), 400.0, 5_000).is_err());
        assert!(apply_calibration(
            &model,
            &key,
            400.0,
            model.created_at_ms + MAX_EVIDENCE_AGE_MS + 1
        )
        .is_err());
        assert!(apply_calibration(&foreign, &key, 400.0, 5_000).is_err());
    }

    #[test]
    fn calibration_requires_matching_compatibility_and_expires() {
        let key = test_key('a');
        let anchors = vec![
            test_anchor(&key, "run-1", 100.0, 110.0, 10),
            test_anchor(&key, "run-2", 200.0, 220.0, 20),
            test_anchor(&key, "run-3", 300.0, 330.0, 30),
        ];
        let model = build_calibration(&anchors, 40, 100).unwrap();

        let applied = apply_calibration(&model, &key, 400.0, 50).unwrap();
        assert!((applied.value - 440.0).abs() < 0.000_001);
        assert!(apply_calibration(&model, &test_key('b'), 400.0, 50).is_err());
        assert!(apply_calibration(&model, &key, 400.0, 140).is_err());
    }

    #[test]
    fn calibration_rejects_malformed_keys_and_unbounded_ttl() {
        // A legacy 64-hex key is explicitly insufficient (audit MT-07 I4).
        let legacy = vec![
            CalibrationAnchor::new(&"a".repeat(64), 100.0, 110.0, 10),
            CalibrationAnchor::new(&"a".repeat(64), 200.0, 220.0, 20),
            CalibrationAnchor::new(&"a".repeat(64), 300.0, 330.0, 30),
        ];
        assert!(build_calibration(&legacy, 40, 100)
            .unwrap_err()
            .contains("legacy compatibility identity"));

        // A malformed v2 digest is rejected.
        let malformed = vec![
            CalibrationAnchor::new("v2:nothex", 100.0, 110.0, 10),
            CalibrationAnchor::new("v2:nothex", 200.0, 220.0, 20),
            CalibrationAnchor::new("v2:nothex", 300.0, 330.0, 30),
        ];
        assert!(build_calibration(&malformed, 40, 100)
            .unwrap_err()
            .contains("sha256"));

        // An anchor that predates the snapshot schema is rejected even with a
        // v2 key.
        let key = test_key('a');
        let mut stale = vec![
            test_anchor(&key, "run-1", 100.0, 110.0, 10),
            test_anchor(&key, "run-2", 200.0, 220.0, 20),
            test_anchor(&key, "run-3", 300.0, 330.0, 30),
        ];
        for anchor in &mut stale {
            anchor.snapshot_schema_version = String::new();
        }
        assert!(build_calibration(&stale, 40, 100)
            .unwrap_err()
            .contains("current execution snapshot schema"));

        // A valid v2 set passes the key checks; the TTL bound still applies.
        let valid = vec![
            test_anchor(&key, "run-1", 100.0, 110.0, 10),
            test_anchor(&key, "run-2", 200.0, 220.0, 20),
            test_anchor(&key, "run-3", 300.0, 330.0, 30),
        ];
        assert!(build_calibration(&valid, 40, 366 * 24 * 60 * 60 * 1_000).is_err());
    }

    #[test]
    fn calibration_records_persist_as_bounded_local_files() {
        let root = std::env::temp_dir().join(format!(
            "localmotive-calibration-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let key = test_key('a');
        let anchors = vec![
            test_anchor(&key, "run-1", 10.0, 11.0, 10),
            test_anchor(&key, "run-2", 20.0, 22.0, 20),
            test_anchor(&key, "run-3", 30.0, 33.0, 30),
        ];
        let anchor = anchors[0].clone();
        let model = build_calibration(&anchors, 50, 100).unwrap();

        persist_calibration_anchor(&root, &anchor).unwrap();
        persist_calibration_model(&root, &model).unwrap();
        let loaded = load_calibration_anchors(&root, &key).unwrap();
        assert_eq!(loaded.records, vec![anchor]);
        assert!(loaded.problems.is_empty());
        let loaded = load_calibration_models(&root, &key).unwrap();
        assert_eq!(loaded.records, vec![model]);
        assert!(loaded.problems.is_empty());

        std::fs::remove_dir_all(root).unwrap();
    }

    fn snapshot_fixture() -> ExecutionSnapshotV2 {
        ExecutionSnapshotV2 {
            schema_version: EXECUTION_SNAPSHOT_SCHEMA.into(),
            scope: SNAPSHOT_SCOPE_LAUNCH_WORKLOAD.into(),
            effective_args: vec![
                "-m".into(),
                "[model]".into(),
                "--threads".into(),
                "8".into(),
            ],
            model_content_sha256: "a".repeat(64),
            model_architecture: "qwen35".into(),
            runtime_sha256: "b".repeat(64),
            runtime_help_sha256: "c".repeat(64),
            runtime_backend: "vulkan".into(),
            runtime_version: "1.0".into(),
            runtime_build: "b10816".into(),
            adapter_ids: vec!["luid:0000:1111".into()],
            driver_versions: vec!["32.0.15.7283".into()],
            host_cpu_model: "AMD Ryzen 9 9950X3D".into(),
            host_platform: "windows".into(),
            host_memory_bytes: 64 * 1024 * 1024 * 1024,
            context_requested: 32768,
            context_effective: Some(32768),
            draft_model_sha256: String::new(),
            mmproj_sha256: String::new(),
            lora_sha256: String::new(),
            workload_sha256: "d".repeat(64),
            harness_version: "0.5.0".into(),
            estimator_version: ESTIMATOR_VERSION.into(),
            unknown_identities: Vec::new(),
        }
    }

    #[test]
    fn mt07_snapshot_key_changes_for_every_material_field() {
        let baseline = snapshot_fixture();
        assert!(baseline.reuse_supported());
        let baseline_key = execution_snapshot_key(&baseline).unwrap();
        assert!(baseline_key.starts_with(EXECUTION_KEY_PREFIX));

        type SnapshotMutation = (&'static str, Box<dyn Fn(&mut ExecutionSnapshotV2)>);
        let mutations: Vec<SnapshotMutation> = vec![
            (
                "effective_args",
                Box::new(|s| s.effective_args.push("--flash-attn".into())),
            ),
            (
                "model_content_sha256",
                Box::new(|s| s.model_content_sha256 = "e".repeat(64)),
            ),
            (
                "model_architecture",
                Box::new(|s| s.model_architecture = "llama".into()),
            ),
            (
                "runtime_sha256",
                Box::new(|s| s.runtime_sha256 = "e".repeat(64)),
            ),
            (
                "runtime_help_sha256",
                Box::new(|s| s.runtime_help_sha256 = "e".repeat(64)),
            ),
            (
                "runtime_backend",
                Box::new(|s| s.runtime_backend = "cuda".into()),
            ),
            (
                "runtime_version",
                Box::new(|s| s.runtime_version = "2.0".into()),
            ),
            (
                "runtime_build",
                Box::new(|s| s.runtime_build = "b20000".into()),
            ),
            (
                "adapter_ids",
                Box::new(|s| {
                    s.adapter_ids.push("luid:0000:2222".into());
                    s.driver_versions.push("32.0.15.7283".into());
                }),
            ),
            (
                "driver_versions",
                Box::new(|s| s.driver_versions[0] = "32.0.15.9999".into()),
            ),
            (
                "host_cpu_model",
                Box::new(|s| s.host_cpu_model = "Intel Core i9".into()),
            ),
            (
                "host_platform",
                Box::new(|s| s.host_platform = "linux".into()),
            ),
            (
                "host_memory_bytes",
                Box::new(|s| s.host_memory_bytes = 32 * 1024 * 1024 * 1024),
            ),
            (
                "context_requested",
                Box::new(|s| s.context_requested = 8192),
            ),
            (
                "context_effective",
                Box::new(|s| s.context_effective = Some(8192)),
            ),
            (
                "draft_model_sha256",
                Box::new(|s| s.draft_model_sha256 = "e".repeat(64)),
            ),
            (
                "mmproj_sha256",
                Box::new(|s| s.mmproj_sha256 = "e".repeat(64)),
            ),
            ("lora_sha256", Box::new(|s| s.lora_sha256 = "e".repeat(64))),
            (
                "workload_sha256",
                Box::new(|s| s.workload_sha256 = "e".repeat(64)),
            ),
            (
                "harness_version",
                Box::new(|s| s.harness_version = "0.6.0".into()),
            ),
            (
                "estimator_version",
                Box::new(|s| s.estimator_version = "other.v9".into()),
            ),
        ];
        for (label, mutate) in mutations {
            let mut changed = baseline.clone();
            mutate(&mut changed);
            let key = execution_snapshot_key(&changed).unwrap();
            assert_ne!(key, baseline_key, "field {label} did not change the key");
        }
    }

    #[test]
    fn mt07_cpu_only_and_hardware_changes_are_distinguished() {
        // CPU-only machine: no adapters, empty driver list, cpu backend.
        let mut cpu_only = snapshot_fixture();
        cpu_only.runtime_backend = "cpu".into();
        cpu_only.adapter_ids.clear();
        cpu_only.driver_versions.clear();
        cpu_only.host_cpu_model = "AMD Ryzen 9 9950X3D".into();
        let cpu_key = execution_snapshot_key(&cpu_only).unwrap();

        // The same GPU with a changed CPU must not share the identity.
        let mut changed_cpu = snapshot_fixture();
        changed_cpu.host_cpu_model = "Intel Core Ultra 9".into();
        assert_ne!(
            execution_snapshot_key(&changed_cpu).unwrap(),
            execution_snapshot_key(&snapshot_fixture()).unwrap()
        );

        // Fit-reduced effective context changes the identity.
        let mut fit_reduced = snapshot_fixture();
        fit_reduced.context_requested = 32768;
        fit_reduced.context_effective = Some(16384);
        assert_ne!(
            execution_snapshot_key(&fit_reduced).unwrap(),
            execution_snapshot_key(&snapshot_fixture()).unwrap()
        );

        // The same draft companion file with changed draft settings differs
        // through the effective arguments.
        let mut draft_a = snapshot_fixture();
        draft_a.draft_model_sha256 = "f".repeat(64);
        draft_a
            .effective_args
            .extend(["--spec-draft-n-max".into(), "16".into()]);
        let mut draft_b = draft_a.clone();
        draft_b.effective_args.pop();
        draft_b.effective_args.push("64".into());
        assert_ne!(
            execution_snapshot_key(&draft_a).unwrap(),
            execution_snapshot_key(&draft_b).unwrap()
        );

        // Changed LoRA bytes under the same filename change the digest.
        let mut lora_a = snapshot_fixture();
        lora_a.lora_sha256 = "1".repeat(64);
        let mut lora_b = snapshot_fixture();
        lora_b.lora_sha256 = "2".repeat(64);
        assert_ne!(
            execution_snapshot_key(&lora_a).unwrap(),
            execution_snapshot_key(&lora_b).unwrap()
        );

        assert!(cpu_key.starts_with(EXECUTION_KEY_PREFIX));
    }

    #[test]
    fn mt07_unknown_identity_blocks_reuse_and_legacy_keys_stay_out() {
        let mut partial = snapshot_fixture();
        partial.unknown_identities = vec!["driverVersion:luid:0000:1111".into()];
        assert!(!partial.reuse_supported());
        let key = execution_snapshot_key(&partial).unwrap();
        let anchors = vec![
            CalibrationAnchor::new_with_snapshot(
                &key,
                EXECUTION_SNAPSHOT_SCHEMA,
                &partial.unknown_identities,
                100.0,
                110.0,
                10,
            ),
            CalibrationAnchor::new_with_snapshot(
                &key,
                EXECUTION_SNAPSHOT_SCHEMA,
                &partial.unknown_identities,
                200.0,
                220.0,
                20,
            ),
            CalibrationAnchor::new_with_snapshot(
                &key,
                EXECUTION_SNAPSHOT_SCHEMA,
                &partial.unknown_identities,
                300.0,
                330.0,
                30,
            ),
        ];
        let error = build_calibration(&anchors, 40, 100).unwrap_err();
        assert!(error.contains("complete hardware identity"), "{error}");
        assert!(error.contains("driverVersion:luid:0000:1111"), "{error}");

        // A complete snapshot builds.
        let complete_key = execution_snapshot_key(&snapshot_fixture()).unwrap();
        let anchors = vec![
            test_anchor(&complete_key, "run-1", 100.0, 110.0, 10),
            test_anchor(&complete_key, "run-2", 200.0, 220.0, 20),
            test_anchor(&complete_key, "run-3", 300.0, 330.0, 30),
        ];
        assert!(build_calibration(&anchors, 40, 100).is_ok());
    }

    #[test]
    fn mt07_effective_arguments_are_sanitized_of_secrets_and_paths() {
        let args = vec![
            "-m".to_string(),
            "C:/models/model.gguf".to_string(),
            "--api-key-file".to_string(),
            "C:/secrets/llama.key".to_string(),
            "--ssl-cert-file".to_string(),
            "C:/secrets/cert.pem".to_string(),
            "--ssl-key-file".to_string(),
            "C:/secrets/key.pem".to_string(),
            "--lora".to_string(),
            "C:/models/adapter.gguf".to_string(),
            "--model-draft".to_string(),
            "C:/models/draft.gguf".to_string(),
            "--threads".to_string(),
            "8".to_string(),
        ];
        let sanitized = sanitize_effective_args(&args);
        let joined = sanitized.join(" ");
        for secret in [
            "llama.key",
            "cert.pem",
            "key.pem",
            "adapter.gguf",
            "draft.gguf",
            "model.gguf",
        ] {
            assert!(
                !joined.contains(secret),
                "sanitized args leaked {secret}: {joined}"
            );
        }
        assert!(joined.contains("[model]"));
        assert!(joined.contains("[configured]"));
        assert!(joined.contains("[lora]"));
        assert!(joined.contains("[draft-model]"));
        assert!(joined.contains("--threads"));
        assert!(joined.contains("8"));
    }

    #[test]
    fn model_set_digest_changes_when_a_selected_companion_changes() {
        let first = selected_artifact_set_sha256(&["a".repeat(64), "b".repeat(64)]).unwrap();
        let second = selected_artifact_set_sha256(&["a".repeat(64), "c".repeat(64)]).unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn s14_sanitizer_covers_every_path_bearing_flag_with_canaries() {
        // One canary per path-bearing flag the launch builder can emit; none
        // may survive sanitization (audit S-14.I2).
        let canary = |name: &str| format!("C:\\Users\\canary-user\\secret-{name}\\file.bin");
        let flags = [
            "-m",
            "--model",
            "--mmproj",
            "--lora",
            "--lora-scaled",
            "-md",
            "--spec-draft-model",
            "--chat-template-file",
            "--api-key-file",
            "--ssl-key-file",
            "--ssl-cert-file",
        ];
        let mut args = Vec::new();
        for flag in flags {
            args.push(flag.to_string());
            args.push(canary(flag.trim_start_matches('-')));
        }
        args.push("--port".into());
        args.push("8080".into());
        let sanitized = sanitize_effective_args(&args);
        let joined = sanitized.join(" ");
        assert!(
            !joined.contains("canary-user"),
            "a local path survived sanitization: {joined}"
        );
        for flag in flags {
            let position = sanitized
                .iter()
                .position(|token| token == flag)
                .unwrap_or_else(|| panic!("{flag} missing from {joined}"));
            assert!(
                sanitized
                    .get(position + 1)
                    .is_some_and(|value| value.starts_with('[')),
                "{flag} value was not replaced: {joined}"
            );
        }
        // Non-path values survive untouched.
        assert!(sanitized.windows(2).any(|pair| pair == ["--port", "8080"]));
    }

    #[test]
    fn s16_mixed_valid_corrupt_and_oversized_records_still_load_with_valid_history() {
        let root = std::env::temp_dir().join(format!(
            "localmotive-s16-mixed-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&root).unwrap();
        let key = test_key('a');
        let anchors = vec![
            test_anchor(&key, "run-1", 10.0, 11.0, 10),
            test_anchor(&key, "run-2", 20.0, 22.0, 20),
            test_anchor(&key, "run-3", 30.0, 33.0, 30),
        ];
        let anchor = anchors[0].clone();
        let model = build_calibration(&anchors, 50, 100).unwrap();
        persist_calibration_anchor(&root, &anchor).unwrap();
        persist_calibration_model(&root, &model).unwrap();

        // Corrupt, oversized and schema-invalid records beside the valid ones.
        std::fs::write(root.join("anchors").join("corrupt.json"), b"{not json").unwrap();
        std::fs::write(
            root.join("anchors").join("oversized.json"),
            vec![b'x'; (MAX_CALIBRATION_RECORD_BYTES + 1) as usize],
        )
        .unwrap();
        std::fs::write(
            root.join("models").join("wrong-shape.json"),
            br#"{"compatibilityKey":"v2:","bad":true}"#,
        )
        .unwrap();

        let anchors = load_calibration_anchors(&root, &key).unwrap();
        assert_eq!(anchors.records.len(), 1, "the valid anchor must survive");
        assert!(
            anchors.problems.len() >= 2,
            "each bad anchor must be reported: {:?}",
            anchors.problems
        );
        let models = load_calibration_models(&root, &key).unwrap();
        assert_eq!(models.records.len(), 1, "the valid model must survive");
        assert!(!models.problems.is_empty());

        // The bad bytes are preserved in the quarantine directory, not deleted.
        let quarantined: Vec<String> = std::fs::read_dir(root.join("quarantine"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        assert!(quarantined.len() >= 3, "{quarantined:?}");
        assert!(quarantined.iter().any(|name| name.contains("corrupt.json")));

        // A second load no longer sees the bad files and reports no problems.
        let anchors = load_calibration_anchors(&root, &key).unwrap();
        assert!(anchors.problems.is_empty());

        // Explicit cleanup removes the calibration records and keeps the
        // quarantine evidence.
        let removed = clear_calibration_history(&root).unwrap();
        assert_eq!(removed, 2, "one anchor and one model record");
        assert_eq!(
            load_calibration_anchors(&root, &key).unwrap().records.len(),
            0
        );
        assert_eq!(
            load_calibration_models(&root, &key).unwrap().records.len(),
            0
        );
        assert!(root.join("quarantine").exists());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn s16_retention_prunes_the_oldest_records_only() {
        let root = std::env::temp_dir().join(format!(
            "localmotive-s16-retention-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(root.join("anchors")).unwrap();
        for index in 0..5 {
            std::fs::write(root.join("anchors").join(format!("{index:05}.json")), b"{}").unwrap();
        }
        std::fs::write(root.join("anchors").join("notes.txt"), b"keep me").unwrap();
        let removed = prune_records_with(&root, "anchors", 3).unwrap();
        assert_eq!(removed, 2);
        let remaining: Vec<String> = std::fs::read_dir(root.join("anchors"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        assert!(remaining.contains(&"00003.json".to_string()));
        assert!(remaining.contains(&"00004.json".to_string()));
        assert!(!remaining.contains(&"00000.json".to_string()));
        assert!(
            remaining.contains(&"notes.txt".to_string()),
            "non-records survive"
        );
        // No-op below the bound.
        assert_eq!(prune_records_with(&root, "anchors", 10).unwrap(), 0);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn s17_imported_evidence_cannot_claim_provenance_or_review_state() {
        // A file that self-declares "verified" imports as Pending, and the
        // provenance label is forced on import (audit S-17).
        let candidate = serde_json::json!({
            "schema": 1,
            "source": "synthetic-community-file",
            "compatibilityKey": "c".repeat(64),
            "state": "verified",
            "records": [
                {"metric": "decodeTps", "value": 42.0, "unit": "tokensPerSecond", "observedAtMs": 42}
            ],
        });
        let bundle: ExternalEvidenceBundle = serde_json::from_value(candidate).unwrap();
        let imported = validate_external_evidence(bundle).unwrap();
        assert_eq!(imported.state, ExternalEvidenceState::Pending);
        assert_eq!(imported.provenance, ExternalProvenance::ImportedExternal);

        // A review is explicit and terminal; Pending is never a review result.
        assert!(
            review_external_evidence(imported.clone(), ExternalEvidenceState::Verified, false)
                .is_err()
        );
        assert!(
            review_external_evidence(imported.clone(), ExternalEvidenceState::Pending, true)
                .is_err()
        );
        let reviewed =
            review_external_evidence(imported, ExternalEvidenceState::Verified, true).unwrap();
        assert_eq!(reviewed.state, ExternalEvidenceState::Verified);
        assert_eq!(reviewed.provenance, ExternalProvenance::ImportedExternal);

        // Export and reload: the serialized reviewed bundle re-imports as
        // Pending again, and a foreign provenance claim is rejected outright.
        let exported = serde_json::to_string(&reviewed).unwrap();
        let reloaded: ExternalEvidenceBundle = serde_json::from_str(&exported).unwrap();
        let reimported = validate_external_evidence(reloaded).unwrap();
        assert_eq!(reimported.state, ExternalEvidenceState::Pending);
        assert_eq!(reimported.provenance, ExternalProvenance::ImportedExternal);
        let forged = exported.replace("importedExternal", "localRun");
        assert!(
            serde_json::from_str::<ExternalEvidenceBundle>(&forged).is_err(),
            "a bundle cannot claim a locally measured provenance"
        );
    }
}
