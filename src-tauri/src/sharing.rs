use crate::evidence::{AttemptOutcome, BenchmarkManifest, Evidence, Workload};
use crate::measurement::BenchmarkSummaryV2;
use crate::recommend::{QualityStatus, QualitySuiteResult};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_SHARE_EXPORT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareFile {
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareRuntime {
    pub version: String,
    pub build: String,
    pub backend: String,
    pub executable_sha256: String,
    pub help_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareModel {
    pub logical_id: String,
    pub architecture: String,
    pub gguf_header_sha256: String,
    pub shards: Vec<ShareFile>,
    pub companions: Vec<ShareFile>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareHardware {
    pub vendor: String,
    pub backend: Option<String>,
    pub driver: Option<String>,
    pub dedicated_bytes: Evidence<u64>,
    pub shared_bytes: Evidence<u64>,
    pub budget_bytes: Evidence<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareLaunch {
    pub requested_context: u32,
    pub effective_context: Evidence<u32>,
    pub parallel: u16,
    pub gpu_layers: String,
    pub batch: u32,
    pub ubatch: u32,
    pub cache_type_k: String,
    pub cache_type_v: String,
    pub split_mode: String,
    pub tensor_split: String,
    pub main_gpu: u16,
    pub rejected_flags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareObservation {
    pub trial: u16,
    pub duration_ms: f64,
    pub prompt_tokens: u32,
    pub generated_tokens: u32,
    pub prefill_tps: Option<f64>,
    pub decode_tps: Option<f64>,
    pub first_token_ms: Option<f64>,
    pub derived_ttft_ms: Option<f64>,
    pub peak_process_rss_bytes: Evidence<u64>,
    pub outcome: AttemptOutcome,
    pub succeeded: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShareQualityCase {
    pub case_id: String,
    pub status: QualityStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShareQuality {
    pub suite_id: String,
    pub seed: u64,
    pub observed_at_ms: u64,
    pub model_logical_id: String,
    pub runtime_sha256: String,
    pub status: QualityStatus,
    pub cases: Vec<ShareQualityCase>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyReview {
    pub omitted_fields: Vec<String>,
    pub requires_user_confirmation: bool,
}

/// One bounded failure category in the public summary. Codes come from the
/// attempt outcome vocabulary only; raw error strings never travel
/// (audit MT-02).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicFailureCategory {
    pub code: String,
    pub count: u32,
}

/// The public, shareable summary: counts and numeric statistics only. It is
/// built from the internal summary plus the observations that back its
/// counts, so a valid share can never contain a raw trial error (MT-02).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicBenchmarkSummary {
    pub successful_trials: u32,
    pub failed_trials: u32,
    pub prefill_tps: Option<crate::measurement::MetricStats>,
    pub decode_tps: crate::measurement::MetricStats,
    pub first_token_ms: Option<crate::measurement::MetricStats>,
    pub derived_ttft_ms: Option<crate::measurement::MetricStats>,
    pub failure_categories: Vec<PublicFailureCategory>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareBundle {
    pub schema: u32,
    pub created_at_ms: u64,
    pub compatibility_key: String,
    pub runtime: ShareRuntime,
    pub model: ShareModel,
    pub hardware: Vec<ShareHardware>,
    pub launch: ShareLaunch,
    pub workload: Workload,
    pub warmup_outcomes: Vec<AttemptOutcome>,
    pub observations: Vec<ShareObservation>,
    pub terminal_outcome: Option<AttemptOutcome>,
    pub summary: Option<PublicBenchmarkSummary>,
    pub quality: Option<ShareQuality>,
    pub privacy_review: PrivacyReview,
}

fn validate_public_text(label: &str, value: &str, max_bytes: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > max_bytes {
        return Err(format!("{label} must contain 1 to {max_bytes} bytes"));
    }
    let lower = value.to_ascii_lowercase();
    if value.contains('\\')
        || value.contains(":/")
        || value.chars().any(char::is_control)
        || [
            "bearer ", "api-key", "api_key", "password", "token=", "secret",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(format!("{label} contains private or path-like text"));
    }
    Ok(())
}

/// A public export may only carry evidence whose free-form source detail was
/// replaced with an approved identifier and whose notes were dropped; this
/// runs at construction and at every direct persistence boundary
/// (audit MT-02 I3).
fn validate_public_evidence<T>(label: &str, evidence: &Evidence<T>) -> Result<(), String> {
    if !evidence.notes.is_empty() {
        return Err(format!("{label}.notes must be empty in a public export"));
    }
    if !evidence
        .source
        .detail
        .starts_with(PUBLIC_SOURCE_DETAIL_PREFIX)
    {
        return Err(format!(
            "{label}.source.detail must use the public export source vocabulary"
        ));
    }
    validate_public_text(
        &format!("{label}.source.detail"),
        &evidence.source.detail,
        128,
    )
}

fn validate_metric_stats(
    label: &str,
    stats: &crate::measurement::MetricStats,
) -> Result<(), String> {
    let values = [
        stats.mean,
        stats.min,
        stats.max,
        stats.p50,
        stats.p95,
        stats.standard_deviation,
    ];
    if stats.count == 0
        || values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        || stats.min > stats.max
        || stats.mean < stats.min
        || stats.mean > stats.max
        || stats.p50 < stats.min
        || stats.p50 > stats.max
        || stats.p95 < stats.min
        || stats.p95 > stats.max
    {
        return Err(format!("{label} contains invalid statistics"));
    }
    Ok(())
}

/// The only source-detail shape allowed into a public export.
pub const PUBLIC_SOURCE_DETAIL_PREFIX: &str = "public export source: ";

fn public_source_detail(kind: crate::evidence::EvidenceSourceKind) -> String {
    use crate::evidence::EvidenceSourceKind;
    let label = match kind {
        EvidenceSourceKind::FileSystem => "file-system",
        EvidenceSourceKind::GgufMetadata => "gguf-metadata",
        EvidenceSourceKind::Runtime => "runtime",
        EvidenceSourceKind::WindowsApi => "windows-api",
        EvidenceSourceKind::NvidiaSmi => "nvidia-smi",
        EvidenceSourceKind::Cim => "cim",
        EvidenceSourceKind::User => "user",
        EvidenceSourceKind::Catalog => "catalog",
        EvidenceSourceKind::Benchmark => "benchmark",
        EvidenceSourceKind::Calculation => "calculation",
        EvidenceSourceKind::Policy => "policy",
        EvidenceSourceKind::Import => "import",
        EvidenceSourceKind::Unknown => "unknown",
    };
    format!("{PUBLIC_SOURCE_DETAIL_PREFIX}{label}")
}

/// Rebuild one evidence value for public export: value, level, kind, and
/// timestamp survive; free-form source detail becomes an approved identifier
/// and notes are dropped (audit MT-02 I2).
fn public_evidence<T: Clone>(evidence: &Evidence<T>) -> Evidence<T> {
    Evidence {
        value: evidence.value.clone(),
        level: evidence.level,
        source: crate::evidence::EvidenceSource {
            kind: evidence.source.kind,
            detail: public_source_detail(evidence.source.kind),
        },
        observed_at_ms: evidence.observed_at_ms,
        notes: Vec::new(),
    }
}

fn public_failure_category_code(outcome: AttemptOutcome) -> &'static str {
    match outcome {
        AttemptOutcome::Succeeded => "succeeded",
        AttemptOutcome::Failed => "failed",
        AttemptOutcome::TimedOut => "timed-out",
        AttemptOutcome::Cancelled => "cancelled",
    }
}

const PUBLIC_FAILURE_CODES: &[&str] = &["failed", "timed-out", "cancelled"];

fn public_summary(
    summary: &BenchmarkSummaryV2,
    observations: &[crate::evidence::BenchmarkObservation],
) -> Result<PublicBenchmarkSummary, String> {
    let mut categories: Vec<PublicFailureCategory> = Vec::new();
    for observation in observations {
        if observation.outcome == AttemptOutcome::Succeeded {
            continue;
        }
        let code = public_failure_category_code(observation.outcome);
        match categories.iter_mut().find(|category| category.code == code) {
            Some(category) => category.count += 1,
            None => categories.push(PublicFailureCategory {
                code: code.into(),
                count: 1,
            }),
        }
    }
    categories.sort_by(|left, right| left.code.cmp(&right.code));
    let category_total: u32 = categories.iter().map(|category| category.count).sum();
    let failed_trials = u32::try_from(summary.failed_trials)
        .map_err(|_| "Share summary failure count is out of range".to_string())?;
    if category_total != failed_trials {
        return Err("Share summary failure counts do not match the benchmark observations".into());
    }
    Ok(PublicBenchmarkSummary {
        successful_trials: u32::try_from(summary.successful_trials)
            .map_err(|_| "Share summary success count is out of range".to_string())?,
        failed_trials,
        prefill_tps: summary.prefill_tps.clone(),
        decode_tps: summary.decode_tps.clone(),
        first_token_ms: summary.first_token_ms.clone(),
        derived_ttft_ms: summary.derived_ttft_ms.clone(),
        failure_categories: categories,
    })
}

pub fn validate_share_bundle(bundle: &ShareBundle) -> Result<(), String> {
    if bundle.schema != 1 {
        return Err(format!(
            "Unsupported share bundle schema: {}",
            bundle.schema
        ));
    }
    if bundle.created_at_ms == 0 {
        return Err("Share bundle creation time is missing".into());
    }
    crate::evidence::validate_sha256("compatibilityKey", &bundle.compatibility_key)
        .map_err(|error| error.to_string())?;
    for (label, digest) in [
        (
            "runtime.executableSha256",
            bundle.runtime.executable_sha256.as_str(),
        ),
        ("runtime.helpSha256", bundle.runtime.help_sha256.as_str()),
        (
            "model.ggufHeaderSha256",
            bundle.model.gguf_header_sha256.as_str(),
        ),
    ] {
        crate::evidence::validate_sha256(label, digest).map_err(|error| error.to_string())?;
    }
    for (label, value) in [
        ("runtime.version", bundle.runtime.version.as_str()),
        ("runtime.build", bundle.runtime.build.as_str()),
        ("runtime.backend", bundle.runtime.backend.as_str()),
        ("model.logicalId", bundle.model.logical_id.as_str()),
        ("model.architecture", bundle.model.architecture.as_str()),
        ("workload.id", bundle.workload.id.as_str()),
        ("launch.gpuLayers", bundle.launch.gpu_layers.as_str()),
        ("launch.cacheTypeK", bundle.launch.cache_type_k.as_str()),
        ("launch.cacheTypeV", bundle.launch.cache_type_v.as_str()),
        ("launch.splitMode", bundle.launch.split_mode.as_str()),
    ] {
        validate_public_text(label, value, 256)?;
    }
    if !bundle.launch.tensor_split.is_empty() {
        validate_public_text("launch.tensorSplit", &bundle.launch.tensor_split, 1_024)?;
    }
    if bundle.hardware.len() > 64
        || bundle.model.shards.len() > 10_000
        || bundle.model.companions.len() > 256
        || bundle.warmup_outcomes.len() > 10
        || bundle.observations.len() > 100
        || bundle
            .quality
            .as_ref()
            .is_some_and(|quality| quality.cases.len() > 64)
    {
        return Err("Share bundle collection limit exceeded".into());
    }
    for (index, file) in bundle
        .model
        .shards
        .iter()
        .chain(bundle.model.companions.iter())
        .enumerate()
    {
        if file.bytes == 0 {
            return Err(format!("model.files[{index}] has zero bytes"));
        }
        crate::evidence::validate_sha256("model.files.sha256", &file.sha256)
            .map_err(|error| error.to_string())?;
    }
    for (index, hardware) in bundle.hardware.iter().enumerate() {
        validate_public_text(&format!("hardware[{index}].vendor"), &hardware.vendor, 128)?;
        if let Some(backend) = hardware.backend.as_deref() {
            validate_public_text(&format!("hardware[{index}].backend"), backend, 128)?;
        }
        if let Some(driver) = hardware.driver.as_deref() {
            validate_public_text(&format!("hardware[{index}].driver"), driver, 256)?;
        }
    }
    for (index, observation) in bundle.observations.iter().enumerate() {
        let values = [
            observation.duration_ms,
            observation.prefill_tps.unwrap_or(0.0),
            observation.decode_tps.unwrap_or(0.0),
            observation.first_token_ms.unwrap_or(0.0),
            observation.derived_ttft_ms.unwrap_or(0.0),
        ];
        if observation.trial == 0
            || values
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0)
            || observation.succeeded != (observation.outcome == AttemptOutcome::Succeeded)
        {
            return Err(format!("observations[{index}] is invalid"));
        }
    }
    if let Some(summary) = bundle.summary.as_ref() {
        for (label, stats) in [
            ("summary.prefillTps", summary.prefill_tps.as_ref()),
            ("summary.firstTokenMs", summary.first_token_ms.as_ref()),
            ("summary.derivedTtftMs", summary.derived_ttft_ms.as_ref()),
        ] {
            if let Some(stats) = stats {
                validate_metric_stats(label, stats)?;
            }
        }
        validate_metric_stats("summary.decodeTps", &summary.decode_tps)?;
        // The public summary may contain counts and bounded category codes
        // only; anything else is not a valid public export (audit MT-02).
        if summary.failure_categories.len() > PUBLIC_FAILURE_CODES.len() {
            return Err("summary.failureCategories exceeds the bounded vocabulary".into());
        }
        let mut seen = Vec::new();
        let mut total = 0_u32;
        for category in &summary.failure_categories {
            if !PUBLIC_FAILURE_CODES.contains(&category.code.as_str()) || category.count == 0 {
                return Err("summary.failureCategories contains an unbounded category".into());
            }
            if seen.contains(&category.code) {
                return Err("summary.failureCategories repeats a category".into());
            }
            seen.push(category.code.clone());
            total = total.saturating_add(category.count);
        }
        if total != summary.failed_trials {
            return Err("summary.failureCategories do not sum to summary.failedTrials".into());
        }
    }
    for (index, hardware) in bundle.hardware.iter().enumerate() {
        for (label, evidence) in [
            ("dedicatedBytes", &hardware.dedicated_bytes),
            ("sharedBytes", &hardware.shared_bytes),
            ("budgetBytes", &hardware.budget_bytes),
        ] {
            validate_public_evidence(&format!("hardware[{index}].{label}"), evidence)?;
        }
    }
    validate_public_evidence("launch.effectiveContext", &bundle.launch.effective_context)?;
    for (index, observation) in bundle.observations.iter().enumerate() {
        validate_public_evidence(
            &format!("observations[{index}].peakProcessRssBytes"),
            &observation.peak_process_rss_bytes,
        )?;
    }
    if let Some(quality) = bundle.quality.as_ref() {
        validate_public_text("quality.suiteId", &quality.suite_id, 128)?;
        for (index, case) in quality.cases.iter().enumerate() {
            validate_public_text(
                &format!("quality.cases[{index}].caseId"),
                &case.case_id,
                128,
            )?;
        }
    }
    let bytes = serde_json::to_vec(bundle).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_SHARE_EXPORT_BYTES {
        return Err(format!(
            "Share bundle exceeds the {}-byte limit",
            MAX_SHARE_EXPORT_BYTES
        ));
    }
    Ok(())
}

pub fn require_export_confirmation(confirmed: bool) -> Result<(), String> {
    if confirmed {
        Ok(())
    } else {
        Err("Review the privacy omissions and confirm this local export".into())
    }
}

pub fn persist_share_bundle(path: &Path, bundle: &ShareBundle) -> Result<PathBuf, String> {
    let is_json = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));
    if !is_json {
        return Err("Share export path must use the .json extension".into());
    }

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or("Share export path must have a parent directory")?;
    if !parent.is_dir() {
        return Err("Share export parent directory does not exist".into());
    }

    validate_share_bundle(bundle)?;
    let serialized = serde_json::to_vec_pretty(bundle)
        .map_err(|error| format!("Could not serialize share export: {error}"))?;
    if serialized.len() > MAX_SHARE_EXPORT_BYTES {
        return Err(format!(
            "Share export exceeds the {MAX_SHARE_EXPORT_BYTES}-byte limit"
        ));
    }

    let mut output = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err("Share export target already exists".into());
        }
        Err(error) => return Err(format!("Could not create share export: {error}")),
    };

    if let Err(error) = output
        .write_all(&serialized)
        .and_then(|()| output.sync_all())
    {
        drop(output);
        let _ = std::fs::remove_file(path);
        return Err(format!("Could not write share export: {error}"));
    }

    Ok(path.to_path_buf())
}

fn share_file(file: &crate::evidence::FileFact) -> Result<ShareFile, String> {
    Ok(ShareFile {
        bytes: file.bytes,
        sha256: file
            .sha256
            .clone()
            .ok_or("Sharing requires a digest for every artifact file")?,
    })
}

pub fn build_share_bundle(
    manifest: &BenchmarkManifest,
    summary: Option<&BenchmarkSummaryV2>,
    quality: Option<&QualitySuiteResult>,
    compatibility_key: String,
    created_at_ms: u64,
) -> Result<ShareBundle, String> {
    manifest
        .validate_complete()
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("compatibilityKey", &compatibility_key)
        .map_err(|error| error.to_string())?;
    if manifest.compatibility_key.as_deref() != Some(compatibility_key.as_str()) {
        return Err("Share export compatibility key does not match the benchmark manifest".into());
    }
    let recomputed_summary = crate::measurement::summarize_observations(&manifest.observations);
    match (summary, recomputed_summary) {
        (Some(supplied), Ok(recomputed)) if supplied != &recomputed => {
            return Err("Share export summary does not match the benchmark observations".into());
        }
        (Some(_), Err(error)) => {
            return Err(format!(
                "Share export summary has no valid observations: {error}"
            ));
        }
        (None, Ok(_)) => {
            return Err("Share export omitted a summary for successful observations".into());
        }
        _ => {}
    }
    let runtime = manifest
        .runtime
        .as_ref()
        .ok_or("Runtime identity is missing")?;
    let model = manifest.model.as_ref().ok_or("Model identity is missing")?;
    let launch = manifest
        .launch
        .as_ref()
        .ok_or("Launch identity is missing")?;
    let executable_sha256 = runtime
        .executable_sha256
        .clone()
        .ok_or("Sharing requires a runtime executable digest")?;
    let quality = quality
        .map(|suite| -> Result<ShareQuality, String> {
            let observed_at_ms = suite
                .observed_at_ms
                .ok_or("Quality evidence is missing its observation timestamp")?;
            let model_logical_id = suite
                .model_logical_id
                .clone()
                .ok_or("Quality evidence is missing its model identity")?;
            let runtime_sha256 = suite
                .runtime_sha256
                .clone()
                .ok_or("Quality evidence is missing its runtime identity")?;
            if model_logical_id != model.logical_id {
                return Err("Quality evidence model identity does not match the manifest".into());
            }
            if runtime_sha256 != executable_sha256 {
                return Err("Quality evidence runtime identity does not match the manifest".into());
            }
            Ok(ShareQuality {
                suite_id: suite.suite_id.clone(),
                seed: suite.seed,
                observed_at_ms,
                model_logical_id,
                runtime_sha256,
                status: suite.status,
                cases: suite
                    .cases
                    .iter()
                    .map(|item| ShareQualityCase {
                        case_id: item.case_id.clone(),
                        status: item.status,
                    })
                    .collect(),
            })
        })
        .transpose()?;
    let bundle = ShareBundle {
        schema: 1,
        created_at_ms,
        compatibility_key,
        runtime: ShareRuntime {
            version: runtime.version.clone(),
            build: runtime.build.clone(),
            backend: runtime.backend.clone(),
            executable_sha256,
            help_sha256: runtime.help_sha256.clone(),
        },
        model: ShareModel {
            logical_id: model.logical_id.clone(),
            architecture: model.architecture.clone(),
            gguf_header_sha256: model.gguf_header_sha256.clone(),
            shards: model
                .shards
                .iter()
                .map(share_file)
                .collect::<Result<_, _>>()?,
            companions: model
                .companions
                .iter()
                .map(share_file)
                .collect::<Result<_, _>>()?,
        },
        hardware: manifest
            .hardware
            .iter()
            .map(|item| ShareHardware {
                vendor: item.vendor.clone(),
                backend: item.backend.clone(),
                driver: item.driver.clone(),
                dedicated_bytes: public_evidence(&item.dedicated_bytes),
                shared_bytes: public_evidence(&item.shared_bytes),
                budget_bytes: public_evidence(&item.budget_bytes),
            })
            .collect(),
        launch: ShareLaunch {
            requested_context: launch.requested_context,
            effective_context: public_evidence(&launch.effective_context),
            parallel: launch.parallel,
            gpu_layers: launch.gpu_layers.clone(),
            batch: launch.batch,
            ubatch: launch.ubatch,
            cache_type_k: launch.cache_type_k.clone(),
            cache_type_v: launch.cache_type_v.clone(),
            split_mode: launch.split_mode.clone(),
            tensor_split: launch.tensor_split.clone(),
            main_gpu: launch.main_gpu,
            rejected_flags: launch.rejected_flags.clone(),
        },
        workload: manifest.workload.clone(),
        warmup_outcomes: manifest
            .warmups
            .iter()
            .map(|warmup| warmup.outcome)
            .collect(),
        observations: manifest
            .observations
            .iter()
            .map(|item| ShareObservation {
                trial: item.trial,
                duration_ms: item.duration_ms,
                prompt_tokens: item.prompt_tokens,
                generated_tokens: item.generated_tokens,
                prefill_tps: item.prefill_tps,
                decode_tps: item.decode_tps,
                first_token_ms: item.first_token_ms,
                derived_ttft_ms: item.derived_ttft_ms,
                peak_process_rss_bytes: public_evidence(&item.peak_process_rss_bytes),
                outcome: item.outcome,
                succeeded: item.outcome == AttemptOutcome::Succeeded,
            })
            .collect(),
        terminal_outcome: manifest.terminal_outcome,
        summary: summary
            .map(|summary| public_summary(summary, &manifest.observations))
            .transpose()?,
        quality,
        privacy_review: PrivacyReview {
            // Describe the actual serialized policy (audit MT-02 I3): raw
            // errors never travel, and evidence free text is replaced by an
            // approved source vocabulary.
            omitted_fields: vec![
                "filesystemPaths".into(),
                "hostnames".into(),
                "promptsAndResponses".into(),
                "credentialsAndArguments".into(),
                "rawErrors".into(),
                "adapterStableIds".into(),
                "evidenceSourceDetails".into(),
                "evidenceNotes".into(),
            ],
            requires_user_confirmation: true,
        },
    };
    validate_share_bundle(&bundle)?;
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{
        AttemptOutcome, BenchmarkManifest, BenchmarkObservation, FileFact, LaunchFact, ModelFact,
        RuntimeFact,
    };

    fn observed_context(value: u32) -> Evidence<u32> {
        // Persisted/public fixtures carry the sanitized source vocabulary,
        // because the validator now enforces it at every boundary.
        public_evidence(
            &Evidence::known(
                value,
                crate::evidence::EvidenceLevel::Observed,
                crate::evidence::EvidenceSource {
                    kind: crate::evidence::EvidenceSourceKind::Runtime,
                    detail: "llama-server GET /props fixture".into(),
                },
                42,
                Vec::new(),
            )
            .unwrap(),
        )
    }

    fn manifest_fixture() -> BenchmarkManifest {
        BenchmarkManifest {
            compatibility_key: Some("e".repeat(64)),
            runtime: Some(RuntimeFact {
                path: r"C:\Users\Mubarak\secret\llama-server.exe".into(),
                version: "v1".into(),
                build: "42".into(),
                executable_sha256: Some("a".repeat(64)),
                help_sha256: "b".repeat(64),
                backend: "cuda".into(),
            }),
            model: Some(ModelFact {
                logical_id: "model-structural-id".into(),
                architecture: "llama".into(),
                shards: vec![FileFact {
                    path: r"C:\Users\Mubarak\private-model.gguf".into(),
                    bytes: 1_000,
                    sha256: Some("c".repeat(64)),
                }],
                companions: Vec::new(),
                gguf_header_sha256: "d".repeat(64),
            }),
            launch: Some(LaunchFact {
                requested_context: 4_096,
                effective_context: observed_context(4_096),
                parallel: 1,
                gpu_layers: "all".into(),
                batch: 512,
                ubatch: 128,
                cache_type_k: "F16".into(),
                cache_type_v: "F16".into(),
                split_mode: "layer".into(),
                command_args: vec!["--api-key".into(), "super-secret".into()],
                ..LaunchFact::default()
            }),
            observations: vec![BenchmarkObservation {
                trial: 1,
                started_at_ms: 42,
                duration_ms: 1.0,
                outcome: AttemptOutcome::Failed,
                decode_tps: None,
                error: Some("token super-secret failed in C:\\Users\\Mubarak".into()),
                ..BenchmarkObservation::default()
            }],
            ..BenchmarkManifest::default()
        }
    }

    #[test]
    fn share_export_requires_explicit_user_confirmation() {
        assert!(require_export_confirmation(false).is_err());
        assert!(require_export_confirmation(true).is_ok());
    }

    fn mixed_manifest_with_nested_canaries() -> BenchmarkManifest {
        // One successful and one failed observation, with distinct path,
        // account-name, secret, and nested-evidence canaries in the raw
        // failure and in nested notes/details (audit MT-02 V1/V2).
        let mut manifest = manifest_fixture();
        manifest.model.as_mut().unwrap().shards = vec![FileFact {
            path: r"C:\Users\Mubarak\private-model.gguf".into(),
            bytes: 1_000,
            sha256: Some("c".repeat(64)),
        }];
        let mut success = BenchmarkObservation {
            trial: 1,
            started_at_ms: 42,
            duration_ms: 0.5,
            prompt_tokens: 512,
            cached_prompt_tokens: 0,
            generated_tokens: 256,
            prefill_tps: Some(100.0),
            decode_tps: Some(50.0),
            first_token_ms: Some(10.0),
            derived_ttft_ms: Some(11.0),
            peak_process_rss_bytes: Evidence::known(
                1_024,
                crate::evidence::EvidenceLevel::Observed,
                crate::evidence::EvidenceSource {
                    kind: crate::evidence::EvidenceSourceKind::Runtime,
                    detail: "NESTED-DETAIL-CANARY".into(),
                },
                42,
                vec!["NESTED-NOTE-CANARY".into()],
            )
            .unwrap(),
            outcome: AttemptOutcome::Succeeded,
            error: None,
        };
        let failed = BenchmarkObservation {
            trial: 2,
            started_at_ms: 43,
            duration_ms: 0.4,
            outcome: AttemptOutcome::Failed,
            error: Some(
                "launch failed for C:\\Users\\Mubarak under account sato942 with api-key super-secret; LOGTail-CANARY"
                    .into(),
            ),
            ..BenchmarkObservation::default()
        };
        // Keep the success in a local variable name that the compiler allows.
        success.trial = 1;
        manifest.observations = vec![success, failed];
        manifest.hardware = vec![crate::evidence::HardwareFact {
            adapter_id: "adapter-0".into(),
            name: "Test GPU".into(),
            vendor: "NVIDIA".into(),
            backend: Some("cuda".into()),
            driver: Some("1".into()),
            current_usage_bytes: Evidence::unknown(
                crate::evidence::EvidenceSource {
                    kind: crate::evidence::EvidenceSourceKind::Unknown,
                    detail: "NESTED-DETAIL-CANARY".into(),
                },
                42,
                "NESTED-NOTE-CANARY",
            ),
            dedicated_bytes: Evidence::known(
                8,
                crate::evidence::EvidenceLevel::Observed,
                crate::evidence::EvidenceSource {
                    kind: crate::evidence::EvidenceSourceKind::NvidiaSmi,
                    detail: "NESTED-DETAIL-CANARY".into(),
                },
                42,
                vec!["NESTED-NOTE-CANARY".into()],
            )
            .unwrap(),
            shared_bytes: Evidence::unknown(
                crate::evidence::EvidenceSource {
                    kind: crate::evidence::EvidenceSourceKind::Unknown,
                    detail: "NESTED-DETAIL-CANARY".into(),
                },
                42,
                "NESTED-NOTE-CANARY",
            ),
            budget_bytes: Evidence::unknown(
                crate::evidence::EvidenceSource {
                    kind: crate::evidence::EvidenceSourceKind::Unknown,
                    detail: "NESTED-DETAIL-CANARY".into(),
                },
                42,
                "NESTED-NOTE-CANARY",
            ),
        }];
        if let Some(launch) = manifest.launch.as_mut() {
            launch.effective_context.notes = vec!["NESTED-NOTE-CANARY".into()];
        }
        manifest
    }

    #[test]
    fn privacy_export_excludes_paths_prompts_credentials_and_raw_errors() {
        let manifest = mixed_manifest_with_nested_canaries();
        let summary = crate::measurement::summarize_observations(&manifest.observations).unwrap();

        let export = build_share_bundle(&manifest, Some(&summary), None, "e".repeat(64), 42)
            .expect("a mixed manifest with a valid summary must export");
        let json = serde_json::to_string(&export).unwrap();

        for canary in [
            "Mubarak",
            "sato942",
            "super-secret",
            "api-key",
            "private-model.gguf",
            "NESTED-DETAIL-CANARY",
            "NESTED-NOTE-CANARY",
            "LOGTail-CANARY",
        ] {
            assert!(
                !json.contains(canary),
                "{canary} leaked through the public export: {json}"
            );
        }
        assert!(json.contains("model-structural-id"));
        assert!(json.contains("privacyReview"));
        assert!(json.contains("evidenceSourceDetails"));
        // Useful counts and numeric statistics survive the redaction.
        assert!(json.contains("\"decodeTps\""), "{json}");
        assert!(json.contains("\"failureCategories\""), "{json}");
        assert!(json.contains("\"successfulTrials\":1"), "{json}");
        assert!(json.contains("\"failedTrials\":1"), "{json}");
        assert!(json.contains("\"code\":\"failed\""), "{json}");
        assert!(json.contains("public export source:"), "{json}");
        // The privacy review describes the actual serialized policy.
        assert!(json.contains("rawErrors"));
        assert!(json.contains("evidenceNotes"));
    }

    #[test]
    fn mt02_direct_persistence_rejects_unsanitized_evidence_and_raw_failure_text() {
        // The stronger validator must hold at the direct persistence boundary
        // too: a hand-supplied bundle with nested free text is refused
        // (audit MT-02 I3).
        let manifest = mixed_manifest_with_nested_canaries();
        let summary = crate::measurement::summarize_observations(&manifest.observations).unwrap();
        let mut export =
            build_share_bundle(&manifest, Some(&summary), None, "e".repeat(64), 42).unwrap();
        // A tampered bundle: reintroduce a note after construction.
        export.hardware[0].shared_bytes.notes = vec!["NESTED-NOTE-CANARY".into()];
        let directory =
            std::env::temp_dir().join(format!("localmotive-share-boundary-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let target = directory.join("tampered.json");
        let error = persist_share_bundle(&target, &export).unwrap_err();
        assert!(error.contains("notes must be empty"), "{error}");
        assert!(!target.exists(), "a rejected export must not be written");

        // And an unbounded failure category is refused the same way.
        let mut export =
            build_share_bundle(&manifest, Some(&summary), None, "e".repeat(64), 42).unwrap();
        if let Some(pub_summary) = export.summary.as_mut() {
            pub_summary.failure_categories[0].code = "raw: C:\\Users\\Mubarak".into();
        }
        let error = persist_share_bundle(&target, &export).unwrap_err();
        assert!(error.contains("unbounded category"), "{error}");
        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn local_share_persistence_creates_one_new_json_file_without_overwriting() {
        let directory =
            std::env::temp_dir().join(format!("localmotive-share-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let target = directory.join("evidence.json");
        let bundle = ShareBundle {
            schema: 1,
            created_at_ms: 42,
            compatibility_key: "a".repeat(64),
            runtime: ShareRuntime {
                version: "v1".into(),
                build: "1".into(),
                backend: "cpu".into(),
                executable_sha256: "b".repeat(64),
                help_sha256: "c".repeat(64),
            },
            model: ShareModel {
                logical_id: "logical".into(),
                architecture: "llama".into(),
                gguf_header_sha256: "d".repeat(64),
                shards: vec![ShareFile {
                    bytes: 10,
                    sha256: "e".repeat(64),
                }],
                companions: Vec::new(),
            },
            hardware: Vec::new(),
            launch: ShareLaunch {
                requested_context: 4_096,
                effective_context: observed_context(4_096),
                parallel: 1,
                gpu_layers: "0".into(),
                batch: 512,
                ubatch: 128,
                cache_type_k: "f16".into(),
                cache_type_v: "f16".into(),
                split_mode: "none".into(),
                tensor_split: String::new(),
                main_gpu: 0,
                rejected_flags: Vec::new(),
            },
            workload: Workload::default(),
            warmup_outcomes: Vec::new(),
            observations: Vec::new(),
            terminal_outcome: None,
            summary: None,
            quality: None,
            privacy_review: PrivacyReview {
                omitted_fields: vec!["filesystemPaths".into()],
                requires_user_confirmation: true,
            },
        };

        let written = persist_share_bundle(&target, &bundle).unwrap();
        let loaded: ShareBundle =
            serde_json::from_slice(&std::fs::read(&written).unwrap()).unwrap();

        assert_eq!(loaded, bundle);
        assert!(persist_share_bundle(&target, &bundle)
            .unwrap_err()
            .contains("already exists"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn share_export_rejects_a_path_like_workload_identity() {
        let mut manifest = manifest_fixture();
        manifest.workload.id = r#"C:\Users\private\prompt.txt"#.into();

        let error = build_share_bundle(&manifest, None, None, "e".repeat(64), 42).unwrap_err();

        assert!(error.contains("workload.id"));
    }

    #[test]
    fn share_bundle_validation_rejects_future_schemas() {
        let mut export =
            build_share_bundle(&manifest_fixture(), None, None, "e".repeat(64), 42).unwrap();
        export.schema = 2;

        assert!(validate_share_bundle(&export)
            .unwrap_err()
            .contains("schema"));
    }
}
