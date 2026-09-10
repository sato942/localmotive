//! Typed provenance and validation contracts for v0.3 measurement workflows.
//!
//! Evidence categories describe where a value came from. They are not
//! probabilities and must never be promoted by ranking or display code.

use serde::{Deserialize, Serialize};
use std::fmt;

pub const BENCHMARK_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceLevel {
    Exact,
    Observed,
    Derived,
    UserOverride,
    Catalog,
    Heuristic,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceSourceKind {
    FileSystem,
    GgufMetadata,
    Runtime,
    WindowsApi,
    NvidiaSmi,
    Cim,
    User,
    Catalog,
    Benchmark,
    Calculation,
    Policy,
    Import,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSource {
    pub kind: EvidenceSourceKind,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    UnsupportedSchema,
    MissingValue,
    UnexpectedValue,
    InvalidRange,
    LimitExceeded,
    NonFinite,
    InvalidDigest,
    EmptyIdentity,
    EvidenceUpgrade,
    InconsistentEvidence,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DomainError {
    pub code: ErrorCode,
    pub field: String,
    pub message: String,
}

impl DomainError {
    fn new(code: ErrorCode, field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for DomainError {}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Evidence<T> {
    pub value: Option<T>,
    pub level: EvidenceLevel,
    pub source: EvidenceSource,
    pub observed_at_ms: u64,
    pub notes: Vec<String>,
}

impl<T> Evidence<T> {
    pub fn known(
        value: T,
        level: EvidenceLevel,
        source: EvidenceSource,
        observed_at_ms: u64,
        notes: Vec<String>,
    ) -> Result<Self, DomainError> {
        let evidence = Self {
            value: Some(value),
            level,
            source,
            observed_at_ms,
            notes,
        };
        evidence.validate("evidence.value")?;
        Ok(evidence)
    }

    pub fn unknown(source: EvidenceSource, observed_at_ms: u64, note: impl Into<String>) -> Self {
        Self {
            value: None,
            level: EvidenceLevel::Unknown,
            source,
            observed_at_ms,
            notes: vec![note.into()],
        }
    }

    pub fn derived(
        value: T,
        required_levels: &[EvidenceLevel],
        source: EvidenceSource,
        observed_at_ms: u64,
        mut notes: Vec<String>,
    ) -> Self {
        if required_levels.contains(&EvidenceLevel::Unknown) {
            notes.push("At least one required input is unknown.".into());
            return Self {
                value: None,
                level: EvidenceLevel::Unknown,
                source,
                observed_at_ms,
                notes,
            };
        }
        Self {
            value: Some(value),
            level: EvidenceLevel::Derived,
            source,
            observed_at_ms,
            notes,
        }
    }

    pub fn validate(&self, field: &str) -> Result<(), DomainError> {
        if self.observed_at_ms == 0 {
            return Err(DomainError::new(
                ErrorCode::InvalidRange,
                format!("{field}.observedAtMs"),
                "Evidence requires a nonzero observation timestamp",
            ));
        }
        if self.source.detail.trim().is_empty() || self.source.detail.len() > 1_024 {
            return Err(DomainError::new(
                ErrorCode::InvalidRange,
                format!("{field}.source.detail"),
                "Evidence source detail must contain 1 to 1024 bytes",
            ));
        }
        if self.notes.len() > 32
            || self
                .notes
                .iter()
                .any(|note| note.is_empty() || note.len() > 1_024)
        {
            return Err(DomainError::new(
                ErrorCode::LimitExceeded,
                format!("{field}.notes"),
                "Evidence notes exceed the count or text limit",
            ));
        }
        match (&self.value, self.level) {
            (None, EvidenceLevel::Unknown) => {
                if self.notes.iter().any(|note| !note.trim().is_empty()) {
                    Ok(())
                } else {
                    Err(DomainError::new(
                        ErrorCode::MissingValue,
                        field,
                        "Unknown evidence must name the missing or conflicting fact",
                    ))
                }
            }
            (Some(_), EvidenceLevel::Unknown) => Err(DomainError::new(
                ErrorCode::UnexpectedValue,
                field,
                "Unknown evidence cannot contain a value",
            )),
            (None, _) => Err(DomainError::new(
                ErrorCode::MissingValue,
                field,
                "Known evidence requires a value",
            )),
            (Some(_), _) => Ok(()),
        }
    }
}

impl<T: Clone> Evidence<T> {
    /// Preserve evidence provenance. Derive a new value instead of relabelling.
    pub fn relabel(&self, next: EvidenceLevel) -> Result<Self, DomainError> {
        if next == self.level {
            return Ok(self.clone());
        }
        Err(DomainError::new(
            ErrorCode::EvidenceUpgrade,
            "level",
            "Evidence levels cannot be changed after acquisition",
        ))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FitClass {
    Unknown,
    Estimated,
    PreflightLikely,
    PreflightTight,
    RequiresOffload,
    LaunchValidated,
    Measured,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionPath {
    Unknown,
    FullGpu,
    LayerOffload,
    Cpu,
    UnifiedMemory,
    MultiGpu,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CacheMode {
    Cold,
    #[default]
    Warm,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Workload {
    pub id: String,
    pub prompt_tokens: u32,
    pub generation_tokens: u32,
    pub warmups: u16,
    pub trials: u16,
    pub seed: Option<i64>,
    pub concurrency: u16,
    pub stream: bool,
    pub cache_mode: CacheMode,
    pub timeout_ms: u64,
}

impl Default for Workload {
    fn default() -> Self {
        Self {
            id: "technical-explanation-v1".into(),
            prompt_tokens: 512,
            generation_tokens: 256,
            warmups: 1,
            trials: 5,
            seed: Some(42),
            concurrency: 1,
            stream: false,
            cache_mode: CacheMode::Warm,
            timeout_ms: 600_000,
        }
    }
}

impl Workload {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.id.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::EmptyIdentity,
                "workload.id",
                "Workload identity is required",
            ));
        }
        validate_bounded_u64(
            "workload.promptTokens",
            self.prompt_tokens as u64,
            1,
            1_048_576,
        )?;
        validate_bounded_u64(
            "workload.generationTokens",
            self.generation_tokens as u64,
            1,
            65_536,
        )?;
        validate_bounded_u64("workload.warmups", self.warmups as u64, 0, 10)?;
        validate_bounded_u64("workload.trials", self.trials as u64, 1, 100)?;
        validate_bounded_u64("workload.concurrency", self.concurrency as u64, 1, 64)?;
        validate_bounded_u64("workload.timeoutMs", self.timeout_ms, 1_000, 3_600_000)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeFact {
    pub path: String,
    pub version: String,
    pub build: String,
    pub executable_sha256: Option<String>,
    pub help_sha256: String,
    pub backend: String,
}

impl RuntimeFact {
    fn validate(&self) -> Result<(), DomainError> {
        validate_text("runtime.path", &self.path, 32_768)?;
        validate_text("runtime.version", &self.version, 1_024)?;
        validate_text("runtime.build", &self.build, 1_024)?;
        validate_text("runtime.backend", &self.backend, 256)?;
        let executable_sha256 = self.executable_sha256.as_deref().ok_or_else(|| {
            DomainError::new(
                ErrorCode::MissingValue,
                "runtime.executableSha256",
                "A complete runtime fact requires an executable digest",
            )
        })?;
        validate_sha256("runtime.executableSha256", executable_sha256)?;
        validate_sha256("runtime.helpSha256", &self.help_sha256)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileFact {
    pub path: String,
    pub bytes: u64,
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelFact {
    pub logical_id: String,
    pub architecture: String,
    pub shards: Vec<FileFact>,
    pub companions: Vec<FileFact>,
    pub gguf_header_sha256: String,
}

impl ModelFact {
    fn validate(&self) -> Result<(), DomainError> {
        validate_text("model.logicalId", &self.logical_id, 256)?;
        validate_text("model.architecture", &self.architecture, 256)?;
        if self.shards.is_empty() || self.shards.len() > 10_000 || self.companions.len() > 256 {
            return Err(DomainError::new(
                ErrorCode::LimitExceeded,
                "model.files",
                "Model file counts exceed the supported limits",
            ));
        }
        for (index, file) in self.shards.iter().chain(self.companions.iter()).enumerate() {
            validate_text(&format!("model.files[{index}].path"), &file.path, 32_768)?;
            if file.bytes == 0 {
                return Err(DomainError::new(
                    ErrorCode::InvalidRange,
                    format!("model.files[{index}].bytes"),
                    "Artifact files must contain at least one byte",
                ));
            }
            let digest = file.sha256.as_deref().ok_or_else(|| {
                DomainError::new(
                    ErrorCode::MissingValue,
                    format!("model.files[{index}].sha256"),
                    "A complete model fact requires every file digest",
                )
            })?;
            validate_sha256(&format!("model.files[{index}].sha256"), digest)?;
        }
        validate_sha256("model.ggufHeaderSha256", &self.gguf_header_sha256)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HardwareFact {
    pub adapter_id: String,
    pub name: String,
    pub vendor: String,
    pub driver: Option<String>,
    pub backend: Option<String>,
    pub dedicated_bytes: Evidence<u64>,
    pub shared_bytes: Evidence<u64>,
    pub budget_bytes: Evidence<u64>,
    pub current_usage_bytes: Evidence<u64>,
}

impl HardwareFact {
    fn validate(&self, index: usize) -> Result<(), DomainError> {
        let prefix = format!("hardware[{index}]");
        validate_text(&format!("{prefix}.adapterId"), &self.adapter_id, 256)?;
        validate_text(&format!("{prefix}.name"), &self.name, 1_024)?;
        validate_text(&format!("{prefix}.vendor"), &self.vendor, 256)?;
        if let Some(driver) = self.driver.as_deref() {
            validate_text(&format!("{prefix}.driver"), driver, 1_024)?;
        }
        if let Some(backend) = self.backend.as_deref() {
            validate_text(&format!("{prefix}.backend"), backend, 256)?;
        }
        self.dedicated_bytes
            .validate(&format!("{prefix}.dedicatedBytes"))?;
        self.shared_bytes
            .validate(&format!("{prefix}.sharedBytes"))?;
        self.budget_bytes
            .validate(&format!("{prefix}.budgetBytes"))?;
        self.current_usage_bytes
            .validate(&format!("{prefix}.currentUsageBytes"))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaunchFact {
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
    pub command_args: Vec<String>,
    pub rejected_flags: Vec<String>,
}

impl Default for LaunchFact {
    fn default() -> Self {
        Self {
            requested_context: 0,
            effective_context: Evidence {
                value: None,
                level: EvidenceLevel::Unknown,
                source: EvidenceSource {
                    kind: EvidenceSourceKind::Unknown,
                    detail: "Effective context has not been observed".into(),
                },
                observed_at_ms: 0,
                notes: vec!["Effective context has not been observed.".into()],
            },
            parallel: 0,
            gpu_layers: String::new(),
            batch: 0,
            ubatch: 0,
            cache_type_k: String::new(),
            cache_type_v: String::new(),
            split_mode: String::new(),
            tensor_split: String::new(),
            main_gpu: 0,
            command_args: Vec::new(),
            rejected_flags: Vec::new(),
        }
    }
}

impl LaunchFact {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.requested_context == 0 {
            return Err(DomainError::new(
                ErrorCode::InvalidRange,
                "launch.requestedContext",
                "Requested context must be greater than zero",
            ));
        }
        self.effective_context.validate("launch.effectiveContext")?;
        if self.effective_context.value == Some(0) {
            return Err(DomainError::new(
                ErrorCode::InvalidRange,
                "launch.effectiveContext",
                "Observed effective context must be greater than zero",
            ));
        }
        if self.parallel == 0 || self.parallel > 64 || self.batch == 0 || self.ubatch == 0 {
            return Err(DomainError::new(
                ErrorCode::InvalidRange,
                "launch.executionShape",
                "Parallelism, batch, and micro-batch must use supported nonzero values",
            ));
        }
        if self.ubatch > self.batch {
            return Err(DomainError::new(
                ErrorCode::InvalidRange,
                "launch.ubatch",
                "Micro-batch size cannot exceed batch size",
            ));
        }
        for (label, value) in [
            ("launch.gpuLayers", self.gpu_layers.as_str()),
            ("launch.cacheTypeK", self.cache_type_k.as_str()),
            ("launch.cacheTypeV", self.cache_type_v.as_str()),
            ("launch.splitMode", self.split_mode.as_str()),
        ] {
            validate_text(label, value, 256)?;
        }
        if self.tensor_split.len() > 1_024
            || self.command_args.len() > 256
            || self.rejected_flags.len() > 256
            || self
                .command_args
                .iter()
                .chain(self.rejected_flags.iter())
                .any(|value| value.len() > 32_768 || value.chars().any(char::is_control))
        {
            return Err(DomainError::new(
                ErrorCode::LimitExceeded,
                "launch.arguments",
                "Launch argument records exceed the supported limits",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AttemptOutcome {
    #[default]
    Succeeded,
    Failed,
    TimedOut,
    Cancelled,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WarmupObservation {
    pub warmup: u16,
    pub started_at_ms: u64,
    pub duration_ms: f64,
    pub outcome: AttemptOutcome,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkObservation {
    pub trial: u16,
    pub started_at_ms: u64,
    pub duration_ms: f64,
    /// Prompt tokens the runtime evaluated this request (`timings.prompt_n`).
    pub prompt_tokens: u32,
    /// Prompt tokens restored from the runtime's prompt cache
    /// (`timings.cache_n`, b10816 semantics). A warm trial is valid when
    /// processed + cached equals the requested prompt size (audit MT-01).
    #[serde(default)]
    pub cached_prompt_tokens: u32,
    pub generated_tokens: u32,
    pub prefill_tps: Option<f64>,
    pub decode_tps: Option<f64>,
    pub first_token_ms: Option<f64>,
    pub derived_ttft_ms: Option<f64>,
    pub peak_process_rss_bytes: Evidence<u64>,
    pub outcome: AttemptOutcome,
    pub error: Option<String>,
}

impl Default for BenchmarkObservation {
    fn default() -> Self {
        let observed_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
            .unwrap_or(u64::MAX)
            .max(1);
        Self {
            trial: 0,
            started_at_ms: 0,
            duration_ms: 0.0,
            prompt_tokens: 0,
            cached_prompt_tokens: 0,
            generated_tokens: 0,
            prefill_tps: None,
            decode_tps: None,
            first_token_ms: None,
            derived_ttft_ms: None,
            peak_process_rss_bytes: Evidence::unknown(
                EvidenceSource {
                    kind: EvidenceSourceKind::Unknown,
                    detail: "process peak working set not sampled".into(),
                },
                observed_at_ms,
                "Process peak working-set evidence is unavailable.",
            ),
            outcome: AttemptOutcome::Failed,
            error: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkManifest {
    pub schema: u32,
    pub harness_version: String,
    pub compatibility_key: Option<String>,
    pub runtime: Option<RuntimeFact>,
    pub hardware: Vec<HardwareFact>,
    pub model: Option<ModelFact>,
    pub launch: Option<LaunchFact>,
    pub workload: Workload,
    pub warmups: Vec<WarmupObservation>,
    pub observations: Vec<BenchmarkObservation>,
    pub terminal_outcome: Option<AttemptOutcome>,
}

impl Default for BenchmarkManifest {
    fn default() -> Self {
        Self {
            schema: BENCHMARK_SCHEMA_VERSION,
            harness_version: env!("CARGO_PKG_VERSION").into(),
            compatibility_key: None,
            runtime: None,
            hardware: Vec::new(),
            model: None,
            launch: None,
            workload: Workload::default(),
            warmups: Vec::new(),
            observations: Vec::new(),
            terminal_outcome: None,
        }
    }
}

impl BenchmarkManifest {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.schema != BENCHMARK_SCHEMA_VERSION {
            return Err(DomainError::new(
                ErrorCode::UnsupportedSchema,
                "schema",
                format!(
                    "Benchmark schema {} is unsupported; expected {}",
                    self.schema, BENCHMARK_SCHEMA_VERSION
                ),
            ));
        }
        if self.harness_version.trim().is_empty() {
            return Err(DomainError::new(
                ErrorCode::EmptyIdentity,
                "harnessVersion",
                "Harness version is required",
            ));
        }
        self.workload.validate()?;
        if self.harness_version.len() > 128
            || self.hardware.len() > 64
            || self.warmups.len() > self.workload.warmups as usize
            || self.observations.len() > self.workload.trials as usize
        {
            return Err(DomainError::new(
                ErrorCode::LimitExceeded,
                "manifest.collections",
                "Benchmark manifest collections exceed workload or schema limits",
            ));
        }
        if self.terminal_outcome == Some(AttemptOutcome::Succeeded) {
            return Err(DomainError::new(
                ErrorCode::InconsistentEvidence,
                "terminalOutcome",
                "A successful run must not contain a terminal failure outcome",
            ));
        }
        for (index, warmup) in self.warmups.iter().enumerate() {
            validate_attempt(
                &format!("warmups[{index}]"),
                warmup.started_at_ms,
                warmup.duration_ms,
                warmup.outcome,
                warmup.error.as_deref(),
            )?;
        }
        for (index, observation) in self.observations.iter().enumerate() {
            validate_attempt(
                &format!("observations[{index}]"),
                observation.started_at_ms,
                observation.duration_ms,
                observation.outcome,
                observation.error.as_deref(),
            )?;
            for (label, value) in [
                ("prefillTps", observation.prefill_tps),
                ("decodeTps", observation.decode_tps),
                ("firstTokenMs", observation.first_token_ms),
                ("derivedTtftMs", observation.derived_ttft_ms),
            ] {
                if let Some(value) = value {
                    validate_finite(&format!("observations[{index}].{label}"), value)?;
                    if value < 0.0 {
                        return Err(DomainError::new(
                            ErrorCode::InvalidRange,
                            format!("observations[{index}].{label}"),
                            "Observation metrics cannot be negative",
                        ));
                    }
                }
            }
            observation
                .peak_process_rss_bytes
                .validate(&format!("observations[{index}].peakProcessRssBytes"))?;
        }
        Ok(())
    }

    pub fn validate_complete(&self) -> Result<(), DomainError> {
        self.validate()?;
        if self.runtime.is_none() {
            return Err(DomainError::new(
                ErrorCode::MissingValue,
                "runtime",
                "A complete benchmark manifest requires a runtime snapshot",
            ));
        }
        if self.model.is_none() {
            return Err(DomainError::new(
                ErrorCode::MissingValue,
                "model",
                "A complete benchmark manifest requires a model identity",
            ));
        }
        if self.launch.is_none() {
            return Err(DomainError::new(
                ErrorCode::MissingValue,
                "launch",
                "A complete benchmark manifest requires a launch snapshot",
            ));
        }
        let compatibility_key = self.compatibility_key.as_deref().ok_or_else(|| {
            DomainError::new(
                ErrorCode::MissingValue,
                "compatibilityKey",
                "A complete benchmark manifest requires a compatibility key",
            )
        })?;
        validate_sha256("compatibilityKey", compatibility_key)?;
        self.runtime
            .as_ref()
            .expect("runtime presence checked above")
            .validate()?;
        for (index, hardware) in self.hardware.iter().enumerate() {
            hardware.validate(index)?;
        }
        self.model
            .as_ref()
            .expect("model presence checked above")
            .validate()?;
        self.launch
            .as_ref()
            .expect("launch presence checked above")
            .validate()?;
        Ok(())
    }
}

fn validate_text(field: &str, value: &str, max_bytes: usize) -> Result<(), DomainError> {
    if value.trim().is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        Err(DomainError::new(
            ErrorCode::InvalidRange,
            field,
            format!("Text must contain 1 to {max_bytes} bytes without control characters"),
        ))
    } else {
        Ok(())
    }
}

fn validate_attempt(
    field: &str,
    started_at_ms: u64,
    duration_ms: f64,
    outcome: AttemptOutcome,
    error: Option<&str>,
) -> Result<(), DomainError> {
    if started_at_ms == 0 || !duration_ms.is_finite() || duration_ms < 0.0 {
        return Err(DomainError::new(
            ErrorCode::InvalidRange,
            field,
            "Attempt timestamps and durations must be valid",
        ));
    }
    if error.is_some_and(|error| error.is_empty() || error.len() > 4_096) {
        return Err(DomainError::new(
            ErrorCode::LimitExceeded,
            format!("{field}.error"),
            "Attempt errors must contain at most 4096 bytes",
        ));
    }
    if (outcome == AttemptOutcome::Succeeded) != error.is_none() {
        return Err(DomainError::new(
            ErrorCode::InconsistentEvidence,
            format!("{field}.outcome"),
            "Attempt outcome conflicts with its error evidence",
        ));
    }
    Ok(())
}

pub fn validate_finite(field: &str, value: f64) -> Result<(), DomainError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(DomainError::new(
            ErrorCode::NonFinite,
            field,
            "Value must be finite",
        ))
    }
}

pub fn validate_sha256(field: &str, value: &str) -> Result<(), DomainError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(DomainError::new(
            ErrorCode::InvalidDigest,
            field,
            "SHA-256 must contain 64 lowercase hexadecimal characters",
        ))
    }
}

fn validate_bounded_u64(
    field: &str,
    value: u64,
    minimum: u64,
    maximum: u64,
) -> Result<(), DomainError> {
    if value < minimum {
        return Err(DomainError::new(
            ErrorCode::InvalidRange,
            field,
            format!("Value must be at least {minimum}"),
        ));
    }
    if value > maximum {
        return Err(DomainError::new(
            ErrorCode::LimitExceeded,
            field,
            format!("Value cannot exceed {maximum}"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(kind: EvidenceSourceKind) -> EvidenceSource {
        EvidenceSource {
            kind,
            detail: "fixture".into(),
        }
    }

    #[test]
    fn unknown_evidence_has_no_value_and_names_the_missing_fact() {
        let evidence = Evidence::<u64>::unknown(
            source(EvidenceSourceKind::Unknown),
            42,
            "GPU budget was not reported",
        );

        assert_eq!(evidence.level, EvidenceLevel::Unknown);
        assert_eq!(evidence.value, None);
        assert_eq!(evidence.notes, vec!["GPU budget was not reported"]);
        evidence.validate("gpuBudgetBytes").unwrap();
    }

    #[test]
    fn known_evidence_requires_a_value() {
        let evidence = Evidence::<u64> {
            value: None,
            level: EvidenceLevel::Exact,
            source: source(EvidenceSourceKind::FileSystem),
            observed_at_ms: 42,
            notes: Vec::new(),
        };

        let error = evidence.validate("fileBytes").unwrap_err();
        assert_eq!(error.code, ErrorCode::MissingValue);
        assert_eq!(error.field, "fileBytes");
    }

    #[test]
    fn unknown_evidence_rejects_a_value() {
        let evidence = Evidence {
            value: Some(16_u64),
            level: EvidenceLevel::Unknown,
            source: source(EvidenceSourceKind::Unknown),
            observed_at_ms: 42,
            notes: vec!["contradictory probes".into()],
        };

        assert_eq!(
            evidence.validate("availableRamBytes").unwrap_err().code,
            ErrorCode::UnexpectedValue
        );
    }

    #[test]
    fn derived_evidence_propagates_an_unknown_required_input() {
        let evidence = Evidence::derived(
            8_u64,
            &[EvidenceLevel::Exact, EvidenceLevel::Unknown],
            source(EvidenceSourceKind::Calculation),
            42,
            vec!["layers × heads".into()],
        );

        assert_eq!(evidence.level, EvidenceLevel::Unknown);
        assert_eq!(evidence.value, None);
        assert!(evidence
            .notes
            .iter()
            .any(|note| note.contains("required input")));
    }

    #[test]
    fn heuristic_evidence_cannot_be_relabelled_as_observed() {
        let evidence = Evidence::known(
            72.0_f64,
            EvidenceLevel::Heuristic,
            source(EvidenceSourceKind::Policy),
            42,
            Vec::new(),
        )
        .unwrap();

        let error = evidence.relabel(EvidenceLevel::Observed).unwrap_err();
        assert_eq!(error.code, ErrorCode::EvidenceUpgrade);
        assert_eq!(evidence.level, EvidenceLevel::Heuristic);
    }

    #[test]
    fn evidence_rejects_a_zero_observation_timestamp() {
        let error = Evidence::known(
            7_u64,
            EvidenceLevel::Observed,
            source(EvidenceSourceKind::WindowsApi),
            0,
            Vec::new(),
        )
        .unwrap_err();

        assert_eq!(error.code, ErrorCode::InvalidRange);
        assert_eq!(error.field, "evidence.value.observedAtMs");
    }

    #[test]
    fn workload_accepts_the_reproducible_default_protocol() {
        let workload = Workload::default();

        workload.validate().unwrap();
        assert_eq!(workload.warmups, 1);
        assert_eq!(workload.trials, 5);
        assert_eq!(workload.seed, Some(42));
    }

    #[test]
    fn workload_rejects_zero_trials() {
        let workload = Workload {
            trials: 0,
            ..Workload::default()
        };

        let error = workload.validate().unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRange);
        assert_eq!(error.field, "workload.trials");
    }

    #[test]
    fn workload_rejects_excessive_concurrency() {
        let workload = Workload {
            concurrency: 65,
            ..Workload::default()
        };

        let error = workload.validate().unwrap_err();
        assert_eq!(error.code, ErrorCode::LimitExceeded);
        assert_eq!(error.field, "workload.concurrency");
    }

    #[test]
    fn finite_validation_rejects_nan_and_infinity() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let error = validate_finite("metric", value).unwrap_err();
            assert_eq!(error.code, ErrorCode::NonFinite);
        }
        validate_finite("metric", 0.0).unwrap();
    }

    #[test]
    fn sha256_validation_requires_lowercase_hex() {
        let valid = "0123456789abcdef".repeat(4);
        validate_sha256("sha256", &valid).unwrap();

        for invalid in ["abc", &valid.to_uppercase(), &format!("{}g", &valid[..63])] {
            let error = validate_sha256("sha256", invalid).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidDigest);
        }
    }

    #[test]
    fn manifest_rejects_a_future_schema() {
        let manifest = BenchmarkManifest {
            schema: BENCHMARK_SCHEMA_VERSION + 1,
            harness_version: "0.3.0".into(),
            workload: Workload::default(),
            ..BenchmarkManifest::default()
        };

        let error = manifest.validate().unwrap_err();
        assert_eq!(error.code, ErrorCode::UnsupportedSchema);
        assert_eq!(error.field, "schema");
    }

    #[test]
    fn manifest_requires_a_harness_identity() {
        let manifest = BenchmarkManifest {
            schema: BENCHMARK_SCHEMA_VERSION,
            harness_version: " ".into(),
            workload: Workload::default(),
            ..BenchmarkManifest::default()
        };

        let error = manifest.validate().unwrap_err();
        assert_eq!(error.code, ErrorCode::EmptyIdentity);
        assert_eq!(error.field, "harnessVersion");
    }

    #[test]
    fn complete_manifest_requires_a_runtime_snapshot() {
        let manifest = BenchmarkManifest::default();

        let error = manifest.validate_complete().unwrap_err();
        assert_eq!(error.code, ErrorCode::MissingValue);
        assert_eq!(error.field, "runtime");
    }

    #[test]
    fn complete_manifest_requires_a_model_identity() {
        let manifest = BenchmarkManifest {
            runtime: Some(RuntimeFact {
                path: "runtime.exe".into(),
                version: "1".into(),
                build: "1".into(),
                executable_sha256: Some("a".repeat(64)),
                help_sha256: "b".repeat(64),
                backend: "cpu".into(),
            }),
            ..BenchmarkManifest::default()
        };

        let error = manifest.validate_complete().unwrap_err();

        assert_eq!(error.code, ErrorCode::MissingValue);
        assert_eq!(error.field, "model");
    }

    #[test]
    fn complete_manifest_requires_a_launch_snapshot() {
        let manifest = BenchmarkManifest {
            runtime: Some(RuntimeFact {
                path: "runtime.exe".into(),
                version: "1".into(),
                build: "1".into(),
                executable_sha256: Some("a".repeat(64)),
                help_sha256: "b".repeat(64),
                backend: "cpu".into(),
            }),
            model: Some(ModelFact::default()),
            ..BenchmarkManifest::default()
        };

        let error = manifest.validate_complete().unwrap_err();

        assert_eq!(error.code, ErrorCode::MissingValue);
        assert_eq!(error.field, "launch");
    }

    #[test]
    fn launch_fact_rejects_ubatch_larger_than_batch() {
        let launch = LaunchFact {
            requested_context: 4096,
            effective_context: Evidence::known(
                4096,
                EvidenceLevel::Observed,
                source(EvidenceSourceKind::Runtime),
                42,
                Vec::new(),
            )
            .unwrap(),
            parallel: 1,
            batch: 128,
            ubatch: 256,
            ..LaunchFact::default()
        };

        let error = launch.validate().unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRange);
        assert_eq!(error.field, "launch.ubatch");
    }

    #[test]
    fn launch_fact_accepts_unknown_effective_context_evidence() {
        let launch = LaunchFact {
            requested_context: 4096,
            effective_context: Evidence {
                value: None,
                level: EvidenceLevel::Unknown,
                source: source(EvidenceSourceKind::Runtime),
                observed_at_ms: 42,
                notes: vec!["The runtime did not expose /props.".into()],
            },
            parallel: 1,
            gpu_layers: "0".into(),
            batch: 128,
            ubatch: 128,
            cache_type_k: "F16".into(),
            cache_type_v: "F16".into(),
            split_mode: "none".into(),
            ..LaunchFact::default()
        };

        launch.validate().unwrap();
    }

    #[test]
    fn launch_fact_rejects_zero_effective_context() {
        let launch = LaunchFact {
            requested_context: 4096,
            effective_context: Evidence::known(
                0,
                EvidenceLevel::Observed,
                source(EvidenceSourceKind::Runtime),
                42,
                Vec::new(),
            )
            .unwrap(),
            parallel: 1,
            batch: 128,
            ubatch: 128,
            ..LaunchFact::default()
        };

        let error = launch.validate().unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRange);
        assert_eq!(error.field, "launch.effectiveContext");
    }

    #[test]
    fn public_contracts_serialize_with_camel_case_fields_and_values() {
        let evidence = Evidence::known(
            7_u64,
            EvidenceLevel::UserOverride,
            source(EvidenceSourceKind::User),
            42,
            Vec::new(),
        )
        .unwrap();
        let value = serde_json::to_value(evidence).unwrap();

        assert_eq!(value["observedAtMs"], 42);
        assert_eq!(value["level"], "userOverride");
        assert_eq!(value["source"]["kind"], "user");
    }

    #[test]
    fn estimated_result_class_remains_distinct_from_preflight() {
        assert_eq!(
            serde_json::to_string(&FitClass::Estimated).unwrap(),
            "\"estimated\""
        );
        assert_ne!(FitClass::Estimated, FitClass::PreflightLikely);
    }
}
