use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

/// Schema tag for the canonical execution snapshot. A
/// compatibility key carries this prefix plus the SHA-256 of the canonical
/// snapshot JSON, so legacy keys cannot silently masquerade as a current
/// complete identity.
pub const EXECUTION_SNAPSHOT_SCHEMA: &str = "localmotive.execution-snapshot.v2";

/// Snapshot scopes.
pub const SNAPSHOT_SCOPE_LAUNCH: &str = "launch";
pub const SNAPSHOT_SCOPE_LAUNCH_WORKLOAD: &str = "launch+workload";
pub const EXECUTION_KEY_PREFIX: &str = "v2:";
/// Estimator/metric identity: bump when the calibration arithmetic or the
/// measured statistic changes meaning.
pub const ESTIMATOR_VERSION: &str = "decode-tps-mean.v1";

/// The versioned canonical execution snapshot: every material influence on a
/// measurement, derived from the effective launch arguments plus observed
/// facts, never a hand-maintained subset of profile fields.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionSnapshotV2 {
    pub schema_version: String,
    /// What the snapshot scope covers: `launch` for launch-only identities
    /// or `launch+workload` for measured-run identities. The scope is part of
    /// identity and a workload identity can never collide.
    pub scope: String,
    /// The effective launch arguments after capability filtering, with secret
    /// values and volatile paths replaced by identity tokens.
    pub effective_args: Vec<String>,
    pub model_content_sha256: String,
    pub model_architecture: String,
    pub runtime_sha256: String,
    pub runtime_help_sha256: String,
    pub runtime_backend: String,
    pub runtime_version: String,
    pub runtime_build: String,
    pub adapter_ids: Vec<String>,
    pub driver_versions: Vec<String>,
    pub host_cpu_model: String,
    pub host_platform: String,
    pub host_memory_bytes: u64,
    pub context_requested: u32,
    pub context_effective: Option<u32>,
    pub draft_model_sha256: String,
    pub mmproj_sha256: String,
    pub lora_sha256: String,
    pub workload_sha256: String,
    pub harness_version: String,
    pub estimator_version: String,
    /// Material facts this machine could not observe. A non-empty list makes
    /// the snapshot insufficient for identity reuse.
    pub unknown_identities: Vec<String>,
}

impl ExecutionSnapshotV2 {
    pub fn reuse_supported(&self) -> bool {
        self.unknown_identities.is_empty()
    }
}

/// Compute the canonical key for a snapshot: `v2:` plus the SHA-256 of the
/// canonical JSON. Every field participates, so any material change yields a
/// different key.
pub fn execution_snapshot_key(snapshot: &ExecutionSnapshotV2) -> Result<String, String> {
    if snapshot.schema_version != EXECUTION_SNAPSHOT_SCHEMA {
        return Err(format!(
            "Unsupported execution snapshot schema: {}",
            snapshot.schema_version
        ));
    }
    if snapshot.scope != SNAPSHOT_SCOPE_LAUNCH && snapshot.scope != SNAPSHOT_SCOPE_LAUNCH_WORKLOAD {
        return Err(format!(
            "Unsupported execution snapshot scope: {}",
            snapshot.scope
        ));
    }
    crate::evidence::validate_sha256("modelContentSha256", &snapshot.model_content_sha256)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("runtimeSha256", &snapshot.runtime_sha256)
        .map_err(|error| error.to_string())?;
    crate::evidence::validate_sha256("runtimeHelpSha256", &snapshot.runtime_help_sha256)
        .map_err(|error| error.to_string())?;
    if snapshot.scope == SNAPSHOT_SCOPE_LAUNCH_WORKLOAD {
        crate::evidence::validate_sha256("workloadSha256", &snapshot.workload_sha256)
            .map_err(|error| error.to_string())?;
    } else if !snapshot.workload_sha256.is_empty() {
        return Err("A launch-scope execution snapshot must not carry a workload digest".into());
    }
    if snapshot.model_architecture.trim().is_empty()
        || snapshot.runtime_backend.trim().is_empty()
        || snapshot.runtime_version.trim().is_empty()
        || snapshot.runtime_build.trim().is_empty()
        || snapshot.harness_version.trim().is_empty()
        || snapshot.estimator_version.trim().is_empty()
    {
        return Err(
            "Execution snapshot requires model, runtime, harness and estimator identities".into(),
        );
    }
    if snapshot.adapter_ids.len() != snapshot.driver_versions.len() {
        return Err("Execution snapshot adapter and driver identities must align".into());
    }
    let canonical = serde_json::to_vec(snapshot).map_err(|error| error.to_string())?;
    let digest = Sha256::digest(canonical);
    Ok(format!("{EXECUTION_KEY_PREFIX}{}", hex::encode(digest)))
}

/// Compatibility keys must carry the current schema prefix; a legacy key is
/// explicitly insufficient evidence for reuse.
pub fn validate_compatibility_key(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    let Some(hex_digest) = trimmed.strip_prefix(EXECUTION_KEY_PREFIX) else {
        return Err(
            "This record uses a legacy compatibility identity; rebuild it from current measurements"
                .into(),
        );
    };
    if hex_digest.len() != 64
        || !hex_digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err("Compatibility key is not a v2 sha256 digest".into());
    }
    Ok(())
}

/// Replace secret values and volatile paths in the effective launch arguments
/// with stable identity tokens. The API-key and TLS values never enter the
/// snapshot; file-backed influences are represented by their content digests
/// carried elsewhere in the snapshot.
pub fn sanitize_effective_args(args: &[String]) -> Vec<String> {
    let mut sanitized = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        let argument = args[index].as_str();
        let next = args.get(index + 1).map(String::as_str);
        let token = match argument {
            "-m" | "--model" => Some("[model]"),
            "--mmproj" => Some("[mmproj]"),
            "--lora" | "--lora-scaled" => Some("[lora]"),
            // The short draft flags carry local draft paths exactly like the
            // long forms.
            "-md" | "-mdl" | "--draft-model" | "--spec-draft-model" | "--model-draft" => {
                Some("[draft-model]")
            }
            // Any other path-bearing value the launch builder can emit.
            "--chat-template-file" => Some("[configured]"),
            "--api-key-file" | "--ssl-key-file" | "--ssl-cert-file" => Some("[configured]"),
            value if value.ends_with("draft-model") || value.ends_with("model-draft") => {
                Some("[draft-model]")
            }
            _ => None,
        };
        sanitized.push(argument.to_string());
        if let (Some(token), Some(_)) = (token, next) {
            sanitized.push(token.to_string());
            index += 2;
            continue;
        }
        index += 1;
    }
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn snapshot_fixture() -> ExecutionSnapshotV2 {
        ExecutionSnapshotV2 {
            schema_version: EXECUTION_SNAPSHOT_SCHEMA.into(),
            scope: SNAPSHOT_SCOPE_LAUNCH_WORKLOAD.into(),
            effective_args: vec![
                "-m".into(),
                "[model]".into(),
                "--threads".into(),
                "8".into(),
            ],
            model_content_sha256: "a".repeat(64),
            model_architecture: "qwen35".into(),
            runtime_sha256: "b".repeat(64),
            runtime_help_sha256: "c".repeat(64),
            runtime_backend: "vulkan".into(),
            runtime_version: "1.0".into(),
            runtime_build: "b10816".into(),
            adapter_ids: vec!["luid:0000:1111".into()],
            driver_versions: vec!["32.0.15.7283".into()],
            host_cpu_model: "AMD Ryzen 9 9950X3D".into(),
            host_platform: "windows".into(),
            host_memory_bytes: 64 * 1024 * 1024 * 1024,
            context_requested: 32768,
            context_effective: Some(32768),
            draft_model_sha256: String::new(),
            mmproj_sha256: String::new(),
            lora_sha256: String::new(),
            workload_sha256: "d".repeat(64),
            harness_version: "0.5.0".into(),
            estimator_version: ESTIMATOR_VERSION.into(),
            unknown_identities: Vec::new(),
        }
    }

    #[test]
    fn snapshot_key_changes_for_every_material_field() {
        let baseline = snapshot_fixture();
        assert!(baseline.reuse_supported());
        let baseline_key = execution_snapshot_key(&baseline).unwrap();
        assert!(baseline_key.starts_with(EXECUTION_KEY_PREFIX));

        type SnapshotMutation = (&'static str, Box<dyn Fn(&mut ExecutionSnapshotV2)>);
        let mutations: Vec<SnapshotMutation> = vec![
            (
                "effective_args",
                Box::new(|s| s.effective_args.push("--flash-attn".into())),
            ),
            (
                "model_content_sha256",
                Box::new(|s| s.model_content_sha256 = "e".repeat(64)),
            ),
            (
                "model_architecture",
                Box::new(|s| s.model_architecture = "llama".into()),
            ),
            (
                "runtime_sha256",
                Box::new(|s| s.runtime_sha256 = "e".repeat(64)),
            ),
            (
                "runtime_help_sha256",
                Box::new(|s| s.runtime_help_sha256 = "e".repeat(64)),
            ),
            (
                "runtime_backend",
                Box::new(|s| s.runtime_backend = "cuda".into()),
            ),
            (
                "runtime_version",
                Box::new(|s| s.runtime_version = "2.0".into()),
            ),
            (
                "runtime_build",
                Box::new(|s| s.runtime_build = "b20000".into()),
            ),
            (
                "adapter_ids",
                Box::new(|s| {
                    s.adapter_ids.push("luid:0000:2222".into());
                    s.driver_versions.push("32.0.15.7283".into());
                }),
            ),
            (
                "driver_versions",
                Box::new(|s| s.driver_versions[0] = "32.0.15.9999".into()),
            ),
            (
                "host_cpu_model",
                Box::new(|s| s.host_cpu_model = "Intel Core i9".into()),
            ),
            (
                "host_platform",
                Box::new(|s| s.host_platform = "linux".into()),
            ),
            (
                "host_memory_bytes",
                Box::new(|s| s.host_memory_bytes = 32 * 1024 * 1024 * 1024),
            ),
            (
                "context_requested",
                Box::new(|s| s.context_requested = 8192),
            ),
            (
                "context_effective",
                Box::new(|s| s.context_effective = Some(8192)),
            ),
            (
                "draft_model_sha256",
                Box::new(|s| s.draft_model_sha256 = "e".repeat(64)),
            ),
            (
                "mmproj_sha256",
                Box::new(|s| s.mmproj_sha256 = "e".repeat(64)),
            ),
            ("lora_sha256", Box::new(|s| s.lora_sha256 = "e".repeat(64))),
            (
                "workload_sha256",
                Box::new(|s| s.workload_sha256 = "e".repeat(64)),
            ),
            (
                "harness_version",
                Box::new(|s| s.harness_version = "0.6.0".into()),
            ),
            (
                "estimator_version",
                Box::new(|s| s.estimator_version = "other.v9".into()),
            ),
        ];
        for (label, mutate) in mutations {
            let mut changed = baseline.clone();
            mutate(&mut changed);
            let key = execution_snapshot_key(&changed).unwrap();
            assert_ne!(key, baseline_key, "field {label} did not change the key");
        }
    }

    #[test]
    fn cpu_only_and_hardware_changes_are_distinguished() {
        // CPU-only machine: no adapters, empty driver list, cpu backend.
        let mut cpu_only = snapshot_fixture();
        cpu_only.runtime_backend = "cpu".into();
        cpu_only.adapter_ids.clear();
        cpu_only.driver_versions.clear();
        cpu_only.host_cpu_model = "AMD Ryzen 9 9950X3D".into();
        let cpu_key = execution_snapshot_key(&cpu_only).unwrap();

        // The same GPU with a changed CPU must not share the identity.
        let mut changed_cpu = snapshot_fixture();
        changed_cpu.host_cpu_model = "Intel Core Ultra 9".into();
        assert_ne!(
            execution_snapshot_key(&changed_cpu).unwrap(),
            execution_snapshot_key(&snapshot_fixture()).unwrap()
        );

        // Fit-reduced effective context changes the identity.
        let mut fit_reduced = snapshot_fixture();
        fit_reduced.context_requested = 32768;
        fit_reduced.context_effective = Some(16384);
        assert_ne!(
            execution_snapshot_key(&fit_reduced).unwrap(),
            execution_snapshot_key(&snapshot_fixture()).unwrap()
        );

        // The same draft companion file with changed draft settings differs
        // through the effective arguments.
        let mut draft_a = snapshot_fixture();
        draft_a.draft_model_sha256 = "f".repeat(64);
        draft_a
            .effective_args
            .extend(["--spec-draft-n-max".into(), "16".into()]);
        let mut draft_b = draft_a.clone();
        draft_b.effective_args.pop();
        draft_b.effective_args.push("64".into());
        assert_ne!(
            execution_snapshot_key(&draft_a).unwrap(),
            execution_snapshot_key(&draft_b).unwrap()
        );

        // Changed LoRA bytes under the same filename change the digest.
        let mut lora_a = snapshot_fixture();
        lora_a.lora_sha256 = "1".repeat(64);
        let mut lora_b = snapshot_fixture();
        lora_b.lora_sha256 = "2".repeat(64);
        assert_ne!(
            execution_snapshot_key(&lora_a).unwrap(),
            execution_snapshot_key(&lora_b).unwrap()
        );

        assert!(cpu_key.starts_with(EXECUTION_KEY_PREFIX));
    }

    #[test]
    fn unknown_identity_blocks_reuse_and_legacy_keys_stay_out() {
        let mut partial = snapshot_fixture();
        partial.unknown_identities = vec!["driverVersion:luid:0000:1111".into()];
        assert!(!partial.reuse_supported());
        let key = execution_snapshot_key(&partial).unwrap();
        assert!(validate_compatibility_key(&key).is_ok());
        assert!(
            validate_compatibility_key(&"a".repeat(64)).is_err(),
            "legacy keys must not be accepted for snapshot reuse"
        );
    }

    #[test]
    fn effective_arguments_are_sanitized_of_secrets_and_paths() {
        let args = vec![
            "-m".to_string(),
            "C:/models/model.gguf".to_string(),
            "--api-key-file".to_string(),
            "C:/secrets/llama.key".to_string(),
            "--ssl-cert-file".to_string(),
            "C:/secrets/cert.pem".to_string(),
            "--ssl-key-file".to_string(),
            "C:/secrets/key.pem".to_string(),
            "--lora".to_string(),
            "C:/models/adapter.gguf".to_string(),
            "--model-draft".to_string(),
            "C:/models/draft.gguf".to_string(),
            "--threads".to_string(),
            "8".to_string(),
        ];
        let sanitized = sanitize_effective_args(&args);
        let joined = sanitized.join(" ");
        for secret in [
            "llama.key",
            "cert.pem",
            "key.pem",
            "adapter.gguf",
            "draft.gguf",
            "model.gguf",
        ] {
            assert!(
                !joined.contains(secret),
                "sanitized args leaked {secret}: {joined}"
            );
        }
        assert!(joined.contains("[model]"));
        assert!(joined.contains("[configured]"));
        assert!(joined.contains("[lora]"));
        assert!(joined.contains("[draft-model]"));
        assert!(joined.contains("--threads"));
        assert!(joined.contains("8"));
    }

    #[test]
    fn model_set_digest_changes_when_a_selected_companion_changes() {
        let first = selected_artifact_set_sha256(&["a".repeat(64), "b".repeat(64)]).unwrap();
        let second = selected_artifact_set_sha256(&["a".repeat(64), "c".repeat(64)]).unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn sanitizer_covers_every_path_bearing_flag_with_canaries() {
        // One canary per path-bearing flag the launch builder can emit; none
        // may survive sanitization.
        let canary = |name: &str| format!("C:\\Users\\canary-user\\secret-{name}\\file.bin");
        let flags = [
            "-m",
            "--model",
            "--mmproj",
            "--lora",
            "--lora-scaled",
            "-md",
            "--spec-draft-model",
            "--chat-template-file",
            "--api-key-file",
            "--ssl-key-file",
            "--ssl-cert-file",
        ];
        let mut args = Vec::new();
        for flag in flags {
            args.push(flag.to_string());
            args.push(canary(flag.trim_start_matches('-')));
        }
        args.push("--port".into());
        args.push("8080".into());
        let sanitized = sanitize_effective_args(&args);
        let joined = sanitized.join(" ");
        assert!(
            !joined.contains("canary-user"),
            "a local path survived sanitization: {joined}"
        );
        for flag in flags {
            let position = sanitized
                .iter()
                .position(|token| token == flag)
                .unwrap_or_else(|| panic!("{flag} missing from {joined}"));
            assert!(
                sanitized
                    .get(position + 1)
                    .is_some_and(|value| value.starts_with('[')),
                "{flag} value was not replaced: {joined}"
            );
        }
        // Non-path values survive untouched.
        assert!(sanitized.windows(2).any(|pair| pair == ["--port", "8080"]));
    }
}
