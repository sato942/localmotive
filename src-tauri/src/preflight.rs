use crate::evidence::{
    Evidence, EvidenceLevel, EvidenceSource, EvidenceSourceKind, ExecutionPath, FitClass,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KvCacheInputs {
    pub architecture: String,
    pub block_count: Option<u64>,
    pub head_count_kv: Option<u64>,
    pub key_length: Option<u64>,
    pub value_length: Option<u64>,
    pub context: u64,
    pub cache_type_k: String,
    pub cache_type_v: String,
    pub recurrent_or_hybrid: bool,
    pub observed_at_ms: u64,
}

fn cache_element_bytes(cache_type: &str) -> Option<u64> {
    match cache_type.trim().to_ascii_lowercase().as_str() {
        "f32" => Some(4),
        "f16" | "bf16" => Some(2),
        _ => None,
    }
}

fn supports_dense_kv_layout(architecture: &str) -> bool {
    matches!(architecture.trim().to_ascii_lowercase().as_str(), "llama")
}

pub fn estimate_kv_cache_bytes(inputs: &KvCacheInputs) -> Evidence<u64> {
    let source = EvidenceSource {
        kind: EvidenceSourceKind::Calculation,
        detail: "dense KV cache dimensions × context × cache element widths".into(),
    };
    let mut missing = Vec::new();
    if inputs.block_count.is_none() {
        missing.push("block_count");
    }
    if inputs.head_count_kv.is_none() {
        missing.push("head_count_kv");
    }
    if inputs.key_length.is_none() {
        missing.push("key_length");
    }
    if inputs.value_length.is_none() {
        missing.push("value_length");
    }
    if cache_element_bytes(&inputs.cache_type_k).is_none() {
        missing.push("cache_type_k");
    }
    if cache_element_bytes(&inputs.cache_type_v).is_none() {
        missing.push("cache_type_v");
    }
    if !missing.is_empty() {
        return Evidence {
            value: None,
            level: EvidenceLevel::Unknown,
            source,
            observed_at_ms: inputs.observed_at_ms,
            notes: vec![format!(
                "Missing required KV terms: {}.",
                missing.join(", ")
            )],
        };
    }
    if !supports_dense_kv_layout(&inputs.architecture) {
        return Evidence {
            value: None,
            level: EvidenceLevel::Unknown,
            source,
            observed_at_ms: inputs.observed_at_ms,
            notes: vec![format!(
                "Dense KV cache layout is not verified for GGUF architecture '{}'.",
                inputs.architecture
            )],
        };
    }
    let terms = (
        inputs.block_count,
        inputs.head_count_kv,
        inputs.key_length,
        inputs.value_length,
        cache_element_bytes(&inputs.cache_type_k),
        cache_element_bytes(&inputs.cache_type_v),
    );
    let (
        Some(blocks),
        Some(kv_heads),
        Some(key_length),
        Some(value_length),
        Some(key_bytes),
        Some(value_bytes),
    ) = terms
    else {
        return Evidence {
            value: None,
            level: EvidenceLevel::Unknown,
            source,
            observed_at_ms: inputs.observed_at_ms,
            notes: vec!["KV cache terms changed during validation.".into()],
        };
    };
    if inputs.recurrent_or_hybrid || inputs.context == 0 {
        return Evidence {
            value: None,
            level: EvidenceLevel::Unknown,
            source,
            observed_at_ms: inputs.observed_at_ms,
            notes: vec!["Dense KV cache arithmetic does not cover hybrid, recurrent, or zero-context workloads.".into()],
        };
    }
    let per_position = (key_length as u128)
        .checked_mul(key_bytes as u128)
        .and_then(|value| value.checked_add((value_length as u128) * (value_bytes as u128)));
    let value = per_position
        .and_then(|value| value.checked_mul(kv_heads as u128))
        .and_then(|value| value.checked_mul(blocks as u128))
        .and_then(|value| value.checked_mul(inputs.context as u128))
        .and_then(|value| u64::try_from(value).ok());
    match value {
        Some(value) => Evidence {
            value: Some(value),
            level: EvidenceLevel::Derived,
            source,
            observed_at_ms: inputs.observed_at_ms,
            notes: vec![
                "This value excludes runtime allocation overhead and policy reserves.".into(),
            ],
        },
        None => Evidence {
            value: None,
            level: EvidenceLevel::Unknown,
            source,
            observed_at_ms: inputs.observed_at_ms,
            notes: vec!["KV cache arithmetic exceeded the supported byte range.".into()],
        },
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryAssessment {
    pub class: FitClass,
    pub required_bytes: Evidence<u64>,
    pub available_bytes: Evidence<u64>,
    pub policy_reserve_bytes: u64,
    pub assumptions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightFacts {
    pub execution_path: ExecutionPath,
    pub runtime_topology_known: bool,
    pub unverified_requirements: Vec<String>,
    pub requested_context: u64,
    pub native_context: Option<u64>,
    pub runtime_fit_enabled: bool,
    pub weight_bytes: u64,
    pub companion_bytes: u64,
    pub kv: KvCacheInputs,
    pub available_memory: Evidence<u64>,
    pub available_disk: Evidence<u64>,
    pub storage_volumes: Vec<StorageVolumeEvidence>,
    pub reserve_bytes: u64,
    pub offload_possible: bool,
    pub assumptions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightReport {
    pub schema: u32,
    pub class: FitClass,
    pub execution_path: ExecutionPath,
    pub requested_context: Evidence<u64>,
    pub native_context: Evidence<u64>,
    pub effective_context: Evidence<u64>,
    pub weight_bytes: Evidence<u64>,
    pub kv_cache_bytes: Evidence<u64>,
    pub storage_required_bytes: Evidence<u64>,
    pub disk_available_bytes: Evidence<u64>,
    pub storage_volumes: Vec<StorageVolumeEvidence>,
    pub memory: MemoryAssessment,
    pub assumptions: Vec<String>,
    pub unknowns: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAllocationPlan {
    pub adapter_id: String,
    pub weight_bytes: Evidence<u64>,
    pub kv_cache_bytes: Evidence<u64>,
    pub total_bytes: Evidence<u64>,
    pub available_bytes: Evidence<u64>,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageVolumeEvidence {
    pub volume_path: Evidence<String>,
    pub resident_bytes: Evidence<u64>,
    pub available_bytes: Evidence<u64>,
}

fn storage_volumes_with<Resolve, Available>(
    files: &[(PathBuf, u64)],
    observed_at_ms: u64,
    mut resolve_volume: Resolve,
    mut available_bytes: Available,
) -> Vec<StorageVolumeEvidence>
where
    Resolve: FnMut(&Path, u64) -> Evidence<String>,
    Available: FnMut(&Path, u64) -> Evidence<u64>,
{
    let mut seen = BTreeSet::new();
    let mut grouped: BTreeMap<String, (Evidence<String>, PathBuf, Option<u64>)> = BTreeMap::new();
    for (path, bytes) in files {
        if !seen.insert(path.clone()) {
            continue;
        }
        let volume = resolve_volume(path, observed_at_ms);
        let query_path = volume
            .value
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| path.clone());
        let key = volume
            .value
            .clone()
            .unwrap_or_else(|| format!("unknown:{}", path.display()));
        let entry = grouped
            .entry(key)
            .or_insert_with(|| (volume, query_path, Some(0)));
        entry.2 = entry.2.and_then(|total| total.checked_add(*bytes));
    }
    grouped
        .into_values()
        .map(
            |(volume_path, query_path, resident_value)| StorageVolumeEvidence {
                volume_path,
                resident_bytes: Evidence {
                    value: resident_value,
                    level: if resident_value.is_some() {
                        EvidenceLevel::Exact
                    } else {
                        EvidenceLevel::Unknown
                    },
                    source: EvidenceSource {
                        kind: EvidenceSourceKind::FileSystem,
                        detail: "Selected resident artifact bytes on one volume".into(),
                    },
                    observed_at_ms,
                    notes: if resident_value.is_some() {
                        Vec::new()
                    } else {
                        vec!["Selected artifact byte aggregation overflowed on this volume.".into()]
                    },
                },
                available_bytes: available_bytes(&query_path, observed_at_ms),
            },
        )
        .collect()
}

pub fn storage_volume_evidence(
    files: &[(PathBuf, u64)],
    observed_at_ms: u64,
) -> Vec<StorageVolumeEvidence> {
    let canonical_files = files
        .iter()
        .map(|(path, bytes)| {
            (
                std::fs::canonicalize(path).unwrap_or_else(|_| path.clone()),
                *bytes,
            )
        })
        .collect::<Vec<_>>();
    storage_volumes_with(
        &canonical_files,
        observed_at_ms,
        volume_path_evidence,
        available_disk_bytes,
    )
}

fn plan_source(kind: EvidenceSourceKind, detail: &str) -> EvidenceSource {
    EvidenceSource {
        kind,
        detail: detail.into(),
    }
}

pub fn build_device_plan(
    devices: &[(String, Evidence<u64>)],
    weight_bytes: u64,
    kv_cache_bytes: &Evidence<u64>,
    observed_at_ms: u64,
) -> Vec<DeviceAllocationPlan> {
    let devices = if devices.is_empty() {
        vec![(
            "unselected-device".to_string(),
            Evidence::unknown(
                plan_source(
                    EvidenceSourceKind::Policy,
                    "No explicit device was selected",
                ),
                observed_at_ms,
                "Device capacity is unknown",
            ),
        )]
    } else {
        devices.to_vec()
    };
    if devices.len() > 1 {
        return devices
            .into_iter()
            .map(|(adapter_id, available_bytes)| {
                let source = plan_source(
                    EvidenceSourceKind::Policy,
                    "Multi-adapter placement requires runtime-validated tensor allocation",
                );
                DeviceAllocationPlan {
                    adapter_id,
                    weight_bytes: Evidence::unknown(
                        source.clone(),
                        observed_at_ms,
                        "Per-adapter weight placement is unknown",
                    ),
                    kv_cache_bytes: Evidence::unknown(
                        source.clone(),
                        observed_at_ms,
                        "Per-adapter KV placement is unknown",
                    ),
                    total_bytes: Evidence::unknown(
                        source,
                        observed_at_ms,
                        "Per-adapter total memory is unknown",
                    ),
                    available_bytes,
                    note:
                        "No per-device placement is claimed before a validated runtime measurement"
                            .into(),
                }
            })
            .collect();
    }

    let weight = Evidence {
        value: Some(weight_bytes),
        level: EvidenceLevel::Heuristic,
        source: plan_source(
            EvidenceSourceKind::Calculation,
            "GGUF shard bytes used as a conservative in-memory weight proxy",
        ),
        observed_at_ms,
        notes: vec!["File bytes do not prove final runtime allocation".into()],
    };
    let total = match kv_cache_bytes
        .value
        .and_then(|kv_bytes| weight_bytes.checked_add(kv_bytes))
    {
        Some(value) => Evidence::derived(
            value,
            &[weight.level, kv_cache_bytes.level],
            plan_source(
                EvidenceSourceKind::Calculation,
                "Weight proxy plus KV estimate",
            ),
            observed_at_ms,
            vec!["Runtime reserve and allocator overhead remain separate".into()],
        ),
        None => Evidence::unknown(
            plan_source(
                EvidenceSourceKind::Calculation,
                "Weight proxy plus KV estimate",
            ),
            observed_at_ms,
            "Total device allocation is unknown",
        ),
    };
    vec![DeviceAllocationPlan {
        adapter_id: devices[0].0.clone(),
        weight_bytes: weight,
        kv_cache_bytes: kv_cache_bytes.clone(),
        total_bytes: total,
        available_bytes: devices[0].1.clone(),
        note: "Single-device estimate; launch validation remains required".into(),
    }]
}

fn fact<T>(
    value: Option<T>,
    level: EvidenceLevel,
    source: EvidenceSource,
    observed_at_ms: u64,
    unknown_note: &str,
) -> Evidence<T> {
    Evidence {
        level: if value.is_some() {
            level
        } else {
            EvidenceLevel::Unknown
        },
        value,
        source,
        observed_at_ms,
        notes: if unknown_note.is_empty() {
            Vec::new()
        } else {
            vec![unknown_note.into()]
        },
    }
}

pub fn build_preflight_report(facts: PreflightFacts) -> PreflightReport {
    let observed_at_ms = facts.kv.observed_at_ms;
    let file_source = EvidenceSource {
        kind: EvidenceSourceKind::FileSystem,
        detail: "Artifact file metadata".into(),
    };
    let gguf_source = EvidenceSource {
        kind: EvidenceSourceKind::GgufMetadata,
        detail: "GGUF metadata".into(),
    };
    let calculation_source = EvidenceSource {
        kind: EvidenceSourceKind::Calculation,
        detail: "Preflight calculation".into(),
    };
    let requested_context = fact(
        Some(facts.requested_context),
        EvidenceLevel::UserOverride,
        EvidenceSource {
            kind: EvidenceSourceKind::User,
            detail: "Launch profile context".into(),
        },
        observed_at_ms,
        "",
    );
    let native_context = fact(
        facts.native_context,
        EvidenceLevel::Exact,
        gguf_source,
        observed_at_ms,
        "GGUF native context is unknown.",
    );
    let effective_context = fact(
        if facts.runtime_fit_enabled {
            None
        } else {
            Some(facts.requested_context)
        },
        EvidenceLevel::Derived,
        calculation_source.clone(),
        observed_at_ms,
        "The runtime fit policy can change the effective context at launch.",
    );
    let loaded_artifact_bytes = facts.weight_bytes.checked_add(facts.companion_bytes);
    let weight_bytes = Evidence {
        value: loaded_artifact_bytes,
        level: if loaded_artifact_bytes.is_some() {
            EvidenceLevel::Heuristic
        } else {
            EvidenceLevel::Unknown
        },
        source: EvidenceSource {
            kind: EvidenceSourceKind::Calculation,
            detail: "GGUF shard bytes used as a live weight-allocation proxy".into(),
        },
        observed_at_ms,
        notes: if loaded_artifact_bytes.is_some() {
            vec![
                "Artifact file bytes are exact, but live runtime allocation remains unknown."
                    .into(),
            ]
        } else {
            vec!["Loaded artifact byte aggregation overflowed.".into()]
        },
    };
    let storage_footprint = if facts.storage_volumes.is_empty() {
        None
    } else {
        facts
            .storage_volumes
            .iter()
            .try_fold(0_u64, |total, volume| {
                if volume.resident_bytes.level != EvidenceLevel::Exact {
                    return None;
                }
                total.checked_add(volume.resident_bytes.value?)
            })
    };
    let storage_required_bytes = fact(
        storage_footprint,
        EvidenceLevel::Exact,
        file_source,
        observed_at_ms,
        "Exact per-volume artifact storage facts are unavailable or overflowed.",
    );
    let kv_cache_bytes = estimate_kv_cache_bytes(&facts.kv);
    let mut memory = assess_memory(
        &weight_bytes,
        &kv_cache_bytes,
        &facts.available_memory,
        facts.reserve_bytes,
        facts.offload_possible,
        observed_at_ms,
    );
    let mut unknowns = Vec::new();
    let topology_unknown =
        !facts.runtime_topology_known && !matches!(facts.execution_path, ExecutionPath::Cpu);
    if topology_unknown {
        unknowns.push("runtime-to-adapter mapping".into());
        memory.class = FitClass::Unknown;
        memory.assumptions.push(
            "The selected DXGI adapter is not mapped to a verified llama.cpp runtime device."
                .into(),
        );
    }
    if !facts.unverified_requirements.is_empty() {
        memory.class = FitClass::Unknown;
        memory
            .assumptions
            .extend(facts.unverified_requirements.iter().cloned());
        unknowns.extend(facts.unverified_requirements.iter().cloned());
    }
    for (name, evidence_missing) in [
        ("nativeContext", native_context.value.is_none()),
        ("effectiveContext", effective_context.value.is_none()),
        ("kvCacheBytes", kv_cache_bytes.value.is_none()),
        (
            "memoryAvailableBytes",
            facts.available_memory.value.is_none(),
        ),
        ("diskAvailableBytes", facts.available_disk.value.is_none()),
    ] {
        if evidence_missing {
            unknowns.push(name.into());
        }
    }
    let mut assumptions = facts.assumptions;
    assumptions.push(
        "Selected artifacts are already resident; available disk is reported separately and is not treated as duplicate launch storage."
            .into(),
    );
    let class = if native_context
        .value
        .is_some_and(|native| facts.requested_context > native)
    {
        assumptions.push(format!(
            "The requested context {} exceeds the GGUF native context {}.",
            facts.requested_context,
            native_context.value.unwrap_or_default()
        ));
        FitClass::Failed
    } else if native_context.value.is_none()
        || effective_context.value.is_none()
        || storage_required_bytes.value.is_none()
    {
        FitClass::Unknown
    } else {
        memory.class
    };
    PreflightReport {
        schema: 1,
        class,
        execution_path: facts.execution_path,
        requested_context,
        native_context,
        effective_context,
        weight_bytes,
        kv_cache_bytes,
        storage_required_bytes,
        disk_available_bytes: facts.available_disk,
        storage_volumes: facts.storage_volumes,
        memory,
        assumptions,
        unknowns,
    }
}

pub fn assess_memory(
    weights: &Evidence<u64>,
    kv_cache: &Evidence<u64>,
    available: &Evidence<u64>,
    policy_reserve_bytes: u64,
    offload_possible: bool,
    observed_at_ms: u64,
) -> MemoryAssessment {
    let source = EvidenceSource {
        kind: EvidenceSourceKind::Calculation,
        detail: "weights + KV cache + named policy reserve".into(),
    };
    let required = weights
        .value
        .zip(kv_cache.value)
        .and_then(|(weights, kv)| weights.checked_add(kv))
        .and_then(|base| base.checked_add(policy_reserve_bytes));
    let required_bytes = Evidence {
        value: required,
        level: if required.is_some() {
            EvidenceLevel::Derived
        } else {
            EvidenceLevel::Unknown
        },
        source,
        observed_at_ms,
        notes: if required.is_some() {
            Vec::new()
        } else {
            vec!["Required memory is unknown because an input is missing or overflowed.".into()]
        },
    };
    let class = match (weights.value, kv_cache.value, required, available.value) {
        (_, _, Some(required), Some(available)) if required <= available => {
            FitClass::PreflightLikely
        }
        (Some(weights), Some(kv), _, Some(available))
            if weights
                .checked_add(kv)
                .is_some_and(|base| base <= available) =>
        {
            FitClass::PreflightTight
        }
        (Some(_), Some(_), _, Some(_)) if offload_possible => FitClass::RequiresOffload,
        (Some(_), Some(_), _, Some(_)) => FitClass::Failed,
        _ => FitClass::Unknown,
    };
    MemoryAssessment {
        class,
        required_bytes,
        available_bytes: available.clone(),
        policy_reserve_bytes,
        assumptions: vec![format!(
            "A fixed {}-byte policy reserve covers unmodeled runtime allocations.",
            policy_reserve_bytes
        )],
    }
}

#[cfg(windows)]
fn volume_path_evidence(path: &Path, observed_at_ms: u64) -> Evidence<String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows_sys::Win32::Storage::FileSystem::GetVolumePathNameW;

    const MAX_VOLUME_PATH_UNITS: usize = 32_768;
    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut output = vec![0_u16; MAX_VOLUME_PATH_UNITS];
    // SAFETY: `wide` is NUL-terminated, and `output` is writable for the stated length.
    let success = unsafe {
        GetVolumePathNameW(
            wide.as_ptr(),
            output.as_mut_ptr(),
            output.len().try_into().unwrap_or(u32::MAX),
        )
    };
    let source = EvidenceSource {
        kind: EvidenceSourceKind::WindowsApi,
        detail: "GetVolumePathNameW mounted-volume path".into(),
    };
    let length = output.iter().position(|unit| *unit == 0).unwrap_or(0);
    if success != 0 && length > 0 {
        Evidence {
            value: Some(
                OsString::from_wide(&output[..length])
                    .to_string_lossy()
                    .into_owned(),
            ),
            level: EvidenceLevel::Observed,
            source,
            observed_at_ms,
            notes: Vec::new(),
        }
    } else {
        Evidence {
            value: None,
            level: EvidenceLevel::Unknown,
            source,
            observed_at_ms,
            notes: vec![format!(
                "GetVolumePathNameW failed for a selected artifact: {}",
                std::io::Error::last_os_error()
            )],
        }
    }
}

#[cfg(not(windows))]
fn volume_path_evidence(_path: &Path, observed_at_ms: u64) -> Evidence<String> {
    Evidence {
        value: None,
        level: EvidenceLevel::Unknown,
        source: EvidenceSource {
            kind: EvidenceSourceKind::Unknown,
            detail: "Windows mounted-volume probe unavailable".into(),
        },
        observed_at_ms,
        notes: vec!["GetVolumePathNameW is available only on Windows.".into()],
    }
}

#[cfg(windows)]
pub fn available_disk_bytes(path: &Path, observed_at_ms: u64) -> Evidence<u64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let volume_path = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let wide = volume_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut available = 0_u64;
    let mut total = 0_u64;
    let mut total_free = 0_u64;
    // SAFETY: `wide` is NUL-terminated, and each output points to writable `u64` storage.
    let success =
        unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut available, &mut total, &mut total_free) };
    let source = EvidenceSource {
        kind: EvidenceSourceKind::WindowsApi,
        detail: "GetDiskFreeSpaceExW for selected artifact volume".into(),
    };
    if success != 0 {
        Evidence {
            value: Some(available),
            level: EvidenceLevel::Observed,
            source,
            observed_at_ms,
            notes: Vec::new(),
        }
    } else {
        Evidence {
            value: None,
            level: EvidenceLevel::Unknown,
            source,
            observed_at_ms,
            notes: vec![format!(
                "GetDiskFreeSpaceExW failed: {}",
                std::io::Error::last_os_error()
            )],
        }
    }
}

#[cfg(not(windows))]
pub fn available_disk_bytes(_path: &Path, observed_at_ms: u64) -> Evidence<u64> {
    Evidence {
        value: None,
        level: EvidenceLevel::Unknown,
        source: EvidenceSource {
            kind: EvidenceSourceKind::Unknown,
            detail: "Windows disk-capacity probe unavailable".into(),
        },
        observed_at_ms,
        notes: vec!["GetDiskFreeSpaceExW is available only on Windows.".into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(value: u64, level: EvidenceLevel) -> Evidence<u64> {
        Evidence {
            value: Some(value),
            level,
            source: EvidenceSource {
                kind: EvidenceSourceKind::Calculation,
                detail: "fixture".into(),
            },
            observed_at_ms: 42,
            notes: Vec::new(),
        }
    }

    fn storage_volume(resident_bytes: u64) -> StorageVolumeEvidence {
        StorageVolumeEvidence {
            volume_path: Evidence {
                value: Some("C:\\".into()),
                level: EvidenceLevel::Observed,
                source: EvidenceSource {
                    kind: EvidenceSourceKind::WindowsApi,
                    detail: "fixture volume".into(),
                },
                observed_at_ms: 42,
                notes: Vec::new(),
            },
            resident_bytes: bytes(resident_bytes, EvidenceLevel::Exact),
            available_bytes: bytes(10_737_418_240, EvidenceLevel::Observed),
        }
    }

    fn preflight_facts() -> PreflightFacts {
        PreflightFacts {
            execution_path: ExecutionPath::FullGpu,
            runtime_topology_known: true,
            unverified_requirements: Vec::new(),
            requested_context: 4_096,
            native_context: Some(8_192),
            runtime_fit_enabled: false,
            weight_bytes: 1_073_741_824,
            companion_bytes: 67_108_864,
            kv: KvCacheInputs {
                architecture: "llama".into(),
                block_count: Some(32),
                head_count_kv: Some(8),
                key_length: Some(128),
                value_length: Some(128),
                context: 4_096,
                cache_type_k: "f16".into(),
                cache_type_v: "f16".into(),
                recurrent_or_hybrid: false,
                observed_at_ms: 42,
            },
            available_memory: bytes(4_294_967_296, EvidenceLevel::Observed),
            available_disk: bytes(10_737_418_240, EvidenceLevel::Observed),
            storage_volumes: vec![storage_volume(1_140_850_688)],
            reserve_bytes: 536_870_912,
            offload_possible: true,
            assumptions: vec!["Fixture policy".into()],
        }
    }

    #[test]
    fn gpu_preflight_without_verified_runtime_topology_is_unknown() {
        let mut facts = preflight_facts();
        facts.runtime_topology_known = false;

        let report = build_preflight_report(facts);

        assert_eq!(report.class, FitClass::Unknown);
        assert!(report
            .unknowns
            .iter()
            .any(|unknown| unknown.contains("runtime-to-adapter")));
    }

    #[test]
    fn unresolved_companion_compatibility_blocks_positive_preflight() {
        let mut facts = preflight_facts();
        facts.execution_path = ExecutionPath::Cpu;
        facts.unverified_requirements =
            vec!["Draft model token sequence compatibility requires runtime validation.".into()];

        let report = build_preflight_report(facts);

        assert_eq!(report.class, FitClass::Unknown);
        assert!(report
            .unknowns
            .iter()
            .any(|unknown| unknown.contains("Draft model")));
    }

    #[test]
    fn calculates_dense_kv_bytes_only_from_complete_terms() {
        let evidence = estimate_kv_cache_bytes(&KvCacheInputs {
            architecture: "llama".into(),
            block_count: Some(32),
            head_count_kv: Some(8),
            key_length: Some(128),
            value_length: Some(128),
            context: 4096,
            cache_type_k: "f16".into(),
            cache_type_v: "f16".into(),
            recurrent_or_hybrid: false,
            observed_at_ms: 42,
        });

        assert_eq!(evidence.value, Some(536_870_912));
        assert_eq!(evidence.level, crate::evidence::EvidenceLevel::Derived);
        assert_eq!(
            evidence.source.kind,
            crate::evidence::EvidenceSourceKind::Calculation
        );
    }

    #[test]
    fn unsupported_architecture_kv_layout_is_unknown() {
        let evidence = estimate_kv_cache_bytes(&KvCacheInputs {
            architecture: "unverified-architecture".into(),
            block_count: Some(32),
            head_count_kv: Some(8),
            key_length: Some(128),
            value_length: Some(128),
            context: 4096,
            cache_type_k: "f16".into(),
            cache_type_v: "f16".into(),
            recurrent_or_hybrid: false,
            observed_at_ms: 42,
        });

        assert_eq!(evidence.level, EvidenceLevel::Unknown);
        assert!(evidence.notes.join(" ").contains("unverified-architecture"));
    }

    #[test]
    fn names_each_missing_kv_term_in_unknown_evidence() {
        let evidence = estimate_kv_cache_bytes(&KvCacheInputs {
            architecture: "llama".into(),
            block_count: Some(32),
            head_count_kv: Some(8),
            key_length: None,
            value_length: Some(128),
            context: 4096,
            cache_type_k: "f16".into(),
            cache_type_v: "f16".into(),
            recurrent_or_hybrid: false,
            observed_at_ms: 42,
        });

        assert_eq!(evidence.level, crate::evidence::EvidenceLevel::Unknown);
        assert!(evidence.notes.join(" ").contains("key_length"));
    }

    #[test]
    fn classifies_preflight_likely_only_after_named_reserve_fits() {
        let assessment = assess_memory(
            &bytes(1_000, EvidenceLevel::Exact),
            &bytes(200, EvidenceLevel::Derived),
            &bytes(1_500, EvidenceLevel::Observed),
            250,
            true,
            42,
        );

        assert_eq!(assessment.class, crate::evidence::FitClass::PreflightLikely);
        assert_eq!(assessment.required_bytes.value, Some(1_450));
        assert_eq!(assessment.policy_reserve_bytes, 250);
    }

    #[cfg(windows)]
    #[test]
    fn disk_capacity_uses_the_selected_artifact_volume() {
        let evidence = available_disk_bytes(std::env::temp_dir().as_path(), 42);

        assert!(evidence.value.is_some_and(|value| value > 0));
        assert_eq!(evidence.source.kind, EvidenceSourceKind::WindowsApi);
        assert!(evidence.source.detail.contains("GetDiskFreeSpaceExW"));
    }

    #[test]
    fn selected_storage_files_remain_separate_by_volume() {
        let files = vec![
            (std::path::PathBuf::from("C:/models/model-1.gguf"), 10),
            (std::path::PathBuf::from("C:/models/model-2.gguf"), 20),
            (std::path::PathBuf::from("D:/draft/draft.gguf"), 30),
        ];
        let volumes = storage_volumes_with(
            &files,
            42,
            |path: &Path, observed_at_ms| Evidence {
                value: Some(if path.to_string_lossy().starts_with("C:") {
                    "C:\\".into()
                } else {
                    "D:\\".into()
                }),
                level: EvidenceLevel::Observed,
                source: EvidenceSource {
                    kind: EvidenceSourceKind::WindowsApi,
                    detail: "fixture volume resolver".into(),
                },
                observed_at_ms,
                notes: Vec::new(),
            },
            |path: &Path, _observed_at_ms| {
                bytes(
                    if path.to_string_lossy().starts_with("C:") {
                        1_000
                    } else {
                        2_000
                    },
                    EvidenceLevel::Observed,
                )
            },
        );

        assert_eq!(volumes.len(), 2);
        assert_eq!(volumes[0].volume_path.value.as_deref(), Some("C:\\"));
        assert_eq!(volumes[0].resident_bytes.value, Some(30));
        assert_eq!(volumes[0].available_bytes.value, Some(1_000));
        assert_eq!(volumes[1].volume_path.value.as_deref(), Some("D:\\"));
        assert_eq!(volumes[1].resident_bytes.value, Some(30));
        assert_eq!(volumes[1].available_bytes.value, Some(2_000));
    }

    #[test]
    fn preflight_report_keeps_storage_memory_and_context_evidence_separate() {
        let report = build_preflight_report(PreflightFacts {
            execution_path: ExecutionPath::FullGpu,
            runtime_topology_known: true,
            unverified_requirements: Vec::new(),
            requested_context: 4_096,
            native_context: Some(8_192),
            runtime_fit_enabled: false,
            weight_bytes: 1_073_741_824,
            companion_bytes: 67_108_864,
            kv: KvCacheInputs {
                architecture: "llama".into(),
                block_count: Some(32),
                head_count_kv: Some(8),
                key_length: Some(128),
                value_length: Some(128),
                context: 4_096,
                cache_type_k: "f16".into(),
                cache_type_v: "f16".into(),
                recurrent_or_hybrid: false,
                observed_at_ms: 42,
            },
            available_memory: bytes(4_294_967_296, EvidenceLevel::Observed),
            available_disk: bytes(10_737_418_240, EvidenceLevel::Observed),
            storage_volumes: vec![StorageVolumeEvidence {
                volume_path: Evidence {
                    value: Some("C:\\".into()),
                    level: EvidenceLevel::Observed,
                    source: EvidenceSource {
                        kind: EvidenceSourceKind::WindowsApi,
                        detail: "fixture volume".into(),
                    },
                    observed_at_ms: 42,
                    notes: Vec::new(),
                },
                resident_bytes: bytes(1_140_850_688, EvidenceLevel::Exact),
                available_bytes: bytes(10_737_418_240, EvidenceLevel::Observed),
            }],
            reserve_bytes: 536_870_912,
            offload_possible: true,
            assumptions: vec!["Fixture policy".into()],
        });

        assert_eq!(report.class, FitClass::PreflightLikely);
        assert_eq!(report.storage_required_bytes.value, Some(1_140_850_688));
        assert_eq!(
            report.weight_bytes.value,
            report.storage_required_bytes.value
        );
        let expected_memory = report
            .storage_required_bytes
            .value
            .and_then(|bytes| bytes.checked_add(report.kv_cache_bytes.value?))
            .and_then(|bytes| bytes.checked_add(report.memory.policy_reserve_bytes));
        assert_eq!(report.memory.required_bytes.value, expected_memory);
        assert_eq!(report.requested_context.value, Some(4_096));
        assert_eq!(report.native_context.value, Some(8_192));
        assert_eq!(report.storage_volumes.len(), 1);
        assert_eq!(
            report.storage_volumes[0].resident_bytes.value,
            Some(1_140_850_688)
        );
        assert!(report.unknowns.is_empty());
    }

    #[test]
    fn preflight_labels_file_bytes_as_a_weight_proxy_not_exact_runtime_memory() {
        let report = build_preflight_report(preflight_facts());

        assert_eq!(report.weight_bytes.level, EvidenceLevel::Heuristic);
        assert!(report
            .weight_bytes
            .notes
            .iter()
            .any(|note| note.contains("runtime allocation")));
    }

    #[test]
    fn loaded_memory_proxy_overflow_does_not_erase_exact_storage() {
        let mut facts = preflight_facts();
        facts.weight_bytes = u64::MAX;
        facts.companion_bytes = 1;

        let report = build_preflight_report(facts);

        assert_eq!(report.weight_bytes.value, None);
        assert_eq!(report.weight_bytes.level, EvidenceLevel::Unknown);
        assert_eq!(report.storage_required_bytes.value, Some(1_140_850_688));
        assert_eq!(report.storage_required_bytes.level, EvidenceLevel::Exact);
        assert_eq!(report.class, FitClass::Unknown);
    }

    #[test]
    fn resident_artifact_footprint_does_not_require_duplicate_free_space() {
        let mut facts = preflight_facts();
        facts.execution_path = ExecutionPath::Cpu;
        facts.available_disk = bytes(1_000, EvidenceLevel::Observed);

        let report = build_preflight_report(facts);

        assert_eq!(report.class, FitClass::PreflightLikely);
        assert!(report.storage_required_bytes.value.unwrap() > 1_000);
        assert!(report
            .assumptions
            .iter()
            .any(|item| { item.contains("already resident") && item.contains("available disk") }));
    }

    #[test]
    fn exact_storage_footprint_uses_deduplicated_volume_facts() {
        let mut facts = preflight_facts();
        facts.weight_bytes = 100;
        facts.companion_bytes = 100;
        facts.storage_volumes = vec![storage_volume(100)];

        let report = build_preflight_report(facts);

        assert_eq!(report.storage_required_bytes.value, Some(100));
        assert_eq!(report.storage_required_bytes.level, EvidenceLevel::Exact);
    }

    #[test]
    fn preflight_fails_when_requested_context_exceeds_native_context() {
        let mut facts = preflight_facts();
        facts.requested_context = 16_384;

        let report = build_preflight_report(facts);

        assert_eq!(report.class, FitClass::Failed);
        assert!(report
            .assumptions
            .iter()
            .any(|item| item.contains("exceeds the GGUF native context")));
    }

    #[test]
    fn device_plan_is_explicit_for_one_device_and_unknown_for_multiple_devices() {
        let one = build_device_plan(
            &[("gpu-0".into(), bytes(200, EvidenceLevel::Observed))],
            100,
            &bytes(50, EvidenceLevel::Derived),
            42,
        );

        assert_eq!(one.len(), 1);
        assert_eq!(one[0].adapter_id, "gpu-0");
        assert_eq!(one[0].weight_bytes.level, EvidenceLevel::Heuristic);
        assert_eq!(one[0].total_bytes.value, Some(150));
        assert_eq!(one[0].available_bytes.value, Some(200));

        let multiple = build_device_plan(
            &[
                ("gpu-0".into(), bytes(200, EvidenceLevel::Observed)),
                ("gpu-1".into(), bytes(300, EvidenceLevel::Observed)),
            ],
            100,
            &bytes(50, EvidenceLevel::Derived),
            42,
        );

        assert!(multiple
            .iter()
            .all(|entry| entry.total_bytes.level == EvidenceLevel::Unknown));
        assert!(multiple
            .iter()
            .all(|entry| entry.note.contains("placement")));
    }
}
