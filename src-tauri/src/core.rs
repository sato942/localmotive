use crate::artifact::{is_reparse_point, parse_shard_name, ArtifactFileFact};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Companion {
    pub path: String,
    pub name: String,
    pub role: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicalModel {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub first_shard: String,
    pub size_bytes: u64,
    pub shard_count: usize,
    pub expected_shards: usize,
    pub complete: bool,
    pub quant: String,
    pub shards: Vec<ArtifactFileFact>,
    pub companions: Vec<Companion>,
}

/// Suggest a TCP port from a bounded scan of the current socket state.
/// The child bind and health check remain authoritative because this probe releases its socket.
pub fn pick_free_port(host: &str, preferred: u16) -> Result<u16, String> {
    let bind_host = if host == "0.0.0.0" { "127.0.0.1" } else { host };
    let start = preferred.max(1);
    for candidate in start..start.saturating_add(200) {
        if TcpListener::bind((bind_host, candidate)).is_ok() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "Could not find a free port in the bounded range {start}..{}",
        start.saturating_add(199)
    ))
}

/// Probe whether a configured TCP endpoint is available immediately before launch.
/// The post-health listener-owner check remains authoritative because this probe releases its socket.
pub fn probe_port_available(host: &str, port: u16) -> Result<(), String> {
    let listener = TcpListener::bind((host, port))
        .map_err(|error| format!("TCP port {host}:{port} is not available: {error}"))?;
    drop(listener);
    Ok(())
}

/// Rough bits-per-weight for a GGUF quantisation label, used only to order
/// companion candidates by similarity to the target. Unknown labels sort last.
fn quant_weight(label: &str) -> Option<f32> {
    let upper = label.to_ascii_uppercase();
    let table = [
        ("F32", 32.0),
        ("BF16", 16.0),
        ("F16", 16.0),
        ("Q8_0", 8.5),
        ("Q6_K", 6.6),
        ("Q5_K", 5.5),
        ("Q5_0", 5.5),
        ("Q5_1", 5.9),
        ("IQ4_XS", 4.25),
        ("Q4_K", 4.8),
        ("Q4_0", 4.5),
        ("Q4_1", 4.9),
        ("IQ4_NL", 4.5),
        ("IQ3", 3.4),
        ("Q3_K", 3.9),
        ("IQ2", 2.4),
        ("Q2_K", 3.0),
    ];
    // The label that describes the file is the one that appears first; a later
    // "-from-BF16" records provenance, not the file's own quantisation.
    table
        .iter()
        .filter_map(|(needle, bits)| upper.find(needle).map(|at| (at, needle.len(), *bits)))
        .min_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)))
        .map(|(_, _, bits)| bits)
}

/// Order companions so the most useful candidate for `target_quant` leads:
/// grouped by role, then by how close the companion's own quantisation is to
/// the target's, then by name for a stable, reproducible list.
fn rank_companions(items: &mut [Companion], target_quant: &str) {
    let target_bits = quant_weight(target_quant);
    items.sort_by(|a, b| {
        a.role
            .cmp(&b.role)
            .then_with(|| {
                let distance = |c: &Companion| match (target_bits, quant_weight(&c.name)) {
                    (Some(target), Some(bits)) => ((bits - target).abs() * 100.0) as i64,
                    (_, Some(_)) => 10_000,
                    _ => 20_000,
                };
                distance(a).cmp(&distance(b))
            })
            .then_with(|| a.name.cmp(&b.name))
    });
}

fn role_for(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("mmproj-") {
        Some("mmproj")
    } else if lower.contains("dspark") {
        Some("dspark")
    } else if lower.starts_with("mtp-") || lower.contains("-mtp-") {
        Some("mtp")
    } else if lower.contains("dflash") {
        Some("dflash")
    } else if lower.contains("eagle3") || lower.contains("eagle-3") {
        Some("eagle3")
    } else {
        None
    }
}

fn shard_key(name: &str) -> (String, usize) {
    if let Ok(shard) = parse_shard_name(name) {
        return (shard.logical_name, shard.count);
    }
    let stem = name.strip_suffix(".gguf").unwrap_or(name);
    if let Some(of_pos) = stem.rfind("-of-") {
        let expected = stem[of_pos + 4..].parse::<usize>().unwrap_or(1);
        let before = &stem[..of_pos];
        if let Some(dash) = before.rfind('-') {
            if before[dash + 1..].len() == 5
                && before[dash + 1..].chars().all(|c| c.is_ascii_digit())
            {
                return (before[..dash].to_string(), expected);
            }
        }
    }
    (stem.to_string(), 1)
}

fn collect_gguf(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Could not inspect {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
            continue;
        }
        if metadata.is_dir() {
            collect_gguf(&path, out)?;
        } else if metadata.is_file()
            && path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("gguf"))
        {
            out.push(path);
        }
    }
    Ok(())
}

pub fn scan_models(root: &Path) -> Result<Vec<LogicalModel>, String> {
    let root_metadata = fs::symlink_metadata(root)
        .map_err(|error| format!("Could not inspect model root {}: {error}", root.display()))?;
    if root_metadata.file_type().is_symlink()
        || is_reparse_point(&root_metadata)
        || !root_metadata.is_dir()
    {
        return Err(format!("Model root does not exist: {}", root.display()));
    }
    let mut files = Vec::new();
    collect_gguf(root, &mut files)?;
    let mut targets: BTreeMap<(PathBuf, String), Vec<PathBuf>> = BTreeMap::new();
    let mut companions: BTreeMap<PathBuf, Vec<Companion>> = BTreeMap::new();

    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let directory = path.parent().unwrap().to_path_buf();
        if let Some(role) = role_for(&name) {
            companions.entry(directory).or_default().push(Companion {
                path: path.to_string_lossy().to_string(),
                name,
                role: role.to_string(),
                size_bytes: path.metadata().map_err(|e| e.to_string())?.len(),
            });
        } else {
            let (key, _) = shard_key(&name);
            targets.entry((directory, key)).or_default().push(path);
        }
    }

    let mut models = Vec::new();
    for ((directory, key), mut shards) in targets {
        shards.sort_by(|left, right| {
            let identity = |path: &PathBuf| {
                path.file_name()
                    .and_then(|name| parse_shard_name(&name.to_string_lossy()).ok())
                    .map(|shard| shard.index)
                    .unwrap_or(1)
            };
            identity(left)
                .cmp(&identity(right))
                .then_with(|| left.cmp(right))
        });
        let first = shards.first().unwrap();
        let shard_identities: Vec<_> = shards
            .iter()
            .filter_map(|path| {
                path.file_name()
                    .and_then(|name| parse_shard_name(&name.to_string_lossy()).ok())
            })
            .collect();
        let expected = shard_identities
            .iter()
            .map(|shard| shard.count)
            .max()
            .unwrap_or(1);
        let expected_consistent = shard_identities.iter().all(|shard| shard.count == expected);
        let indices: HashSet<_> = shard_identities.iter().map(|shard| shard.index).collect();
        let complete = expected_consistent
            && indices.len() == expected
            && (1..=expected).all(|index| indices.contains(&index));
        let size_bytes = shards
            .iter()
            .map(|p| p.metadata().map(|m| m.len()).unwrap_or(0))
            .sum();
        let shard_facts = shards
            .iter()
            .map(|path| {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let identity = parse_shard_name(&name).ok();
                Ok(ArtifactFileFact {
                    path: path.to_string_lossy().to_string(),
                    name,
                    size_bytes: path.metadata().map_err(|error| error.to_string())?.len(),
                    sha256: None,
                    header_sha256: None,
                    shard_index: identity.as_ref().map(|shard| shard.index),
                    expected_shards: identity.as_ref().map(|shard| shard.count),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let quant = key
            .split('-')
            .rev()
            .find(|part| part.starts_with('Q') || part.starts_with("IQ") || part.starts_with("UD"))
            .unwrap_or("unknown")
            .to_string();
        let family_root = |path: &Path| {
            path.strip_prefix(root)
                .ok()
                .and_then(|relative| relative.components().next())
                .map(|component| root.join(component.as_os_str()))
                .unwrap_or_else(|| path.to_path_buf())
        };
        let target_family = family_root(&directory);
        let mut model_companions: Vec<Companion> = companions
            .iter()
            .filter(|(companion_dir, _)| family_root(companion_dir) == target_family)
            .flat_map(|(_, items)| items.clone())
            .collect();
        rank_companions(&mut model_companions, &quant);
        models.push(LogicalModel {
            id: first.to_string_lossy().to_string(),
            name: key,
            directory: directory.to_string_lossy().to_string(),
            first_shard: first.to_string_lossy().to_string(),
            size_bytes,
            shard_count: shards.len(),
            expected_shards: expected,
            complete,
            quant,
            shards: shard_facts,
            companions: model_companions,
        });
    }
    Ok(models)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct LaunchProfile {
    pub name: String,
    pub runtime: String,
    pub model: String,
    pub draft_model: Option<String>,
    pub mmproj: Option<String>,
    pub host: String,
    pub port: u16,
    pub alias: String,
    pub context: u32,
    pub parallel: u16,
    pub gpu_layers: String,
    pub cpu_moe: u16,
    pub cpu_ffn: u16,
    pub threads: i32,
    pub threads_batch: i32,
    pub batch: u32,
    pub ubatch: u32,
    pub flash_attention: String,
    pub fit: bool,
    pub fit_target: String,
    pub fit_ctx: u32,
    pub kv_offload: bool,
    pub cache_type_k: String,
    pub cache_type_v: String,
    pub load_mode: String,
    pub lazy_mode: String,
    pub split_mode: String,
    pub tensor_split: String,
    pub main_gpu: u16,
    pub device: String,
    pub continuous_batching: bool,
    pub cache_prompt: bool,
    pub cache_reuse: u32,
    pub cache_ram: i32,
    pub context_checkpoints: u16,
    pub context_shift: bool,
    pub warmup: bool,
    pub sleep_idle_seconds: i32,
    pub timeout: u32,
    pub threads_http: i32,
    pub sse_ping_interval: i32,
    pub metrics: bool,
    pub slots: bool,
    pub web_ui: bool,
    pub cors_origins: String,
    pub api_key_file: String,
    pub ssl_key_file: String,
    pub ssl_cert_file: String,
    pub jinja: bool,
    pub reasoning: String,
    pub reasoning_effort: String,
    pub reasoning_budget: i32,
    pub reasoning_preserve: bool,
    pub chat_template_file: String,
    pub temperature: f32,
    pub top_k: i32,
    pub top_p: f32,
    pub min_p: f32,
    pub repeat_penalty: f32,
    pub repeat_last_n: i32,
    pub seed: i64,
    pub dry_multiplier: f32,
    pub dry_base: f32,
    pub spec_type: String,
    pub draft_max: u16,
    pub draft_min: u16,
    pub draft_p_min: f32,
    pub draft_p_split: f32,
    pub draft_gpu_layers: String,
    pub draft_cache_type_k: String,
    pub draft_cache_type_v: String,
    pub ngram_match: u16,
    pub ngram_min: u16,
    pub ngram_max: u16,
    pub ngram_size_n: u16,
    pub ngram_size_m: u16,
    pub ngram_min_hits: u16,
    pub mmproj_offload: bool,
    pub mmproj_device: String,
    pub image_min_tokens: u32,
    pub image_max_tokens: u32,
    pub lora: String,
    pub lora_scaled: String,
    pub override_tensor: String,
    pub override_kv: String,
    pub verbosity: u8,
    pub log_timestamps: bool,
    pub extra_args: Vec<String>,
}

impl Default for LaunchProfile {
    fn default() -> Self {
        Self {
            name: "New profile".into(),
            runtime: r"C:\llama\llama-server.exe".into(),
            model: String::new(),
            draft_model: None,
            mmproj: None,
            host: "127.0.0.1".into(),
            port: 8080,
            alias: String::new(),
            context: 8192,
            parallel: 1,
            gpu_layers: "all".into(),
            cpu_moe: 0,
            cpu_ffn: 0,
            threads: -1,
            threads_batch: -1,
            batch: 2048,
            ubatch: 512,
            flash_attention: "auto".into(),
            fit: true,
            fit_target: "1024".into(),
            fit_ctx: 4096,
            kv_offload: true,
            cache_type_k: "f16".into(),
            cache_type_v: "f16".into(),
            load_mode: "auto".into(),
            lazy_mode: "auto".into(),
            split_mode: "layer".into(),
            tensor_split: String::new(),
            main_gpu: 0,
            device: String::new(),
            continuous_batching: true,
            cache_prompt: true,
            cache_reuse: 0,
            cache_ram: 8192,
            context_checkpoints: 32,
            context_shift: false,
            warmup: true,
            sleep_idle_seconds: -1,
            timeout: 3600,
            threads_http: -1,
            sse_ping_interval: 30,
            metrics: true,
            slots: true,
            web_ui: true,
            cors_origins: "localhost".into(),
            api_key_file: String::new(),
            ssl_key_file: String::new(),
            ssl_cert_file: String::new(),
            jinja: true,
            reasoning: "auto".into(),
            reasoning_effort: "default".into(),
            reasoning_budget: -1,
            reasoning_preserve: false,
            chat_template_file: String::new(),
            temperature: 0.8,
            top_k: 40,
            top_p: 0.95,
            min_p: 0.05,
            repeat_penalty: 1.0,
            repeat_last_n: 64,
            seed: -1,
            dry_multiplier: 0.0,
            dry_base: 1.75,
            spec_type: "none".into(),
            draft_max: 3,
            draft_min: 0,
            draft_p_min: 0.0,
            draft_p_split: 0.10,
            draft_gpu_layers: "auto".into(),
            draft_cache_type_k: "f16".into(),
            draft_cache_type_v: "f16".into(),
            ngram_match: 24,
            ngram_min: 48,
            ngram_max: 64,
            ngram_size_n: 12,
            ngram_size_m: 48,
            ngram_min_hits: 1,
            mmproj_offload: true,
            mmproj_device: "auto".into(),
            image_min_tokens: 0,
            image_max_tokens: 0,
            lora: String::new(),
            lora_scaled: String::new(),
            override_tensor: String::new(),
            override_kv: String::new(),
            verbosity: 3,
            log_timestamps: true,
            extra_args: Vec::new(),
        }
    }
}

impl LaunchProfile {
    pub fn build_args(&self) -> Result<Vec<String>, String> {
        if self.alias.trim().is_empty() {
            return Err("Model alias is required".into());
        }
        if self.port == 0 {
            return Err("Port must be between 1 and 65535".into());
        }
        if self.context == 0 || self.parallel == 0 || self.batch == 0 || self.ubatch == 0 {
            return Err("Context, slots, batch, and uBatch must be greater than zero".into());
        }
        if self.ubatch > self.batch {
            return Err("Physical uBatch cannot exceed logical batch size".into());
        }
        let gpu_layers = self.gpu_layers.trim();
        if !matches!(gpu_layers, "auto" | "all") && gpu_layers.parse::<u32>().is_err() {
            return Err("GPU layers must be a nonnegative number, auto, or all".into());
        }
        if !self.tensor_split.trim().is_empty() {
            let mut has_positive_fraction = false;
            for fraction in self.tensor_split.split(',') {
                let value = fraction
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value >= 0.0)
                    .ok_or("Tensor split requires comma-separated finite nonnegative values")?;
                has_positive_fraction |= value > 0.0;
            }
            if !has_positive_fraction {
                return Err("Tensor split requires at least one positive value".into());
            }
        }
        if self.spec_type.starts_with("draft-") && self.draft_min > self.draft_max {
            return Err("Speculative draft minimum cannot exceed maximum".into());
        }
        if self.spec_type == "ngram-mod" && self.ngram_min > self.ngram_max {
            return Err("N-gram draft minimum cannot exceed maximum".into());
        }
        if self.image_max_tokens > 0 && self.image_min_tokens > self.image_max_tokens {
            return Err("Minimum image token budget cannot exceed maximum".into());
        }
        if self.ssl_key_file.is_empty() != self.ssl_cert_file.is_empty() {
            return Err("SSL private key and certificate must be configured together".into());
        }
        let host = self.host.trim();
        if host.is_empty() {
            return Err("Host is required".into());
        }
        let ip_host = host
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
            .unwrap_or(host);
        let parsed_host = if ip_host.eq_ignore_ascii_case("localhost") {
            None
        } else {
            Some(ip_host.parse::<std::net::IpAddr>().map_err(|_| {
                "Host must be an IP address or localhost; DNS names are not safe bind targets"
            })?)
        };
        let loopback = ip_host.eq_ignore_ascii_case("localhost")
            || parsed_host.is_some_and(|address| address.is_loopback());
        if loopback
            && self.api_key_file.trim().is_empty()
            && matches!(self.cors_origins.trim(), "" | "*")
        {
            return Err("Wildcard CORS requires an API key file".into());
        }
        if !loopback && self.api_key_file.trim().is_empty() {
            return Err("A non-loopback host requires an API key file".into());
        }
        if !loopback && matches!(self.cors_origins.trim(), "" | "*") {
            return Err("A non-loopback host requires restricted CORS origins".into());
        }
        let model_backed = matches!(
            self.spec_type.as_str(),
            "draft-simple" | "draft-eagle3" | "draft-dflash" | "draft-dspark"
        );
        if model_backed && self.draft_model.as_deref().unwrap_or("").is_empty() {
            return Err(format!("{} requires a draft model", self.spec_type));
        }

        let mut args = Vec::new();
        let push = |args: &mut Vec<String>, flag: &str, value: String| {
            args.extend([flag.to_string(), value]);
        };
        push(&mut args, "-m", self.model.clone());
        push(&mut args, "--host", self.host.clone());
        push(&mut args, "--port", self.port.to_string());
        push(&mut args, "--alias", self.alias.clone());
        push(&mut args, "-c", self.context.to_string());
        push(&mut args, "-np", self.parallel.to_string());
        push(&mut args, "-ngl", self.gpu_layers.clone());
        push(&mut args, "-b", self.batch.to_string());
        push(&mut args, "-ub", self.ubatch.to_string());
        push(&mut args, "--flash-attn", self.flash_attention.clone());
        push(
            &mut args,
            "--fit",
            if self.fit { "on" } else { "off" }.into(),
        );
        if self.fit {
            if !self.fit_target.trim().is_empty() {
                push(&mut args, "--fit-target", self.fit_target.clone());
            }
            push(&mut args, "--fit-ctx", self.fit_ctx.to_string());
        }
        if self.threads >= 0 {
            push(&mut args, "-t", self.threads.to_string());
        }
        if self.threads_batch >= 0 {
            push(&mut args, "-tb", self.threads_batch.to_string());
        }
        args.push(
            if self.kv_offload {
                "--kv-offload"
            } else {
                "--no-kv-offload"
            }
            .into(),
        );
        push(&mut args, "-ctk", self.cache_type_k.clone());
        push(&mut args, "-ctv", self.cache_type_v.clone());
        push(&mut args, "--load-mode", self.load_mode.clone());
        push(&mut args, "--lazy-mode", self.lazy_mode.clone());
        push(&mut args, "--split-mode", self.split_mode.clone());
        push(&mut args, "--main-gpu", self.main_gpu.to_string());
        if !self.tensor_split.trim().is_empty() {
            push(&mut args, "--tensor-split", self.tensor_split.clone());
        }
        if !self.device.trim().is_empty() {
            push(&mut args, "--device", self.device.clone());
        }
        if self.cpu_moe > 0 {
            push(&mut args, "-ncmoe", self.cpu_moe.to_string());
        }
        if self.cpu_ffn > 0 {
            push(&mut args, "-ncffn", self.cpu_ffn.to_string());
        }

        args.push(
            if self.continuous_batching {
                "--cont-batching"
            } else {
                "--no-cont-batching"
            }
            .into(),
        );
        args.push(
            if self.cache_prompt {
                "--cache-prompt"
            } else {
                "--no-cache-prompt"
            }
            .into(),
        );
        push(&mut args, "--cache-reuse", self.cache_reuse.to_string());
        push(&mut args, "--cache-ram", self.cache_ram.to_string());
        push(
            &mut args,
            "--ctx-checkpoints",
            self.context_checkpoints.to_string(),
        );
        args.push(
            if self.context_shift {
                "--context-shift"
            } else {
                "--no-context-shift"
            }
            .into(),
        );
        args.push(
            if self.warmup {
                "--warmup"
            } else {
                "--no-warmup"
            }
            .into(),
        );
        push(
            &mut args,
            "--sleep-idle-seconds",
            self.sleep_idle_seconds.to_string(),
        );
        push(&mut args, "--timeout", self.timeout.to_string());
        if self.threads_http >= 0 {
            push(&mut args, "--threads-http", self.threads_http.to_string());
        }
        push(
            &mut args,
            "--sse-ping-interval",
            self.sse_ping_interval.to_string(),
        );
        if self.metrics {
            args.push("--metrics".into());
        }
        args.push(if self.slots { "--slots" } else { "--no-slots" }.into());
        args.push(if self.web_ui { "--webui" } else { "--no-webui" }.into());

        if !self.cors_origins.trim().is_empty() {
            push(&mut args, "--cors-origins", self.cors_origins.clone());
        }
        if !self.api_key_file.trim().is_empty() {
            push(&mut args, "--api-key-file", self.api_key_file.clone());
        }
        if !self.ssl_key_file.trim().is_empty() {
            push(&mut args, "--ssl-key-file", self.ssl_key_file.clone());
        }
        if !self.ssl_cert_file.trim().is_empty() {
            push(&mut args, "--ssl-cert-file", self.ssl_cert_file.clone());
        }

        args.push(if self.jinja { "--jinja" } else { "--no-jinja" }.into());
        push(&mut args, "--reasoning", self.reasoning.clone());
        if self.reasoning_effort != "default" {
            push(
                &mut args,
                "--reasoning-effort",
                self.reasoning_effort.clone(),
            );
        }
        push(
            &mut args,
            "--reasoning-budget",
            self.reasoning_budget.to_string(),
        );
        if self.reasoning_preserve {
            args.push("--reasoning-preserve".into());
        }
        if !self.chat_template_file.trim().is_empty() {
            push(
                &mut args,
                "--chat-template-file",
                self.chat_template_file.clone(),
            );
        }

        push(&mut args, "--temperature", self.temperature.to_string());
        push(&mut args, "--top-k", self.top_k.to_string());
        push(&mut args, "--top-p", self.top_p.to_string());
        push(&mut args, "--min-p", self.min_p.to_string());
        push(
            &mut args,
            "--repeat-penalty",
            self.repeat_penalty.to_string(),
        );
        push(&mut args, "--repeat-last-n", self.repeat_last_n.to_string());
        push(&mut args, "--seed", self.seed.to_string());
        push(
            &mut args,
            "--dry-multiplier",
            self.dry_multiplier.to_string(),
        );
        push(&mut args, "--dry-base", self.dry_base.to_string());

        if let Some(path) = self.draft_model.as_ref().filter(|s| !s.is_empty()) {
            push(&mut args, "-md", path.clone());
        }
        if self.spec_type != "none" {
            push(&mut args, "--spec-type", self.spec_type.clone());
            if self.spec_type.starts_with("draft-") {
                push(&mut args, "--spec-draft-n-max", self.draft_max.to_string());
                push(&mut args, "--spec-draft-n-min", self.draft_min.to_string());
                push(
                    &mut args,
                    "--spec-draft-p-min",
                    self.draft_p_min.to_string(),
                );
                push(
                    &mut args,
                    "--spec-draft-p-split",
                    self.draft_p_split.to_string(),
                );
                push(&mut args, "--spec-draft-ngl", self.draft_gpu_layers.clone());
                push(
                    &mut args,
                    "--spec-draft-type-k",
                    self.draft_cache_type_k.clone(),
                );
                push(
                    &mut args,
                    "--spec-draft-type-v",
                    self.draft_cache_type_v.clone(),
                );
            }
            match self.spec_type.as_str() {
                "ngram-mod" => {
                    push(
                        &mut args,
                        "--spec-ngram-mod-n-match",
                        self.ngram_match.to_string(),
                    );
                    push(
                        &mut args,
                        "--spec-ngram-mod-n-min",
                        self.ngram_min.to_string(),
                    );
                    push(
                        &mut args,
                        "--spec-ngram-mod-n-max",
                        self.ngram_max.to_string(),
                    );
                }
                "ngram-simple" | "ngram-map-k" | "ngram-map-k4v" => {
                    let prefix = format!("--spec-{}", self.spec_type);
                    push(
                        &mut args,
                        &format!("{prefix}-size-n"),
                        self.ngram_size_n.to_string(),
                    );
                    push(
                        &mut args,
                        &format!("{prefix}-size-m"),
                        self.ngram_size_m.to_string(),
                    );
                    push(
                        &mut args,
                        &format!("{prefix}-min-hits"),
                        self.ngram_min_hits.to_string(),
                    );
                }
                _ => {}
            }
        }

        if let Some(path) = self.mmproj.as_ref().filter(|s| !s.is_empty()) {
            push(&mut args, "--mmproj", path.clone());
            args.push(
                if self.mmproj_offload {
                    "--mmproj-offload"
                } else {
                    "--no-mmproj-offload"
                }
                .into(),
            );
            push(&mut args, "--mmproj-device", self.mmproj_device.clone());
            if self.image_min_tokens > 0 {
                push(
                    &mut args,
                    "--image-min-tokens",
                    self.image_min_tokens.to_string(),
                );
            }
            if self.image_max_tokens > 0 {
                push(
                    &mut args,
                    "--image-max-tokens",
                    self.image_max_tokens.to_string(),
                );
            }
        }
        if !self.lora.trim().is_empty() {
            push(&mut args, "--lora", self.lora.clone());
        }
        if !self.lora_scaled.trim().is_empty() {
            push(&mut args, "--lora-scaled", self.lora_scaled.clone());
        }
        if !self.override_tensor.trim().is_empty() {
            push(&mut args, "--override-tensor", self.override_tensor.clone());
        }
        if !self.override_kv.trim().is_empty() {
            push(&mut args, "--override-kv", self.override_kv.clone());
        }
        push(&mut args, "--log-verbosity", self.verbosity.to_string());
        args.push(
            if self.log_timestamps {
                "--log-timestamps"
            } else {
                "--no-log-timestamps"
            }
            .into(),
        );
        validate_raw_extra_arguments(&args, &self.extra_args)?;
        args.extend(self.extra_args.clone());
        Ok(args)
    }

    pub fn display_command_with_args(&self, args: &[String]) -> String {
        let quote = |s: &str| {
            if s.contains(' ') {
                format!("\"{}\"", s)
            } else {
                s.to_string()
            }
        };
        std::iter::once(quote(&self.runtime))
            .chain(args.iter().map(|argument| quote(argument)))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn argument_flag(token: &str) -> Option<&str> {
    let flag = token.split_once('=').map_or(token, |(flag, _)| flag);
    let bytes = flag.as_bytes();
    (flag.starts_with('-')
        && flag.len() > 1
        && bytes.get(1).is_some_and(|byte| !byte.is_ascii_digit()))
    .then_some(flag)
}

fn validate_raw_extra_arguments(
    managed_args: &[String],
    extra_args: &[String],
) -> Result<(), String> {
    const MAX_EXTRA_ARGUMENTS: usize = 32;
    if extra_args.len() > MAX_EXTRA_ARGUMENTS {
        return Err(format!(
            "Raw extra arguments are limited to {MAX_EXTRA_ARGUMENTS} tokens"
        ));
    }
    const RESTRICTED_FLAGS: &[&str] = &[
        "--path",
        "--slot-save-path",
        "--webui-mcp-proxy",
        "-ag",
        "-dr",
        "-hf",
        "-hff",
        "-hfr",
        "-hft",
        "-mmu",
        "-mu",
    ];
    const RESTRICTED_PREFIXES: &[&str] = &[
        "--agent",
        "--api-key",
        "--docker-",
        "--hf-",
        "--mcp-",
        "--mmproj-url",
        "--model-url",
        "--models-",
        "--rpc",
        "--tools",
    ];
    const MANAGED_FLAG_ALIASES: &[&str] = &[
        "--batch-size",
        "--cache-type-k",
        "--cache-type-v",
        "--ctx-size",
        "--gpu-layers",
        "--model",
        "--n-gpu-layers",
        "--parallel",
        "--threads",
        "--threads-batch",
        "--ubatch-size",
    ];
    let managed_flags = managed_args
        .iter()
        .filter_map(|argument| argument_flag(argument))
        .collect::<HashSet<_>>();

    for argument in extra_args {
        const MAX_EXTRA_ARGUMENT_BYTES: usize = 1_024;
        if argument.len() > MAX_EXTRA_ARGUMENT_BYTES {
            return Err(format!(
                "Each raw extra argument is limited to {MAX_EXTRA_ARGUMENT_BYTES} bytes"
            ));
        }
        let Some(flag) = argument_flag(argument) else {
            return Err("Each raw extra argument must be one --flag or --flag=value token".into());
        };
        if managed_flags.contains(flag) || MANAGED_FLAG_ALIASES.contains(&flag) {
            return Err(format!(
                "Raw extra argument {flag} cannot override a managed profile flag"
            ));
        }
        if RESTRICTED_FLAGS.contains(&flag)
            || RESTRICTED_PREFIXES
                .iter()
                .any(|prefix| flag.starts_with(prefix))
        {
            return Err(format!(
                "Raw extra argument {flag} is not allowed at this trust boundary"
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCapabilities {
    pub path: String,
    pub version: String,
    pub build: String,
    pub commit: String,
    pub help_sha256: String,
    pub spec_types: Vec<String>,
    pub supported_flags: Vec<String>,
    pub metrics: bool,
    pub multimodal: bool,
    pub fit: bool,
}

pub fn parse_supported_flags(help: &str) -> Vec<String> {
    let mut flags = help
        .split_whitespace()
        .filter_map(|token| {
            let clean = token
                .trim_matches(|c: char| matches!(c, ',' | '[' | ']' | '(' | ')' | '`' | ':' | ';'));
            let bytes = clean.as_bytes();
            if clean.starts_with('-')
                && clean.len() > 1
                && bytes.get(1).is_some_and(|byte| !byte.is_ascii_digit())
            {
                Some(clean.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    flags.sort();
    flags.dedup();
    flags
}

pub fn manifest_safe_args(args: &[String]) -> Vec<String> {
    const SECRET_VALUE_FLAGS: &[&str] = &["--api-key"];
    const SENSITIVE_VALUE_FLAGS: &[&str] = &[
        "-m",
        "--model-draft",
        "--mmproj",
        "--api-key-file",
        "--ssl-key-file",
        "--ssl-cert-file",
        "--chat-template-file",
    ];

    let fingerprint = |value: &str| {
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    };
    let mut safe = Vec::with_capacity(args.len());
    let mut redact_secret_next = false;
    let mut redact_next = false;
    for argument in args {
        if redact_secret_next {
            safe.push("[REDACTED]".into());
            redact_secret_next = false;
            continue;
        }
        if redact_next {
            safe.push(fingerprint(argument));
            redact_next = false;
            continue;
        }
        if let Some((flag, value)) = argument.split_once('=') {
            if SECRET_VALUE_FLAGS.contains(&flag) {
                safe.push(format!("{flag}=[REDACTED]"));
                continue;
            }
            if SENSITIVE_VALUE_FLAGS.contains(&flag) {
                safe.push(format!("{flag}={}", fingerprint(value)));
                continue;
            }
        }
        safe.push(argument.clone());
        redact_secret_next = SECRET_VALUE_FLAGS.contains(&argument.as_str());
        redact_next = SENSITIVE_VALUE_FLAGS.contains(&argument.as_str());
    }
    safe
}

pub fn filter_supported_args(
    args: &[String],
    supported_flags: &[String],
) -> (Vec<String>, Vec<String>) {
    let supported = supported_flags
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut filtered = Vec::new();
    let mut omitted = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if let Some(flag) = argument_flag(token).filter(|flag| !supported.contains(*flag)) {
            omitted.push(flag.to_string());
            index += 1;
            if !token.contains('=') && index < args.len() && argument_flag(&args[index]).is_none() {
                index += 1;
            }
            continue;
        }
        filtered.push(token.clone());
        index += 1;
    }
    (filtered, omitted)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectedLaunchArgument {
    pub flag: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchArgumentValidation {
    pub effective_args: Vec<String>,
    pub rejected: Vec<RejectedLaunchArgument>,
    pub command: String,
}

pub fn validate_launch_arguments(
    profile: &LaunchProfile,
    capabilities: &RuntimeCapabilities,
) -> Result<LaunchArgumentValidation, String> {
    let raw_args = profile.build_args()?;
    let managed_end = raw_args
        .len()
        .checked_sub(profile.extra_args.len())
        .ok_or_else(|| "Raw launch argument accounting is inconsistent".to_string())?;
    let managed_flags = raw_args[..managed_end]
        .iter()
        .filter_map(|argument| argument_flag(argument))
        .collect::<HashSet<_>>();
    let (effective_args, omitted) = filter_supported_args(&raw_args, &capabilities.supported_flags);
    let unsupported_required = omitted
        .iter()
        .filter(|flag| managed_flags.contains(flag.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unsupported_required.is_empty() {
        return Err(format!(
            "Runtime help does not advertise required launch flags: {}",
            unsupported_required.join(", ")
        ));
    }
    let rejected = omitted
        .into_iter()
        .map(|flag| RejectedLaunchArgument {
            reason: format!("Runtime help does not advertise {flag}"),
            flag,
        })
        .collect();
    let command = profile.display_command_with_args(&effective_args);
    Ok(LaunchArgumentValidation {
        effective_args,
        rejected,
        command,
    })
}

pub fn parse_capabilities(version: &str, help: &str) -> RuntimeCapabilities {
    let extract = |marker: &str| {
        version
            .split(marker)
            .nth(1)
            .and_then(|s| {
                s.split(|c: char| c == ',' || c == ')' || c.is_whitespace())
                    .find(|x| !x.is_empty())
            })
            .unwrap_or("unknown")
            .to_string()
    };
    let spec_types = help
        .lines()
        .find_map(|line| line.trim().strip_prefix("--spec-type "))
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or("none")
        .split(',')
        .map(str::to_string)
        .collect();
    RuntimeCapabilities {
        path: String::new(),
        version: version.trim().to_string(),
        build: extract("build "),
        commit: extract("commit "),
        help_sha256: hex::encode(Sha256::digest(help.as_bytes())),
        spec_types,
        supported_flags: parse_supported_flags(help),
        metrics: help.contains("--metrics"),
        multimodal: help.contains("--mmproj"),
        fit: help.contains("--fit"),
    }
}

#[cfg(test)]
const RUNTIME_PROBE_TIMEOUT: Duration = Duration::from_millis(250);
#[cfg(not(test))]
const RUNTIME_PROBE_TIMEOUT: Duration = Duration::from_secs(10);
const RUNTIME_PROBE_STREAM_LIMIT: usize = 2 * 1024 * 1024;

fn read_runtime_probe_stream(stream: &mut impl Read) -> std::io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8 * 1024];
    let mut exceeded = false;
    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = RUNTIME_PROBE_STREAM_LIMIT.saturating_sub(bytes.len());
        bytes.extend_from_slice(&buffer[..read.min(remaining)]);
        exceeded |= read > remaining;
    }
    Ok((bytes, exceeded))
}

fn run_runtime_probe(path: &Path, arg: &str) -> Result<String, String> {
    let mut child = crate::proc::hidden_command(path)
        .arg(arg)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("Could not run runtime {arg} probe: {error}"))?;
    let Some(mut stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("Runtime {arg} probe did not expose stdout"));
    };
    let Some(mut stderr) = child.stderr.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("Runtime {arg} probe did not expose stderr"));
    };
    let stdout_reader = std::thread::spawn(move || read_runtime_probe_stream(&mut stdout));
    let stderr_reader = std::thread::spawn(move || read_runtime_probe_stream(&mut stderr));
    let started = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < RUNTIME_PROBE_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(format!(
                    "Runtime {arg} probe timed out after {} seconds",
                    RUNTIME_PROBE_TIMEOUT.as_secs_f32()
                ));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(format!("Could not inspect runtime {arg} probe: {error}"));
            }
        }
    };
    let (stdout, stdout_exceeded) = stdout_reader
        .join()
        .map_err(|_| format!("Runtime {arg} stdout reader failed"))?
        .map_err(|error| format!("Could not read runtime {arg} stdout: {error}"))?;
    let (stderr, stderr_exceeded) = stderr_reader
        .join()
        .map_err(|_| format!("Runtime {arg} stderr reader failed"))?
        .map_err(|error| format!("Could not read runtime {arg} stderr: {error}"))?;
    if stdout_exceeded || stderr_exceeded {
        return Err(format!(
            "Runtime {arg} probe exceeded the {} byte output limit",
            RUNTIME_PROBE_STREAM_LIMIT
        ));
    }
    let text = String::from_utf8_lossy(&[stdout, stderr].concat()).to_string();
    if !status.success() {
        let status = status
            .code()
            .map(|code| format!("code {code}"))
            .unwrap_or_else(|| "a signal".into());
        return Err(format!(
            "Runtime {arg} probe exited with {status}: {}",
            text.trim()
        ));
    }
    Ok(text)
}

pub fn inspect_runtime(path: &Path) -> Result<RuntimeCapabilities, String> {
    crate::artifact::validate_regular_non_reparse_file("Runtime", path)?;
    let version = run_runtime_probe(path, "--version")?;
    let help = run_runtime_probe(path, "--help")?;
    let mut caps = parse_capabilities(&version, &help);
    caps.path = path.to_string_lossy().to_string();
    Ok(caps)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSummary {
    pub samples: Vec<f64>,
    pub mean_tps: f64,
    pub median_tps: f64,
    pub min_tps: f64,
    pub max_tps: f64,
    pub tokens: u32,
    pub repeats: u16,
}

pub fn summarize_benchmark(
    mut samples: Vec<f64>,
    tokens: u32,
    repeats: u16,
) -> Result<BenchmarkSummary, String> {
    if samples.is_empty() {
        return Err("No benchmark samples were collected".into());
    }
    if samples.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return Err("Benchmark samples must be positive finite values".into());
    }
    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    samples.sort_by(|a, b| a.total_cmp(b));
    let median = if samples.len().is_multiple_of(2) {
        (samples[samples.len() / 2 - 1] + samples[samples.len() / 2]) / 2.0
    } else {
        samples[samples.len() / 2]
    };
    Ok(BenchmarkSummary {
        mean_tps: mean,
        median_tps: median,
        min_tps: samples[0],
        max_tps: *samples.last().unwrap(),
        samples,
        tokens,
        repeats,
    })
}

pub fn parse_tps(body: &str) -> Result<f64, String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    value["timings"]["predicted_per_second"]
        .as_f64()
        .filter(|v| *v > 0.0)
        .ok_or_else(|| "llama-server response did not include generation throughput".into())
}

fn completion_request(host: &str, port: u16, tokens: u32) -> Result<f64, String> {
    let connect_host = match host {
        "0.0.0.0" => "127.0.0.1",
        "::" | "[::]" => "::1",
        value => value,
    };
    let body = serde_json::json!({
        "prompt": "Write a detailed technical explanation of speculative decoding, including verification, acceptance, and performance tradeoffs.",
        "n_predict": tokens,
        "temperature": 0,
        "seed": 42,
        "ignore_eos": true,
        "stream": false
    }).to_string();
    let mut stream = TcpStream::connect((connect_host, port)).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(600)))
        .map_err(|e| e.to_string())?;
    let request = format!(
        "POST /completion HTTP/1.1\r\nHost: {connect_host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| e.to_string())?;
    let (headers, payload) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| "Invalid HTTP response from llama-server".to_string())?;
    if !headers.contains(" 200 ") {
        return Err(format!(
            "Benchmark request failed: {}",
            headers.lines().next().unwrap_or(headers)
        ));
    }
    parse_tps(payload)
}

pub fn benchmark_server(
    host: &str,
    port: u16,
    tokens: u32,
    repeats: u16,
) -> Result<BenchmarkSummary, String> {
    if repeats == 0 || repeats > 10 {
        return Err("Repeats must be between 1 and 10".into());
    }
    completion_request(host, port, tokens.min(64))?;
    let mut samples = Vec::new();
    for _ in 0..repeats {
        samples.push(completion_request(host, port, tokens)?);
    }
    summarize_benchmark(samples, tokens, repeats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn launch_profile_rejects_port_zero() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            model: "fixture.gguf".into(),
            port: 0,
            ..LaunchProfile::default()
        };

        assert!(profile.build_args().unwrap_err().contains("Port"));
    }

    #[test]
    fn launch_profile_recognizes_the_ip_loopback_ranges() {
        for host in ["127.0.0.2", "::1", "[::1]"] {
            let profile = LaunchProfile {
                alias: "fixture".into(),
                model: "fixture.gguf".into(),
                host: host.into(),
                api_key_file: String::new(),
                ..LaunchProfile::default()
            };

            assert!(
                profile.build_args().is_ok(),
                "loopback host {host} required network credentials"
            );
        }
    }

    #[test]
    fn launch_profile_rejects_wildcard_cors_without_authentication() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            model: "fixture.gguf".into(),
            host: "127.0.0.1".into(),
            cors_origins: "*".into(),
            api_key_file: String::new(),
            ..LaunchProfile::default()
        };

        let error = profile.build_args().unwrap_err();

        assert!(error.contains("Wildcard CORS requires an API key file"));
    }

    #[test]
    fn launch_profile_rejects_dns_hosts_that_could_resolve_off_machine() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            model: "fixture.gguf".into(),
            host: "example.com".into(),
            api_key_file: "key.txt".into(),
            cors_origins: "https://example.com".into(),
            ..LaunchProfile::default()
        };

        let error = profile.build_args().unwrap_err();

        assert!(error.contains("IP address"), "unexpected error: {error}");
    }

    #[test]
    fn raw_extra_arguments_cannot_override_managed_profile_flags() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            model: "fixture.gguf".into(),
            extra_args: vec!["--host=0.0.0.0".into()],
            ..LaunchProfile::default()
        };

        let error = profile.build_args().unwrap_err();

        assert!(error.contains("--host"), "unexpected error: {error}");
        assert!(error.contains("managed"), "unexpected error: {error}");
    }

    #[test]
    fn raw_extra_arguments_cannot_override_managed_long_aliases() {
        for argument in [
            "--model=other.gguf",
            "--ctx-size=1",
            "--n-gpu-layers=0",
            "--threads=1",
            "--batch-size=1",
        ] {
            let profile = LaunchProfile {
                alias: "fixture".into(),
                model: "fixture.gguf".into(),
                extra_args: vec![argument.into()],
                ..LaunchProfile::default()
            };

            let error = profile.build_args().unwrap_err();

            assert!(
                error.contains("managed"),
                "{argument} produced unexpected error: {error}"
            );
        }
    }

    #[test]
    fn raw_extra_arguments_reject_inline_secrets_without_echoing_values() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            model: "fixture.gguf".into(),
            extra_args: vec!["--api-key=[REDACTED]".into()],
            ..LaunchProfile::default()
        };

        let error = profile.build_args().unwrap_err();

        assert!(error.contains("--api-key"), "unexpected error: {error}");
        assert!(!error.contains("[REDACTED]"));
    }

    #[test]
    fn raw_extra_arguments_require_one_self_contained_option_per_token() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            model: "fixture.gguf".into(),
            extra_args: vec!["--check-tensors".into(), "unexpected-value".into()],
            ..LaunchProfile::default()
        };

        let error = profile.build_args().unwrap_err();

        assert!(error.contains("--flag=value"), "unexpected error: {error}");
    }

    #[test]
    fn raw_extra_arguments_reject_privilege_expanding_runtime_flags() {
        for argument in [
            "--model-url=https://example.invalid/model.gguf",
            "--tools-invoke",
            "--webui-mcp-proxy",
        ] {
            let profile = LaunchProfile {
                alias: "fixture".into(),
                model: "fixture.gguf".into(),
                extra_args: vec![argument.into()],
                ..LaunchProfile::default()
            };

            let error = profile.build_args().unwrap_err();

            assert!(
                error.contains("trust boundary"),
                "{argument} produced unexpected error: {error}"
            );
        }
    }

    #[test]
    fn raw_extra_argument_count_is_bounded() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            model: "fixture.gguf".into(),
            extra_args: (0..33).map(|index| format!("--safe-{index}")).collect(),
            ..LaunchProfile::default()
        };

        let error = profile.build_args().unwrap_err();

        assert!(error.contains("32"), "unexpected error: {error}");
    }

    #[test]
    fn raw_extra_argument_length_is_bounded() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            model: "fixture.gguf".into(),
            extra_args: vec![format!("--safe={}", "x".repeat(1_025))],
            ..LaunchProfile::default()
        };

        let error = profile.build_args().unwrap_err();

        assert!(error.contains("1024"), "unexpected error: {error}");
        assert!(!error.contains(&"x".repeat(1_025)));
    }

    #[test]
    fn runtime_filter_accepts_a_supported_equals_form_option() {
        let args = vec!["--safe-experimental=value".into()];
        let supported = vec!["--safe-experimental".into()];

        let (effective, rejected) = filter_supported_args(&args, &supported);

        assert_eq!(effective, args);
        assert!(rejected.is_empty());
    }

    #[test]
    fn runtime_inspection_rejects_a_failed_version_probe() {
        let error = inspect_runtime(&std::env::current_exe().unwrap()).unwrap_err();

        assert!(error.contains("--version"), "unexpected error: {error}");
        assert!(error.contains("exited"), "unexpected error: {error}");
    }

    #[cfg(windows)]
    #[test]
    fn runtime_inspection_times_out_a_stalled_probe() {
        let root = std::env::temp_dir().join(format!(
            "gguf-pilot-runtime-probe-timeout-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let source = root.join("probe.rs");
        let executable = root.join("probe.exe");
        fs::write(
            &source,
            r#"
use std::{env, thread, time::Duration};

fn main() {
    match env::args().nth(1).as_deref() {
        Some("--version") => {
            thread::sleep(Duration::from_secs(2));
            println!("fixture runtime");
        }
        Some("--help") => println!("-m, --model FNAME"),
        _ => {}
    }
}
"#,
        )
        .unwrap();
        let status = crate::proc::hidden_command("rustc")
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .status()
            .unwrap();
        assert!(status.success());

        let error = inspect_runtime(&executable).unwrap_err();

        assert!(error.contains("timed out"), "unexpected error: {error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn runtime_inspection_rejects_excessive_probe_output() {
        let root = std::env::temp_dir().join(format!(
            "gguf-pilot-runtime-probe-output-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let source = root.join("probe.rs");
        let executable = root.join("probe.exe");
        fs::write(
            &source,
            r#"
use std::{env, io::{self, Write}};

fn main() {
    match env::args().nth(1).as_deref() {
        Some("--version") => println!("fixture runtime"),
        Some("--help") => io::stdout().write_all(&vec![b'x'; 5 * 1024 * 1024]).unwrap(),
        _ => {}
    }
}
"#,
        )
        .unwrap();
        let status = crate::proc::hidden_command("rustc")
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .status()
            .unwrap();
        assert!(status.success());

        let error = inspect_runtime(&executable).unwrap_err();

        assert!(error.contains("output limit"), "unexpected error: {error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn free_port_prefers_the_requested_one_then_walks_upward() {
        // Requested port free -> use it.
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let taken = listener.local_addr().unwrap().port();
        drop(listener);
        assert_eq!(pick_free_port("127.0.0.1", taken).unwrap(), taken);

        // Requested port occupied -> the next free port above it.
        let held = TcpListener::bind(("127.0.0.1", taken)).unwrap();
        let next = pick_free_port("127.0.0.1", taken).unwrap();
        assert!(next > taken, "expected a port above {taken}, got {next}");
        assert!(
            TcpListener::bind(("127.0.0.1", next)).is_ok(),
            "returned port {next} was not actually free"
        );
        drop(held);
    }

    #[test]
    fn port_probe_rejects_an_occupied_port() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();

        let error = probe_port_available("127.0.0.1", port).unwrap_err();

        assert!(
            error.contains(&port.to_string()),
            "unexpected error: {error}"
        );
        assert!(error.contains("not available"), "unexpected error: {error}");
    }

    #[test]
    fn companions_of_one_role_are_ranked_so_the_closest_quant_leads() {
        // Real case: DeepSeek-V4-Flash-0731 ships 8 DSpark drafts (7 in a
        // `dspark/` subfolder plus one at the family root). All are genuine
        // companions, but the list must be ordered so the variant matching the
        // target's own quantisation is offered first instead of an arbitrary one.
        let root = std::env::temp_dir().join(format!("gguf-pilot-rank-{}", std::process::id()));
        let family = root.join("DeepSeek-V4-Flash-0731");
        let drafts = family.join("dspark");
        let target_dir = family.join("UD-IQ4_XS");
        fs::create_dir_all(&drafts).unwrap();
        fs::create_dir_all(&target_dir).unwrap();
        for index in 1..=2 {
            fs::write(
                target_dir.join(format!(
                    "DeepSeek-V4-Flash-0731-UD-IQ4_XS-{index:05}-of-00002.gguf"
                )),
                b"x",
            )
            .unwrap();
        }
        for name in [
            "dspark-DeepSeek-V4-Flash-0731-BF16.gguf",
            "dspark-DeepSeek-V4-Flash-0731-Q6_K.gguf",
            "dspark-DeepSeek-V4-Flash-0731-Q4_K_M-hybrid.gguf",
        ] {
            fs::write(drafts.join(name), b"x").unwrap();
        }
        fs::write(family.join("dspark-DeepSeek-V4-Flash-0731-Q8_0.gguf"), b"x").unwrap();

        let models = scan_models(&root).unwrap();
        let target = models
            .iter()
            .find(|m| m.name.contains("UD-IQ4_XS"))
            .expect("target model");
        let dspark: Vec<&Companion> = target
            .companions
            .iter()
            .filter(|c| c.role == "dspark")
            .collect();
        assert_eq!(dspark.len(), 4, "every real companion is still listed");
        assert!(
            dspark[0].name.contains("Q4_K_M"),
            "closest quant should lead, got {}",
            dspark[0].name
        );
        // Deterministic order, so the UI and the suggested profile agree.
        let again = scan_models(&root).unwrap();
        let again_names: Vec<&str> = again
            .iter()
            .find(|m| m.name.contains("UD-IQ4_XS"))
            .unwrap()
            .companions
            .iter()
            .filter(|c| c.role == "dspark")
            .map(|c| c.name.as_str())
            .collect();
        let first_names: Vec<&str> = dspark.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(first_names, again_names);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn quant_weight_reads_the_primary_label_not_a_provenance_suffix() {
        // "Q6_K-from-BF16" is a Q6_K model derived from BF16; the leading label
        // is the one that describes the file.
        assert_eq!(
            quant_weight("dspark-X-Q6_K-from-BF16-hybrid.gguf"),
            Some(6.6)
        );
        assert_eq!(quant_weight("dspark-X-Q6_K-from-Q8.gguf"), Some(6.6));
        assert_eq!(quant_weight("dspark-X-BF16.gguf"), Some(16.0));
        assert_eq!(quant_weight("X-UD-IQ4_XS-00001-of-00004.gguf"), Some(4.25));
        assert_eq!(quant_weight("no-quant-here.gguf"), None);
    }

    /// Opt-in probe against the operator's real model tree. Ignored by default
    /// because it depends on local files:
    /// `cargo test real_model_tree -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_model_tree_companion_order() {
        let root = std::path::PathBuf::from(r"C:\models");
        if !root.is_dir() {
            eprintln!("skipped: {} not present", root.display());
            return;
        }
        for model in scan_models(&root).unwrap() {
            if model.companions.len() > 1 {
                println!(
                    "{} (quant {}, {} companions)",
                    model.name,
                    model.quant,
                    model.companions.len()
                );
                for c in &model.companions {
                    println!("   {:8} {}", c.role, c.name);
                }
            }
        }
    }

    #[test]
    fn scan_groups_complete_shards_and_marks_companions() {
        let root = std::env::temp_dir().join(format!("gguf-pilot-scan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("ModelA")).unwrap();
        fs::write(root.join("ModelA/ModelA-Q4-00001-of-00002.gguf"), b"a").unwrap();
        fs::write(root.join("ModelA/ModelA-Q4-00002-of-00002.gguf"), b"b").unwrap();
        fs::write(root.join("ModelA/mmproj-ModelA-F16.gguf"), b"v").unwrap();
        fs::write(root.join("ModelA/ModelA-DSpark-F16.gguf"), b"d").unwrap();

        let models = scan_models(Path::new(&root)).unwrap();
        assert_eq!(models.len(), 1);
        let model = &models[0];
        assert_eq!(model.shard_count, 2);
        assert_eq!(model.expected_shards, 2);
        assert!(model.complete);
        assert_eq!(
            model
                .shards
                .iter()
                .map(|shard| shard.shard_index)
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2)]
        );
        assert_eq!(
            model
                .shards
                .iter()
                .map(|shard| shard.size_bytes)
                .sum::<u64>(),
            model.size_bytes
        );
        assert_eq!(model.companions.len(), 2);
        assert!(model.companions.iter().any(|c| c.role == "mmproj"));
        assert!(model.companions.iter().any(|c| c.role == "dspark"));

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn scan_does_not_follow_a_directory_reparse_point() {
        use std::os::windows::fs::symlink_dir;

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("gguf-pilot-scan-root-{nonce}"));
        let outside = std::env::temp_dir().join(format!("gguf-pilot-scan-outside-{nonce}"));
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("private-Q4_K_M.gguf"), b"outside").unwrap();
        let junction = root.join("junction");
        if symlink_dir(&outside, &junction).is_err() {
            let status = crate::proc::hidden_command("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(&junction)
                .arg(&outside)
                .status()
                .unwrap();
            assert!(status.success());
        }

        let models = scan_models(&root).unwrap();

        assert!(models.is_empty());
        fs::remove_dir(junction).unwrap();
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }

    #[test]
    fn scan_rejects_conflicting_expected_shard_counts() {
        let root = std::env::temp_dir().join(format!(
            "gguf-pilot-conflicting-shards-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("ModelA")).unwrap();
        fs::write(root.join("ModelA/ModelA-Q4-00001-of-00002.gguf"), b"a").unwrap();
        fs::write(root.join("ModelA/ModelA-Q4-00002-of-00003.gguf"), b"b").unwrap();

        let models = scan_models(&root).unwrap();

        assert_eq!(models.len(), 1);
        assert!(!models[0].complete);
        assert_eq!(models[0].expected_shards, 3);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_attaches_companion_from_nested_folder_to_parent_target() {
        let root = std::env::temp_dir().join(format!("gguf-pilot-nested-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Qwen/MTP")).unwrap();
        fs::write(root.join("Qwen/Qwen-Q6.gguf"), b"target").unwrap();
        fs::write(root.join("Qwen/MTP/mtp-Qwen-Q4.gguf"), b"draft").unwrap();

        let models = scan_models(&root).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].companions.len(), 1);
        assert_eq!(models[0].companions[0].role, "mtp");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_attaches_companion_from_sibling_folder_in_same_family() {
        let root = std::env::temp_dir().join(format!("gguf-pilot-sibling-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("DeepSeek/UD-IQ4_XS")).unwrap();
        fs::create_dir_all(root.join("DeepSeek/dspark")).unwrap();
        fs::write(root.join("DeepSeek/UD-IQ4_XS/DeepSeek-IQ4.gguf"), b"target").unwrap();
        fs::write(
            root.join("DeepSeek/dspark/dspark-DeepSeek-Q8.gguf"),
            b"draft",
        )
        .unwrap();

        let models = scan_models(&root).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].companions.len(), 1);
        assert_eq!(models[0].companions[0].role, "dspark");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn profile_builds_explicit_dspark_command() {
        let profile = LaunchProfile {
            name: "LFM fast".into(),
            runtime: r"C:\llama\llama-server.exe".into(),
            model: r"C:\models\LFM\target.gguf".into(),
            draft_model: Some(r"C:\models\LFM\draft.gguf".into()),
            mmproj: None,
            host: "127.0.0.1".into(),
            port: 3339,
            alias: "lfm-local".into(),
            context: 4096,
            parallel: 1,
            gpu_layers: "all".into(),
            cpu_moe: 0,
            batch: 2048,
            ubatch: 512,
            flash_attention: "on".into(),
            fit: false,
            spec_type: "draft-dspark".into(),
            draft_max: 5,
            ..LaunchProfile::default()
        };

        let args = profile.build_args().unwrap();
        assert!(args
            .windows(2)
            .any(|w| w == ["--spec-type", "draft-dspark"]));
        assert!(args
            .windows(2)
            .any(|w| w == ["-md", r"C:\models\LFM\draft.gguf"]));
        assert!(args.windows(2).any(|w| w == ["--alias", "lfm-local"]));
        assert!(args.contains(&"--fit".to_string()));
        assert!(args.contains(&"off".to_string()));
    }

    #[test]
    fn profile_emits_useful_runtime_and_advanced_options() {
        let profile = LaunchProfile {
            model: r"C:\models\target.gguf".into(),
            alias: "target-local".into(),
            threads: 16,
            threads_batch: 32,
            flash_attention: "auto".into(),
            cache_type_k: "q8_0".into(),
            cache_type_v: "q8_0".into(),
            load_mode: "mmap+mlock".into(),
            lazy_mode: "auto".into(),
            split_mode: "layer".into(),
            cache_prompt: true,
            continuous_batching: true,
            reasoning: "auto".into(),
            reasoning_effort: "high".into(),
            temperature: 1.0,
            top_p: 0.95,
            spec_type: "ngram-mod".into(),
            ngram_match: 24,
            ngram_min: 48,
            ngram_max: 64,
            ..LaunchProfile::default()
        };

        let args = profile.build_args().unwrap();
        for expected in [
            ["-t", "16"],
            ["-tb", "32"],
            ["-ctk", "q8_0"],
            ["-ctv", "q8_0"],
            ["--load-mode", "mmap+mlock"],
            ["--reasoning", "auto"],
            ["--reasoning-effort", "high"],
            ["--spec-ngram-mod-n-match", "24"],
        ] {
            assert!(args.windows(2).any(|window| window == expected));
        }
        assert!(args.contains(&"--cache-prompt".to_string()));
        assert!(args.contains(&"--cont-batching".to_string()));
    }

    #[test]
    fn non_loopback_profile_requires_authentication_and_restricted_cors() {
        let mut profile = LaunchProfile {
            model: r"C:\models\target.gguf".into(),
            alias: "lan-model".into(),
            host: "0.0.0.0".into(),
            ..LaunchProfile::default()
        };
        assert!(profile.build_args().unwrap_err().contains("API key file"));

        profile.api_key_file = r"C:\secrets\llama.keys".into();
        profile.cors_origins = "*".into();
        assert!(profile.build_args().unwrap_err().contains("CORS"));

        profile.cors_origins = "http://192.168.1.20:3000".into();
        assert!(profile.build_args().is_ok());
    }

    #[test]
    fn runtime_filter_omits_options_not_advertised_by_selected_build() {
        let args = vec![
            "-m".into(),
            "model.gguf".into(),
            "--load-mode".into(),
            "auto".into(),
            "--lazy-mode".into(),
            "auto".into(),
            "--metrics".into(),
        ];
        let supported = vec!["-m".into(), "--load-mode".into(), "--metrics".into()];
        let (filtered, omitted) = filter_supported_args(&args, &supported);
        assert!(filtered.contains(&"--load-mode".to_string()));
        assert!(!filtered.contains(&"--lazy-mode".to_string()));
        assert_eq!(omitted, vec!["--lazy-mode"]);
    }

    #[test]
    fn launch_argument_validation_explains_each_rejected_flag() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            extra_args: vec!["--experimental-fixture".into()],
            ..LaunchProfile::default()
        };
        let raw = profile.build_args().unwrap();
        let supported_flags = raw
            .iter()
            .filter(|token| token.starts_with('-'))
            .filter(|token| token.as_str() != "--experimental-fixture")
            .cloned()
            .collect::<Vec<_>>();
        let capabilities = RuntimeCapabilities {
            path: profile.runtime.clone(),
            version: "fixture".into(),
            build: "fixture".into(),
            commit: "fixture".into(),
            help_sha256: "a".repeat(64),
            spec_types: Vec::new(),
            supported_flags,
            metrics: false,
            multimodal: true,
            fit: true,
        };

        let validation = validate_launch_arguments(&profile, &capabilities).unwrap();

        assert_eq!(validation.rejected.len(), 1);
        assert_eq!(validation.rejected[0].flag, "--experimental-fixture");
        assert!(validation.rejected[0].reason.contains("does not advertise"));
        assert!(!validation.command.contains("--experimental-fixture"));
    }

    #[test]
    fn launch_argument_validation_rejects_an_unsupported_required_model_flag() {
        let profile = LaunchProfile {
            alias: "fixture".into(),
            ..LaunchProfile::default()
        };
        let supported_flags = profile
            .build_args()
            .unwrap()
            .into_iter()
            .filter(|token| token.starts_with('-') && token != "-m")
            .collect();
        let capabilities = RuntimeCapabilities {
            path: profile.runtime.clone(),
            version: "fixture".into(),
            build: "fixture".into(),
            commit: "fixture".into(),
            help_sha256: "a".repeat(64),
            spec_types: Vec::new(),
            supported_flags,
            metrics: true,
            multimodal: true,
            fit: true,
        };

        let error = validate_launch_arguments(&profile, &capabilities).unwrap_err();

        assert!(error.contains("-m"), "unexpected error: {error}");
        assert!(error.contains("required"), "unexpected error: {error}");
    }

    #[test]
    fn manifest_arguments_fingerprint_sensitive_path_values() {
        let first = vec![
            "--api-key-file".into(),
            r"C:\private\keys.txt".into(),
            "--ctx-size".into(),
            "4096".into(),
        ];
        let second = vec![
            "--api-key-file".into(),
            r"C:\other\keys.txt".into(),
            "--ctx-size".into(),
            "4096".into(),
        ];

        let first_safe = manifest_safe_args(&first);
        let second_safe = manifest_safe_args(&second);

        assert!(!first_safe.join(" ").contains("private"));
        assert!(!first_safe.join(" ").contains("keys.txt"));
        assert_ne!(first_safe, second_safe);
        assert_eq!(first_safe[2..], ["--ctx-size", "4096"]);
    }

    #[test]
    fn manifest_arguments_redact_direct_api_keys() {
        let secret = "super-secret-benchmark-key";
        let args = vec![
            "--api-key".into(),
            secret.into(),
            format!("--api-key={secret}"),
        ];

        let safe = manifest_safe_args(&args);

        assert!(!safe.iter().any(|value| value.contains(secret)));
        assert_eq!(safe[1], "[REDACTED]");
        assert_eq!(safe[2], "--api-key=[REDACTED]");
    }

    #[test]
    fn profile_rejects_inconsistent_advanced_ranges() {
        let mut profile = LaunchProfile {
            model: r"C:\models\target.gguf".into(),
            alias: "invalid-ranges".into(),
            spec_type: "draft-mtp".into(),
            draft_min: 6,
            draft_max: 3,
            ..LaunchProfile::default()
        };
        assert!(profile.build_args().unwrap_err().contains("draft minimum"));

        profile.draft_min = 0;
        profile.image_min_tokens = 2048;
        profile.image_max_tokens = 1024;
        assert!(profile.build_args().unwrap_err().contains("image token"));
    }

    #[test]
    fn profile_rejects_malformed_gpu_placement_values() {
        for (gpu_layers, tensor_split) in [
            ("not-a-layer-count", ""),
            ("all", "0.5,,0.5"),
            ("all", "0,0"),
            ("all", "0.5,not-a-fraction"),
        ] {
            let profile = LaunchProfile {
                alias: "fixture".into(),
                model: "fixture.gguf".into(),
                gpu_layers: gpu_layers.into(),
                tensor_split: tensor_split.into(),
                ..LaunchProfile::default()
            };

            assert!(
                profile.build_args().is_err(),
                "accepted gpu_layers={gpu_layers:?}, tensor_split={tensor_split:?}"
            );
        }
    }

    #[test]
    fn capability_parser_uses_runtime_help_as_truth() {
        let help = "--spec-type none,draft-mtp,draft-dspark,ngram-mod\n--metrics enable metrics\n--mmproj FILE";
        let caps = parse_capabilities("version: build 10679, commit abc123", help);
        assert_eq!(caps.build, "10679");
        assert_eq!(caps.commit, "abc123");
        assert_eq!(
            caps.spec_types,
            vec!["none", "draft-mtp", "draft-dspark", "ngram-mod"]
        );
        assert!(caps.metrics);
        assert!(caps.multimodal);
        assert!(!caps.spec_types.iter().any(|x| x == "draft-dspark2"));
    }

    #[test]
    fn runtime_help_digest_preserves_exact_probe_bytes() {
        let compact = parse_capabilities("version: test", "--model FILE\n--metrics");
        let spaced = parse_capabilities("version: test", "--model FILE  \n--metrics");

        assert_eq!(compact.supported_flags, spaced.supported_flags);
        assert_ne!(compact.help_sha256, spaced.help_sha256);
    }

    #[test]
    fn benchmark_summary_reports_stable_statistics() {
        let summary = summarize_benchmark(vec![360.0, 362.0, 361.0], 2048, 3).unwrap();
        assert_eq!(summary.mean_tps, 361.0);
        assert_eq!(summary.median_tps, 361.0);
        assert_eq!(summary.min_tps, 360.0);
        assert_eq!(summary.max_tps, 362.0);
        assert_eq!(summary.tokens, 2048);
        assert_eq!(summary.repeats, 3);
    }

    #[test]
    fn timing_parser_accepts_llama_completion_response() {
        let body = r#"{"timings":{"predicted_per_second":478.25}}"#;
        assert_eq!(parse_tps(body).unwrap(), 478.25);
    }
}
