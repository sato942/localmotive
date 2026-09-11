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

/// Hard cap on how many profile fields one proposal may change. The system
/// prompt states this policy; the loop enforces it in Rust too, because a
/// model that ignores instructions must not turn one paid reply into an
/// unattributable multi-field measurement (audit MT-03).
pub const MAX_CHANGED_FIELDS_PER_PROPOSAL: usize = 3;

/// Overall wall-clock budget for one tuning session, so no combination of
/// slow measurements and talkative advisors can keep a session alive
/// indefinitely (audit MT-03, MT-04).
pub const TUNING_DEADLINE_SECS: u64 = 45 * 60;

/// Independent loop budgets. All three bound different resources: total
/// advisor calls (paid requests), measured trials, and consecutive
/// rejections (no-op, duplicate, or invalid proposals). A valid no-op reply
/// consumes the advisor-call budget even though it records no trial.
#[derive(Clone, Copy, Debug)]
pub struct TuningBudgets {
    pub max_advisor_calls: u32,
    pub max_consecutive_rejections: u32,
    pub deadline: Option<std::time::Instant>,
}

impl Default for TuningBudgets {
    fn default() -> Self {
        Self {
            max_advisor_calls: 40,
            max_consecutive_rejections: 3,
            deadline: None,
        }
    }
}

/// Fields whose changes can alter generated output quality rather than only
/// speed; the session reports them with the winner (audit MT-11 I4).
pub const QUALITY_AFFECTING_FIELDS: &[&str] = &["cacheTypeK", "cacheTypeV", "specType"];

/// One measurement as the bench saw it: the summary, the command that ran,
/// and the observed effective per-slot context from the running server. The
/// observed value is what the requested-capacity objective is checked against
/// (audit MT-11).
#[derive(Clone, Debug)]
pub struct TrialMeasurement {
    pub summary: BenchmarkSummary,
    pub command: String,
    pub effective_context: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinalVerification {
    pub baseline_tps: f64,
    pub winner_tps: f64,
    /// Relative improvement the winner had to beat (2x the baseline drift,
    /// floored at [`MIN_MATERIAL_IMPROVEMENT`]).
    pub required_improvement: f64,
    pub confirmed: bool,
}

pub const MIN_MATERIAL_IMPROVEMENT: f64 = 0.03;

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
    /// Observed effective per-slot context of the trial server, when the
    /// runtime reported it (audit MT-11 I1/I3).
    #[serde(default)]
    pub effective_context: Option<u32>,
    /// Standard deviation of the trial samples, so improvement claims can be
    /// checked against observed variation (audit MT-11 I4).
    #[serde(default)]
    pub std_dev: Option<f64>,
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
    fn measure(&mut self, profile: &LaunchProfile) -> Result<TrialMeasurement, String>;
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
    /// The measured objective, stated plainly (audit MT-11 I1/I2): the
    /// retained harness measures short-prompt decode throughput at an
    /// allocated context, not a filled full-context workload.
    #[serde(default)]
    pub objective: String,
    /// The requested per-slot context every scored candidate had to observe.
    #[serde(default)]
    pub required_effective_context: u32,
    /// Baseline and winner remeasured after the loop; `None` when the
    /// verification could not run (cancelled or failed measurement).
    #[serde(default)]
    pub final_verification: Option<FinalVerification>,
    /// Winner changes that can alter output quality and were NOT quality
    /// gated in this session (audit MT-11 I4).
    #[serde(default)]
    pub quality_affecting_changes: Vec<String>,
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
    /// The workload the retained harness measures (audit MT-11 I1): generated
    /// token count and repeats go into the objective label and the brief.
    pub measured_tokens: u32,
    pub measured_repeats: u16,
    pub budgets: TuningBudgets,
    /// The user's Stop signal. Checked before and after every advisor call
    /// and before every measurement; cancellation is a terminal outcome, not
    /// a candidate failure (audit MT-04).
    pub cancel: Option<&'a std::sync::atomic::AtomicBool>,
}

/// Record one bounded rejection (no-op, duplicate, invalid, or oversized) as
/// a failed trial so the reason stays visible to the advisor and the user
/// without spending a measurement (audit MT-03).
fn push_rejection(
    trials: &mut Vec<TuningTrial>,
    on_trial: &mut dyn FnMut(&TuningTrial),
    proposal: &Proposal,
    reason: String,
) {
    let trial = TuningTrial {
        index: trials.len() as u32,
        changes: proposal.changes.clone(),
        rationale: proposal.rationale.clone(),
        mean_tps: None,
        median_tps: None,
        error: Some(reason),
        command: String::new(),
        effective_context: None,
        std_dev: None,
    };
    on_trial(&trial);
    trials.push(trial);
}

/// The measured objective, stated plainly (audit MT-11 I1/I2): the retained
/// harness scores short-prompt decode throughput at the allocated context.
fn objective_label(inputs: &TuningInputs) -> String {
    format!(
        "Short-prompt decode throughput: {} repeats × {} generated tokens on the fixed harness prompt (prompt occupancy is a few dozen tokens) at an allocated context of {} tokens; output quality and latency are not measured",
        inputs.measured_repeats, inputs.measured_tokens, inputs.target_context
    )
}

/// Sample standard deviation of the trial samples; zero when undefined.
fn sample_std_dev(samples: &[f64]) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|sample| (sample - mean).powi(2))
        .sum::<f64>()
        / (samples.len() as f64 - 1.0);
    variance.sqrt()
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
    let required_context = inputs.target_context;

    let record = |trials: &mut Vec<TuningTrial>,
                  best: &mut Option<(u32, f64, LaunchProfile)>,
                  on_trial: &mut dyn FnMut(&TuningTrial),
                  profile: &LaunchProfile,
                  changes: BTreeMap<String, serde_json::Value>,
                  rationale: String,
                  outcome: Result<TrialMeasurement, String>| {
        let index = trials.len() as u32;
        let trial = match outcome {
            Ok(measurement) => {
                let scoreable = match measurement.effective_context {
                    Some(observed) if observed >= required_context => Ok(observed),
                    Some(observed) => Err(format!(
                        "Effective per-slot context was {observed} tokens, below the required {required_context}; the candidate cannot win the requested-capacity objective"
                    )),
                    None => Err(
                        "The effective per-slot context could not be observed, so the candidate cannot be scored for the requested-capacity objective"
                            .into(),
                    ),
                };
                match scoreable {
                    Ok(observed) => {
                        if best
                            .as_ref()
                            .is_none_or(|(_, tps, _)| measurement.summary.mean_tps > *tps)
                        {
                            *best = Some((index, measurement.summary.mean_tps, profile.clone()));
                        }
                        TuningTrial {
                            index,
                            changes,
                            rationale,
                            mean_tps: Some(measurement.summary.mean_tps),
                            median_tps: Some(measurement.summary.median_tps),
                            error: None,
                            command: measurement.command,
                            effective_context: Some(observed),
                            std_dev: Some(sample_std_dev(&measurement.summary.samples)),
                        }
                    }
                    Err(error) => TuningTrial {
                        index,
                        changes,
                        rationale,
                        mean_tps: None,
                        median_tps: None,
                        error: Some(error),
                        command: measurement.command,
                        effective_context: measurement.effective_context,
                        std_dev: None,
                    },
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
                effective_context: None,
                std_dev: None,
            },
        };
        on_trial(&trial);
        trials.push(trial);
    };

    let is_cancelled = |inputs: &TuningInputs| {
        inputs
            .cancel
            .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed))
    };
    let deadline_reached = |inputs: &TuningInputs| {
        inputs
            .budgets
            .deadline
            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
    };
    if is_cancelled(inputs) {
        return Ok(TuningReport {
            baseline_tps: None,
            best_index: None,
            best_tps: None,
            best_profile: baseline.clone(),
            trials,
            stopped_reason: "Cancelled by the user before the baseline measurement".into(),
            objective: objective_label(inputs),
            required_effective_context: required_context,
            final_verification: None,
            quality_affecting_changes: Vec::new(),
        });
    }
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
        if is_cancelled(inputs) {
            trials.pop();
            return Ok(TuningReport {
                baseline_tps: None,
                best_index: None,
                best_tps: None,
                best_profile: baseline.clone(),
                trials,
                stopped_reason: "Cancelled by the user during the baseline measurement".into(),
                objective: objective_label(inputs),
                required_effective_context: required_context,
                final_verification: None,
                quality_affecting_changes: Vec::new(),
            });
        }
        return Err(format!(
            "Baseline failed, so there is nothing to tune from: {}",
            trials[0].error.clone().unwrap_or_default()
        ));
    }

    // Every effective configuration seen so far, canonicalized AFTER coercion
    // and companion resolution, starting with the baseline. Comparing raw
    // proposal maps misses nonempty maps that normalize to an earlier
    // configuration (audit MT-03).
    let mut seen_effective: Vec<BTreeMap<String, serde_json::Value>> = vec![BTreeMap::new()];
    let mut stopped_reason = "Trial budget exhausted".to_string();
    let mut consecutive_rejections = 0_u32;
    let mut consecutive_advisor_failures = 0_u32;
    let mut advisor_calls = 0_u32;
    // Measured trials only: rejection rows are recorded in the history but
    // must not silently spend the measurement budget (audit MT-03).
    let mut measured_trials = 1_u32;
    while measured_trials <= inputs.max_trials {
        if is_cancelled(inputs) {
            stopped_reason = "Cancelled by the user".into();
            break;
        }
        if deadline_reached(inputs) {
            stopped_reason = format!(
                "Overall tuning deadline reached ({} minutes)",
                TUNING_DEADLINE_SECS / 60
            );
            break;
        }
        if advisor_calls >= inputs.budgets.max_advisor_calls {
            stopped_reason = format!(
                "Advisor call budget exhausted ({} calls)",
                inputs.budgets.max_advisor_calls
            );
            break;
        }
        let remaining = inputs.max_trials + 1 - measured_trials;
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
        advisor_calls += 1;
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
        if is_cancelled(inputs) {
            stopped_reason = "Cancelled by the user".into();
            break;
        }
        if proposal.done || proposal.changes.is_empty() {
            stopped_reason = if proposal.rationale.is_empty() {
                "Advisor declared convergence".into()
            } else {
                format!("Advisor stopped: {}", proposal.rationale)
            };
            break;
        }
        // The advertised three-fields-per-trial policy is a hard limit here
        // (audit MT-03 I4). `draftModel` is tuner-owned noise, not a field the
        // model actually chose.
        let changed_fields = proposal
            .changes
            .keys()
            .filter(|field| field.as_str() != "draftModel")
            .count();
        if changed_fields > MAX_CHANGED_FIELDS_PER_PROPOSAL {
            consecutive_rejections += 1;
            let reason = format!(
                "Rejected: the proposal changed {changed_fields} fields; at most {MAX_CHANGED_FIELDS_PER_PROPOSAL} are allowed per trial"
            );
            push_rejection(&mut trials, &mut on_trial, &proposal, reason);
            if consecutive_rejections >= inputs.budgets.max_consecutive_rejections {
                stopped_reason = "Advisor repeatedly proposed changes outside the policy".into();
                break;
            }
            if advisor_calls >= inputs.budgets.max_advisor_calls {
                stopped_reason = format!(
                    "Advisor call budget exhausted ({} calls)",
                    inputs.budgets.max_advisor_calls
                );
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
                consecutive_rejections += 1;
                push_rejection(
                    &mut trials,
                    &mut on_trial,
                    &proposal,
                    format!("Rejected before launch: {error}"),
                );
                if consecutive_rejections >= inputs.budgets.max_consecutive_rejections {
                    stopped_reason = "Advisor repeatedly proposed invalid changes".into();
                    break;
                }
                if advisor_calls >= inputs.budgets.max_advisor_calls {
                    stopped_reason = format!(
                        "Advisor call budget exhausted ({} calls)",
                        inputs.budgets.max_advisor_calls
                    );
                    break;
                }
                continue;
            }
        };
        // A proposal that leaves the effective configuration unchanged is a
        // bounded rejection with its reason in the history, never an
        // unbounded continuation (audit MT-03).
        if applied.is_empty() {
            consecutive_rejections += 1;
            push_rejection(
                &mut trials,
                &mut on_trial,
                &proposal,
                "Rejected: a no-op proposal that leaves the effective configuration unchanged"
                    .into(),
            );
            if consecutive_rejections >= inputs.budgets.max_consecutive_rejections {
                stopped_reason = "Advisor repeatedly proposed no-op or duplicate changes".into();
                break;
            }
            if advisor_calls >= inputs.budgets.max_advisor_calls {
                stopped_reason = format!(
                    "Advisor call budget exhausted ({} calls)",
                    inputs.budgets.max_advisor_calls
                );
                break;
            }
            continue;
        }
        if seen_effective.iter().any(|seen| seen == &applied) {
            consecutive_rejections += 1;
            push_rejection(
                &mut trials,
                &mut on_trial,
                &proposal,
                "Rejected: this configuration was already measured in an earlier trial".into(),
            );
            if consecutive_rejections >= inputs.budgets.max_consecutive_rejections {
                stopped_reason = "Advisor repeatedly proposed no-op or duplicate changes".into();
                break;
            }
            if advisor_calls >= inputs.budgets.max_advisor_calls {
                stopped_reason = format!(
                    "Advisor call budget exhausted ({} calls)",
                    inputs.budgets.max_advisor_calls
                );
                break;
            }
            continue;
        }
        consecutive_rejections = 0;
        seen_effective.push(applied.clone());
        measured_trials += 1;
        let outcome = bench.measure(&candidate);
        if is_cancelled(inputs) {
            // Cancellation during a measurement is terminal and is never
            // recorded as a candidate failure or success (audit MT-04).
            stopped_reason = "Cancelled by the user during a measurement".into();
            break;
        }
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

    let mut quality_affecting_changes: Vec<String> = Vec::new();
    // Final verification (audit MT-11 I4): remeasure the baseline and the
    // finalist, and require a material improvement beyond the observed
    // baseline drift. A winner that cannot clear it is not reported.
    let mut final_verification: Option<FinalVerification> = None;
    let mut confirmed_winner_tps: Option<(u32, f64, LaunchProfile)> = None;
    // Trial index 0 is the baseline; when nothing beat it there is no winner
    // to re-verify, and re-measuring the baseline against itself would only
    // invent a failure note.
    if let Some((index, _winner_tps, winner_profile)) =
        best.clone().filter(|(index, _, _)| *index != 0)
    {
        if is_cancelled(inputs) {
            stopped_reason = format!("{stopped_reason}; final verification skipped (cancelled)");
        } else {
            quality_affecting_changes = trials
                .iter()
                .find(|trial| trial.index == index)
                .map(|trial| {
                    trial
                        .changes
                        .keys()
                        .filter(|field| QUALITY_AFFECTING_FIELDS.contains(&field.as_str()))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            quality_affecting_changes.sort();
            let remeasure_baseline = bench.measure(&baseline);
            let remeasure_winner = if is_cancelled(inputs) {
                Err("Cancelled".into())
            } else {
                bench.measure(&winner_profile)
            };
            match (remeasure_baseline, remeasure_winner) {
                (Ok(baseline_again), Ok(winner_again)) => {
                    let b = baseline_again.summary.mean_tps;
                    let w = winner_again.summary.mean_tps;
                    let drift = if b.abs() > f64::EPSILON {
                        (b - baseline_tps.unwrap_or(b)).abs() / b.abs()
                    } else {
                        0.0
                    };
                    let required = (2.0 * drift).max(MIN_MATERIAL_IMPROVEMENT);
                    let confirmed = w > b * (1.0 + required);
                    final_verification = Some(FinalVerification {
                        baseline_tps: b,
                        winner_tps: w,
                        required_improvement: required,
                        confirmed,
                    });
                    if confirmed {
                        confirmed_winner_tps = Some((index, w, winner_profile.clone()));
                    } else {
                        stopped_reason = format!(
                            "{stopped_reason}; the measured winner did not clear the material-improvement bar (winner {:.2} tok/s vs baseline {:.2} tok/s, required +{:.1}%)",
                            w,
                            b,
                            required * 100.0
                        );
                    }
                }
                _ => {
                    stopped_reason = format!(
                        "{stopped_reason}; the winner could not be re-verified (remeasurement failed or was cancelled)"
                    );
                }
            }
        }
    }
    let (mut best_index, mut best_tps, mut best_profile) = match confirmed_winner_tps {
        Some((index, tps, profile)) => (Some(index), Some(tps), profile),
        None => (None, None, baseline.clone()),
    };
    let _ = &mut best_index;
    let _ = &mut best_tps;
    let _ = &mut best_profile;
    Ok(TuningReport {
        baseline_tps,
        best_index,
        best_tps,
        best_profile,
        trials,
        stopped_reason,
        objective: objective_label(inputs),
        required_effective_context: required_context,
        final_verification,
        quality_affecting_changes,
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
            help_sha256: "a".repeat(64),
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
            system_memory: crate::runtime::detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
            unassigned_nvidia: Vec::new(),
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
        fn measure(&mut self, profile: &LaunchProfile) -> Result<TrialMeasurement, String> {
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
            Ok(TrialMeasurement {
                summary: summarize_benchmark(vec![tps, tps + 1.0], 256, 2).unwrap(),
                command: format!(
                    "cmd draft={} fa={}",
                    profile.draft_max, profile.flash_attention
                ),
                // A one-slot server at the target context: the gate passes.
                effective_context: Some(4096 / u32::from(profile.parallel.max(1))),
            })
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
            measured_tokens: 256,
            measured_repeats: 2,
            budgets: TuningBudgets::default(),
            cancel: None,
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
            6,
            "the rejected proposal must not be measured; the winner and baseline are re-measured for final verification"
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
            measured_tokens: 256,
            measured_repeats: 2,
            budgets: TuningBudgets::default(),
            cancel: None,
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
        assert_eq!(
            bench.calls.len(),
            4,
            "garbled replies are never measured; the winner and baseline are re-measured for final verification"
        );
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
            measured_tokens: 256,
            measured_repeats: 2,
            budgets: TuningBudgets::default(),
            cancel: None,
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

    /// Advisor that repeats one fixed reply and counts every call, so budget
    /// assertions observe the exact number of (paid) requests.
    struct CountingAdvisor {
        reply: String,
        calls: usize,
        cancel_after: Option<(usize, std::sync::Arc<std::sync::atomic::AtomicBool>)>,
    }
    impl Advisor for CountingAdvisor {
        fn propose(&mut self, _brief: &TuningBrief) -> Result<Proposal, String> {
            self.calls += 1;
            if let Some((after, flag)) = &self.cancel_after {
                if self.calls >= *after {
                    flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
            parse_proposal(&self.reply)
        }
    }

    fn tuning_inputs<'a>(
        hardware: &'a HardwareInfo,
        capabilities: &'a RuntimeCapabilities,
        budgets: TuningBudgets,
        cancel: Option<&'a std::sync::atomic::AtomicBool>,
    ) -> TuningInputs<'a> {
        TuningInputs {
            objective: "max tok/s",
            target_context: 4096,
            hardware,
            system_ram_bytes: None,
            gguf: None,
            capabilities,
            companions: &[],
            max_trials: 6,
            measured_tokens: 256,
            measured_repeats: 2,
            budgets,
            cancel,
        }
    }

    #[test]
    fn mt03_repeated_effective_noops_terminate_within_the_advisor_call_budget() {
        // The audited defect: a valid proposal that changes nothing reset the
        // rejection counter and looped forever. A deterministic advisor that
        // always proposes the existing value must now terminate inside the
        // independent advisor-call budget, with every no-op preserved as a
        // bounded rejection (audit MT-03 V1).
        let base = base_profile(); // threads defaults to -1 in LaunchProfile::default()
        let hw = hardware();
        let cap = caps();
        let budgets = TuningBudgets {
            max_advisor_calls: 5,
            max_consecutive_rejections: 100,
            deadline: None,
        };
        let inputs = tuning_inputs(&hw, &cap, budgets, None);
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = CountingAdvisor {
            reply: r#"{"changes":{"threads":-1},"rationale":"auto threads are best","done":false}"#
                .into(),
            calls: 0,
            cancel_after: None,
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();

        assert_eq!(
            advisor.calls, 5,
            "the advisor-call budget must bind exactly"
        );
        assert_eq!(
            report.stopped_reason, "Advisor call budget exhausted (5 calls)",
            "{}",
            report.stopped_reason
        );
        assert_eq!(
            report.trials.len(),
            6,
            "baseline plus five recorded rejections"
        );
        assert_eq!(
            bench.calls.len(),
            1,
            "no no-op configuration is ever measured"
        );
        for trial in &report.trials[1..] {
            assert!(
                trial
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("no-op")),
                "rejection reason must be recorded: {trial:?}"
            );
        }
    }

    #[test]
    fn mt03_noop_rejections_are_also_bounded_by_the_consecutive_rejection_limit() {
        // With a generous call budget the terminal outcome must come from the
        // rejection counter itself: repeated no-ops end the session long
        // before the advisor-call budget (audit MT-03 I3).
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let budgets = TuningBudgets {
            max_advisor_calls: 100,
            max_consecutive_rejections: 3,
            deadline: None,
        };
        let inputs = tuning_inputs(&hw, &cap, budgets, None);
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = CountingAdvisor {
            reply: r#"{"changes":{"threads":-1},"rationale":"no-op again","done":false}"#.into(),
            calls: 0,
            cancel_after: None,
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();

        assert_eq!(advisor.calls, 3, "three rejections end the session");
        assert_eq!(
            report.stopped_reason,
            "Advisor repeatedly proposed no-op or duplicate changes"
        );
        assert_eq!(
            report.trials.len(),
            4,
            "baseline plus three recorded rejections"
        );
    }

    #[test]
    fn mt03_coercions_echoes_and_oversize_proposals_are_bounded_rejections() {
        // Numeric-string coercion to the existing value, a bare draftModel
        // echo, and a four-field change are all rejected with a recorded
        // reason; the fourth consecutive rejection ends the session (MT-03 V2).
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let inputs = tuning_inputs(&hw, &cap, TuningBudgets::default(), None);
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                r#"{"changes":{"threads":"-1"},"rationale":"coerced no-op","done":false}"#,
                r#"{"changes":{"draftModel":"C:\\m\\d.gguf"},"rationale":"echo","done":false}"#,
                r#"{"changes":{"gpuLayers":10,"batch":1024,"ubatch":512,"flashAttention":"on"},"rationale":"too many","done":false}"#,
            ],
            briefs_seen: vec![],
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();

        assert_eq!(bench.calls.len(), 1, "rejections are never measured");
        assert_eq!(report.trials.len(), 4, "{:#?}", report.trials);
        assert!(report.trials[1].error.as_deref().unwrap().contains("no-op"));
        assert!(report.trials[2].error.as_deref().unwrap().contains("no-op"));
        assert!(
            report.trials[3]
                .error
                .as_deref()
                .unwrap()
                .contains("changed 4 fields"),
            "{:?}",
            report.trials[3].error
        );
        assert_eq!(
            report.stopped_reason,
            "Advisor repeatedly proposed changes outside the policy"
        );
    }

    #[test]
    fn mt03_a_reordered_duplicate_of_a_measured_configuration_is_rejected() {
        // Comparing canonicalized effective changes (not raw maps) catches a
        // nonempty proposal that normalizes to an already-measured
        // configuration (audit MT-03 I2).
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let inputs = tuning_inputs(&hw, &cap, TuningBudgets::default(), None);
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                r#"{"changes":{"flashAttention":"on"},"rationale":"fa","done":false}"#,
                r#"{"changes":{"threads":-1,"flashAttention":"on"},"rationale":"same effect via a different map","done":false}"#,
                r#"{"changes":{},"rationale":"converged","done":true}"#,
            ],
            briefs_seen: vec![],
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();

        assert_eq!(
            bench.calls.len(),
            4,
            "the duplicate is not measured again; the winner and baseline are re-measured for final verification"
        );
        assert!(
            report.trials[2]
                .error
                .as_deref()
                .is_some_and(|error| error.contains("already measured")),
            "{:?}",
            report.trials[2].error
        );
        assert_eq!(report.stopped_reason, "Advisor stopped: converged");
    }

    #[test]
    fn mt03_cancellation_during_a_noop_sequence_stops_before_the_next_advisor_call() {
        // Combined with the lifecycle cancellation: once Stop arrives, a
        // talkative no-op advisor never gets another paid call (MT-03 V3).
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let inputs = tuning_inputs(&hw, &cap, TuningBudgets::default(), Some(flag.as_ref()));
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = CountingAdvisor {
            reply: r#"{"changes":{"threads":-1},"rationale":"no-op again","done":false}"#.into(),
            calls: 0,
            cancel_after: Some((2, flag.clone())),
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();

        assert_eq!(advisor.calls, 2, "cancellation forbids a third paid call");
        assert_eq!(report.stopped_reason, "Cancelled by the user");
        assert_eq!(
            report.trials.len(),
            2,
            "baseline plus the first no-op rejection"
        );
    }

    #[test]
    fn mt03_the_overall_deadline_stops_the_loop_with_the_documented_reason() {
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let budgets = TuningBudgets {
            max_advisor_calls: 100,
            max_consecutive_rejections: 100,
            deadline: Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
        };
        let inputs = tuning_inputs(&hw, &cap, budgets, None);
        let mut bench = FakeBench { calls: vec![] };
        let mut advisor = CountingAdvisor {
            reply: r#"{"changes":{"flashAttention":"on"},"rationale":"fa","done":false}"#.into(),
            calls: 0,
            cancel_after: None,
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();

        assert_eq!(
            advisor.calls, 0,
            "an expired deadline forbids the first paid call"
        );
        assert!(
            report.stopped_reason.contains("deadline"),
            "{}",
            report.stopped_reason
        );
        assert_eq!(report.trials.len(), 1, "only the baseline was recorded");
    }

    struct CancelDuringMeasureBench {
        calls: usize,
        flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }
    impl Bench for CancelDuringMeasureBench {
        fn measure(&mut self, _profile: &LaunchProfile) -> Result<TrialMeasurement, String> {
            self.calls += 1;
            if self.calls > 1 {
                // The user presses Stop while this candidate is running; the
                // legacy path would surface the resulting failure as an
                // ordinary candidate error.
                self.flag.store(true, std::sync::atomic::Ordering::Relaxed);
                return Err("server exited with code 1".into());
            }
            Ok(TrialMeasurement {
                summary: summarize_benchmark(vec![100.0], 256, 1).unwrap(),
                command: "cmd".into(),
                effective_context: Some(4096),
            })
        }
    }

    /// A bench scripted per call: (tps, observed effective context). Calls
    /// beyond the script repeat the last entry.
    struct ScriptedBench {
        calls: Vec<LaunchProfile>,
        script: Vec<(f64, Option<u32>)>,
    }
    impl Bench for ScriptedBench {
        fn measure(&mut self, profile: &LaunchProfile) -> Result<TrialMeasurement, String> {
            let index = self.calls.len().min(self.script.len() - 1);
            self.calls.push(profile.clone());
            let (tps, ctx) = self.script[index];
            Ok(TrialMeasurement {
                summary: summarize_benchmark(vec![tps, tps + 1.0], 256, 2).unwrap(),
                command: "cmd".into(),
                effective_context: ctx,
            })
        }
    }

    #[test]
    fn mt11_reduced_or_unobserved_effective_context_cannot_win() {
        // V1: a candidate whose parallelism divides the context below target
        // (4096/4 = 1024) must not silently win, even at a much higher mean.
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let inputs = tuning_inputs(&hw, &cap, TuningBudgets::default(), None);
        let mut bench = ScriptedBench {
            calls: vec![],
            // baseline 100 @4096; candidate 999 @1024 (parallel divides);
            // then verification re-measurements are not reached because no
            // candidate is scoreable.
            script: vec![
                (100.0, Some(4096)),
                (999.0, Some(1024)),
                (100.0, Some(4096)),
            ],
        };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                r#"{"changes":{"flashAttention":"on"},"rationale":"fast","done":false}"#,
                r#"{"changes":{},"rationale":"converged","done":true}"#,
            ],
            briefs_seen: vec![],
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();
        let candidate = report.trials.iter().find(|trial| trial.index == 1).unwrap();
        let error = candidate.error.as_deref().unwrap();
        assert!(error.contains("below the required 4096"), "{error}");
        assert_eq!(
            report.best_index, None,
            "an unscoreable candidate cannot win"
        );
        assert_eq!(
            report.best_profile.parallel, base.parallel,
            "the reported profile stays the baseline when nothing scored"
        );

        // A candidate whose context could not be observed is rejected too.
        let mut bench = ScriptedBench {
            calls: vec![],
            script: vec![(100.0, Some(4096)), (999.0, None), (100.0, Some(4096))],
        };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                r#"{"changes":{"flashAttention":"on"},"rationale":"fast","done":false}"#,
                r#"{"changes":{},"rationale":"converged","done":true}"#,
            ],
            briefs_seen: vec![],
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();
        let error = report.trials[1].error.as_deref().unwrap();
        assert!(error.contains("could not be observed"), "{error}");
        assert_eq!(report.best_index, None);
    }

    #[test]
    fn mt11_winner_needs_material_improvement_beyond_observed_variation() {
        // V3: a winner that only edges past the baseline inside the observed
        // drift is not reported; the final verification records why.
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let inputs = tuning_inputs(&hw, &cap, TuningBudgets::default(), None);
        // baseline 100; candidate 101 (+1%); verification remeasures baseline
        // 100 and the candidate 101: below the 3% floor.
        let mut bench = ScriptedBench {
            calls: vec![],
            script: vec![
                (100.0, Some(4096)),
                (101.0, Some(4096)),
                (100.0, Some(4096)),
                (101.0, Some(4096)),
            ],
        };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                r#"{"changes":{"flashAttention":"on"},"rationale":"tiny","done":false}"#,
                r#"{"changes":{},"rationale":"converged","done":true}"#,
            ],
            briefs_seen: vec![],
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();
        let verification = report.final_verification.expect("verification ran");
        assert!(!verification.confirmed);
        assert_eq!(
            report.best_index, None,
            "an unconfirmed winner is not reported"
        );
        assert!(
            report.stopped_reason.contains("material-improvement"),
            "{}",
            report.stopped_reason
        );

        // A large, clear improvement is confirmed by the re-measurement.
        let mut bench = ScriptedBench {
            calls: vec![],
            script: vec![
                (100.0, Some(4096)),
                (130.0, Some(4096)),
                (100.0, Some(4096)),
                (130.0, Some(4096)),
            ],
        };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                r#"{"changes":{"flashAttention":"on"},"rationale":"clear win","done":false}"#,
                r#"{"changes":{},"rationale":"converged","done":true}"#,
            ],
            briefs_seen: vec![],
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();
        let verification = report.final_verification.expect("verification ran");
        assert!(verification.confirmed);
        assert_eq!(report.best_index, Some(1));
        assert_eq!(report.best_tps, Some(130.5));
    }

    #[test]
    fn mt11_objective_label_and_quality_affecting_changes_are_reported() {
        // I1/I2/I4: the report states the retained short-prompt objective,
        // the per-slot requirement, and flags winner changes that affect
        // output quality without a quality gate.
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let inputs = tuning_inputs(&hw, &cap, TuningBudgets::default(), None);
        let mut bench = ScriptedBench {
            calls: vec![],
            script: vec![
                (100.0, Some(4096)),
                (130.0, Some(4096)),
                (100.0, Some(4096)),
                (130.0, Some(4096)),
            ],
        };
        let mut advisor = ScriptedAdvisor {
            replies: vec![
                r#"{"changes":{"cacheTypeK":"q8_0"},"rationale":"kv quality trade","done":false}"#,
                r#"{"changes":{},"rationale":"converged","done":true}"#,
            ],
            briefs_seen: vec![],
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();
        assert_eq!(report.required_effective_context, 4096);
        assert!(
            report.objective.contains("Short-prompt decode throughput"),
            "{}",
            report.objective
        );
        assert!(
            report
                .objective
                .contains("quality and latency are not measured"),
            "{}",
            report.objective
        );
        assert_eq!(
            report.quality_affecting_changes,
            vec!["cacheTypeK".to_string()]
        );
        // Sample standard deviation of [130, 131] is sqrt(1/2); compare
        // against the computed value instead of a magic literal.
        let expected = (0.5_f64).sqrt();
        let std_dev = report.trials[1]
            .std_dev
            .expect("a scored trial records its spread");
        assert!(
            (std_dev - expected).abs() < 1e-9,
            "spread of [130, 131] should be {expected}, got {std_dev}"
        );
    }

    #[test]
    fn mt04_cancellation_during_a_measurement_is_terminal_not_a_candidate_failure() {
        // A cancelled candidate must never appear as a failed configuration,
        // and the loop must stop proposing instead of continuing (MT-04 V1).
        let base = base_profile();
        let hw = hardware();
        let cap = caps();
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let inputs = tuning_inputs(&hw, &cap, TuningBudgets::default(), Some(flag.as_ref()));
        let mut bench = CancelDuringMeasureBench {
            calls: 0,
            flag: flag.clone(),
        };
        let mut advisor = CountingAdvisor {
            reply: r#"{"changes":{"flashAttention":"on"},"rationale":"fa","done":false}"#.into(),
            calls: 0,
            cancel_after: None,
        };
        let report = run_tuning(&base, &inputs, &mut bench, &mut advisor, |_| {}).unwrap();

        assert_eq!(bench.calls, 2, "the candidate was attempted once");
        assert_eq!(advisor.calls, 1, "no proposal follows the cancellation");
        assert_eq!(
            report.stopped_reason,
            "Cancelled by the user during a measurement"
        );
        assert_eq!(
            report.trials.len(),
            1,
            "cancellation is not recorded as a candidate failure: {:#?}",
            report.trials
        );
    }
}
