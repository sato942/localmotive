use crate::artifact::is_reparse_point;
use crate::evidence::EvidenceLevel;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_CALIBRATION_RECORD_BYTES: u64 = 64 * 1024;
const MAX_CALIBRATION_RECORDS: usize = 10_000;
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
    pub compatibility_key: String,
    pub estimated_value: f64,
    pub measured_value: f64,
    pub observed_at_ms: u64,
}

impl CalibrationAnchor {
    pub fn new(
        compatibility_key: &str,
        estimated_value: f64,
        measured_value: f64,
        observed_at_ms: u64,
    ) -> Self {
        Self {
            compatibility_key: compatibility_key.into(),
            estimated_value,
            measured_value,
            observed_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationModel {
    pub compatibility_key: String,
    pub factor: f64,
    pub residual_standard_deviation: f64,
    pub anchor_count: usize,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
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
    crate::evidence::validate_sha256("compatibilityKey", key).map_err(|error| error.to_string())?;
    if created_at_ms == 0 || ttl_ms == 0 || ttl_ms > MAX_TTL_MS {
        return Err(format!(
            "Calibration timestamps require a positive creation time and TTL no greater than {MAX_TTL_MS} ms"
        ));
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
    Ok(CalibrationModel {
        compatibility_key: key.into(),
        factor,
        residual_standard_deviation: variance.sqrt(),
        anchor_count: anchors.len(),
        created_at_ms,
        expires_at_ms: created_at_ms
            .checked_add(ttl_ms)
            .ok_or("Calibration expiry overflowed")?,
    })
}

pub fn apply_calibration(
    model: &CalibrationModel,
    compatibility_key: &str,
    estimated_value: f64,
    now_ms: u64,
) -> Result<CalibratedEstimate, String> {
    crate::evidence::validate_sha256("compatibilityKey", compatibility_key)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("model.compatibilityKey", &model.compatibility_key)
        .map_err(|error| error.to_string())?;
    if compatibility_key != model.compatibility_key {
        return Err("Calibration compatibility key does not match".into());
    }
    if now_ms == 0 || now_ms >= model.expires_at_ms {
        return Err("Calibration has expired".into());
    }
    if !estimated_value.is_finite() || estimated_value <= 0.0 || estimated_value > 1_000_000_000.0 {
        return Err("Estimated value must be positive and finite".into());
    }
    let value = estimated_value * model.factor;
    let uncertainty = estimated_value * model.residual_standard_deviation * 1.96;
    if !model.factor.is_finite()
        || model.factor <= 0.0
        || !model.residual_standard_deviation.is_finite()
        || model.residual_standard_deviation < 0.0
        || !value.is_finite()
        || !uncertainty.is_finite()
    {
        return Err("Calibration model produced a non-finite result".into());
    }
    Ok(CalibratedEstimate {
        value,
        lower_bound: (value - uncertainty).max(0.0),
        upper_bound: value + uncertainty,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEvidenceBundle {
    pub schema: u32,
    pub source: String,
    pub compatibility_key: String,
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
    bundle.state = ExternalEvidenceState::Pending;
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

fn validate_persisted_anchor(anchor: &CalibrationAnchor) -> Result<(), String> {
    crate::evidence::validate_sha256("compatibilityKey", &anchor.compatibility_key)
        .map_err(|error| error.to_string())?;
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

fn validate_persisted_model(model: &CalibrationModel) -> Result<(), String> {
    crate::evidence::validate_sha256("compatibilityKey", &model.compatibility_key)
        .map_err(|error| error.to_string())?;
    if model.created_at_ms == 0
        || model.expires_at_ms <= model.created_at_ms
        || model.expires_at_ms - model.created_at_ms > 365 * 24 * 60 * 60 * 1_000
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

pub fn persist_calibration_anchor(
    root: &Path,
    anchor: &CalibrationAnchor,
) -> Result<PathBuf, String> {
    validate_persisted_anchor(anchor)?;
    persist_record(
        root,
        "anchors",
        &format!(
            "{}-{}.json",
            anchor.observed_at_ms,
            &anchor.compatibility_key[..16]
        ),
        anchor,
    )
}

pub fn load_calibration_anchors(
    root: &Path,
    compatibility_key: &str,
) -> Result<Vec<CalibrationAnchor>, String> {
    crate::evidence::validate_sha256("compatibilityKey", compatibility_key)
        .map_err(|error| error.to_string())?;
    let mut records = Vec::new();
    for path in record_paths(root, "anchors")? {
        let record: CalibrationAnchor = read_bounded_record(&path)?;
        validate_persisted_anchor(&record)?;
        if record.compatibility_key == compatibility_key {
            records.push(record);
        }
    }
    records.sort_by_key(|record| record.observed_at_ms);
    records.dedup();
    Ok(records)
}

pub fn persist_calibration_model(root: &Path, model: &CalibrationModel) -> Result<PathBuf, String> {
    validate_persisted_model(model)?;
    persist_record(
        root,
        "models",
        &format!(
            "{}-{}.json",
            model.created_at_ms,
            &model.compatibility_key[..16]
        ),
        model,
    )
}

pub fn load_calibration_models(
    root: &Path,
    compatibility_key: &str,
) -> Result<Vec<CalibrationModel>, String> {
    crate::evidence::validate_sha256("compatibilityKey", compatibility_key)
        .map_err(|error| error.to_string())?;
    let mut records = Vec::new();
    for path in record_paths(root, "models")? {
        let record: CalibrationModel = read_bounded_record(&path)?;
        validate_persisted_model(&record)?;
        if record.compatibility_key == compatibility_key {
            records.push(record);
        }
    }
    records.sort_by_key(|record| record.created_at_ms);
    records.dedup();
    Ok(records)
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

    #[test]
    fn calibration_requires_matching_compatibility_and_expires() {
        let key = "a".repeat(64);
        let anchors = vec![
            CalibrationAnchor::new(&key, 100.0, 110.0, 10),
            CalibrationAnchor::new(&key, 200.0, 220.0, 20),
            CalibrationAnchor::new(&key, 300.0, 330.0, 30),
        ];
        let model = build_calibration(&anchors, 40, 100).unwrap();

        let applied = apply_calibration(&model, &key, 400.0, 50).unwrap();
        assert!((applied.value - 440.0).abs() < 0.000_001);
        assert!(apply_calibration(&model, "key-b", 400.0, 50).is_err());
        assert!(apply_calibration(&model, &key, 400.0, 140).is_err());
    }

    #[test]
    fn calibration_rejects_malformed_keys_and_unbounded_ttl() {
        let anchors = vec![
            CalibrationAnchor::new("key-a", 100.0, 110.0, 10),
            CalibrationAnchor::new("key-a", 200.0, 220.0, 20),
            CalibrationAnchor::new("key-a", 300.0, 330.0, 30),
        ];

        assert!(build_calibration(&anchors, 40, 100)
            .unwrap_err()
            .contains("compatibilityKey"));

        let key = "a".repeat(64);
        let valid = anchors
            .into_iter()
            .map(|mut anchor| {
                anchor.compatibility_key = key.clone();
                anchor
            })
            .collect::<Vec<_>>();
        assert!(build_calibration(&valid, 40, 366 * 24 * 60 * 60 * 1_000).is_err());
    }

    #[test]
    fn calibration_records_persist_as_bounded_local_files() {
        let root = std::env::temp_dir().join(format!(
            "gguf-pilot-calibration-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let key = "a".repeat(64);
        let anchor = CalibrationAnchor::new(&key, 10.0, 11.0, 42);
        let model =
            build_calibration(&[anchor.clone(), anchor.clone(), anchor.clone()], 50, 100).unwrap();

        persist_calibration_anchor(&root, &anchor).unwrap();
        persist_calibration_model(&root, &model).unwrap();
        assert_eq!(load_calibration_anchors(&root, &key).unwrap(), vec![anchor]);
        assert_eq!(load_calibration_models(&root, &key).unwrap(), vec![model]);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn model_set_digest_changes_when_a_selected_companion_changes() {
        let first = selected_artifact_set_sha256(&["a".repeat(64), "b".repeat(64)]).unwrap();
        let second = selected_artifact_set_sha256(&["a".repeat(64), "c".repeat(64)]).unwrap();

        assert_ne!(first, second);
    }
}
