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
    pub summary: Option<BenchmarkSummaryV2>,
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
                dedicated_bytes: item.dedicated_bytes.clone(),
                shared_bytes: item.shared_bytes.clone(),
                budget_bytes: item.budget_bytes.clone(),
            })
            .collect(),
        launch: ShareLaunch {
            requested_context: launch.requested_context,
            effective_context: launch.effective_context.clone(),
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
                peak_process_rss_bytes: item.peak_process_rss_bytes.clone(),
                outcome: item.outcome,
                succeeded: item.outcome == AttemptOutcome::Succeeded,
            })
            .collect(),
        terminal_outcome: manifest.terminal_outcome,
        summary: summary.cloned(),
        quality,
        privacy_review: PrivacyReview {
            omitted_fields: vec![
                "filesystemPaths".into(),
                "hostnames".into(),
                "promptsAndResponses".into(),
                "credentialsAndArguments".into(),
                "rawErrors".into(),
                "adapterStableIds".into(),
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
        Evidence::known(
            value,
            crate::evidence::EvidenceLevel::Observed,
            crate::evidence::EvidenceSource {
                kind: crate::evidence::EvidenceSourceKind::Runtime,
                detail: "llama-server GET /props fixture".into(),
            },
            42,
            Vec::new(),
        )
        .unwrap()
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

    #[test]
    fn privacy_export_excludes_paths_prompts_credentials_and_raw_errors() {
        let manifest = manifest_fixture();

        let export = build_share_bundle(&manifest, None, None, "e".repeat(64), 42).unwrap();
        let json = serde_json::to_string(&export).unwrap();

        assert!(!json.contains("Mubarak"));
        assert!(!json.contains("super-secret"));
        assert!(!json.contains("api-key"));
        assert!(!json.contains("private-model.gguf"));
        assert!(json.contains("model-structural-id"));
        assert!(json.contains("privacyReview"));
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
