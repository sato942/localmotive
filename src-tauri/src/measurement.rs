use crate::evidence::{
    AttemptOutcome, BenchmarkManifest, BenchmarkObservation, CacheMode, Evidence, EvidenceSource,
    EvidenceSourceKind, FitClass, WarmupObservation, Workload,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_HTTP_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_HTTP_REQUEST_BYTES: usize = 16 * 1024 * 1024;
const BENCHMARK_PROMPT: &str = "Explain deterministic local inference measurement with fixed inputs, explicit evidence, and reproducible results. ";
static MANIFEST_NONCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MetricStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub p50: f64,
    pub p95: f64,
    pub min: f64,
    pub max: f64,
    pub standard_deviation: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSummaryV2 {
    pub result_class: FitClass,
    pub successful_trials: usize,
    pub failed_trials: usize,
    pub prefill_tps: Option<MetricStats>,
    pub decode_tps: MetricStats,
    pub first_token_ms: Option<MetricStats>,
    pub derived_ttft_ms: Option<MetricStats>,
    pub failures: Vec<String>,
}

fn validate_observation(observation: &BenchmarkObservation) -> Result<(), String> {
    if observation.trial == 0 {
        return Err("Benchmark observation trial must be positive".into());
    }
    if !observation.duration_ms.is_finite() || observation.duration_ms < 0.0 {
        return Err(format!(
            "Benchmark observation trial {} durationMs must be finite and non-negative",
            observation.trial
        ));
    }
    for (field, value) in [
        ("prefillTps", observation.prefill_tps),
        ("decodeTps", observation.decode_tps),
        ("firstTokenMs", observation.first_token_ms),
        ("derivedTtftMs", observation.derived_ttft_ms),
    ] {
        if value.is_some_and(|value| !value.is_finite() || value <= 0.0) {
            return Err(format!(
                "Benchmark observation trial {} {field} must be positive and finite",
                observation.trial
            ));
        }
    }
    let outcome_matches_error = matches!(
        (observation.outcome, observation.error.as_ref()),
        (AttemptOutcome::Succeeded, None)
            | (
                AttemptOutcome::Failed | AttemptOutcome::TimedOut | AttemptOutcome::Cancelled,
                Some(_)
            )
    );
    if !outcome_matches_error {
        return Err(format!(
            "Benchmark observation trial {} outcome conflicts with its error evidence",
            observation.trial
        ));
    }
    if observation.outcome == AttemptOutcome::Succeeded
        && (observation.decode_tps.is_none()
            || observation
                .prompt_tokens
                .saturating_add(observation.cached_prompt_tokens)
                == 0
            || observation.generated_tokens == 0)
    {
        return Err(format!(
            "Benchmark observation trial {} requires tokens and decodeTps when successful",
            observation.trial
        ));
    }
    Ok(())
}

/// Bound one acquisition-time failure string so a rich launch error (which
/// can carry a runtime log tail) can never make the final manifest fail
/// validation and discard the whole run, including earlier successes
/// (audit MT-12). The cut is Unicode-safe and the truncation is explicit.
pub const MAX_OBSERVATION_ERROR_BYTES: usize = 4_096;

pub fn bound_observation_error(error: &str) -> String {
    const MARKER: &str =
        " … [message truncated; the full runtime output remains in the local run log]";
    if error.len() <= MAX_OBSERVATION_ERROR_BYTES {
        return error.to_string();
    }
    let budget = MAX_OBSERVATION_ERROR_BYTES.saturating_sub(MARKER.len());
    let mut cut = budget;
    while cut > 0 && !error.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}{MARKER}", &error[..cut])
}

fn metric_stats(values: impl Iterator<Item = f64>) -> Option<MetricStats> {
    let mut values = values
        .filter(|value| value.is_finite() && *value > 0.0)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let count = values.len();
    let scale = values[count - 1];
    let normalized_mean = values.iter().map(|value| value / scale).sum::<f64>() / count as f64;
    let mean = normalized_mean * scale;
    let median = if count % 2 == 0 {
        values[count / 2 - 1] / 2.0 + values[count / 2] / 2.0
    } else {
        values[count / 2]
    };
    let normalized_variance = values
        .iter()
        .map(|value| {
            let difference = value / scale - normalized_mean;
            difference * difference
        })
        .sum::<f64>()
        / count as f64;
    let percentile = |percent: usize| {
        let rank = (percent * count).div_ceil(100).max(1);
        values[rank - 1]
    };
    Some(MetricStats {
        count,
        mean,
        median,
        p50: percentile(50),
        p95: percentile(95),
        min: values[0],
        max: values[count - 1],
        standard_deviation: normalized_variance.sqrt() * scale,
    })
}

pub fn summarize_observations(
    observations: &[BenchmarkObservation],
) -> Result<BenchmarkSummaryV2, String> {
    for observation in observations {
        validate_observation(observation)?;
    }
    let successful = |item: &&BenchmarkObservation| item.outcome == AttemptOutcome::Succeeded;
    let decode_tps = metric_stats(
        observations
            .iter()
            .filter(successful)
            .filter_map(|item| item.decode_tps),
    )
    .ok_or("No successful decode-throughput observations were collected")?;
    let failures = observations
        .iter()
        .filter_map(|item| {
            item.error
                .as_ref()
                .map(|error| format!("trial {}: {error}", item.trial))
        })
        .collect::<Vec<_>>();
    let successful_trials = observations
        .iter()
        .filter(|item| item.outcome == AttemptOutcome::Succeeded)
        .count();
    Ok(BenchmarkSummaryV2 {
        result_class: FitClass::Measured,
        successful_trials,
        failed_trials: observations.len().saturating_sub(successful_trials),
        prefill_tps: metric_stats(
            observations
                .iter()
                .filter(successful)
                .filter_map(|item| item.prefill_tps),
        ),
        decode_tps,
        first_token_ms: metric_stats(
            observations
                .iter()
                .filter(successful)
                .filter_map(|item| item.first_token_ms),
        ),
        derived_ttft_ms: metric_stats(
            observations
                .iter()
                .filter(successful)
                .filter_map(|item| item.derived_ttft_ms),
        ),
        failures,
    })
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletionTiming {
    /// Prompt tokens the runtime evaluated for this request (`prompt_n`).
    pub prompt_tokens: u32,
    /// Prompt tokens restored from the runtime prompt cache (`cache_n`,
    /// b10816 semantics). The requested prompt is complete when
    /// `prompt_tokens + cached_prompt_tokens` equals the requested size
    /// (audit MT-01).
    pub cached_prompt_tokens: u32,
    pub generated_tokens: u32,
    pub prefill_tps: f64,
    pub decode_tps: f64,
    pub first_token_ms: Option<f64>,
    pub derived_ttft_ms: f64,
    pub peak_process_rss_bytes: Evidence<u64>,
}

fn unknown_process_peak_rss_bytes() -> Evidence<u64> {
    Evidence::unknown(
        EvidenceSource {
            kind: EvidenceSourceKind::Runtime,
            detail: "process peak working set was not sampled".into(),
        },
        observed_at_ms().unwrap_or(0),
        "Process peak working-set evidence is unavailable for this attempt.",
    )
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkloadRun {
    pub warmups: Vec<WarmupObservation>,
    pub observations: Vec<BenchmarkObservation>,
    pub terminal_outcome: Option<AttemptOutcome>,
}

fn observed_at_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "System time is outside the supported range".into())
}

fn failed_outcome(error: &str) -> AttemptOutcome {
    let normalized = error.to_ascii_lowercase();
    if normalized.contains("cancelled") || normalized.contains("canceled") {
        AttemptOutcome::Cancelled
    } else if normalized.contains("timed out") || normalized.contains("timeout") {
        AttemptOutcome::TimedOut
    } else {
        AttemptOutcome::Failed
    }
}

pub fn parse_completion_timing(body: &str) -> Result<CompletionTiming, String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|error| error.to_string())?;
    let timings = value
        .get("timings")
        .ok_or("llama-server response did not include timings")?;
    let positive = |name: &str| {
        timings
            .get(name)
            .and_then(serde_json::Value::as_f64)
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| format!("llama-server timing {name} must be positive and finite"))
    };
    let tokens = |name: &str| {
        timings
            .get(name)
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| format!("llama-server timing {name} must be a positive token count"))
    };
    let prompt_ms = positive("prompt_ms")?;
    let predicted_per_token_ms = positive("predicted_per_token_ms")?;
    let first_token_ms = timings
        .get("first_token_ms")
        .and_then(serde_json::Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0);
    // `cache_n` is absent on runtimes without prompt caching: treat the
    // whole prompt as processed in that case, and reject a present value
    // that is not a token count.
    let cached_prompt_tokens = match timings.get("cache_n") {
        None | Some(serde_json::Value::Null) => 0,
        Some(value) => value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or("llama-server timing cache_n must be a non-negative token count")?,
    };
    Ok(CompletionTiming {
        prompt_tokens: tokens("prompt_n")?,
        cached_prompt_tokens,
        generated_tokens: tokens("predicted_n")?,
        prefill_tps: positive("prompt_per_second")?,
        decode_tps: positive("predicted_per_second")?,
        first_token_ms,
        derived_ttft_ms: prompt_ms + predicted_per_token_ms,
        peak_process_rss_bytes: unknown_process_peak_rss_bytes(),
    })
}

pub fn parse_completion_content(body: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|error| error.to_string())?;
    value
        .get("content")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "llama-server response did not include completion content".into())
}

pub fn run_workload_with<F>(
    workload: &Workload,
    cancelled: &AtomicBool,
    mut request: F,
) -> Result<WorkloadRun, String>
where
    F: FnMut() -> Result<CompletionTiming, String>,
{
    workload.validate().map_err(|error| error.to_string())?;
    if workload.concurrency != 1 {
        return Err("The v0.3 sequential harness currently supports concurrency 1 only".into());
    }
    if workload.cache_mode == CacheMode::Cold {
        return Err(
            "Cold-cache benchmarking requires a fresh runtime for each measured trial".into(),
        );
    }
    let mut run = WorkloadRun {
        warmups: Vec::with_capacity(usize::from(workload.warmups)),
        observations: Vec::with_capacity(usize::from(workload.trials)),
        terminal_outcome: None,
    };
    for warmup in 1..=workload.warmups {
        let started_at_ms = observed_at_ms()?;
        if cancelled.load(Ordering::Relaxed) {
            run.warmups.push(WarmupObservation {
                warmup,
                started_at_ms,
                outcome: AttemptOutcome::Cancelled,
                error: Some("Benchmark cancelled".into()),
                ..WarmupObservation::default()
            });
            run.terminal_outcome = Some(AttemptOutcome::Cancelled);
            return Ok(run);
        }
        let started = Instant::now();
        let result = request();
        let duration_ms = started.elapsed().as_secs_f64() * 1_000.0;
        match result {
            Ok(_) => run.warmups.push(WarmupObservation {
                warmup,
                started_at_ms,
                duration_ms,
                outcome: AttemptOutcome::Succeeded,
                error: None,
            }),
            Err(error) => {
                let outcome = failed_outcome(&error);
                run.warmups.push(WarmupObservation {
                    warmup,
                    started_at_ms,
                    duration_ms,
                    outcome,
                    error: Some(bound_observation_error(&error)),
                });
                run.terminal_outcome = Some(outcome);
                return Ok(run);
            }
        }
    }
    for trial in 1..=workload.trials {
        if cancelled.load(Ordering::Relaxed) {
            run.observations.push(BenchmarkObservation {
                trial,
                started_at_ms: observed_at_ms()?,
                outcome: AttemptOutcome::Cancelled,
                error: Some("Benchmark cancelled".into()),
                ..BenchmarkObservation::default()
            });
            run.terminal_outcome = Some(AttemptOutcome::Cancelled);
            break;
        }
        let started_at_ms = observed_at_ms()?;
        let started = Instant::now();
        let result = request();
        let duration_ms = started.elapsed().as_secs_f64() * 1_000.0;
        let observation = match result {
            Ok(timing) => BenchmarkObservation {
                trial,
                started_at_ms,
                duration_ms,
                prompt_tokens: timing.prompt_tokens,
                cached_prompt_tokens: timing.cached_prompt_tokens,
                generated_tokens: timing.generated_tokens,
                prefill_tps: Some(timing.prefill_tps),
                decode_tps: Some(timing.decode_tps),
                first_token_ms: timing.first_token_ms,
                derived_ttft_ms: Some(timing.derived_ttft_ms),
                peak_process_rss_bytes: timing.peak_process_rss_bytes,
                outcome: AttemptOutcome::Succeeded,
                error: None,
            },
            Err(error) => {
                let outcome = failed_outcome(&error);
                // Bounded at acquisition: outcome/category first, then the
                // visible truncation marker (audit MT-12).
                BenchmarkObservation {
                    trial,
                    started_at_ms,
                    duration_ms,
                    outcome,
                    error: Some(bound_observation_error(&error)),
                    ..BenchmarkObservation::default()
                }
            }
        };
        let terminal_outcome =
            (observation.outcome == AttemptOutcome::Cancelled).then_some(AttemptOutcome::Cancelled);
        run.observations.push(observation);
        if let Some(outcome) = terminal_outcome {
            run.terminal_outcome = Some(outcome);
            break;
        }
    }
    Ok(run)
}

pub fn run_cold_workload_with<F>(
    workload: &Workload,
    cancelled: &AtomicBool,
    request_with_fresh_runtime: F,
) -> Result<WorkloadRun, String>
where
    F: FnMut() -> Result<CompletionTiming, String>,
{
    workload.validate().map_err(|error| error.to_string())?;
    if workload.cache_mode != CacheMode::Cold {
        return Err("The cold-cache harness requires cacheMode cold".into());
    }
    let mut attempt_shape = workload.clone();
    attempt_shape.cache_mode = CacheMode::Warm;
    run_workload_with(&attempt_shape, cancelled, request_with_fresh_runtime)
}

fn post_json(
    host: &str,
    port: u16,
    path: &str,
    body: &str,
    timeout: Duration,
    cancelled: Option<&AtomicBool>,
) -> Result<String, String> {
    if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        return Err("Benchmark cancelled before the request".into());
    }
    if host.contains(['\r', '\n']) {
        return Err("Benchmark host contains an invalid header character".into());
    }
    if body.len() > MAX_HTTP_REQUEST_BYTES {
        return Err(format!(
            "Benchmark request exceeds the {MAX_HTTP_REQUEST_BYTES}-byte limit"
        ));
    }
    let connect_host = match host {
        "0.0.0.0" => "127.0.0.1",
        "::" | "[::]" => "::1",
        value => value,
    };
    let mut stream = TcpStream::connect((connect_host, port)).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_millis(100).min(timeout)))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| error.to_string())?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {connect_host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| error.to_string())?;

    let started = Instant::now();
    let mut response_bytes = Vec::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        if cancelled.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err("Benchmark cancelled while waiting for a response".into());
        }
        if started.elapsed() >= timeout {
            return Err("Benchmark request timed out".into());
        }
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                if response_bytes.len().saturating_add(read) > MAX_HTTP_RESPONSE_BYTES as usize {
                    return Err(format!(
                        "Benchmark response exceeds the {}-byte limit",
                        MAX_HTTP_RESPONSE_BYTES
                    ));
                }
                response_bytes.extend_from_slice(&buffer[..read]);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    let response = String::from_utf8(response_bytes)
        .map_err(|_| "Benchmark response was not valid UTF-8".to_string())?;
    let (headers, payload) = response
        .split_once("\r\n\r\n")
        .ok_or("Invalid HTTP response from llama-server")?;
    let status = headers.lines().next().unwrap_or_default();
    if !(status.starts_with("HTTP/1.1 200 ") || status.starts_with("HTTP/1.0 200 ")) {
        return Err(format!("Benchmark request failed: {status}"));
    }
    Ok(payload.into())
}

fn exact_prompt_tokens(
    host: &str,
    port: u16,
    target: u32,
    timeout: Duration,
    cancelled: Option<&AtomicBool>,
) -> Result<Vec<i32>, String> {
    let body = serde_json::json!({
        "content": BENCHMARK_PROMPT,
        "add_special": false,
        "parse_special": true,
        "with_pieces": false
    })
    .to_string();
    let payload = post_json(host, port, "/tokenize", &body, timeout, cancelled)?;
    let value: serde_json::Value =
        serde_json::from_str(&payload).map_err(|error| error.to_string())?;
    let source = value
        .get("tokens")
        .and_then(serde_json::Value::as_array)
        .ok_or("llama-server tokenize response did not include a token array")?
        .iter()
        .map(|token| {
            token
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or_else(|| "llama-server returned an invalid token ID".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if source.is_empty() {
        return Err("llama-server returned an empty benchmark prompt tokenization".into());
    }
    let target = usize::try_from(target).map_err(|_| "Prompt token target is unsupported")?;
    Ok(source.into_iter().cycle().take(target).collect())
}

fn completion_request_with_prompt_tokens_inner(
    host: &str,
    port: u16,
    workload: &Workload,
    prompt: &[i32],
    cancelled: Option<&AtomicBool>,
) -> Result<CompletionTiming, String> {
    workload.validate().map_err(|error| error.to_string())?;
    if workload.stream {
        return Err(
            "This benchmark harness does not claim direct TTFT for streamed responses".into(),
        );
    }
    if prompt.len() != workload.prompt_tokens as usize {
        return Err(format!(
            "Prepared prompt contains {} tokens; expected {}",
            prompt.len(),
            workload.prompt_tokens
        ));
    }
    let timeout = Duration::from_millis(workload.timeout_ms);
    let body = serde_json::json!({
        "prompt": prompt,
        "n_predict": workload.generation_tokens,
        "temperature": 0.0,
        "seed": workload.seed.unwrap_or(42),
        "ignore_eos": true,
        "stream": false,
        "cache_prompt": matches!(workload.cache_mode, crate::evidence::CacheMode::Warm)
    })
    .to_string();
    let payload = post_json(host, port, "/completion", &body, timeout, cancelled)?;
    let timing = parse_completion_timing(&payload)?;
    // b10816 reports newly evaluated prompt tokens separately from tokens
    // restored from the prompt cache; the requested prompt is fully accounted
    // for when both counts sum to the requested size. This keeps cached warm
    // trials valid without accepting a genuinely wrong workload (audit MT-01).
    let accounted_prompt_tokens = timing
        .prompt_tokens
        .checked_add(timing.cached_prompt_tokens)
        .ok_or("llama-server prompt token counts overflowed")?;
    if accounted_prompt_tokens != workload.prompt_tokens {
        return Err(format!(
            "llama-server evaluated {} prompt tokens and restored {} from cache; expected a total of {}",
            timing.prompt_tokens, timing.cached_prompt_tokens, workload.prompt_tokens
        ));
    }
    if timing.generated_tokens != workload.generation_tokens {
        return Err(format!(
            "llama-server generated {} tokens; expected {}",
            timing.generated_tokens, workload.generation_tokens
        ));
    }
    Ok(timing)
}

pub fn prepare_exact_prompt_tokens_cancellable(
    host: &str,
    port: u16,
    workload: &Workload,
    cancelled: &AtomicBool,
) -> Result<Vec<i32>, String> {
    workload.validate().map_err(|error| error.to_string())?;
    exact_prompt_tokens(
        host,
        port,
        workload.prompt_tokens,
        Duration::from_millis(workload.timeout_ms),
        Some(cancelled),
    )
}

pub fn completion_request_with_prompt_tokens_cancellable(
    host: &str,
    port: u16,
    workload: &Workload,
    prompt: &[i32],
    cancelled: &AtomicBool,
) -> Result<CompletionTiming, String> {
    completion_request_with_prompt_tokens_inner(host, port, workload, prompt, Some(cancelled))
}

fn completion_request_inner(
    host: &str,
    port: u16,
    workload: &Workload,
    cancelled: Option<&AtomicBool>,
) -> Result<CompletionTiming, String> {
    workload.validate().map_err(|error| error.to_string())?;
    let prompt = exact_prompt_tokens(
        host,
        port,
        workload.prompt_tokens,
        Duration::from_millis(workload.timeout_ms),
        cancelled,
    )?;
    completion_request_with_prompt_tokens_inner(host, port, workload, &prompt, cancelled)
}

pub fn completion_request(
    host: &str,
    port: u16,
    workload: &Workload,
) -> Result<CompletionTiming, String> {
    completion_request_inner(host, port, workload, None)
}

pub fn completion_request_cancellable(
    host: &str,
    port: u16,
    workload: &Workload,
    cancelled: &AtomicBool,
) -> Result<CompletionTiming, String> {
    completion_request_inner(host, port, workload, Some(cancelled))
}

pub fn quality_completion_request(host: &str, port: u16, prompt: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "prompt": prompt,
        "n_predict": 64,
        "temperature": 0.0,
        "seed": 42,
        "ignore_eos": false,
        "stream": false,
        "cache_prompt": false
    })
    .to_string();
    let payload = post_json(
        host,
        port,
        "/completion",
        &body,
        Duration::from_secs(120),
        None,
    )?;
    parse_completion_content(&payload)
}

pub fn persist_manifest(directory: &Path, manifest: &BenchmarkManifest) -> Result<PathBuf, String> {
    manifest
        .validate_complete()
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let workload_id = manifest
        .workload
        .id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(64)
        .collect::<String>();
    let observed_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    let nonce = MANIFEST_NONCE.fetch_add(1, Ordering::Relaxed);
    let file_name = format!("benchmark-{observed_at_ms}-{workload_id}-{nonce}.json");
    let target = directory.join(&file_name);
    let temporary = directory.join(format!(".{file_name}.tmp"));
    let bytes = serde_json::to_vec_pretty(manifest).map_err(|error| error.to_string())?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    drop(file);
    if let Err(error) = fs::rename(&temporary, &target) {
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    Ok(target)
}

pub fn validate_replay_compatibility(
    manifest: &BenchmarkManifest,
    logical_model_id: &str,
    command_args: &[String],
    current_compatibility_key: &str,
) -> Result<(), String> {
    let model = manifest
        .model
        .as_ref()
        .ok_or("Replay manifest does not contain a model identity")?;
    if model.logical_id != logical_model_id {
        return Err("Replay model identity does not match the running server".into());
    }
    let launch = manifest
        .launch
        .as_ref()
        .ok_or("Replay manifest does not contain a launch snapshot")?;
    if launch.command_args != command_args {
        return Err("Replay launch arguments do not match the running server".into());
    }
    let recorded_compatibility_key = manifest
        .compatibility_key
        .as_deref()
        .ok_or("Replay manifest does not contain a compatibility identity")?;
    crate::evidence::validate_sha256("compatibilityKey", recorded_compatibility_key)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("currentCompatibilityKey", current_compatibility_key)
        .map_err(|error| error.to_string())?;
    if recorded_compatibility_key != current_compatibility_key {
        return Err("Replay compatibility identity does not match the running server".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{LaunchFact, ModelFact, RuntimeFact};
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;
    use std::thread;

    fn read_http_request(stream: &mut TcpStream) -> String {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 4_096];
        loop {
            let read = stream.read(&mut buffer).unwrap();
            assert!(read > 0, "request ended before its declared body");
            request.extend_from_slice(&buffer[..read]);
            let Some(headers_end) = request.windows(4).position(|part| part == b"\r\n\r\n") else {
                continue;
            };
            let headers_end = headers_end + 4;
            let headers = String::from_utf8_lossy(&request[..headers_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    line.strip_prefix("Content-Length: ")
                        .and_then(|value| value.parse::<usize>().ok())
                })
                .unwrap_or(0);
            if request.len() >= headers_end + content_length {
                return String::from_utf8(request).unwrap();
            }
        }
    }

    fn observed_context(value: u32) -> crate::evidence::Evidence<u32> {
        crate::evidence::Evidence::known(
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

    fn observation(
        trial: u16,
        decode_tps: Option<f64>,
        error: Option<&str>,
    ) -> BenchmarkObservation {
        BenchmarkObservation {
            trial,
            started_at_ms: 42 + u64::from(trial),
            duration_ms: 100.0,
            prompt_tokens: 32,
            cached_prompt_tokens: 0,
            generated_tokens: 16,
            prefill_tps: decode_tps.map(|value| value * 2.0),
            decode_tps,
            first_token_ms: None,
            derived_ttft_ms: decode_tps.map(|value| 1_000.0 / value),
            peak_process_rss_bytes: unknown_process_peak_rss_bytes(),
            outcome: if error.is_some() {
                AttemptOutcome::Failed
            } else {
                AttemptOutcome::Succeeded
            },
            error: error.map(str::to_string),
        }
    }

    #[test]
    fn summary_keeps_failed_trials_without_discarding_valid_statistics() {
        let observations = vec![
            observation(1, Some(10.0), None),
            observation(2, Some(20.0), None),
            observation(3, None, Some("timeout")),
            observation(4, Some(30.0), None),
        ];

        let summary = summarize_observations(&observations).unwrap();

        assert_eq!(summary.result_class, FitClass::Measured);
        assert_eq!(summary.successful_trials, 3);
        assert_eq!(summary.failed_trials, 1);
        assert_eq!(summary.decode_tps.p50, 20.0);
        assert_eq!(summary.decode_tps.p95, 30.0);
        assert_eq!(summary.failures, vec!["trial 3: timeout"]);
    }

    #[test]
    fn metric_summary_reports_median_separately_from_nearest_rank_p50() {
        let stats = metric_stats([10.0, 20.0, 30.0, 40.0].into_iter()).unwrap();

        assert_eq!(stats.median, 25.0);
        assert_eq!(stats.p50, 20.0);
    }

    #[test]
    fn metric_summary_remains_finite_for_large_finite_observations() {
        let stats = metric_stats([f64::MAX / 2.0, f64::MAX].into_iter()).unwrap();

        assert!(stats.mean.is_finite());
        assert!(stats.median.is_finite());
        assert!(stats.standard_deviation.is_finite());
    }

    #[test]
    fn summary_rejects_non_finite_raw_observations() {
        let mut invalid = observation(1, Some(10.0), None);
        invalid.decode_tps = Some(f64::NAN);
        let observations = vec![invalid];

        let error = summarize_observations(&observations).unwrap_err();

        assert!(error.contains("decodeTps"));
        assert!(error.contains("finite"));
    }

    #[test]
    fn summary_rejects_an_outcome_that_conflicts_with_raw_error_evidence() {
        let mut invalid = observation(1, Some(10.0), None);
        invalid.outcome = AttemptOutcome::Failed;

        let error = summarize_observations(&[invalid]).unwrap_err();

        assert!(error.contains("outcome"));
    }

    #[test]
    fn completion_timing_keeps_prefill_decode_and_derived_ttft_distinct() {
        let timing = parse_completion_timing(
            r#"{"timings":{"prompt_n":32,"prompt_ms":80.0,"prompt_per_second":400.0,"predicted_n":16,"predicted_ms":200.0,"predicted_per_second":80.0,"predicted_per_token_ms":12.5}}"#,
        )
        .unwrap();

        assert_eq!(timing.prompt_tokens, 32);
        assert_eq!(timing.generated_tokens, 16);
        assert_eq!(timing.prefill_tps, 400.0);
        assert_eq!(timing.decode_tps, 80.0);
        assert_eq!(timing.derived_ttft_ms, 92.5);
        assert!(timing.first_token_ms.is_none());
    }

    #[test]
    fn completion_content_parser_reads_llama_server_text() {
        assert_eq!(
            parse_completion_content(r#"{"content":"READY","stop":true}"#).unwrap(),
            "READY"
        );
    }

    #[test]
    fn workload_runner_executes_warmups_and_preserves_trial_failures() {
        let workload = Workload {
            warmups: 1,
            trials: 2,
            ..Workload::default()
        };
        let cancelled = AtomicBool::new(false);
        let mut calls = 0;

        let run = run_workload_with(&workload, &cancelled, || {
            calls += 1;
            if calls == 2 {
                Err("controlled failure".into())
            } else {
                Ok(CompletionTiming {
                    prompt_tokens: 32,
                    cached_prompt_tokens: 0,
                    generated_tokens: 16,
                    prefill_tps: 100.0,
                    decode_tps: 50.0,
                    first_token_ms: None,
                    derived_ttft_ms: 25.0,
                    peak_process_rss_bytes: unknown_process_peak_rss_bytes(),
                })
            }
        })
        .unwrap();

        assert_eq!(calls, 3);
        assert_eq!(run.warmups.len(), 1);
        assert_eq!(run.warmups[0].outcome, AttemptOutcome::Succeeded);
        assert_eq!(run.observations.len(), 2);
        assert_eq!(
            run.observations[0].error.as_deref(),
            Some("controlled failure")
        );
        assert_eq!(run.observations[1].decode_tps, Some(50.0));
        assert!(run.terminal_outcome.is_none());
    }

    #[test]
    fn workload_runner_preserves_process_memory_source_evidence() {
        let workload = Workload {
            warmups: 0,
            trials: 1,
            ..Workload::default()
        };
        let process_memory = crate::evidence::Evidence::known(
            123_456_u64,
            crate::evidence::EvidenceLevel::Observed,
            crate::evidence::EvidenceSource {
                kind: crate::evidence::EvidenceSourceKind::WindowsApi,
                detail: "GetProcessMemoryInfo(PeakWorkingSetSize)".into(),
            },
            42,
            Vec::new(),
        )
        .unwrap();

        let run = run_workload_with(&workload, &AtomicBool::new(false), || {
            Ok(CompletionTiming {
                prompt_tokens: workload.prompt_tokens,
                cached_prompt_tokens: 0,
                generated_tokens: workload.generation_tokens,
                prefill_tps: 100.0,
                decode_tps: 50.0,
                first_token_ms: None,
                derived_ttft_ms: 25.0,
                peak_process_rss_bytes: process_memory.clone(),
            })
        })
        .unwrap();

        assert_eq!(
            run.observations[0].peak_process_rss_bytes.value,
            Some(123_456)
        );
        assert_eq!(
            run.observations[0].peak_process_rss_bytes.source.detail,
            "GetProcessMemoryInfo(PeakWorkingSetSize)"
        );
    }

    #[test]
    fn workload_runner_preserves_warmup_failure_as_a_terminal_outcome() {
        let workload = Workload {
            warmups: 1,
            trials: 2,
            ..Workload::default()
        };

        let run = run_workload_with(&workload, &AtomicBool::new(false), || {
            Err("request timed out".into())
        })
        .unwrap();

        assert_eq!(run.warmups.len(), 1);
        assert_eq!(run.warmups[0].outcome, AttemptOutcome::TimedOut);
        assert!(run.observations.is_empty());
        assert_eq!(run.terminal_outcome, Some(AttemptOutcome::TimedOut));
    }

    #[test]
    fn workload_runner_records_cancellation_before_the_next_trial() {
        let cancelled = AtomicBool::new(true);
        let workload = Workload {
            warmups: 0,
            trials: 2,
            ..Workload::default()
        };

        let run = run_workload_with(&workload, &cancelled, || {
            unreachable!("cancellation must stop before a request")
        })
        .unwrap();

        assert_eq!(run.observations.len(), 1);
        assert_eq!(run.observations[0].outcome, AttemptOutcome::Cancelled);
        assert_eq!(run.terminal_outcome, Some(AttemptOutcome::Cancelled));
    }

    #[test]
    fn in_flight_cancellation_stops_the_current_trial_once() {
        let cancelled = AtomicBool::new(false);
        let workload = Workload {
            warmups: 0,
            trials: 3,
            ..Workload::default()
        };

        let run = run_workload_with(&workload, &cancelled, || {
            cancelled.store(true, Ordering::Relaxed);
            Err("Benchmark cancelled while waiting for a response".into())
        })
        .unwrap();

        assert_eq!(run.observations.len(), 1);
        assert_eq!(run.observations[0].outcome, AttemptOutcome::Cancelled);
        assert_eq!(run.terminal_outcome, Some(AttemptOutcome::Cancelled));
    }

    #[test]
    fn sequential_harness_rejects_unimplemented_concurrency() {
        let workload = Workload {
            concurrency: 2,
            ..Workload::default()
        };

        let error = run_workload_with(&workload, &AtomicBool::new(false), || {
            unreachable!("unsupported concurrency must fail before a request")
        })
        .unwrap_err();

        assert!(error.contains("concurrency 1"));
    }

    #[test]
    fn sequential_harness_rejects_cold_cache_without_a_fresh_runtime() {
        let workload = Workload {
            cache_mode: CacheMode::Cold,
            ..Workload::default()
        };

        let error = run_workload_with(&workload, &AtomicBool::new(false), || {
            unreachable!("cold-cache mode must fail before a request")
        })
        .unwrap_err();

        assert!(error.contains("fresh runtime"));
    }

    #[test]
    fn cold_harness_executes_one_fresh_runtime_callback_per_attempt() {
        let workload = Workload {
            cache_mode: CacheMode::Cold,
            warmups: 1,
            trials: 2,
            ..Workload::default()
        };
        let cancelled = AtomicBool::new(false);
        let mut fresh_runtime_starts = 0;

        let run = run_cold_workload_with(&workload, &cancelled, || {
            fresh_runtime_starts += 1;
            Ok(CompletionTiming {
                prompt_tokens: workload.prompt_tokens,
                cached_prompt_tokens: 0,
                generated_tokens: workload.generation_tokens,
                prefill_tps: 100.0,
                decode_tps: 50.0,
                first_token_ms: None,
                derived_ttft_ms: 25.0,
                peak_process_rss_bytes: unknown_process_peak_rss_bytes(),
            })
        })
        .unwrap();

        assert_eq!(fresh_runtime_starts, 3);
        assert_eq!(run.warmups.len(), 1);
        assert_eq!(run.observations.len(), 2);
    }

    #[test]
    fn completion_request_uses_the_fixed_workload_protocol() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut tokenize_stream, _) = listener.accept().unwrap();
            let request = read_http_request(&mut tokenize_stream);
            assert!(request.starts_with("POST /tokenize HTTP/1.1"));
            let tokens = r#"{"tokens":[10]}"#;
            write!(
                tokenize_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                tokens.len(),
                tokens
            )
            .unwrap();
            drop(tokenize_stream);

            let (mut completion_stream, _) = listener.accept().unwrap();
            let request = read_http_request(&mut completion_stream);
            assert!(request.contains("POST /completion"));
            assert!(request.contains("\"seed\":42"));
            assert!(request.contains("\"temperature\":0.0"));
            assert!(request.contains("\"stream\":false"));
            let body = r#"{"timings":{"prompt_n":512,"prompt_ms":80.0,"prompt_per_second":6400.0,"predicted_n":16,"predicted_ms":200.0,"predicted_per_second":80.0,"predicted_per_token_ms":12.5}}"#;
            write!(
                completion_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        let workload = Workload {
            generation_tokens: 16,
            ..Workload::default()
        };

        let timing = completion_request("127.0.0.1", port, &workload).unwrap();

        assert_eq!(timing.decode_tps, 80.0);
        server.join().unwrap();
    }

    #[test]
    fn completion_request_sends_the_exact_prompt_token_target() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut tokenize_stream, _) = listener.accept().unwrap();
            let tokenize_request = read_http_request(&mut tokenize_stream);
            assert!(tokenize_request.starts_with("POST /tokenize HTTP/1.1"));
            let tokenize_body = r#"{"tokens":[10,20,30]}"#;
            write!(
                tokenize_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                tokenize_body.len(),
                tokenize_body
            )
            .unwrap();
            drop(tokenize_stream);

            let (mut completion_stream, _) = listener.accept().unwrap();
            let completion_request = read_http_request(&mut completion_stream);
            assert!(completion_request.starts_with("POST /completion HTTP/1.1"));
            let (_, body) = completion_request.split_once("\r\n\r\n").unwrap();
            let request: serde_json::Value = serde_json::from_str(body).unwrap();
            assert_eq!(request["prompt"], serde_json::json!([10, 20, 30, 10, 20]));
            let completion_body = r#"{"timings":{"prompt_n":5,"prompt_ms":80.0,"prompt_per_second":62.5,"predicted_n":2,"predicted_ms":40.0,"predicted_per_second":50.0,"predicted_per_token_ms":20.0}}"#;
            write!(
                completion_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                completion_body.len(),
                completion_body
            )
            .unwrap();
        });
        let workload = Workload {
            prompt_tokens: 5,
            generation_tokens: 2,
            ..Workload::default()
        };

        let timing = completion_request("127.0.0.1", port, &workload).unwrap();

        assert_eq!(timing.prompt_tokens, 5);
        server.join().unwrap();
    }

    #[test]
    fn completion_request_rejects_a_prompt_token_count_mismatch() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut tokenize_stream, _) = listener.accept().unwrap();
            let _ = read_http_request(&mut tokenize_stream);
            let tokenize_body = r#"{"tokens":[10]}"#;
            write!(
                tokenize_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                tokenize_body.len(),
                tokenize_body
            )
            .unwrap();
            drop(tokenize_stream);

            let (mut completion_stream, _) = listener.accept().unwrap();
            let _ = read_http_request(&mut completion_stream);
            let body = r#"{"timings":{"prompt_n":4,"prompt_ms":80.0,"prompt_per_second":50.0,"predicted_n":2,"predicted_ms":40.0,"predicted_per_second":50.0,"predicted_per_token_ms":20.0}}"#;
            write!(
                completion_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        let workload = Workload {
            prompt_tokens: 5,
            generation_tokens: 2,
            ..Workload::default()
        };

        let error = completion_request("127.0.0.1", port, &workload).unwrap_err();

        assert!(
            error.contains(
                "evaluated 4 prompt tokens and restored 0 from cache; expected a total of 5"
            ),
            "{error}"
        );
        server.join().unwrap();
    }

    #[test]
    fn completion_request_rejects_a_generation_token_count_mismatch() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut tokenize_stream, _) = listener.accept().unwrap();
            let _ = read_http_request(&mut tokenize_stream);
            let tokenize_body = r#"{"tokens":[10]}"#;
            write!(
                tokenize_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                tokenize_body.len(),
                tokenize_body
            )
            .unwrap();
            drop(tokenize_stream);

            let (mut completion_stream, _) = listener.accept().unwrap();
            let _ = read_http_request(&mut completion_stream);
            let body = r#"{"timings":{"prompt_n":5,"prompt_ms":80.0,"prompt_per_second":62.5,"predicted_n":1,"predicted_ms":20.0,"predicted_per_second":50.0,"predicted_per_token_ms":20.0}}"#;
            write!(
                completion_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        let workload = Workload {
            prompt_tokens: 5,
            generation_tokens: 2,
            ..Workload::default()
        };

        let error = completion_request("127.0.0.1", port, &workload).unwrap_err();

        assert!(error.contains("generated 1 tokens; expected 2"));
        server.join().unwrap();
    }

    #[test]
    fn prepared_prompt_tokens_are_reused_across_measured_trials() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut tokenize_stream, _) = listener.accept().unwrap();
            assert!(read_http_request(&mut tokenize_stream).starts_with("POST /tokenize HTTP/1.1"));
            let tokenize_body = r#"{"tokens":[10,20]}"#;
            write!(
                tokenize_stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                tokenize_body.len(),
                tokenize_body
            )
            .unwrap();
            drop(tokenize_stream);

            for _ in 0..2 {
                let (mut completion_stream, _) = listener.accept().unwrap();
                let request = read_http_request(&mut completion_stream);
                assert!(request.starts_with("POST /completion HTTP/1.1"));
                let completion_body = r#"{"timings":{"prompt_n":3,"prompt_ms":30.0,"prompt_per_second":100.0,"predicted_n":2,"predicted_ms":40.0,"predicted_per_second":50.0,"predicted_per_token_ms":20.0}}"#;
                write!(
                    completion_stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    completion_body.len(),
                    completion_body
                )
                .unwrap();
            }
        });
        let workload = Workload {
            prompt_tokens: 3,
            generation_tokens: 2,
            warmups: 0,
            trials: 2,
            ..Workload::default()
        };
        let cancelled = AtomicBool::new(false);
        let prompt =
            prepare_exact_prompt_tokens_cancellable("127.0.0.1", port, &workload, &cancelled)
                .unwrap();

        let run = run_workload_with(&workload, &cancelled, || {
            completion_request_with_prompt_tokens_cancellable(
                "127.0.0.1",
                port,
                &workload,
                &prompt,
                &cancelled,
            )
        })
        .unwrap();

        assert_eq!(run.observations.len(), 2);
        server.join().unwrap();
    }

    #[test]
    fn cancellation_interrupts_an_in_flight_response_wait() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            std::thread::sleep(Duration::from_secs(2));
        });
        let cancelled = Arc::new(AtomicBool::new(false));
        let trigger = cancelled.clone();
        let cancel_thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            trigger.store(true, Ordering::Relaxed);
        });
        let workload = Workload {
            timeout_ms: 5_000,
            ..Workload::default()
        };
        let started = Instant::now();

        let error =
            completion_request_cancellable("127.0.0.1", port, &workload, &cancelled).unwrap_err();

        assert!(error.contains("cancelled"));
        assert!(started.elapsed() < Duration::from_secs(1));
        cancel_thread.join().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn manifest_persistence_is_atomic_and_round_trips_raw_observations() {
        let directory = std::env::temp_dir().join(format!(
            "localmotive-manifest-persist-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        let manifest = BenchmarkManifest {
            compatibility_key: Some("c".repeat(64)),
            runtime: Some(RuntimeFact {
                path: "runtime.exe".into(),
                version: "1".into(),
                build: "1".into(),
                executable_sha256: Some("a".repeat(64)),
                help_sha256: "b".repeat(64),
                backend: "cpu".into(),
            }),
            model: Some(ModelFact {
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
            launch: Some(LaunchFact {
                requested_context: 4_096,
                effective_context: observed_context(4_096),
                parallel: 1,
                gpu_layers: "0".into(),
                batch: 512,
                ubatch: 128,
                cache_type_k: "F16".into(),
                cache_type_v: "F16".into(),
                split_mode: "none".into(),
                ..LaunchFact::default()
            }),
            observations: vec![observation(1, Some(50.0), None)],
            ..BenchmarkManifest::default()
        };

        let path = persist_manifest(&directory, &manifest).unwrap();
        let loaded: BenchmarkManifest = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();

        assert_eq!(loaded, manifest);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn replay_rejects_a_changed_launch_command() {
        let manifest = BenchmarkManifest {
            compatibility_key: Some("a".repeat(64)),
            model: Some(ModelFact {
                logical_id: "model-a".into(),
                ..ModelFact::default()
            }),
            launch: Some(LaunchFact {
                command_args: vec!["--ctx-size".into(), "4096".into()],
                effective_context: observed_context(4_096),
                batch: 512,
                ubatch: 128,
                ..LaunchFact::default()
            }),
            ..BenchmarkManifest::default()
        };

        let error = validate_replay_compatibility(
            &manifest,
            "model-a",
            &["--ctx-size".into(), "8192".into()],
            &"a".repeat(64),
        )
        .unwrap_err();

        assert!(error.contains("launch arguments"));
    }

    #[test]
    fn replay_rejects_a_changed_compatibility_identity() {
        let manifest = BenchmarkManifest {
            compatibility_key: Some("a".repeat(64)),
            model: Some(ModelFact {
                logical_id: "model-a".into(),
                ..ModelFact::default()
            }),
            launch: Some(LaunchFact {
                command_args: vec!["--ctx-size".into(), "4096".into()],
                effective_context: observed_context(4_096),
                batch: 512,
                ubatch: 128,
                ..LaunchFact::default()
            }),
            ..BenchmarkManifest::default()
        };

        let error = validate_replay_compatibility(
            &manifest,
            "model-a",
            &["--ctx-size".into(), "4096".into()],
            &"b".repeat(64),
        )
        .unwrap_err();

        assert!(error.contains("compatibility identity"));
    }

    /// Serve `attempts` benchmark attempts: for each, one tokenize response
    /// and one completion response whose body comes from `bodies[index]`.
    fn serve_cache_protocol(bodies: Vec<String>) -> (u16, std::thread::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            let mut seen = Vec::new();
            for body in &bodies {
                let (mut tokenize_stream, _) = listener.accept().unwrap();
                seen.push(read_http_request(&mut tokenize_stream));
                let tokenize_body = format!(r#"{{"tokens":[{}]}}"#, ["10"; 8].join(","));
                write!(
                    tokenize_stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    tokenize_body.len(),
                    tokenize_body
                )
                .unwrap();
                drop(tokenize_stream);

                let (mut completion_stream, _) = listener.accept().unwrap();
                seen.push(read_http_request(&mut completion_stream));
                write!(
                    completion_stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .unwrap();
                drop(completion_stream);
            }
            seen
        });
        (port, handle)
    }

    fn cached_timing_body(prompt_n: u32, cache_n: u32, predicted_n: u32) -> String {
        format!(
            r#"{{"timings":{{"prompt_n":{prompt_n},"cache_n":{cache_n},"prompt_ms":80.0,"prompt_per_second":50.0,"predicted_n":{predicted_n},"predicted_ms":40.0,"predicted_per_second":50.0,"predicted_per_token_ms":20.0}}}}"#
        )
    }

    #[test]
    fn mt01_warm_cache_reuse_is_counted_against_processed_plus_cached_tokens() {
        // b10816: the first request processes the whole prompt, later requests
        // restore it from cache. The default warm protocol must accept the
        // cached hit (audit MT-01 V1).
        let bodies = vec![
            cached_timing_body(512, 0, 256),
            cached_timing_body(1, 511, 256),
            cached_timing_body(1, 511, 256),
        ];
        let (port, server) = serve_cache_protocol(bodies);
        let workload = Workload {
            prompt_tokens: 512,
            generation_tokens: 256,
            warmups: 1,
            trials: 2,
            ..Workload::default()
        };
        let cancelled = AtomicBool::new(false);
        let run = run_workload_with(&workload, &cancelled, || {
            completion_request("127.0.0.1", port, &workload)
        })
        .unwrap();
        let requests = server.join().unwrap();

        assert_eq!(run.warmups.len(), 1);
        assert_eq!(run.warmups[0].outcome, AttemptOutcome::Succeeded);
        assert_eq!(run.observations.len(), 2);
        for observation in &run.observations {
            assert_eq!(
                observation.outcome,
                AttemptOutcome::Succeeded,
                "{observation:?}"
            );
            assert_eq!(observation.prompt_tokens, 1);
            assert_eq!(observation.cached_prompt_tokens, 511);
            assert_eq!(observation.generated_tokens, 256);
        }
        // The protocol is genuinely warm: later requests ask for prompt reuse.
        assert!(
            requests
                .iter()
                .filter(|request| request.contains("cache_prompt"))
                .count()
                >= 2,
            "warm requests must be explicit about cache policy"
        );
        let summary = summarize_observations(&run.observations).unwrap();
        assert_eq!(summary.successful_trials, 2);
    }

    #[test]
    fn mt01_inconsistent_or_missing_cache_accounting_is_rejected() {
        // (a) A runtime without cache_n still works: absent means 0 cached.
        let bodies = vec![
            r#"{"timings":{"prompt_n":512,"prompt_ms":80.0,"prompt_per_second":50.0,"predicted_n":256,"predicted_ms":40.0,"predicted_per_second":50.0,"predicted_per_token_ms":20.0}}"#.to_string(),
        ];
        let (port, server) = serve_cache_protocol(bodies);
        let workload = Workload {
            prompt_tokens: 512,
            generation_tokens: 256,
            warmups: 0,
            trials: 1,
            ..Workload::default()
        };
        let cancelled = AtomicBool::new(false);
        let run = run_workload_with(&workload, &cancelled, || {
            completion_request("127.0.0.1", port, &workload)
        })
        .unwrap();
        server.join().unwrap();
        assert_eq!(run.observations[0].outcome, AttemptOutcome::Succeeded);
        assert_eq!(run.observations[0].cached_prompt_tokens, 0);

        // (b) Processed + cached that miss the requested total are wrong.
        let bodies = vec![cached_timing_body(1, 100, 256)];
        let (port, server) = serve_cache_protocol(bodies);
        let error = run_workload_with(&workload, &cancelled, || {
            completion_request("127.0.0.1", port, &workload)
        })
        .unwrap();
        server.join().unwrap();
        assert_eq!(error.observations[0].outcome, AttemptOutcome::Failed);
        assert!(
            error.observations[0]
                .error
                .as_deref()
                .unwrap()
                .contains("expected a total of 512"),
            "{:?}",
            error.observations[0].error
        );

        // (c) Cache accounting cannot excuse a short generation.
        let bodies = vec![cached_timing_body(1, 511, 100)];
        let (port, server) = serve_cache_protocol(bodies);
        let short = run_workload_with(&workload, &cancelled, || {
            completion_request("127.0.0.1", port, &workload)
        })
        .unwrap();
        server.join().unwrap();
        assert_eq!(short.observations[0].outcome, AttemptOutcome::Failed);
        assert!(
            short.observations[0]
                .error
                .as_deref()
                .unwrap()
                .contains("generated 100 tokens"),
            "{:?}",
            short.observations[0].error
        );

        // (d) A non-numeric cache_n is a protocol error, not silently zero.
        let bodies = vec![r#"{"timings":{"prompt_n":512,"cache_n":"lots","prompt_ms":80.0,"prompt_per_second":50.0,"predicted_n":256,"predicted_ms":40.0,"predicted_per_second":50.0,"predicted_per_token_ms":20.0}}"#.to_string()];
        let (port, server) = serve_cache_protocol(bodies);
        let malformed = run_workload_with(&workload, &cancelled, || {
            completion_request("127.0.0.1", port, &workload)
        })
        .unwrap();
        server.join().unwrap();
        assert_eq!(malformed.observations[0].outcome, AttemptOutcome::Failed);
        assert!(
            malformed.observations[0]
                .error
                .as_deref()
                .unwrap()
                .contains("cache_n"),
            "{:?}",
            malformed.observations[0].error
        );
    }

    #[test]
    fn mt12_oversize_failure_text_is_bounded_at_acquisition_with_a_visible_marker() {
        // A rich cold-start failure (structured evidence plus a runtime log
        // tail) must not turn into a manifest-validation rejection that
        // discards the run (audit MT-12). The bound happens at acquisition.
        let huge = format!(
            "cold start failed: {}é{}",
            "x".repeat(5_000),
            "log tail with multibyte é characters; ".repeat(80)
        );
        assert!(huge.len() > MAX_OBSERVATION_ERROR_BYTES);
        let workload = Workload {
            warmups: 0,
            trials: 1,
            ..Workload::default()
        };
        let cancelled = AtomicBool::new(false);
        let run = run_workload_with(&workload, &cancelled, || Err(huge.clone())).unwrap();

        let stored = run.observations[0].error.as_deref().unwrap();
        assert!(
            stored.len() <= MAX_OBSERVATION_ERROR_BYTES,
            "stored error is {} bytes",
            stored.len()
        );
        assert!(stored.contains("[message truncated"), "{stored}");
        assert!(stored.starts_with("cold start failed:"), "{stored}");
        assert_eq!(run.observations[0].outcome, AttemptOutcome::Failed);
        assert_eq!(
            run.terminal_outcome, None,
            "a failed trial is recorded, not terminal"
        );

        // Warmup failures take the same bound and do end the run.
        let workload = Workload {
            warmups: 1,
            trials: 1,
            ..Workload::default()
        };
        let run = run_workload_with(&workload, &cancelled, || Err(huge.clone())).unwrap();
        let stored = run.warmups[0].error.as_deref().unwrap();
        assert!(stored.len() <= MAX_OBSERVATION_ERROR_BYTES);
        assert!(stored.contains("[message truncated"));
        assert_eq!(run.terminal_outcome, Some(AttemptOutcome::Failed));

        // Boundary: a message of exactly the limit is untouched.
        let exact = "y".repeat(MAX_OBSERVATION_ERROR_BYTES);
        assert_eq!(bound_observation_error(&exact), exact);
        // One byte over: bounded with the marker, still valid UTF-8.
        let over = format!("{exact}é");
        let bounded = bound_observation_error(&over);
        assert!(bounded.len() <= MAX_OBSERVATION_ERROR_BYTES);
        assert!(bounded.contains("[message truncated"));
    }

    #[test]
    fn mt12_successful_observations_survive_a_later_oversize_failure() {
        // Two good trials plus one oversized failure: the summary and the
        // earlier measurements must survive finalization (audit MT-12 V2).
        let huge = "z".repeat(9_000);
        let attempts = AtomicBool::new(false);
        let workload = Workload {
            warmups: 0,
            trials: 3,
            ..Workload::default()
        };
        let cancelled = AtomicBool::new(false);
        let mut index = 0;
        let run = run_workload_with(&workload, &cancelled, || {
            index += 1;
            if index == 2 {
                Err(huge.clone())
            } else {
                Ok(CompletionTiming {
                    prompt_tokens: workload.prompt_tokens,
                    cached_prompt_tokens: 0,
                    generated_tokens: workload.generation_tokens,
                    prefill_tps: 100.0,
                    decode_tps: 50.0,
                    first_token_ms: None,
                    derived_ttft_ms: 25.0,
                    peak_process_rss_bytes: unknown_process_peak_rss_bytes(),
                })
            }
        })
        .unwrap();
        let _ = &attempts;

        assert_eq!(run.observations.len(), 3);
        assert_eq!(run.observations[0].outcome, AttemptOutcome::Succeeded);
        assert_eq!(run.observations[2].outcome, AttemptOutcome::Succeeded);
        let failure = run.observations[1].error.as_deref().unwrap();
        assert!(failure.len() <= MAX_OBSERVATION_ERROR_BYTES);
        let summary = summarize_observations(&run.observations).unwrap();
        assert_eq!(summary.successful_trials, 2);
        assert_eq!(summary.failed_trials, 1);
    }
}
