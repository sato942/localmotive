use crate::evidence::FitClass;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum QualityStatus {
    NotRun,
    Passed,
    Failed,
    Error,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QualityCaseResult {
    pub case_id: String,
    pub status: QualityStatus,
    pub detail: String,
}

pub fn evaluate_quality_case(case_id: &str, response: &str) -> QualityCaseResult {
    let (status, detail) = match case_id {
        "exact-ready-v1" if response.trim() == "READY" => (
            QualityStatus::Passed,
            "Response matched READY exactly".into(),
        ),
        "exact-ready-v1" => (
            QualityStatus::Failed,
            "Response did not match READY exactly".into(),
        ),
        "json-status-v1" => match serde_json::from_str::<serde_json::Value>(response) {
            Ok(value)
                if value.as_object().is_some_and(|object| {
                    object.len() == 1
                        && object.get("status").and_then(serde_json::Value::as_str) == Some("ok")
                }) =>
            {
                (
                    QualityStatus::Passed,
                    "Response matched the required JSON structure".into(),
                )
            }
            Ok(_) => (
                QualityStatus::Failed,
                "JSON response did not contain exactly {\"status\":\"ok\"}".into(),
            ),
            Err(_) => (QualityStatus::Failed, "Response was not valid JSON".into()),
        },
        _ => (
            QualityStatus::Error,
            "Unknown quality case identifier".into(),
        ),
    };
    QualityCaseResult {
        case_id: case_id.into(),
        status,
        detail,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QualitySuiteResult {
    pub suite_id: String,
    pub seed: u64,
    pub observed_at_ms: Option<u64>,
    pub model_logical_id: Option<String>,
    pub runtime_sha256: Option<String>,
    /// Full model-content identity (first-shard SHA-256) captured before the
    /// requests; quality attaches only to runs with the same content
    /// (audit MT-09).
    #[serde(default)]
    pub model_content_sha256: Option<String>,
    /// The launch-scope execution-snapshot key of the serving configuration
    /// (audit MT-09).
    #[serde(default)]
    pub compatibility_key: Option<String>,
    /// The suite's structural scope; `structural-smoke.v1` means the two
    /// format/obedience cases only (audit MT-09 I4).
    #[serde(default)]
    pub suite_version: String,
    #[serde(default)]
    pub cases_planned: u16,
    pub status: QualityStatus,
    pub cases: Vec<QualityCaseResult>,
}

pub fn run_quality_suite_with<F>(mut request: F) -> QualitySuiteResult
where
    F: FnMut(&str, &str) -> Result<String, String>,
{
    const CASES: [(&str, &str); 2] = [
        (
            "exact-ready-v1",
            "Reply with exactly the uppercase word READY and no other text.",
        ),
        (
            "json-status-v1",
            "Return one JSON object with exactly one key named status and string value ok.",
        ),
    ];
    let cases = CASES
        .into_iter()
        .map(|(case_id, prompt)| match request(case_id, prompt) {
            Ok(response) => evaluate_quality_case(case_id, &response),
            Err(error) => QualityCaseResult {
                case_id: case_id.into(),
                status: QualityStatus::Error,
                detail: error,
            },
        })
        .collect::<Vec<_>>();
    let status = if cases.iter().any(|item| item.status == QualityStatus::Error) {
        QualityStatus::Error
    } else if cases
        .iter()
        .any(|item| item.status == QualityStatus::Failed)
    {
        QualityStatus::Failed
    } else {
        QualityStatus::Passed
    };
    QualitySuiteResult {
        suite_id: "localmotive-structural-v1".into(),
        seed: 42,
        observed_at_ms: None,
        model_logical_id: None,
        runtime_sha256: None,
        model_content_sha256: None,
        compatibility_key: None,
        suite_version: "structural-smoke.v1".into(),
        cases_planned: cases.len() as u16,
        status,
        cases,
    }
}

/// The identity policy for attaching quality evidence to a measured run
/// (audit MT-09): full model content, runtime executable, and the
/// launch-scope execution identity must all match. Missing identities are
/// refusals, not assumptions.
pub fn quality_attachment_decision(
    manifest: &crate::evidence::BenchmarkManifest,
    quality: &QualitySuiteResult,
) -> Result<(), String> {
    let launch_key = manifest
        .launch_compatibility_key
        .as_deref()
        .filter(|key| !key.is_empty())
        .ok_or(
            "This benchmark manifest does not carry a launch-scope execution identity; quality evidence cannot attach to it",
        )?;
    let quality_key = quality
        .compatibility_key
        .as_deref()
        .filter(|key| !key.is_empty())
        .ok_or("The quality result does not carry an execution identity")?;
    if launch_key != quality_key {
        return Err(format!(
            "Quality evidence was measured on a different configuration (quality {quality_key}, run {launch_key})"
        ));
    }
    let model_sha = manifest
        .model
        .as_ref()
        .and_then(|model| model.shards.first())
        .and_then(|shard| shard.sha256.as_deref())
        .filter(|digest| !digest.is_empty())
        .ok_or("The benchmark manifest does not carry full model content identity")?;
    let quality_model_sha = quality
        .model_content_sha256
        .as_deref()
        .filter(|digest| !digest.is_empty())
        .ok_or("The quality result does not carry full model content identity")?;
    if model_sha != quality_model_sha {
        return Err("Quality evidence came from different model content than the run".into());
    }
    let runtime_sha = manifest
        .runtime
        .as_ref()
        .and_then(|runtime| runtime.executable_sha256.as_deref())
        .filter(|digest| !digest.is_empty())
        .ok_or("The benchmark manifest does not carry the runtime executable identity")?;
    let quality_runtime_sha = quality
        .runtime_sha256
        .as_deref()
        .filter(|digest| !digest.is_empty())
        .ok_or("The quality result does not carry the runtime executable identity")?;
    if runtime_sha != quality_runtime_sha {
        return Err("Quality evidence came from a different runtime than the run".into());
    }
    Ok(())
}

/// Build a ranking candidate from a persisted benchmark manifest. Quality
/// evidence attaches only through [`quality_attachment_decision`]; a refused
/// attachment is an error, never a silent omission (audit MT-09 I3).
pub fn candidate_from_manifest(
    id: &str,
    manifest: &crate::evidence::BenchmarkManifest,
    result_class: crate::evidence::FitClass,
    quality: Option<&QualitySuiteResult>,
) -> Result<CandidateEvidence, String> {
    manifest
        .validate_complete()
        .map_err(|error| error.to_string())?;
    let summary = crate::measurement::summarize_observations(&manifest.observations)?;
    let peak_memory_bytes = manifest
        .observations
        .iter()
        .filter_map(|observation| observation.peak_process_rss_bytes.value)
        .max();
    let storage_bytes = manifest.model.as_ref().map(|model| {
        model
            .shards
            .iter()
            .chain(model.companions.iter())
            .map(|file| file.bytes)
            .sum::<u64>()
    });
    let quality_pass_rate = match quality {
        Some(quality) => {
            quality_attachment_decision(manifest, quality)?;
            let scored = quality
                .cases
                .iter()
                .filter(|case| matches!(case.status, QualityStatus::Passed | QualityStatus::Failed))
                .count();
            if scored == 0 {
                None
            } else {
                let passed = quality
                    .cases
                    .iter()
                    .filter(|case| case.status == QualityStatus::Passed)
                    .count();
                Some(passed as f64 / scored as f64)
            }
        }
        None => None,
    };
    Ok(CandidateEvidence {
        id: id.into(),
        result_class,
        decode_tps: Some(summary.decode_tps.mean),
        prefill_tps: summary.prefill_tps.as_ref().map(|stats| stats.mean),
        p95_latency_ms: summary.first_token_ms.as_ref().map(|stats| stats.p95),
        peak_memory_bytes,
        quality_pass_rate,
        storage_bytes,
    })
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CandidateEvidence {
    pub id: String,
    pub result_class: FitClass,
    pub decode_tps: Option<f64>,
    pub prefill_tps: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub peak_memory_bytes: Option<u64>,
    pub quality_pass_rate: Option<f64>,
    pub storage_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RecommendationConstraints {
    pub min_decode_tps: Option<f64>,
    pub max_p95_latency_ms: Option<f64>,
    pub max_peak_memory_bytes: Option<u64>,
    pub min_quality_pass_rate: Option<f64>,
    pub max_storage_bytes: Option<u64>,
    pub require_measured: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ObjectiveWeights {
    pub decode_tps: f64,
    pub prefill_tps: f64,
    pub latency: f64,
    pub memory: f64,
    pub quality: f64,
    pub storage: f64,
}

impl Default for ObjectiveWeights {
    fn default() -> Self {
        Self {
            decode_tps: 0.30,
            prefill_tps: 0.10,
            latency: 0.20,
            memory: 0.15,
            quality: 0.20,
            storage: 0.05,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScoreComponent {
    pub objective: String,
    pub normalized: f64,
    pub weight: f64,
    pub contribution: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankedCandidate {
    pub id: String,
    pub feasible: bool,
    pub violations: Vec<String>,
    pub pareto: bool,
    pub dominated_by: Vec<String>,
    pub preference_score: Option<f64>,
    pub score_components: Vec<ScoreComponent>,
}

type ObjectiveExtractor = fn(&CandidateEvidence) -> Option<f64>;
type ObjectiveSpec = (&'static str, f64, bool, ObjectiveExtractor);

fn violations(candidate: &CandidateEvidence, limits: &RecommendationConstraints) -> Vec<String> {
    let mut result = Vec::new();
    if matches!(
        candidate.result_class,
        FitClass::Unknown | FitClass::Estimated | FitClass::Failed | FitClass::RequiresOffload
    ) {
        result.push(format!(
            "Result class {:?} is not runnable",
            candidate.result_class
        ));
    }
    if limits.require_measured && candidate.result_class != FitClass::Measured {
        result.push("Measured evidence is required".into());
    }
    let required_min = [
        (
            "decode throughput",
            candidate.decode_tps,
            limits.min_decode_tps,
        ),
        (
            "quality pass rate",
            candidate.quality_pass_rate,
            limits.min_quality_pass_rate,
        ),
    ];
    for (name, actual, limit) in required_min {
        if let Some(limit) = limit {
            match actual {
                Some(actual) if actual >= limit => {}
                Some(actual) => result.push(format!("{name} {actual} is below {limit}")),
                None => result.push(format!("{name} is unknown")),
            }
        }
    }
    if let Some(limit) = limits.max_p95_latency_ms {
        match candidate.p95_latency_ms {
            Some(actual) if actual <= limit => {}
            Some(actual) => result.push(format!("p95 latency {actual} exceeds {limit}")),
            None => result.push("p95 latency is unknown".into()),
        }
    }
    for (name, actual, limit) in [
        (
            "peak memory",
            candidate.peak_memory_bytes,
            limits.max_peak_memory_bytes,
        ),
        ("storage", candidate.storage_bytes, limits.max_storage_bytes),
    ] {
        if let Some(limit) = limit {
            match actual {
                Some(actual) if actual <= limit => {}
                Some(actual) => result.push(format!("{name} {actual} exceeds {limit}")),
                None => result.push(format!("{name} is unknown")),
            }
        }
    }
    result
}

fn dominates(left: &CandidateEvidence, right: &CandidateEvidence) -> bool {
    let mut compared = false;
    let mut strictly_better = false;
    for (left, right, maximize) in [
        (left.decode_tps, right.decode_tps, true),
        (left.prefill_tps, right.prefill_tps, true),
        (left.p95_latency_ms, right.p95_latency_ms, false),
        (
            left.peak_memory_bytes.map(|value| value as f64),
            right.peak_memory_bytes.map(|value| value as f64),
            false,
        ),
        (left.quality_pass_rate, right.quality_pass_rate, true),
        (
            left.storage_bytes.map(|value| value as f64),
            right.storage_bytes.map(|value| value as f64),
            false,
        ),
    ] {
        let (Some(left), Some(right)) = (left, right) else {
            continue;
        };
        compared = true;
        if (maximize && left < right) || (!maximize && left > right) {
            return false;
        }
        strictly_better |= left != right;
    }
    compared && strictly_better
}

fn normalized(value: f64, minimum: f64, maximum: f64, maximize: bool) -> f64 {
    if maximum == minimum {
        return 1.0;
    }
    let increasing = (value - minimum) / (maximum - minimum);
    if maximize {
        increasing
    } else {
        1.0 - increasing
    }
}

fn validate_optional_metric(
    label: &str,
    value: Option<f64>,
    minimum: f64,
    maximum: f64,
) -> Result<(), String> {
    if value.is_some_and(|value| !value.is_finite() || value < minimum || value > maximum) {
        return Err(format!(
            "{label} must be finite and between {minimum} and {maximum}"
        ));
    }
    Ok(())
}

fn validate_candidate(candidate: &CandidateEvidence) -> Result<(), String> {
    if candidate.id.trim().is_empty() || candidate.id.len() > 256 {
        return Err("Candidate id must contain 1 to 256 bytes".into());
    }
    validate_optional_metric(
        "decodeTps",
        candidate.decode_tps,
        f64::MIN_POSITIVE,
        1_000_000.0,
    )?;
    validate_optional_metric(
        "prefillTps",
        candidate.prefill_tps,
        f64::MIN_POSITIVE,
        10_000_000.0,
    )?;
    validate_optional_metric("p95LatencyMs", candidate.p95_latency_ms, 0.0, 86_400_000.0)?;
    validate_optional_metric("qualityPassRate", candidate.quality_pass_rate, 0.0, 1.0)
}

fn validate_constraints(limits: &RecommendationConstraints) -> Result<(), String> {
    validate_optional_metric(
        "minDecodeTps",
        limits.min_decode_tps,
        f64::MIN_POSITIVE,
        1_000_000.0,
    )?;
    validate_optional_metric(
        "maxP95LatencyMs",
        limits.max_p95_latency_ms,
        0.0,
        86_400_000.0,
    )?;
    validate_optional_metric("minQualityPassRate", limits.min_quality_pass_rate, 0.0, 1.0)?;
    if limits.max_peak_memory_bytes == Some(0) || limits.max_storage_bytes == Some(0) {
        return Err("Byte constraints must be greater than zero when present".into());
    }
    Ok(())
}

pub fn rank_candidates(
    candidates: &[CandidateEvidence],
    limits: &RecommendationConstraints,
    weights: &ObjectiveWeights,
) -> Result<Vec<RankedCandidate>, String> {
    if candidates.is_empty() {
        return Err("At least one candidate is required".into());
    }
    if candidates.len() > 10_000 {
        return Err("Candidate count exceeds the limit of 10000".into());
    }
    validate_constraints(limits)?;
    for candidate in candidates {
        validate_candidate(candidate)?;
    }
    let weight_values = [
        weights.decode_tps,
        weights.prefill_tps,
        weights.latency,
        weights.memory,
        weights.quality,
        weights.storage,
    ];
    let weight_sum = weight_values.iter().sum::<f64>();
    if weight_values
        .iter()
        .any(|weight| !weight.is_finite() || *weight < 0.0 || *weight > 1_000_000.0)
        || !weight_sum.is_finite()
        || weight_sum <= 0.0
    {
        return Err("Objective weights must be finite, non-negative, and not all zero".into());
    }
    let violations = candidates
        .iter()
        .map(|candidate| violations(candidate, limits))
        .collect::<Vec<_>>();
    let feasible_indices = violations
        .iter()
        .enumerate()
        .filter_map(|(index, items)| items.is_empty().then_some(index))
        .collect::<Vec<_>>();
    let ranges = |extract: fn(&CandidateEvidence) -> Option<f64>| {
        let values = feasible_indices
            .iter()
            .filter_map(|index| extract(&candidates[*index]))
            .collect::<Vec<_>>();
        values
            .iter()
            .copied()
            .reduce(f64::min)
            .zip(values.iter().copied().reduce(f64::max))
    };
    let objectives: [ObjectiveSpec; 6] = [
        ("decodeTps", weights.decode_tps, true, |item| {
            item.decode_tps
        }),
        ("prefillTps", weights.prefill_tps, true, |item| {
            item.prefill_tps
        }),
        ("p95LatencyMs", weights.latency, false, |item| {
            item.p95_latency_ms
        }),
        ("peakMemoryBytes", weights.memory, false, |item| {
            item.peak_memory_bytes.map(|value| value as f64)
        }),
        ("qualityPassRate", weights.quality, true, |item| {
            item.quality_pass_rate
        }),
        ("storageBytes", weights.storage, false, |item| {
            item.storage_bytes.map(|value| value as f64)
        }),
    ];
    let mut ranked = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.iter().enumerate() {
        let feasible = violations[index].is_empty();
        let mut dominated_by = if feasible {
            feasible_indices
                .iter()
                .filter(|other| **other != index && dominates(&candidates[**other], candidate))
                .map(|other| candidates[*other].id.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        dominated_by.sort();
        let mut score_components = Vec::new();
        if feasible {
            for (name, weight, maximize, extract) in objectives {
                if weight == 0.0 {
                    continue;
                }
                if let (Some(value), Some((minimum, maximum))) =
                    (extract(candidate), ranges(extract))
                {
                    let normalized = normalized(value, minimum, maximum, maximize);
                    score_components.push(ScoreComponent {
                        objective: name.into(),
                        normalized,
                        weight,
                        contribution: normalized * weight,
                    });
                }
            }
        }
        let total_weight = score_components.iter().map(|item| item.weight).sum::<f64>();
        let preference_score = (total_weight > 0.0).then(|| {
            score_components
                .iter()
                .map(|item| item.contribution)
                .sum::<f64>()
                / total_weight
        });
        ranked.push(RankedCandidate {
            id: candidate.id.clone(),
            feasible,
            violations: violations[index].clone(),
            pareto: feasible && dominated_by.is_empty(),
            dominated_by,
            preference_score,
            score_components,
        });
    }
    ranked.sort_by(|left, right| {
        right
            .feasible
            .cmp(&left.feasible)
            .then_with(|| right.pareto.cmp(&left.pareto))
            .then_with(|| {
                right
                    .preference_score
                    .unwrap_or(f64::NEG_INFINITY)
                    .total_cmp(&left.preference_score.unwrap_or(f64::NEG_INFINITY))
            })
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(ranked)
}

#[cfg(test)]
mod tests {

    fn mt09_manifest_fixture(launch_key: Option<String>) -> crate::evidence::BenchmarkManifest {
        use crate::evidence::{
            AttemptOutcome, BenchmarkManifest, BenchmarkObservation, Evidence, EvidenceSource,
            EvidenceSourceKind, RuntimeFact,
        };
        let mut manifest = BenchmarkManifest {
            compatibility_key: Some(format!("v2:{}", "c".repeat(64))),
            launch_compatibility_key: launch_key,
            execution_snapshot_schema: crate::calibration::EXECUTION_SNAPSHOT_SCHEMA.into(),
            runtime: Some(RuntimeFact {
                path: "runtime.exe".into(),
                version: "1".into(),
                build: "1".into(),
                executable_sha256: Some("f".repeat(64)),
                help_sha256: "b".repeat(64),
                backend: "cpu".into(),
            }),
            model: Some(crate::evidence::ModelFact {
                logical_id: "fixture".into(),
                architecture: "llama".into(),
                shards: vec![crate::evidence::FileFact {
                    path: "model.gguf".into(),
                    bytes: 1,
                    sha256: Some("d".repeat(64)),
                }],
                companions: Vec::new(),
                gguf_header_sha256: "e".repeat(64),
            }),
            launch: Some(crate::evidence::LaunchFact {
                requested_context: 4_096,
                effective_context: Evidence {
                    value: Some(4_096),
                    level: crate::evidence::EvidenceLevel::Observed,
                    source: EvidenceSource {
                        kind: EvidenceSourceKind::Runtime,
                        detail: "fixture".into(),
                    },
                    observed_at_ms: 1,
                    notes: Vec::new(),
                },
                parallel: 1,
                batch: 512,
                ubatch: 128,
                gpu_layers: "all".into(),
                cache_type_k: "f16".into(),
                cache_type_v: "f16".into(),
                split_mode: "none".into(),
                ..crate::evidence::LaunchFact::default()
            }),
            ..BenchmarkManifest::default()
        };
        manifest.workload.trials = 2;
        for trial in 1..=2u16 {
            manifest.observations.push(BenchmarkObservation {
                trial,
                started_at_ms: 100 + u64::from(trial),
                duration_ms: 100.0,
                prompt_tokens: 8,
                cached_prompt_tokens: 0,
                generated_tokens: 16,
                prefill_tps: Some(10.0),
                decode_tps: Some(50.0),
                first_token_ms: Some(5.0),
                derived_ttft_ms: None,
                peak_process_rss_bytes: Evidence::unknown(
                    EvidenceSource {
                        kind: EvidenceSourceKind::Runtime,
                        detail: "fixture".into(),
                    },
                    100 + u64::from(trial),
                    "fixture",
                ),
                outcome: AttemptOutcome::Succeeded,
                error: None,
            });
        }
        manifest
    }

    fn mt09_quality_fixture(key: &str, model_sha: &str, runtime_sha: &str) -> QualitySuiteResult {
        QualitySuiteResult {
            suite_id: "localmotive-structural-v1".into(),
            seed: 42,
            observed_at_ms: Some(1),
            model_logical_id: Some("fixture".into()),
            runtime_sha256: Some(runtime_sha.into()),
            model_content_sha256: Some(model_sha.into()),
            compatibility_key: Some(key.into()),
            suite_version: "structural-smoke.v1".into(),
            cases_planned: 2,
            status: QualityStatus::Passed,
            cases: vec![
                QualityCaseResult {
                    case_id: "exact-ready-v1".into(),
                    status: QualityStatus::Passed,
                    detail: "ok".into(),
                },
                QualityCaseResult {
                    case_id: "json-status-v1".into(),
                    status: QualityStatus::Passed,
                    detail: "ok".into(),
                },
            ],
        }
    }

    #[test]
    fn mt09_quality_joins_only_with_matching_identity() {
        let launch_key = format!("v2:{}", "a".repeat(64));
        let manifest = mt09_manifest_fixture(Some(launch_key.clone()));
        let matching = mt09_quality_fixture(&launch_key, &"d".repeat(64), &"f".repeat(64));
        let candidate = candidate_from_manifest(
            "fixture",
            &manifest,
            crate::evidence::FitClass::Measured,
            Some(&matching),
        )
        .expect("a fully matching quality result attaches");
        assert_eq!(candidate.quality_pass_rate, Some(1.0));

        // A different launch identity (changed KV precision, speculation,
        // LoRA, companions, or effective args) refuses attachment.
        let other_key = format!("v2:{}", "9".repeat(64));
        let other_launch = mt09_quality_fixture(&other_key, &"d".repeat(64), &"f".repeat(64));
        let error = candidate_from_manifest(
            "fixture",
            &manifest,
            crate::evidence::FitClass::Measured,
            Some(&other_launch),
        )
        .unwrap_err();
        assert!(error.contains("different configuration"), "{error}");

        // Changed model tensor bytes with the same header and size.
        let other_model = mt09_quality_fixture(&launch_key, &"1".repeat(64), &"f".repeat(64));
        let error = candidate_from_manifest(
            "fixture",
            &manifest,
            crate::evidence::FitClass::Measured,
            Some(&other_model),
        )
        .unwrap_err();
        assert!(error.contains("different model content"), "{error}");

        // Another runtime executable.
        let other_runtime = mt09_quality_fixture(&launch_key, &"d".repeat(64), &"2".repeat(64));
        let error = candidate_from_manifest(
            "fixture",
            &manifest,
            crate::evidence::FitClass::Measured,
            Some(&other_runtime),
        )
        .unwrap_err();
        assert!(error.contains("different runtime"), "{error}");

        // A quality result without an identity never attaches.
        let mut anonymous = mt09_quality_fixture(&launch_key, &"d".repeat(64), &"f".repeat(64));
        anonymous.compatibility_key = None;
        let error = candidate_from_manifest(
            "fixture",
            &manifest,
            crate::evidence::FitClass::Measured,
            Some(&anonymous),
        )
        .unwrap_err();
        assert!(
            error.contains("does not carry an execution identity"),
            "{error}"
        );
    }

    #[test]
    fn mt09_manifests_without_a_launch_identity_refuse_quality() {
        let manifest = mt09_manifest_fixture(None);
        let quality = mt09_quality_fixture(
            &format!("v2:{}", "a".repeat(64)),
            &"d".repeat(64),
            &"f".repeat(64),
        );
        let error = candidate_from_manifest(
            "fixture",
            &manifest,
            crate::evidence::FitClass::Measured,
            Some(&quality),
        )
        .unwrap_err();
        assert!(error.contains("launch-scope execution identity"), "{error}");

        // Without quality the candidate still builds.
        let candidate = candidate_from_manifest(
            "fixture",
            &manifest,
            crate::evidence::FitClass::Measured,
            None,
        )
        .expect("quality-free candidates still rank");
        assert_eq!(candidate.quality_pass_rate, None);
    }

    #[test]
    fn mt09_suite_is_labelled_as_structural_smoke() {
        let suite = run_quality_suite_with(|_, prompt| {
            if prompt.contains("READY") {
                Ok("READY".into())
            } else {
                Ok("{\"status\":\"ok\"}".into())
            }
        });
        assert_eq!(suite.suite_version, "structural-smoke.v1");
        assert_eq!(suite.cases_planned, 2);
        assert_eq!(suite.status, QualityStatus::Passed);
        assert_eq!(suite.cases.len(), 2);
    }

    use super::*;

    #[test]
    fn structural_quality_checks_are_deterministic_and_report_failures() {
        let exact = evaluate_quality_case("exact-ready-v1", " READY\n");
        let json = evaluate_quality_case("json-status-v1", r#"{"status":"ok"}"#);
        let invalid = evaluate_quality_case("json-status-v1", "status: ok");

        assert_eq!(exact.status, QualityStatus::Passed);
        assert_eq!(json.status, QualityStatus::Passed);
        assert_eq!(invalid.status, QualityStatus::Failed);
        assert!(invalid.detail.contains("JSON"));
    }

    #[test]
    fn quality_suite_reports_case_results_without_a_speed_score() {
        let result = run_quality_suite_with(|case_id, _prompt| match case_id {
            "exact-ready-v1" => Ok("READY".into()),
            "json-status-v1" => Ok(r#"{"status":"wrong"}"#.into()),
            _ => unreachable!(),
        });

        assert_eq!(result.status, QualityStatus::Failed);
        assert_eq!(result.cases.len(), 2);
        assert_eq!(result.seed, 42);
        assert!(result.observed_at_ms.is_none());
        assert!(result.model_logical_id.is_none());
        assert!(result.runtime_sha256.is_none());
        assert_eq!(result.cases[0].status, QualityStatus::Passed);
        assert_eq!(result.cases[1].status, QualityStatus::Failed);
    }

    #[test]
    fn ranking_keeps_tradeoffs_on_the_pareto_frontier_and_names_dominators() {
        let candidate = |id: &str, class: FitClass, decode_tps: f64, latency: f64, memory: u64| {
            CandidateEvidence {
                id: id.into(),
                result_class: class,
                decode_tps: Some(decode_tps),
                prefill_tps: Some(decode_tps * 2.0),
                p95_latency_ms: Some(latency),
                peak_memory_bytes: Some(memory),
                quality_pass_rate: Some(0.9),
                storage_bytes: Some(1_000),
            }
        };
        let candidates = vec![
            candidate("fast", FitClass::Measured, 100.0, 100.0, 8_000),
            candidate("lean", FitClass::Measured, 90.0, 80.0, 6_000),
            candidate("dominated", FitClass::Measured, 80.0, 120.0, 8_000),
            candidate("estimated", FitClass::Estimated, 300.0, 20.0, 2_000),
            candidate("failed", FitClass::Failed, 200.0, 40.0, 4_000),
        ];

        let ranked = rank_candidates(
            &candidates,
            &RecommendationConstraints::default(),
            &ObjectiveWeights::default(),
        )
        .unwrap();

        assert!(ranked.iter().find(|item| item.id == "fast").unwrap().pareto);
        assert!(ranked.iter().find(|item| item.id == "lean").unwrap().pareto);
        let dominated = ranked.iter().find(|item| item.id == "dominated").unwrap();
        assert!(!dominated.pareto);
        assert!(dominated.dominated_by.contains(&"fast".into()));
        assert!(
            !ranked
                .iter()
                .find(|item| item.id == "failed")
                .unwrap()
                .feasible
        );
        assert!(
            !ranked
                .iter()
                .find(|item| item.id == "estimated")
                .unwrap()
                .feasible
        );
    }

    #[test]
    fn ranking_rejects_non_finite_or_out_of_domain_candidate_metrics() {
        let candidate = CandidateEvidence {
            id: "invalid".into(),
            result_class: FitClass::Measured,
            decode_tps: Some(f64::NAN),
            prefill_tps: Some(10.0),
            p95_latency_ms: Some(-1.0),
            peak_memory_bytes: Some(1),
            quality_pass_rate: Some(1.1),
            storage_bytes: Some(1),
        };

        let error = rank_candidates(
            &[candidate],
            &RecommendationConstraints::default(),
            &ObjectiveWeights::default(),
        )
        .unwrap_err();

        assert!(error.contains("decodeTps"));
    }

    #[test]
    fn ranking_rejects_invalid_constraint_domains() {
        let candidate = CandidateEvidence {
            id: "candidate".into(),
            result_class: FitClass::Measured,
            decode_tps: Some(10.0),
            prefill_tps: Some(20.0),
            p95_latency_ms: Some(1.0),
            peak_memory_bytes: Some(1),
            quality_pass_rate: Some(0.9),
            storage_bytes: Some(1),
        };
        let limits = RecommendationConstraints {
            min_quality_pass_rate: Some(1.1),
            ..RecommendationConstraints::default()
        };

        let error =
            rank_candidates(&[candidate], &limits, &ObjectiveWeights::default()).unwrap_err();

        assert!(error.contains("minQualityPassRate"));
    }
}
