use serde::{Deserialize, Serialize};
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
    pub companions: Vec<Companion>,
}

/// First free TCP port at or above `preferred` on `host`, scanning a bounded
/// window. Used so the default 8080 does not collide with anything already
/// listening (another llama-server, a dev server, a proxy).
pub fn pick_free_port(host: &str, preferred: u16) -> Result<u16, String> {
    let bind_host = if host == "0.0.0.0" { "127.0.0.1" } else { host };
    let start = preferred.max(1);
    for candidate in start..start.saturating_add(200) {
        if TcpListener::bind((bind_host, candidate)).is_ok() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "No free port found between {start} and {} on {host}",
        start.saturating_add(200)
    ))
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
        if path.is_dir() {
            collect_gguf(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("gguf"))
        {
            out.push(path);
        }
    }
    Ok(())
}

pub fn scan_models(root: &Path) -> Result<Vec<LogicalModel>, String> {
    if !root.is_dir() {
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
        shards.sort();
        let first = shards.first().unwrap();
        let (_, expected) = shard_key(&first.file_name().unwrap().to_string_lossy());
        let size_bytes = shards
            .iter()
            .map(|p| p.metadata().map(|m| m.len()).unwrap_or(0))
            .sum();
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
            complete: shards.len() == expected,
            quant,
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
        if self.context == 0 || self.parallel == 0 || self.batch == 0 || self.ubatch == 0 {
            return Err("Context, slots, batch, and uBatch must be greater than zero".into());
        }
        if self.ubatch > self.batch {
            return Err("Physical uBatch cannot exceed logical batch size".into());
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
        let loopback = matches!(self.host.as_str(), "127.0.0.1" | "localhost" | "::1");
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCapabilities {
    pub path: String,
    pub version: String,
    pub build: String,
    pub commit: String,
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

pub fn filter_supported_args(
    args: &[String],
    supported_flags: &[String],
) -> (Vec<String>, Vec<String>) {
    let supported = supported_flags
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let is_flag = |token: &str| {
        token.starts_with('-')
            && token.len() > 1
            && token
                .as_bytes()
                .get(1)
                .is_some_and(|byte| !byte.is_ascii_digit())
    };
    let mut filtered = Vec::new();
    let mut omitted = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        if is_flag(token) && !supported.contains(token.as_str()) {
            omitted.push(token.clone());
            index += 1;
            if index < args.len() && !is_flag(&args[index]) {
                index += 1;
            }
            continue;
        }
        filtered.push(token.clone());
        index += 1;
    }
    (filtered, omitted)
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
        spec_types,
        supported_flags: parse_supported_flags(help),
        metrics: help.contains("--metrics"),
        multimodal: help.contains("--mmproj"),
        fit: help.contains("--fit"),
    }
}

pub fn inspect_runtime(path: &Path) -> Result<RuntimeCapabilities, String> {
    if !path.is_file() {
        return Err(format!("Runtime does not exist: {}", path.display()));
    }
    let run = |arg: &str| {
        crate::proc::hidden_command(path)
            .arg(arg)
            .output()
            .map_err(|e| e.to_string())
            .map(|o| String::from_utf8_lossy(&[o.stdout, o.stderr].concat()).to_string())
    };
    let version = run("--version")?;
    let help = run("--help")?;
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
        assert_eq!(model.companions.len(), 2);
        assert!(model.companions.iter().any(|c| c.role == "mmproj"));
        assert!(model.companions.iter().any(|c| c.role == "dspark"));

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
            extra_args: vec!["--jinja".into()],
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
