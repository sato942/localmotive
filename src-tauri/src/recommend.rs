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
        status,
        cases,
    }
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
