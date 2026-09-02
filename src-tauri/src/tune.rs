//! AI-assisted throughput tuning.
//!
//! The cloud model never touches the machine. It receives a structured brief
//! (hardware, GGUF facts, runtime capabilities, the current profile, and every
//! measurement so far) and replies with a JSON proposal that changes only
//! whitelisted, throughput-relevant profile fields. This module owns the
//! whitelist, the prompt, the proposal parsing/validation, the profile
//! application, and the measure-propose-measure loop. Networking and process
//! supervision are injected so the loop is unit-testable without a GPU.

use crate::core::{BenchmarkSummary, LaunchProfile, RuntimeCapabilities};
use crate::gguf::GgufSummary;
use crate::runtime::HardwareInfo;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Fields the tuner may change, with the reason each one is throughput-relevant.
/// Anything not listed here is rejected even if the cloud model proposes it.
pub const TUNABLE_FIELDS: &[(&str, &str)] = &[
    ("gpuLayers", "layers offloaded to GPU; `all` or a count"),
    ("cpuMoe", "MoE expert layers kept on CPU"),
    ("cpuFfn", "dense FFN layers kept on CPU"),
    ("threads", "generation threads (-1 = auto)"),
    ("threadsBatch", "prompt-processing threads (-1 = auto)"),
    ("batch", "logical batch size"),
    ("ubatch", "physical micro-batch size (<= batch)"),
    ("flashAttention", "auto | on | off"),
    ("kvOffload", "keep KV cache on GPU"),
    (
        "cacheTypeK",
        "KV cache K type: f16 bf16 q8_0 q4_0 q4_1 iq4_nl q5_0 q5_1 f32",
    ),
    ("cacheTypeV", "KV cache V type (same choices)"),
    ("fit", "smart memory fit on/off"),
    ("fitTarget", "fit margin in MiB"),
    ("splitMode", "none | layer | row | tensor"),
    ("tensorSplit", "per-GPU split ratios, e.g. 3,1"),
    ("mainGpu", "primary GPU index"),
    ("specType", "speculative method advertised by the runtime"),
    ("draftMax", "maximum draft tokens per step"),
    ("draftMin", "minimum draft tokens per step"),
    ("draftPMin", "draft acceptance probability floor"),
    ("draftPSplit", "draft split probability"),
    ("draftGpuLayers", "draft model GPU layers"),
    ("ngramMatch", "ngram-mod match length"),
    ("ngramMin", "ngram-mod minimum draft"),
    ("ngramMax", "ngram-mod maximum draft"),
    ("ngramSizeN", "ngram map lookup size"),
    ("ngramSizeM", "ngram map draft size"),
    ("ngramMinHits", "ngram map minimum hits"),
    ("continuousBatching", "continuous batching on/off"),
    ("parallel", "parallel slots (context is divided among them)"),
];

pub fn is_tunable(field: &str) -> bool {
    TUNABLE_FIELDS.iter().any(|(name, _)| *name == field)
}

/// A single measured configuration in the tuning history.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TuningTrial {
    pub index: u32,
    /// Only the fields that differ from the baseline profile.
    pub changes: BTreeMap<String, serde_json::Value>,
    pub rationale: String,
    /// None when the server failed to start or the benchmark failed.
    pub mean_tps: Option<f64>,
    pub median_tps: Option<f64>,
    pub error: Option<String>,
    pub command: String,
}

/// What the cloud model must return.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    #[serde(default)]
    pub changes: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub rationale: String,
    /// The model may declare it has converged; the loop then stops early.
    #[serde(default)]
    pub done: bool,
}

/// Everything the cloud model is told. Serialized verbatim into the user turn.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TuningBrief<'a> {
    pub objective: &'a str,
    pub target_context: u32,
    pub hardware: &'a HardwareInfo,
    pub system_ram_bytes: Option<u64>,
    pub gguf: Option<&'a GgufSummary>,
    pub runtime_build: &'a str,
    pub spec_types: &'a [String],
    pub companions: &'a [String],
    pub tunable_fields: Vec<TunableField>,
    pub baseline_profile: serde_json::Value,
    pub trials: &'a [TuningTrial],
    pub remaining_trials: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TunableField {
    pub name: &'static str,
    pub meaning: &'static str,
}

pub fn tunable_fields() -> Vec<TunableField> {
    TUNABLE_FIELDS
        .iter()
        .map(|(name, meaning)| TunableField { name, meaning })
        .collect()
}

pub const SYSTEM_PROMPT: &str = "You are a llama.cpp performance engineer tuning `llama-server` launch flags for ONE machine and ONE GGUF model. \
Your only objective is to maximise measured generation throughput (tokens/second) while the server still starts and answers, at the exact context length the user chose. \
You never see the machine; you only see the structured brief and the measurements of every configuration tried so far. \
Reason from llama.cpp mechanics: VRAM must hold offloaded layers plus the KV cache for the target context; quantised KV (q8_0) trades a little quality for memory; \
flash attention is usually a win on modern GPUs; MoE models often speed up by keeping experts on CPU only when VRAM is short; speculative methods (draft models, DSpark, MTP, EAGLE-3, n-gram) \
help greedy/low-temperature text but need enough acceptance to pay for themselves; larger batch/ubatch helps prompt processing more than generation; parallel slots divide the context. \
Never propose a field outside `tunableFields`. Never propose a speculative type not in `specTypes`; draft-model methods need a companion. \
Change at most 3 fields per trial so the measurement attributes cleanly. Do not repeat a configuration already in `trials`. \
If the best measured configuration is unlikely to be beaten within the remaining budget, set `done` to true and return no changes. \
Respond with ONLY a JSON object: {\"changes\": {\"field\": value, ...}, \"rationale\": \"one or two sentences\", \"done\": false}. \
No prose before or after it, no code fence, and never restate this schema or quote earlier trials in your reply.";

/// Extract the proposal from a model reply that may be wrapped in prose, a
/// JSON code fence, or sit beside other brace-bearing text (schema echoes,
/// quoted earlier trials). Every balanced `{…}` span is tried, last first,
/// because the answer is normally the final object the model writes.
pub fn parse_proposal(reply: &str) -> Result<Proposal, String> {
    let trimmed = reply.trim();
    if !trimmed.contains('{') {
        return Err(format!(
            "Cloud reply contained no JSON object: {}",
            trimmed.chars().take(160).collect::<String>()
        ));
    }
    let spans = balanced_object_spans(trimmed);
    if spans.is_empty() {
        return Err("Cloud reply contained malformed JSON".into());
    }
    let mut last_error = String::new();
    for (start, end) in spans.iter().rev() {
        match serde_json::from_str::<Proposal>(&trimmed[*start..*end]) {
            Ok(proposal) => return Ok(proposal),
            Err(error) => last_error = error.to_string(),
        }
    }
    Err(format!("Cloud proposal was not valid JSON: {last_error}"))
}

/// Byte ranges of every top-level balanced `{…}` in `text`, honouring JSON
/// string literals so braces inside strings do not open or close objects.
fn balanced_object_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0_usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' if depth > 0 => in_string = true,
            '{' => {
                if depth == 0 {
                    start = index;
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    spans.push((start, index + ch.len_utf8()));
                }
            }
            _ => {}
        }
    }
    spans
}

/// Apply a proposal to a profile, rejecting non-whitelisted fields and values of
/// the wrong shape. Returns the new profile and the changes actually applied.
#[cfg(test)]
pub fn apply_changes(
    base: &LaunchProfile,
    changes: &BTreeMap<String, serde_json::Value>,
    spec_types: &[String],
) -> Result<(LaunchProfile, BTreeMap<String, serde_json::Value>), String> {
    apply_changes_with_companions(base, changes, spec_types, &[])
}

/// The companion role a model-backed speculative method consumes.
fn companion_role_for(spec_type: &str) -> Option<&'static str> {
    match spec_type {
        "draft-dspark" => Some("dspark"),
        "draft-dflash" => Some("dflash"),
        "draft-eagle3" => Some("eagle3"),
        "draft-mtp" => Some("mtp"),
        "draft-simple" => Some("draft"),
        _ => None,
    }
}

/// Like [`apply_changes`], but when the advisor selects a model-backed
/// speculative method the matching companion (`"<role>: <path>"`, as listed in
/// the brief) is attached as the draft model. `draftModel` itself stays
/// non-tunable: the advisor chooses the *method*, the tuner supplies the file.
pub fn apply_changes_with_companions(
    base: &LaunchProfile,
    changes: &BTreeMap<String, serde_json::Value>,
    spec_types: &[String],
    companions: &[String],
) -> Result<(LaunchProfile, BTreeMap<String, serde_json::Value>), String> {
    let mut json = serde_json::to_value(base).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Profile did not serialise to an object".to_string())?;
    let mut applied = BTreeMap::new();
    for (field, value) in changes {
        if field == "draftModel" {
            // Tuner-owned: derived from `specType` + companions below. Advisors
            // echo it from the trial history; that is noise, not a proposal.
            continue;
        }
        if !is_tunable(field) {
            return Err(format!("`{field}` is not a tunable field"));
        }
        if field == "specType" {
            let requested = value
                .as_str()
                .ok_or_else(|| "`specType` must be a string".to_string())?;
            if !spec_types.iter().any(|known| known == requested) {
                return Err(format!(
                    "`specType` {requested} is not advertised by this runtime"
                ));
            }
        }
        let current = object
            .get(field)
            .ok_or_else(|| format!("`{field}` is not a profile field"))?;
        let coerced = coerce_like(current, value)
            .ok_or_else(|| format!("`{field}` received a value of the wrong type: {value}"))?;
        if &coerced != current {
            applied.insert(field.clone(), coerced.clone());
        }
        object.insert(field.clone(), coerced);
    }

    // Resolve the draft model from the selected method, not from the advisor.
    if let Some(spec_type) = object.get("specType").and_then(|v| v.as_str()) {
        let spec_type = spec_type.to_string();
        let current_draft = object
            .get("draftModel")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        match companion_role_for(&spec_type) {
            Some(role) => {
                let prefix = format!("{role}:");
                let path = companions
                    .iter()
                    .find_map(|entry| entry.strip_prefix(&prefix).map(|p| p.trim().to_string()))
                    .filter(|p| !p.is_empty());
                match path {
                    Some(path) => {
                        object.insert("draftModel".into(), serde_json::Value::String(path));
                    }
                    None if current_draft.is_empty() => {
                        return Err(format!(
                            "{spec_type} needs a draft model but this model has no {role} companion"
                        ));
                    }
                    None => {}
                }
            }
            None if !current_draft.is_empty() && changes.contains_key("specType") => {
                object.insert("draftModel".into(), serde_json::Value::Null);
            }
            None => {}
        }
    }

    let profile: LaunchProfile = serde_json::from_value(json)
        .map_err(|error| format!("Proposal broke the profile: {error}"))?;
    // Re-run the profile's own validation so an invalid combination is rejected
    // before a server is ever launched with it.
    profile.build_args()?;
    Ok((profile, applied))
}

/// Coerce a proposed value into the JSON shape of the existing field so
/// `"99"` becomes `99` for numeric fields and `"all"` stays a string for
/// string fields. Returns None when no sensible coercion exists.
fn coerce_like(
    current: &serde_json::Value,
    proposed: &serde_json::Value,
) -> Option<serde_json::Value> {
    use serde_json::Value;
    match current {
        Value::Bool(_) => match proposed {
            Value::Bool(b) => Some(Value::Bool(*b)),
            Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
                "true" | "on" | "yes" => Some(Value::Bool(true)),
                "false" | "off" | "no" => Some(Value::Bool(false)),
                _ => None,
            },
            _ => None,
        },
        Value::Number(n) => {
            let text = match proposed {
                Value::Number(m) => m.to_string(),
                Value::String(s) => s.trim().to_string(),
                _ => return None,
            };
            if n.is_f64() {
                text.parse::<f64>()
                    .ok()
                    .and_then(|f| serde_json::Number::from_f64(f).map(Value::Number))
            } else if n.is_i64() {
                text.parse::<i64>().ok().map(|i| Value::Number(i.into()))
            } else {
                text.parse::<u64>().ok().map(|u| Value::Number(u.into()))
            }
        }
        Value::String(_) => match proposed {
            Value::String(s) => Some(Value::String(s.clone())),
            Value::Number(m) => Some(Value::String(m.to_string())),
            Value::Bool(b) => Some(Value::String(if *b { "on" } else { "off" }.into())),
            _ => None,
        },
        _ => None,
    }
}

/// Outcome of measuring one configuration. Injected so tests never launch a server.
pub trait Bench {
    fn measure(&mut self, profile: &LaunchProfile) -> Result<(BenchmarkSummary, String), String>;
}

/// The cloud model. Injected so tests never make HTTP calls.
pub trait Advisor {
    fn propose(&mut self, brief: &TuningBrief) -> Result<Proposal, String>;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TuningReport {
    pub baseline_tps: Option<f64>,
    pub best_index: Option<u32>,
    pub best_tps: Option<f64>,
    pub best_profile: LaunchProfile,
    pub trials: Vec<TuningTrial>,
    pub stopped_reason: String,
}

pub struct TuningInputs<'a> {
    pub objective: &'a str,
    pub target_context: u32,
    pub hardware: &'a HardwareInfo,
    pub system_ram_bytes: Option<u64>,
    pub gguf: Option<&'a GgufSummary>,
    pub capabilities: &'a RuntimeCapabilities,
    pub companions: &'a [String],
    pub max_trials: u32,
}

/// Measure the baseline, then alternate propose → validate → measure until the
/// budget is spent or the advisor declares convergence. Failed launches are
/// recorded as trials (with the error) so the advisor can learn from them.
pub fn run_tuning<B: Bench, A: Advisor>(
    base: &LaunchProfile,
    inputs: &TuningInputs,
    bench: &mut B,
    advisor: &mut A,
    mut on_trial: impl FnMut(&TuningTrial),
) -> Result<TuningReport, String> {
    let mut baseline = base.clone();
    baseline.context = inputs.target_context;
    let baseline_json = serde_json::to_value(&baseline).map_err(|error| error.to_string())?;

    let mut trials: Vec<TuningTrial> = Vec::new();
    let mut best: Option<(u32, f64, LaunchProfile)> = None;

    let record = |trials: &mut Vec<TuningTrial>,
                  best: &mut Option<(u32, f64, LaunchProfile)>,
                  on_trial: &mut dyn FnMut(&TuningTrial),
                  profile: &LaunchProfile,
                  changes: BTreeMap<String, serde_json::Value>,
                  rationale: String,
                  outcome: Result<(BenchmarkSummary, String), String>| {
        let index = trials.len() as u32;
        let trial = match outcome {
            Ok((summary, command)) => {
                if best
                    .as_ref()
                    .is_none_or(|(_, tps, _)| summary.mean_tps > *tps)
                {
                    *best = Some((index, summary.mean_tps, profile.clone()));
                }
                TuningTrial {
                    index,
                    changes,
                    rationale,
                    mean_tps: Some(summary.mean_tps),
                    median_tps: Some(summary.median_tps),
                    error: None,
                    command,
                }
            }
            Err(error) => TuningTrial {
                index,
                changes,
                rationale,
                mean_tps: None,
                median_tps: None,
                error: Some(error),
                command: String::new(),
            },
        };
        on_trial(&trial);
        trials.push(trial);
    };

    record(
        &mut trials,
        &mut best,
        &mut on_trial,
        &baseline,
        BTreeMap::new(),
        "Baseline: the profile exactly as configured, at the target context.".into(),
        bench.measure(&baseline),
    );
    let baseline_tps = trials[0].mean_tps;
    if baseline_tps.is_none() {
        return Err(format!(
            "Baseline failed, so there is nothing to tune from: {}",
            trials[0].error.clone().unwrap_or_default()
        ));
    }

    let mut stopped_reason = "Trial budget exhausted".to_string();
    let mut consecutive_rejections = 0_u32;
    let mut consecutive_advisor_failures = 0_u32;
    while (trials.len() as u32) < inputs.max_trials + 1 {
        let remaining = inputs.max_trials + 1 - trials.len() as u32;
        let brief = TuningBrief {
            objective: inputs.objective,
            target_context: inputs.target_context,
            hardware: inputs.hardware,
            system_ram_bytes: inputs.system_ram_bytes,
            gguf: inputs.gguf,
            runtime_build: &inputs.capabilities.build,
            spec_types: &inputs.capabilities.spec_types,
            companions: inputs.companions,
            tunable_fields: tunable_fields(),
            baseline_profile: baseline_json.clone(),
            trials: &trials,
            remaining_trials: remaining,
        };
        let proposal = match advisor.propose(&brief) {
            Ok(proposal) => {
                consecutive_advisor_failures = 0;
                proposal
            }
            Err(error) => {
                consecutive_advisor_failures += 1;
                if consecutive_advisor_failures >= 3 {
                    stopped_reason = format!("Cloud advisor failed 3 times in a row: {error}");
                    break;
                }
                continue;
            }
        };
        if proposal.done || proposal.changes.is_empty() {
            stopped_reason = if proposal.rationale.is_empty() {
                "Advisor declared convergence".into()
            } else {
                format!("Advisor stopped: {}", proposal.rationale)
            };
            break;
        }
        if trials.iter().any(|trial| trial.changes == proposal.changes) {
            consecutive_rejections += 1;
            if consecutive_rejections >= 2 {
                stopped_reason = "Advisor kept repeating configurations".into();
                break;
            }
            continue;
        }
        let (candidate, applied) = match apply_changes_with_companions(
            &baseline,
            &proposal.changes,
            &inputs.capabilities.spec_types,
            inputs.companions,
        ) {
            Ok(result) => result,
            Err(error) => {
                // Record the rejection as a failed trial so the advisor sees it,
                // without spending a measurement.
                let index = trials.len() as u32;
                let trial = TuningTrial {
                    index,
                    changes: proposal.changes.clone(),
                    rationale: proposal.rationale.clone(),
                    mean_tps: None,
                    median_tps: None,
                    error: Some(format!("Rejected before launch: {error}")),
                    command: String::new(),
                };
                on_trial(&trial);
                trials.push(trial);
                consecutive_rejections += 1;
                if consecutive_rejections >= 3 {
                    stopped_reason = "Advisor repeatedly proposed invalid changes".into();
                    break;
                }
                continue;
            }
        };
        consecutive_rejections = 0;
        if applied.is_empty() {
            continue;
        }
        let outcome = bench.measure(&candidate);
        record(
            &mut trials,
            &mut best,
            &mut on_trial,
            &candidate,
            applied,
            proposal.rationale,
            outcome,
        );
    }

    let (best_index, best_tps, best_profile) = match best {
        Some((index, tps, profile)) => (Some(index), Some(tps), profile),
        None => (None, None, baseline.clone()),
    };
    Ok(TuningReport {
        baseline_tps,
        best_index,
        best_tps,
        best_profile,
        trials,
        stopped_reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::summarize_benchmark;

    fn caps() -> RuntimeCapabilities {
        RuntimeCapabilities {
            path: "x".into(),
            version: "v".into(),
            build: "10679".into(),
            commit: "abc".into(),
            spec_types: vec!["none".into(), "draft-dspark".into(), "ngram-mod".into()],
            supported_flags: vec![],
            metrics: true,
            multimodal: false,
            fit: true,
        }
    }

    fn hardware() -> HardwareInfo {
        HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["Test GPU".into()],
            vendor: "nvidia".into(),
            cuda_major: Some(13),
            driver_version: "1".into(),
            detection_status: "fixture".into(),
            recommendation: String::new(),
        }
    }

    fn base_profile() -> LaunchProfile {
        LaunchProfile {
            alias: "test".into(),
            model: "m.gguf".into(),
            draft_model: Some("d.gguf".into()),
            ..LaunchProfile::default()
        }
    }

    #[test]
    fn whitelist_excludes_identity_network_and_secret_fields() {
        for forbidden in [
            "host",
            "port",
            "alias",
            "model",
            "apiKeyFile",
            "extraArgs",
            "runtime",
            "corsOrigins",
            "context",
        ] {
            assert!(!is_tunable(forbidden), "{forbidden} must not be tunable");
        }
        for allowed in [
            "gpuLayers",
            "cacheTypeK",
            "specType",
            "ubatch",
            "flashAttention",
        ] {
            assert!(is_tunable(allowed), "{allowed} must be tunable");
        }
    }

    #[test]
    fn parses_proposal_wrapped_in_prose_and_fence() {
        let reply = "Sure. Here is my plan:\n```json\n{\"changes\": {\"cacheTypeK\": \"q8_0\", \"draftMax\": 5}, \"rationale\": \"more draft\", \"done\": false}\n```\nGood luck.";
        let proposal = parse_proposal(reply).unwrap();
        assert_eq!(proposal.changes["cacheTypeK"], "q8_0");
        assert_eq!(proposal.changes["draftMax"], 5);
        assert!(!proposal.done);
        assert!(parse_proposal("no json here").is_err());
    }

    #[test]
    fn parses_proposal_when_prose_contains_other_braces() {
        // Seen live: the advisor echoes the schema in prose, then answers. A
        // first-brace..last-brace slice spans both objects and fails with
        // "trailing characters". The parser must find the balanced object that
        // actually deserialises as a proposal.
        let reply = "The format is {\"changes\": {...}, \"rationale\": \"...\"} as requested.\n\n{\"changes\": {\"flashAttention\": \"on\"}, \"rationale\": \"FA helps here\", \"done\": false}\n\nLet me know {if} you want more.";
        let proposal = parse_proposal(reply).unwrap();
        assert_eq!(proposal.changes["flashAttention"], "on");
        assert_eq!(proposal.rationale, "FA helps here");

        // Two complete proposal-shaped objects: the last one is the answer, the
        // earlier one is the model quoting a previous trial.
        let reply = "Previously I proposed {\"changes\": {\"batch\": 4096}, \"rationale\": \"old\", \"done\": false}. Now:\n{\"changes\": {\"ubatch\": 1024}, \"rationale\": \"new\", \"done\": false}";
        let proposal = parse_proposal(reply).unwrap();
        assert_eq!(proposal.changes["ubatch"], 1024);
        assert_eq!(proposal.rationale, "new");

        // Braces inside JSON strings must not confuse the scanner.
        let reply = "{\"changes\": {\"cacheTypeV\": \"q8_0\"}, \"rationale\": \"use {q8_0} for V\", \"done\": false}";
        assert_eq!(parse_proposal(reply).unwrap().rationale, "use {q8_0} for V");
    }

    #[test]
    fn apply_changes_attaches_matching_companion_when_advisor_selects_draft_method() {
        // Live finding: the brief advertises a DSpark companion, the advisor
        // proposes draft-dspark, and the proposal was rejected because
        // `draftModel` is (rightly) not tunable. The tuner must attach the
        // companion whose role matches the selected method itself.
        let companions = vec![
            "mmproj: C:\\m\\proj.gguf".to_string(),
            "dspark: C:\\m\\LFM-DSpark-F16.gguf".to_string(),
        ];
        let mut changes = BTreeMap::new();
        changes.insert("specType".to_string(), serde_json::json!("draft-dspark"));
        changes.insert("draftMax".to_string(), serde_json::json!(8));
        let (profile, applied) = apply_changes_with_companions(
            &base_profile(),
            &changes,
            &caps().spec_types,
            &companions,
        )
        .unwrap();
        assert_eq!(profile.spec_type, "draft-dspark");
        assert_eq!(
            profile.draft_model.as_deref(),
            Some("C:\\m\\LFM-DSpark-F16.gguf")
        );
        // The companion is a consequence of the method, not a proposed change:
        // it must not appear in the recorded changes (otherwise the advisor
        // echoes it back and the duplicate detector stops matching).
        assert!(
            !applied.contains_key("draftModel"),
            "draftModel leaked into changes: {applied:?}"
        );
        assert_eq!(applied.len(), 2, "{applied:?}");

        // Live finding: the advisor echoes `draftModel` from the trial history.
        // The tuner owns that field, so an echoed value is ignored, not rejected.
        let mut echoed = changes.clone();
        echoed.insert(
            "draftModel".to_string(),
            serde_json::json!("C:\\m\\LFM-DSpark-F16.gguf"),
        );
        echoed.insert("draftMax".to_string(), serde_json::json!(12));
        let (profile2, applied2) = apply_changes_with_companions(
            &base_profile(),
            &echoed,
            &caps().spec_types,
            &companions,
        )
        .unwrap();
        assert_eq!(profile2.draft_max, 12);
        assert_eq!(
            profile2.draft_model.as_deref(),
            Some("C:\\m\\LFM-DSpark-F16.gguf")
        );
        assert!(!applied2.contains_key("draftModel"));

        // No matching companion and no draft on the profile: the rejection
        // names what is missing instead of launching a server that will fail.
        let mut bare = base_profile();
        bare.draft_model = None;
        let error = apply_changes_with_companions(
            &bare,
            &changes,
            &caps().spec_types,
            &["mmproj: C:\\m\\proj.gguf".to_string()],
        )
        .unwrap_err();
        assert!(error.contains("no dspark companion"), "{error}");

        // A draft already on the profile is kept when no companion matches.
        let (kept, _) =
            apply_changes_with_companions(&base_profile(), &changes, &caps().spec_types, &[])
                .unwrap();
        assert_eq!(kept.draft_model.as_deref(), Some("d.gguf"));

        // Switching back to a non-model method drops the attached draft model
        // so a stale companion is not passed to a method that cannot use it.
        let mut back = BTreeMap::new();
        back.insert("specType".to_string(), serde_json::json!("none"));
        let (profile, _) =
            apply_changes_with_companions(&profile, &back, &caps().spec_types, &companions)
                .unwrap();
        assert_eq!(profile.spec_type, "none");
        assert!(profile.draft_model.is_none());
    }

    #[test]
    fn apply_changes_rejects_fields_outside_whitelist() {
        let mut changes = BTreeMap::new();
        changes.insert("port".to_string(), serde_json::json!(9999));
        let error = apply_changes(&base_profile(), &changes, &caps().spec_types).unwrap_err();
        assert!(error.contains("not a tunable field"));
    }

    #[test]
    fn apply_changes_coerces_types_and_reports_only_real_differences() {
        let mut changes = BTreeMap::new();
        changes.insert("draftMax".to_string(), serde_json::json!("5"));
        changes.insert("flashAttention".to_string(), serde_json::json!(true));
        changes.insert("kvOffload".to_string(), serde_json::json!("on"));
        changes.insert("batch".to_string(), serde_json::json!(2048)); // unchanged
        let (profile, applied) =
            apply_changes(&base_profile(), &changes, &caps().spec_types).unwrap();
        assert_eq!(profile.draft_max, 5);
        assert_eq!(profile.flash_attention, "on");
        assert!(profile.kv_offload);
        assert_eq!(
            applied.len(),
            2,
            "unchanged batch and already-true kvOffload are not reported: {applied:?}"
        );
        assert!(applied.contains_key("draftMax") && applied.contains_key("flashAttention"));
    }

    #[test]
    fn apply_changes_rejects_unadvertised_speculation_and_invalid_combinations() {
        let mut changes = BTreeMap::new();
        changes.insert("specType".to_string(), serde_json::json!("draft-eagle3"));
        assert!(apply_changes(&base_profile(), &changes, &caps().spec_types)
            .unwrap_err()
            .contains("not advertised"));

        let mut bad = BTreeMap::new();
        bad.insert("ubatch".to_string(), serde_json::json!(4096)); // > batch 2048
        assert!(apply_changes(&base_profile(), &bad, &caps().spec_types)
            .unwrap_err()
            .contains("uBatch"));
    }

    struct FakeBench {
        calls: Vec<LaunchProfile>,
    }
    impl Bench for FakeBench {
        fn measure(
            &mut self,
            profile: &LaunchProfile,
        ) -> Result<(BenchmarkSummary, String), String> {
            self.calls.push(profile.clone());
            if profile.cache_type_k == "q4_0" {
                return Err("server exited with code 1".into());
            }
            let tps = 100.0
                + if profile.flash_attention == "on" {
                    20.0
                } else {
                    0.0
                }
                + if profile.draft_max == 5 { 15.0 } else { 0.0 };
            Ok((
                summarize_benchmark(vec![tps, tps + 1.0], 256, 2).unwrap(),
                format!(
                    "cmd draft={} fa={}",
                    profile.draft_max, profile.flash_attention
                ),
            ))
        }
    }

    struct ScriptedAdvisor {
        replies: Vec<&'static str>,
        briefs_seen: Vec<usize>,
    }
    impl Advisor for ScriptedAdvisor {
        fn propose(&mut self, brief: &TuningBrief) -> Result<Proposal, String> {
            self.briefs_seen.push(brief.trials.len());
            if self.replies.is_empty() {
                return Ok(Proposal {
                    changes: BTreeMap::new(),
                    rationale: "converged".into(),
                    done: true,
                });
            }
            parse_proposal(self.replies.remove(0))
        }
    }

    #[test]
    fn tuning_loop_measures_baseline_learns_from_failures_and_keeps_the_best() {
        let base = base_profile();
        let inputs = TuningInputs {
            objective: "max tok/s",
            target_context: 4096,
            hardware: &hardware(),
            system_ram_bytes: Some(1 << 36),
            gguf: None,
            capabilities: &caps(),
            companions: &["d.gguf".to_string()],
            max_trials: 6,
        };
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                r#"{"changes":{"flashAttention":"on"},"rationale":"fa","done":false}"#,
                r#"{"changes":{"cacheTypeK":"q4_0"},"rationale":"crash test","done":false}"#,
                r#"{"changes":{"port":1},"rationale":"illegal","done":false}"#,
                r#"{"changes":{"flashAttention":"on","draftMax":5},"rationale":"both","done":false}"#,
            ],
            briefs_seen: vec![],
        };
        let mut seen = Vec::new();
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |trial| {
            seen.push(trial.index)
        })
        .unwrap();

        assert_eq!(report.baseline_tps, Some(100.5));
        assert_eq!(report.trials.len(), 5, "{:#?}", report.trials);
        assert!(report.trials[0].changes.is_empty());
        assert_eq!(report.trials[1].mean_tps, Some(120.5));
        assert!(report.trials[2]
            .error
            .as_deref()
            .unwrap()
            .contains("exited"));
        assert!(report.trials[3]
            .error
            .as_deref()
            .unwrap()
            .contains("Rejected before launch"));
        assert_eq!(report.trials[4].mean_tps, Some(135.5));
        assert_eq!(report.best_index, Some(4));
        assert_eq!(report.best_tps, Some(135.5));
        assert_eq!(report.best_profile.draft_max, 5);
        assert_eq!(
            report.best_profile.context, 4096,
            "target context is enforced on every trial"
        );
        assert_eq!(report.stopped_reason, "Advisor stopped: converged");
        assert_eq!(
            bench.calls.len(),
            4,
            "the rejected proposal must not be measured"
        );
        assert_eq!(seen, vec![0, 1, 2, 3, 4]);
        assert_eq!(
            advisor.briefs_seen.first(),
            Some(&1),
            "advisor sees the baseline trial in its first brief"
        );
    }

    #[test]
    fn tuning_loop_retries_a_garbled_advisor_reply_before_giving_up() {
        // One unparseable reply (network hiccup, a model rambling without JSON)
        // must cost a retry, not the whole session. Three in a row ends it.
        let base = base_profile();
        let inputs = TuningInputs {
            objective: "max tok/s",
            target_context: 4096,
            hardware: &hardware(),
            system_ram_bytes: None,
            gguf: None,
            capabilities: &caps(),
            companions: &[],
            max_trials: 4,
        };
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                "I think flash attention would help, let me reason about it first…",
                r#"{"changes":{"flashAttention":"on"},"rationale":"fa","done":false}"#,
                "garbled",
                "still garbled",
                "and again",
                r#"{"changes":{"draftMax":5},"rationale":"never reached","done":false}"#,
            ],
            briefs_seen: vec![],
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();
        assert_eq!(report.trials.len(), 2, "{:#?}", report.trials);
        assert_eq!(report.trials[1].mean_tps, Some(120.5));
        assert_eq!(report.best_index, Some(1));
        assert!(
            report
                .stopped_reason
                .starts_with("Cloud advisor failed 3 times in a row"),
            "{}",
            report.stopped_reason
        );
        assert_eq!(bench.calls.len(), 2, "garbled replies are never measured");
    }

    #[test]
    fn tuning_loop_fails_fast_when_baseline_cannot_run() {
        let mut base = base_profile();
        base.cache_type_k = "q4_0".into();
        let inputs = TuningInputs {
            objective: "x",
            target_context: 2048,
            hardware: &hardware(),
            system_ram_bytes: None,
            gguf: None,
            capabilities: &caps(),
            companions: &[],
            max_trials: 3,
        };
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = ScriptedAdvisor {
            replies: vec![],
            briefs_seen: vec![],
        };
        let error = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap_err();
        assert!(error.contains("Baseline failed"));
        assert!(
            advisor.briefs_seen.is_empty(),
            "advisor is never consulted without a baseline"
        );
    }

    #[test]
    fn brief_serialises_with_camel_case_and_every_tunable_field() {
        let caps = caps();
        let hw = hardware();
        let brief = TuningBrief {
            objective: "o",
            target_context: 8192,
            hardware: &hw,
            system_ram_bytes: None,
            gguf: None,
            runtime_build: &caps.build,
            spec_types: &caps.spec_types,
            companions: &[],
            tunable_fields: tunable_fields(),
            baseline_profile: serde_json::json!({}),
            trials: &[],
            remaining_trials: 4,
        };
        let json = serde_json::to_value(&brief).unwrap();
        assert_eq!(json["targetContext"], 8192);
        assert_eq!(
            json["tunableFields"].as_array().unwrap().len(),
            TUNABLE_FIELDS.len()
        );
        assert!(SYSTEM_PROMPT.contains("ONLY a JSON object"));
    }
}
