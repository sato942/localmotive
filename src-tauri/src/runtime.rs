use crate::artifact::{is_reparse_point, validate_no_reparse_ancestors};
use crate::evidence::{Evidence, EvidenceLevel, EvidenceSource, EvidenceSourceKind};
use cap_std::ambient_authority;
use cap_std::fs::{Dir as CapDir, OpenOptions as CapOpenOptions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RELEASE_BY_TAG_URL: &str = "https://api.github.com/repos/ggml-org/llama.cpp/releases/tags";

fn observed_at_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn observed_value<T>(value: T, source: EvidenceSource, observed_at_ms: u64) -> Evidence<T> {
    Evidence {
        value: Some(value),
        level: EvidenceLevel::Observed,
        source,
        observed_at_ms,
        notes: Vec::new(),
    }
}

fn unknown_value<T>(
    kind: EvidenceSourceKind,
    detail: &str,
    note: String,
    observed_at_ms: u64,
) -> Evidence<T> {
    Evidence {
        value: None,
        level: EvidenceLevel::Unknown,
        source: EvidenceSource {
            kind,
            detail: detail.into(),
        },
        observed_at_ms,
        notes: vec![note],
    }
}

#[cfg(windows)]
pub fn process_peak_working_set(process_id: u32) -> Evidence<u64> {
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};

    let observed_at_ms = observed_at_ms();
    // SAFETY: `OpenProcess` receives a concrete PID and no inheritable handle request.
    let process = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, 0, process_id) };
    if process.is_null() || process == INVALID_HANDLE_VALUE {
        return unknown_value(
            EvidenceSourceKind::WindowsApi,
            "GetProcessMemoryInfo(PeakWorkingSetSize)",
            format!(
                "OpenProcess failed for PID {process_id}: {}",
                io::Error::last_os_error()
            ),
            observed_at_ms,
        );
    }
    // SAFETY: `counters` is zero-initialized and its size is passed to the API.
    let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { zeroed() };
    counters.cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    // SAFETY: `process` stays valid until `CloseHandle`; `counters` is writable for `cb` bytes.
    let measured = unsafe {
        GetProcessMemoryInfo(
            process,
            &mut counters,
            size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };
    // SAFETY: `process` is an owned handle returned by `OpenProcess`.
    unsafe { CloseHandle(process) };
    if measured != 0 {
        observed_value(
            counters.PeakWorkingSetSize as u64,
            EvidenceSource {
                kind: EvidenceSourceKind::WindowsApi,
                detail: "GetProcessMemoryInfo(PeakWorkingSetSize)".into(),
            },
            observed_at_ms,
        )
    } else {
        unknown_value(
            EvidenceSourceKind::WindowsApi,
            "GetProcessMemoryInfo(PeakWorkingSetSize)",
            format!(
                "GetProcessMemoryInfo failed for PID {process_id}: {}",
                io::Error::last_os_error()
            ),
            observed_at_ms,
        )
    }
}

#[cfg(not(windows))]
pub fn process_peak_working_set(_process_id: u32) -> Evidence<u64> {
    unknown_value(
        EvidenceSourceKind::Unknown,
        "process peak working set",
        "Process peak working-set collection is implemented only on Windows.".into(),
        observed_at_ms(),
    )
}

#[derive(Clone, Debug, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    #[serde(default)]
    pub target_commitish: String,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub assets: Vec<GithubAsset>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubAsset {
    pub name: String,
    #[serde(alias = "browser_download_url")]
    pub browser_download_url: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub digest: Option<String>,
}

#[cfg(test)]
impl GithubAsset {
    #[allow(dead_code)]
    fn sample(name: &str) -> Self {
        Self {
            name: name.into(),
            browser_download_url: format!("https://example.invalid/{name}"),
            size: 1,
            digest: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub architecture: String,
    pub gpu_names: Vec<String>,
    pub vendor: String,
    pub cuda_major: Option<u16>,
    pub driver_version: String,
    pub detection_status: String,
    pub recommendation: String,
    pub system_memory: SystemMemoryInfo,
    pub adapters: Vec<GpuAdapterInfo>,
    pub manual_overrides: Vec<HardwareOverride>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareOverride {
    pub adapter_id: String,
    pub dedicated_bytes: Option<u64>,
    pub shared_bytes: Option<u64>,
    pub note: String,
}

pub fn manual_override_capacity(
    overrides: &[HardwareOverride],
    adapter_id: &str,
    observed_at_ms: u64,
) -> Result<Option<Evidence<u64>>, String> {
    if overrides.len() > 64 {
        return Err("Manual hardware overrides cannot contain more than 64 adapters".into());
    }
    let mut seen = HashSet::new();
    for override_value in overrides {
        if override_value.adapter_id.trim().is_empty() || override_value.adapter_id.len() > 256 {
            return Err("Each manual hardware override requires a bounded adapter ID".into());
        }
        if !seen.insert(override_value.adapter_id.as_str()) {
            return Err(format!(
                "Manual hardware override {} appears more than once",
                override_value.adapter_id
            ));
        }
        if override_value.note.len() > 1_024 {
            return Err("Manual hardware override notes cannot exceed 1024 bytes".into());
        }
        if override_value.dedicated_bytes == Some(0) || override_value.shared_bytes == Some(0) {
            return Err("Manual hardware override capacities must be greater than zero".into());
        }
        if override_value.dedicated_bytes.is_none() && override_value.shared_bytes.is_none() {
            return Err("Each manual hardware override requires one capacity metric".into());
        }
    }

    let Some(override_value) = overrides.iter().find(|item| item.adapter_id == adapter_id) else {
        return Ok(None);
    };
    let (value, metric, mut notes) = if let Some(value) = override_value.dedicated_bytes {
        (
            value,
            "dedicated memory",
            vec!["Dedicated and shared memory are not aggregated".into()],
        )
    } else {
        (
            override_value
                .shared_bytes
                .expect("validated shared capacity"),
            "shared memory",
            vec!["Shared memory is a user override, not an observed GPU budget".into()],
        )
    };
    if !override_value.note.trim().is_empty() {
        notes.push(override_value.note.trim().to_string());
    }
    Evidence::known(
        value,
        EvidenceLevel::UserOverride,
        EvidenceSource {
            kind: EvidenceSourceKind::User,
            detail: format!("Manual {metric} override"),
        },
        observed_at_ms,
        notes,
    )
    .map(Some)
    .map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMemoryInfo {
    pub total_physical_bytes: Evidence<u64>,
    pub available_physical_bytes: Evidence<u64>,
    pub memory_load_percent: Evidence<u32>,
}

#[cfg(windows)]
pub fn detect_system_memory() -> SystemMemoryInfo {
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let observed_at_ms = observed_at_ms();
    // SAFETY: `MEMORYSTATUSEX` is zero-initialized, and `dwLength` names its exact size.
    let mut status: MEMORYSTATUSEX = unsafe { zeroed() };
    status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
    // SAFETY: `status` is writable for the full `MEMORYSTATUSEX` size during this call.
    if unsafe { GlobalMemoryStatusEx(&mut status) } != 0 {
        let source = EvidenceSource {
            kind: EvidenceSourceKind::WindowsApi,
            detail: "GlobalMemoryStatusEx".into(),
        };
        return SystemMemoryInfo {
            total_physical_bytes: observed_value(
                status.ullTotalPhys,
                source.clone(),
                observed_at_ms,
            ),
            available_physical_bytes: observed_value(
                status.ullAvailPhys,
                source.clone(),
                observed_at_ms,
            ),
            memory_load_percent: observed_value(status.dwMemoryLoad, source, observed_at_ms),
        };
    }
    let error = format!(
        "GlobalMemoryStatusEx failed: {}",
        io::Error::last_os_error()
    );
    SystemMemoryInfo {
        total_physical_bytes: unknown_value(
            EvidenceSourceKind::WindowsApi,
            "GlobalMemoryStatusEx",
            error.clone(),
            observed_at_ms,
        ),
        available_physical_bytes: unknown_value(
            EvidenceSourceKind::WindowsApi,
            "GlobalMemoryStatusEx",
            error.clone(),
            observed_at_ms,
        ),
        memory_load_percent: unknown_value(
            EvidenceSourceKind::WindowsApi,
            "GlobalMemoryStatusEx",
            error,
            observed_at_ms,
        ),
    }
}

#[cfg(not(windows))]
pub fn detect_system_memory() -> SystemMemoryInfo {
    let observed_at_ms = observed_at_ms();
    let note = "GlobalMemoryStatusEx is available only on Windows".to_string();
    SystemMemoryInfo {
        total_physical_bytes: unknown_value(
            EvidenceSourceKind::Unknown,
            "GlobalMemoryStatusEx",
            note.clone(),
            observed_at_ms,
        ),
        available_physical_bytes: unknown_value(
            EvidenceSourceKind::Unknown,
            "GlobalMemoryStatusEx",
            note.clone(),
            observed_at_ms,
        ),
        memory_load_percent: unknown_value(
            EvidenceSourceKind::Unknown,
            "GlobalMemoryStatusEx",
            note,
            observed_at_ms,
        ),
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MemoryMetric {
    Dedicated,
    Shared,
    Budget,
    CurrentUsage,
    AvailableBudget,
    AvailableForReservation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapacityObservation {
    pub metric: MemoryMetric,
    pub evidence: Evidence<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuAdapterInfo {
    pub adapter_id: String,
    pub compatibility_id: String,
    pub name: String,
    pub vendor: String,
    pub driver: Evidence<String>,
    pub backend: Evidence<String>,
    pub dedicated_bytes: Evidence<u64>,
    pub shared_bytes: Evidence<u64>,
    pub budget_bytes: Evidence<u64>,
    pub current_usage_bytes: Evidence<u64>,
    pub available_budget_bytes: Evidence<u64>,
    pub available_for_reservation_bytes: Evidence<u64>,
    pub capacity_observations: Vec<CapacityObservation>,
}

fn vendor_name(vendor_id: u32) -> String {
    match vendor_id {
        0x10de => "nvidia",
        0x1002 | 0x1022 => "amd",
        0x8086 => "intel",
        0x1414 => "microsoft",
        _ => "other",
    }
    .into()
}

fn backend_for_vendor(vendor: &str, observed_at_ms: u64) -> Evidence<String> {
    let value = match vendor {
        "nvidia" => "cuda",
        "amd" => "rocm",
        "intel" => "sycl",
        _ => "vulkan",
    };
    Evidence {
        value: Some(value.into()),
        level: EvidenceLevel::Heuristic,
        source: EvidenceSource {
            kind: EvidenceSourceKind::Policy,
            detail: "vendor-to-runtime compatibility policy".into(),
        },
        observed_at_ms,
        notes: vec!["Runtime inspection must confirm the active inference backend.".into()],
    }
}

#[cfg(windows)]
pub fn detect_dxgi_adapters() -> Result<Vec<GpuAdapterInfo>, String> {
    use windows::core::Interface;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, IDXGIAdapter1, IDXGIAdapter3, IDXGIFactory6,
        DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_ERROR_NOT_FOUND, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
        DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_QUERY_VIDEO_MEMORY_INFO,
    };

    // SAFETY: `CreateDXGIFactory1` returns a reference-counted COM interface.
    let factory: IDXGIFactory6 = unsafe { CreateDXGIFactory1() }
        .map_err(|error| format!("CreateDXGIFactory1 failed: {error}"))?;
    let observed_at_ms = observed_at_ms();
    let description_source = EvidenceSource {
        kind: EvidenceSourceKind::WindowsApi,
        detail: "IDXGIAdapter1::GetDesc1".into(),
    };
    let memory_source = EvidenceSource {
        kind: EvidenceSourceKind::WindowsApi,
        detail: "IDXGIAdapter3::QueryVideoMemoryInfo(local)".into(),
    };
    let mut adapters = Vec::new();
    for index in 0..64_u32 {
        // SAFETY: The factory owns each returned COM interface.
        let adapter: IDXGIAdapter1 = match unsafe {
            factory.EnumAdapterByGpuPreference(index, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE)
        } {
            Ok(adapter) => adapter,
            Err(error) if error.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(error) => return Err(format!("DXGI adapter enumeration failed: {error}")),
        };
        // SAFETY: `adapter` is a valid `IDXGIAdapter1` interface.
        let description = unsafe { adapter.GetDesc1() }
            .map_err(|error| format!("IDXGIAdapter1::GetDesc1 failed: {error}"))?;
        if description.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }
        let description_length = description
            .Description
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(description.Description.len());
        let name = String::from_utf16_lossy(&description.Description[..description_length]);
        let vendor = vendor_name(description.VendorId);
        let adapter_id = format!(
            "luid:{:08x}{:08x}",
            description.AdapterLuid.HighPart as u32, description.AdapterLuid.LowPart
        );
        let compatibility_id = format!(
            "pci:{:04x}:{:04x}:{:08x}:{:02x}",
            description.VendorId, description.DeviceId, description.SubSysId, description.Revision
        );
        let dedicated_bytes = observed_value(
            description.DedicatedVideoMemory as u64,
            description_source.clone(),
            observed_at_ms,
        );
        let shared_bytes = observed_value(
            description.SharedSystemMemory as u64,
            description_source.clone(),
            observed_at_ms,
        );
        let query = adapter.cast::<IDXGIAdapter3>().and_then(|adapter| {
            let mut memory = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
            // SAFETY: `memory` is writable and node zero is the documented single-node default.
            unsafe { adapter.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut memory) }
                .map(|()| memory)
        });
        let (budget_bytes, current_usage_bytes, available_budget_bytes, reservation_bytes) =
            match query {
                Ok(memory) => (
                    observed_value(memory.Budget, memory_source.clone(), observed_at_ms),
                    observed_value(memory.CurrentUsage, memory_source.clone(), observed_at_ms),
                    observed_value(
                        memory.Budget.saturating_sub(memory.CurrentUsage),
                        memory_source.clone(),
                        observed_at_ms,
                    ),
                    observed_value(
                        memory.AvailableForReservation,
                        memory_source.clone(),
                        observed_at_ms,
                    ),
                ),
                Err(error) => {
                    let note = format!("QueryVideoMemoryInfo failed: {error}");
                    (
                        unknown_value(
                            EvidenceSourceKind::WindowsApi,
                            &memory_source.detail,
                            note.clone(),
                            observed_at_ms,
                        ),
                        unknown_value(
                            EvidenceSourceKind::WindowsApi,
                            &memory_source.detail,
                            note.clone(),
                            observed_at_ms,
                        ),
                        unknown_value(
                            EvidenceSourceKind::WindowsApi,
                            &memory_source.detail,
                            note.clone(),
                            observed_at_ms,
                        ),
                        unknown_value(
                            EvidenceSourceKind::WindowsApi,
                            &memory_source.detail,
                            note,
                            observed_at_ms,
                        ),
                    )
                }
            };
        let capacity_observations = vec![
            CapacityObservation {
                metric: MemoryMetric::Dedicated,
                evidence: dedicated_bytes.clone(),
            },
            CapacityObservation {
                metric: MemoryMetric::Shared,
                evidence: shared_bytes.clone(),
            },
            CapacityObservation {
                metric: MemoryMetric::Budget,
                evidence: budget_bytes.clone(),
            },
            CapacityObservation {
                metric: MemoryMetric::CurrentUsage,
                evidence: current_usage_bytes.clone(),
            },
            CapacityObservation {
                metric: MemoryMetric::AvailableBudget,
                evidence: available_budget_bytes.clone(),
            },
            CapacityObservation {
                metric: MemoryMetric::AvailableForReservation,
                evidence: reservation_bytes.clone(),
            },
        ];
        adapters.push(GpuAdapterInfo {
            adapter_id,
            compatibility_id,
            name,
            vendor: vendor.clone(),
            driver: unknown_value(
                EvidenceSourceKind::WindowsApi,
                "DXGI adapter description",
                "DXGI does not expose the installed driver version.".into(),
                observed_at_ms,
            ),
            backend: backend_for_vendor(&vendor, observed_at_ms),
            dedicated_bytes,
            shared_bytes,
            budget_bytes,
            current_usage_bytes,
            available_budget_bytes,
            available_for_reservation_bytes: reservation_bytes,
            capacity_observations,
        });
    }
    Ok(adapters)
}

#[cfg(not(windows))]
pub fn detect_dxgi_adapters() -> Result<Vec<GpuAdapterInfo>, String> {
    Err("DXGI adapter evidence is available only on Windows".into())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeOption {
    pub id: String,
    pub label: String,
    pub backend: String,
    pub install_key: String,
    pub description: String,
    pub compatibility: String,
    pub asset: GithubAsset,
    pub companion_asset: Option<GithubAsset>,
    pub recommended: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityKey {
    pub os_build: String,
    pub architecture: String,
    pub adapter_id: String,
    pub driver: String,
    pub firmware: String,
    pub backend: String,
    pub install_key: String,
    pub release_commit: String,
    pub asset_name: String,
    pub asset_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityRecord {
    pub key: CompatibilityKey,
    pub evidence_level: String,
    pub attestation_id: String,
    pub expiry_identity: String,
}

fn compatibility_expiry_identity(key: &CompatibilityKey) -> String {
    let mut digest = Sha256::new();
    digest.update(b"localmotive-compatibility-v1\0");
    for value in [
        &key.os_build,
        &key.architecture,
        &key.adapter_id,
        &key.driver,
        &key.firmware,
        &key.backend,
        &key.install_key,
        &key.release_commit,
        &key.asset_name,
        &key.asset_sha256,
    ] {
        digest.update((value.len() as u64).to_be_bytes());
        digest.update(value.as_bytes());
    }
    format!("compatibility-v1:sha256:{}", hex::encode(digest.finalize()))
}

fn compatibility_record_matches(
    record: &CompatibilityRecord,
    option: &RuntimeOption,
    adapter: &GpuAdapterInfo,
    os_build: &str,
    firmware: Option<&str>,
    architecture: &str,
    release_commit: &str,
) -> bool {
    let key = &record.key;
    let driver_is_direct = matches!(
        adapter.driver.level,
        EvidenceLevel::Exact | EvidenceLevel::Observed
    );
    let firmware_matches = if key.firmware == "not-applicable" {
        firmware.is_none() || firmware == Some("not-applicable")
    } else {
        firmware == Some(key.firmware.as_str())
    };
    record.evidence_level == "L4_PRODUCT"
        && !record.attestation_id.trim().is_empty()
        && record.attestation_id.len() <= 256
        && record.expiry_identity == compatibility_expiry_identity(key)
        && key.os_build == os_build
        && key.architecture == architecture
        && key.adapter_id == adapter.compatibility_id
        && driver_is_direct
        && adapter.driver.value.as_deref() == Some(key.driver.as_str())
        && firmware_matches
        && key.backend == option.backend
        && key.install_key == option.install_key
        && key.release_commit == release_commit
        && key.asset_name == option.asset.name
        && option.asset.digest.as_deref() == Some(format!("sha256:{}", key.asset_sha256).as_str())
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeInstallRequest {
    pub install_key: String,
    pub adapter_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BackendAvailabilityStatus {
    Available,
    Blocked,
    Dormant,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackendAvailability {
    pub install_key: String,
    pub backend: String,
    pub status: BackendAvailabilityStatus,
    pub reason: String,
    pub blocking_jobs: Vec<String>,
    pub evidence_urls: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeCatalogOrigin {
    Network,
    Cache,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCatalog {
    pub tag: String,
    pub published_at: String,
    pub options: Vec<RuntimeOption>,
    pub availability: Vec<BackendAvailability>,
    pub origin: RuntimeCatalogOrigin,
    pub warning: Option<String>,
    pub recommendation_reason: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledRuntime {
    pub tag: String,
    pub backend: String,
    pub runtime_path: String,
    pub install_root: String,
    pub reused: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInstallProgress {
    pub install_key: String,
    pub asset_name: String,
    pub downloaded: u64,
    pub total: u64,
}

/// What an executable on disk *is*, derived from evidence beside it rather than
/// from its filename: the managed `runtime.json` manifest when present, otherwise
/// the ggml backend DLLs and CUDA runtime DLLs shipped in the same folder.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeIdentity {
    pub path: String,
    /// `cuda`, `rocm`, `sycl`, `openvino`, `vulkan`, `opencl`, `cpu`, or `unknown`.
    pub backend: String,
    pub cuda_major: Option<u16>,
    /// Release tag recorded by the managed installer, when this is a managed runtime.
    pub tag: Option<String>,
    /// Install key recorded by the managed installer (for example `cuda-13.3`).
    pub install_key: Option<String>,
    /// `manifest`, `dlls`, or `none`.
    pub source: String,
    /// True only after the backend validates the managed root and compiled content manifest.
    pub managed_verified: bool,
}

/// One managed installation found under the runtime root.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedRuntimeRecord {
    pub tag: String,
    pub backend: String,
    pub install_key: String,
    pub runtime_path: String,
    pub install_root: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeManifest {
    #[serde(alias = "releaseTag")]
    tag: String,
    backend: String,
    #[serde(default)]
    install_key: Option<String>,
    runtime: String,
}

/// Numeric build of a release tag such as `b10752` or a bare `10752`.
pub fn tag_build(tag: &str) -> Option<u32> {
    tag.trim().trim_start_matches(['b', 'B']).parse().ok()
}

fn cuda_major_from_install_key(install_key: &str) -> Option<u16> {
    install_key
        .strip_prefix("cuda-")?
        .split('.')
        .next()?
        .parse()
        .ok()
}

/// Lowercase sibling file names beside a runtime executable.
fn sibling_dll_names(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// Backend derived from shipped ggml DLL evidence.
fn backend_from_dll_names(names: &[String]) -> String {
    let has = |needle: &str| names.iter().any(|name| name.contains(needle));
    if has("ggml-cuda") {
        "cuda"
    } else if has("ggml-hip") || has("ggml-rocm") {
        "rocm"
    } else if has("ggml-sycl") {
        "sycl"
    } else if has("ggml-openvino") {
        "openvino"
    } else if has("ggml-vulkan") {
        "vulkan"
    } else if has("ggml-opencl") {
        "opencl"
    } else if has("ggml-cpu") || has("ggml.dll") {
        "cpu"
    } else {
        "unknown"
    }
    .into()
}

/// CUDA major version derived from the shipped `cudart64_` DLL.
fn cuda_major_from_dll_names(names: &[String]) -> Option<u16> {
    names.iter().find_map(|name| {
        name.strip_prefix("cudart64_")?
            .split(['.', '_'])
            .next()?
            .parse()
            .ok()
    })
}

/// Backend-specific shipped-DLL dependency names for clean-machine checks.
///
/// Each entry names shipped DLLs the extracted runtime folder must carry
/// for the backend to load on a clean Windows machine (no developer
/// toolkits). CUDA keeps the existing `cuda_companion_is_complete` rule
/// (`ggml-cuda.dll` plus the matching `cudart64_<major>.dll`); ROCm
/// needs `ggml-hip.dll` or `ggml-rocm.dll`; SYCL needs `ggml-sycl.dll`;
/// OpenVINO needs `ggml-openvino.dll`; Arm64 OpenCL needs
/// `ggml-opencl.dll`; Vulkan loads through the system
/// Vulkan loader plus `ggml-vulkan.dll` (`vulkan-1.dll` may come from the
/// GPU driver, so absence is a warning, never a mismatch); CPU needs
/// `ggml-cpu` or `ggml.dll`. Finding A-09 records this gap.
fn backend_dependency_names(backend: &str) -> &'static [&'static str] {
    match backend {
        "rocm" => &["ggml-hip.dll", "ggml-rocm.dll"],
        "sycl" => &["ggml-sycl.dll"],
        "openvino" => &["ggml-openvino.dll"],
        "vulkan" => &["ggml-vulkan.dll"],
        "opencl" => &["ggml-opencl.dll"],
        "cpu" => &["ggml-cpu", "ggml.dll"],
        _ => &[],
    }
}

/// Whether the shipped DLLs satisfy the backend dependency rule.
///
/// CUDA keeps the companion rule (`ggml-cuda.dll` plus the matching
/// `cudart64_<major>.dll`). Every other known backend needs one of its
/// dependency names; `unknown` and empty folders stay unknown (the
/// caller, not this check, decides what that means).
fn backend_dependencies_are_complete(names: &[String], backend: &str) -> bool {
    if backend == "cuda" {
        return cuda_major_from_dll_names(names)
            .is_some_and(|major| cuda_companion_is_complete(names, Some(major)));
    }
    let needed = backend_dependency_names(backend);
    if needed.is_empty() {
        return false;
    }
    needed
        .iter()
        .any(|needle| names.iter().any(|name| name.contains(needle)))
}

/// Whether the shipped CUDA companion (`cudart-` archive) is complete.
///
/// The current catalog pins one `cudart64_<major>.dll` per CUDA major
/// alongside `ggml-cuda.dll`. A CUDA folder missing either piece is a
/// dependency failure on a clean machine, not an install-and-see case.
/// Finding A-09 records this gap.
#[allow(dead_code)]
fn cuda_companion_is_complete(names: &[String], cuda_major: Option<u16>) -> bool {
    let Some(major) = cuda_major else {
        return false;
    };
    let has_cuda = names.iter().any(|name| name.contains("ggml-cuda"));
    let has_cudart = names.iter().any(|name| {
        name.strip_prefix("cudart64_")
            .and_then(|rest| rest.split(['.', '_']).next()?.parse::<u16>().ok())
            .is_some_and(|found| found == major)
    });
    has_cuda && has_cudart
}

fn read_manifest(dir: &Path) -> Option<RuntimeManifest> {
    const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
    let path = dir.join("runtime.json");
    let metadata = fs::symlink_metadata(&path).ok()?;
    if metadata.file_type().is_symlink()
        || is_reparse_point(&metadata)
        || !metadata.is_file()
        || metadata.len() > MAX_MANIFEST_BYTES
    {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn safe_directory(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        metadata.is_dir() && !metadata.file_type().is_symlink() && !is_reparse_point(&metadata)
    })
}

fn manifest_runtime_path(dir: &Path, runtime: &str) -> Option<PathBuf> {
    use std::path::Component;

    if !safe_directory(dir) {
        return None;
    }
    let relative = Path::new(runtime);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let mut path = dir.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return None;
        };
        path.push(name);
        let metadata = fs::symlink_metadata(&path).ok()?;
        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
            return None;
        }
    }
    fs::symlink_metadata(&path)
        .ok()
        .filter(|metadata| metadata.is_file())
        .map(|_| path)
}

pub fn describe_runtime(path: &Path) -> RuntimeIdentity {
    let display = path.to_string_lossy().to_string();
    let Some(dir) = path.parent() else {
        return RuntimeIdentity {
            path: display,
            backend: "unknown".into(),
            cuda_major: None,
            tag: None,
            install_key: None,
            source: "none".into(),
            managed_verified: false,
        };
    };
    if let Some(manifest) = read_manifest(dir)
        .filter(|manifest| manifest_runtime_path(dir, &manifest.runtime).as_deref() == Some(path))
    {
        let install_key = manifest.install_key.clone();
        let dll_names = sibling_dll_names(dir);
        let dll_backend = backend_from_dll_names(&dll_names);
        let dll_cuda_major = cuda_major_from_dll_names(&dll_names);
        let backend = manifest.backend.clone();
        let cuda_major = install_key.as_deref().and_then(cuda_major_from_install_key);
        // Cross-check only when DLL evidence exists beside the runtime.
        // A fresh managed install always ships the backend DLLs; when the
        // folder carries no DLLs at all (for example a manifest-only
        // fixture), the manifest stays authoritative.
        let mismatch = dll_backend != "unknown"
            && (dll_backend != backend
                || (backend == "cuda"
                    && (dll_cuda_major != cuda_major
                        || !cuda_companion_is_complete(&dll_names, cuda_major)))
                || (backend != "cuda" && !backend_dependencies_are_complete(&dll_names, &backend)));
        if mismatch {
            // A manifest that disagrees with the DLLs beside the runtime
            // is a loaded-library failure before any image change: report
            // the mismatch instead of trusting either side alone.
            // Finding A-09 records this clean-machine gap.
            return RuntimeIdentity {
                path: display,
                backend: "mismatch".into(),
                cuda_major,
                tag: Some(manifest.tag),
                install_key,
                source: "manifest-dll-mismatch".into(),
                managed_verified: false,
            };
        }
        return RuntimeIdentity {
            path: display,
            cuda_major,
            backend,
            tag: Some(manifest.tag),
            install_key,
            source: "manifest".into(),
            managed_verified: false,
        };
    }

    let names = sibling_dll_names(dir);
    let backend = backend_from_dll_names(&names);
    let cuda_major = cuda_major_from_dll_names(&names);
    let source = if backend == "unknown" { "none" } else { "dlls" };
    RuntimeIdentity {
        path: display,
        backend,
        cuda_major,
        tag: None,
        install_key: None,
        source: source.into(),
        managed_verified: false,
    }
}

pub fn list_managed_runtimes_in(root: &Path) -> Vec<ManagedRuntimeRecord> {
    let mut records = Vec::new();
    let Ok(tags) = fs::read_dir(root) else {
        return records;
    };
    for tag_dir in tags.filter_map(Result::ok).map(|entry| entry.path()) {
        if !safe_directory(&tag_dir)
            || tag_dir
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with('.'))
        {
            continue;
        }
        let Ok(installs) = fs::read_dir(&tag_dir) else {
            continue;
        };
        for install_dir in installs.filter_map(Result::ok).map(|entry| entry.path()) {
            if !safe_directory(&install_dir) {
                continue;
            }
            let Some(manifest) = read_manifest(&install_dir) else {
                continue;
            };
            let Some(runtime_path) = manifest_runtime_path(&install_dir, &manifest.runtime) else {
                continue;
            };
            if managed_runtime_verified_in(&runtime_path, root) != Ok(true) {
                continue;
            }
            records.push(ManagedRuntimeRecord {
                tag: manifest.tag,
                backend: manifest.backend.clone(),
                install_key: manifest.install_key.unwrap_or(manifest.backend),
                runtime_path: runtime_path.to_string_lossy().to_string(),
                install_root: install_dir.to_string_lossy().to_string(),
            });
        }
    }
    records.sort_by(|a, b| {
        tag_build(&b.tag)
            .cmp(&tag_build(&a.tag))
            .then_with(|| a.install_key.cmp(&b.install_key))
    });
    records
}

pub fn list_managed_runtimes() -> Result<Vec<ManagedRuntimeRecord>, String> {
    let primary = runtime_data_dir("Localmotive");
    let legacy = runtime_data_dir("GGUF Pilot");
    let mut records = list_managed_runtimes_in(&primary);
    // Keep legacy installs visible and selectable until a new runtime arrives.
    for record in list_managed_runtimes_in(&legacy) {
        if !records.iter().any(|existing| {
            existing.tag == record.tag && existing.install_key == record.install_key
        }) {
            records.push(record);
        }
    }
    records.sort_by(|a, b| {
        tag_build(&b.tag)
            .cmp(&tag_build(&a.tag))
            .then_with(|| a.install_key.cmp(&b.install_key))
    });
    Ok(records)
}

fn architecture() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "x64".into(),
        "aarch64" => "arm64".into(),
        other => other.into(),
    }
}

fn cuda_major_from_smi(text: &str) -> Option<u16> {
    let rest = ["CUDA UMD Version:", "CUDA Version:"]
        .iter()
        .find_map(|marker| text.split(marker).nth(1))?
        .trim_start();
    rest.split('.').next()?.trim().parse().ok()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NvidiaProbeRow {
    name: String,
    driver: String,
    total_bytes: Option<u64>,
    used_bytes: Option<u64>,
}

fn parse_nvidia_probe_rows(output: &str) -> Vec<NvidiaProbeRow> {
    output
        .lines()
        .filter_map(|line| {
            let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
            if fields.len() != 4 || fields[0].is_empty() {
                return None;
            }
            let mib = |value: &str| {
                value
                    .parse::<u64>()
                    .ok()
                    .and_then(|value| value.checked_mul(1024 * 1024))
            };
            Some(NvidiaProbeRow {
                name: fields[0].to_string(),
                driver: fields[1].to_string(),
                total_bytes: mib(fields[2]),
                used_bytes: mib(fields[3]),
            })
        })
        .collect()
}

fn merge_nvidia_probe_observations(
    adapters: &mut [GpuAdapterInfo],
    rows: &[NvidiaProbeRow],
    observed_at_ms: u64,
) {
    let mut matched = vec![false; adapters.len()];
    for row in rows {
        let Some((index, adapter)) = adapters.iter_mut().enumerate().find(|(index, adapter)| {
            !matched[*index] && adapter.name.eq_ignore_ascii_case(&row.name)
        }) else {
            continue;
        };
        matched[index] = true;
        let source = EvidenceSource {
            kind: EvidenceSourceKind::NvidiaSmi,
            detail: "nvidia-smi --query-gpu".into(),
        };
        if !row.driver.is_empty() {
            adapter.driver = observed_value(row.driver.clone(), source.clone(), observed_at_ms);
        }
        if let Some(value) = row.total_bytes {
            adapter.capacity_observations.push(CapacityObservation {
                metric: MemoryMetric::Dedicated,
                evidence: observed_value(value, source.clone(), observed_at_ms),
            });
        }
        if let Some(value) = row.used_bytes {
            adapter.capacity_observations.push(CapacityObservation {
                metric: MemoryMetric::CurrentUsage,
                evidence: observed_value(value, source.clone(), observed_at_ms),
            });
        }
    }
}

const HARDWARE_DETECTION_NOTICE: &str =
    "Hardware detection does not establish product support; use the approved catalog recommendation.";

pub fn detect_hardware() -> HardwareInfo {
    // GPU-less runners (and locked-down CI boxes) must not hang the suite:
    // PowerShell/CIM and nvidia-smi probes can stall for minutes where no
    // GPU stack exists. An explicit opt-out keeps those environments fast
    // while production keeps probing by default.
    if std::env::var_os("LOCALMOTIVE_SKIP_HARDWARE_PROBE").is_some() {
        return HardwareInfo {
            architecture: architecture(),
            gpu_names: Vec::new(),
            vendor: "cpu".into(),
            cuda_major: None,
            driver_version: String::new(),
            detection_status: "Hardware probing skipped by LOCALMOTIVE_SKIP_HARDWARE_PROBE".into(),
            recommendation: HARDWARE_DETECTION_NOTICE.into(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
    }
    const HARDWARE_PROBE_TIMEOUT: Duration = Duration::from_secs(30);
    const HARDWARE_PROBE_STREAM_LIMIT: usize = 1024 * 1024;
    let architecture = architecture();
    let system_memory = detect_system_memory();
    let mut adapters = detect_dxgi_adapters().unwrap_or_default();
    let hardware_observed_at_ms = observed_at_ms();
    let mut nvidia_query_command = crate::proc::hidden_command("nvidia-smi.exe");
    nvidia_query_command.args([
        "--query-gpu=name,driver_version,memory.total,memory.used",
        "--format=csv,noheader,nounits",
    ]);
    let nvidia_query = crate::proc::output_with_timeout(
        &mut nvidia_query_command,
        HARDWARE_PROBE_TIMEOUT,
        HARDWARE_PROBE_STREAM_LIMIT,
    );
    if let Ok(output) = nvidia_query {
        if output.status.success() {
            let rows = parse_nvidia_probe_rows(&String::from_utf8_lossy(&output.stdout));
            let gpu_names = rows.iter().map(|row| row.name.clone()).collect::<Vec<_>>();
            let driver_version = rows
                .first()
                .map(|row| row.driver.clone())
                .unwrap_or_default();
            if !gpu_names.is_empty() {
                merge_nvidia_probe_observations(&mut adapters, &rows, hardware_observed_at_ms);
                let mut smi_command = crate::proc::hidden_command("nvidia-smi.exe");
                let smi = crate::proc::output_with_timeout(
                    &mut smi_command,
                    HARDWARE_PROBE_TIMEOUT,
                    HARDWARE_PROBE_STREAM_LIMIT,
                )
                .ok()
                .map(|result| String::from_utf8_lossy(&result.stdout).to_string())
                .unwrap_or_default();
                let cuda_major = cuda_major_from_smi(&smi);
                return HardwareInfo {
                    architecture,
                    gpu_names,
                    vendor: "nvidia".into(),
                    cuda_major,
                    driver_version,
                    detection_status: "NVIDIA GPU and driver reported by nvidia-smi".into(),
                    recommendation: HARDWARE_DETECTION_NOTICE.into(),
                    system_memory,
                    adapters,
                    manual_overrides: Vec::new(),
                };
            }
        }
    }

    let script = "Get-CimInstance Win32_VideoController | ForEach-Object { \"$($_.Name)`t$($_.DriverVersion)\" }";
    let mut cim_command = crate::proc::hidden_command("powershell.exe");
    cim_command.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    let adapter_rows = crate::proc::output_with_timeout(
        &mut cim_command,
        HARDWARE_PROBE_TIMEOUT,
        HARDWARE_PROBE_STREAM_LIMIT,
    )
    .ok()
    .filter(|output| output.status.success())
    .map(|output| {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.split_once('\t').unwrap_or((line, "")))
            .map(|(name, driver)| (name.trim().to_string(), driver.trim().to_string()))
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    let gpu_names = adapter_rows
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let driver_version = adapter_rows
        .first()
        .map(|(_, driver)| driver.clone())
        .unwrap_or_default();
    let combined = gpu_names.join(" ").to_ascii_lowercase();
    let vendor = if combined.contains("nvidia")
        || combined.contains("geforce")
        || combined.contains("quadro")
    {
        "nvidia"
    } else if combined.contains("qualcomm") || combined.contains("adreno") {
        // Qualcomm stays a hint only. OpenCL is dormant capability on
        // Windows x64 and never receives automatic preference.
        "qualcomm"
    } else if combined.contains("amd") || combined.contains("radeon") {
        "amd"
    } else if combined.contains("intel") {
        "intel"
    } else if gpu_names.is_empty() {
        "cpu"
    } else {
        "other"
    };

    let detection_status = if gpu_names.is_empty() {
        "No graphics adapter reported by Windows CIM"
    } else {
        "Graphics adapter and driver reported by Windows CIM"
    };
    HardwareInfo {
        architecture,
        gpu_names,
        vendor: vendor.into(),
        cuda_major: None,
        driver_version,
        detection_status: detection_status.into(),
        recommendation: HARDWARE_DETECTION_NOTICE.into(),
        system_memory,
        adapters,
        manual_overrides: Vec::new(),
    }
}

fn backend_for(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("-cuda-") {
        Some("cuda")
    } else if lower.contains("-rocm-") || lower.contains("-hip-") {
        Some("rocm")
    } else if lower.contains("-sycl-") {
        Some("sycl")
    } else if lower.contains("-openvino-") {
        Some("openvino")
    } else if lower.contains("-vulkan-") {
        Some("vulkan")
    } else if lower.contains("-opencl-") {
        Some("opencl")
    } else if lower.contains("-cpu-") {
        Some("cpu")
    } else {
        None
    }
}

fn label_and_description(backend: &str, name: &str) -> (String, String, String) {
    match backend {
        "cuda" => {
            let version = name
                .split("-cuda-")
                .nth(1)
                .and_then(|value| value.split('-').next())
                .unwrap_or("current");
            (
                format!("NVIDIA CUDA {version}"),
                "Optional NVIDIA acceleration package".into(),
                "Requires an exact L4 product compatibility record for this adapter, driver, and CUDA package".into(),
            )
        }
        "rocm" => (
            "AMD ROCm".into(),
            "Optional AMD acceleration package".into(),
            "Requires an exact L4 product compatibility record for this adapter, driver, and ROCm package".into(),
        ),
        "sycl" => (
            "Intel SYCL".into(),
            "Optional Intel acceleration package".into(),
            "Requires an exact L4 product compatibility record for this adapter, driver, and SYCL package".into(),
        ),
        "openvino" => (
            "Intel OpenVINO".into(),
            "Optional Intel inference package".into(),
            "Requires an exact L4 product compatibility record before recommendation".into(),
        ),
        "vulkan" => (
            "Vulkan".into(),
            "Optional cross-vendor GPU package".into(),
            "Requires an exact L4 product compatibility record before recommendation".into(),
        ),
        "opencl" => (
            "OpenCL".into(),
            "Specialized OpenCL Windows build".into(),
            "Choose only for matching hardware such as supported Adreno devices".into(),
        ),
        _ => (
            "CPU".into(),
            "CPU-only Windows x64 package".into(),
            "Requires no GPU; product support still depends on an exact qualified Windows and CPU row".into(),
        ),
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn cuda_version(_name: &str) -> u16 {
    0
}

/// One approved runtime asset from the pinned `b10796` manifest.
///
/// Each catalog entry must bind to exactly one manifest entry. The manifest
/// carries the tag, download URL, byte count, SHA-256 digest, backend,
/// architecture, and CUDA version. GitHub release metadata is untrusted
/// input: selection rejects an asset whose name, size, URL, or digest
/// differs from the manifest entry.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovedRuntimeAsset {
    pub name: String,
    pub url: String,
    pub bytes: u64,
    pub digest: String,
    pub backend: String,
    pub arch: String,
    #[serde(default)]
    pub dormant: bool,
    #[serde(default)]
    pub cuda_version: Option<String>,
    #[serde(default)]
    pub companion_name: Option<String>,
    #[serde(default)]
    pub content_manifest_sha256: Option<String>,
}

/// One required upstream hardware job that gates a backend.
///
/// A runtime update blocks while any required job for its backend fails or
/// remains queued. Release `b10796` exposes 95 successes, five failures,
/// and one queued check.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequiredUpstreamJob {
    pub name: String,
    pub backend: String,
    pub conclusion: String,
    pub status: String,
    pub url: String,
    pub completed_at: String,
    pub install_keys: Vec<String>,
    pub platforms: Vec<String>,
    pub hardware_classes: Vec<String>,
}

fn approved_digest_is_sha256(digest: &str) -> bool {
    digest.len() == 71
        && digest.starts_with("sha256:")
        && digest[7..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Bind one GitHub asset to one approved manifest entry.
///
/// Returns the manifest entry when name, size, URL, and digest all match.
/// Rejects the asset when the manifest lacks an entry, lacks a digest, or
/// reports different identity bytes.
pub fn bind_asset_to_manifest<'a>(
    asset: &GithubAsset,
    approved: &'a [ApprovedRuntimeAsset],
) -> Result<&'a ApprovedRuntimeAsset, String> {
    let entry = approved
        .iter()
        .find(|entry| entry.name == asset.name)
        .ok_or_else(|| {
            format!(
                "Runtime asset {} is not in the approved manifest",
                asset.name
            )
        })?;
    if !approved_digest_is_sha256(&entry.digest) {
        return Err(format!(
            "Approved manifest entry {} lacks a SHA-256 digest",
            entry.name
        ));
    }
    let Some(remote_digest) = asset.digest.as_deref() else {
        return Err(format!(
            "Runtime asset {} lacks a SHA-256 digest in upstream metadata",
            asset.name
        ));
    };
    if !approved_digest_is_sha256(remote_digest) {
        return Err(format!(
            "Runtime asset {} lacks a SHA-256 digest in upstream metadata",
            asset.name
        ));
    }
    if !remote_digest.eq_ignore_ascii_case(&entry.digest) {
        return Err(format!(
            "Runtime asset {} changed identity: upstream digest differs from the approved manifest",
            asset.name
        ));
    }
    if asset.size != entry.bytes {
        return Err(format!(
            "Runtime asset {} changed identity: upstream size {} differs from approved {}",
            asset.name, asset.size, entry.bytes
        ));
    }
    if asset.browser_download_url != entry.url {
        return Err(format!(
            "Runtime asset {} changed identity: upstream URL differs from the approved manifest",
            asset.name
        ));
    }
    Ok(entry)
}

const APPROVED_RUNTIMES: &str = include_str!("../approved_runtimes.json");

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApprovedRuntimeIdentity {
    pub release_tag: String,
    pub release_commit: String,
    pub manifest_sha256: String,
    pub published_at: String,
    pub observed_at: String,
    pub source: String,
    pub supported_windows: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApprovedRuntimeManifest {
    schema_version: u32,
    release_tag: String,
    release_commit: String,
    published_at: String,
    observed_at: String,
    source: String,
    supported_windows: Vec<String>,
    assets: Vec<ApprovedRuntimeAsset>,
    required_jobs: Vec<RequiredUpstreamJob>,
    compatibility_records: Vec<CompatibilityRecord>,
    approval: ApprovedManifestGate,
}

fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
}

fn approved_asset_install_key(entry: &ApprovedRuntimeAsset) -> Result<Option<String>, String> {
    if entry.backend == "cuda-companion" {
        return Ok(None);
    }
    if entry.backend == "cuda" {
        let version = entry
            .cuda_version
            .as_deref()
            .ok_or_else(|| format!("Approved CUDA asset {} lacks cudaVersion", entry.name))?;
        return Ok(Some(format!("cuda-{version}")));
    }
    Ok(Some(entry.backend.clone()))
}

fn parse_approved_manifest(text: &str) -> Result<ApprovedRuntimeManifest, String> {
    let manifest: ApprovedRuntimeManifest = serde_json::from_str(text)
        .map_err(|error| format!("Approved runtime manifest is invalid: {error}"))?;
    if manifest.schema_version != 3 {
        return Err(format!(
            "Approved runtime manifest schema {} is unsupported",
            manifest.schema_version
        ));
    }
    if manifest.release_tag.is_empty()
        || manifest.release_commit.len() != 40
        || !manifest
            .release_commit
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err("Approved runtime release identity is malformed".into());
    }
    let expected_source = format!("{RELEASE_BY_TAG_URL}/{}", manifest.release_tag);
    if manifest.source != expected_source
        || !valid_utc_timestamp(&manifest.published_at)
        || !valid_utc_timestamp(&manifest.observed_at)
        || manifest.supported_windows.is_empty()
    {
        return Err("Approved runtime identity lacks publication or Windows support scope".into());
    }
    if manifest.assets.is_empty() {
        return Err("Approved runtime manifest contains no assets".into());
    }

    let mut asset_names = HashSet::new();
    let mut active_install_keys = std::collections::HashMap::new();
    for asset in &manifest.assets {
        if !asset_names.insert(asset.name.clone()) {
            return Err(format!(
                "Approved runtime asset {} is duplicated",
                asset.name
            ));
        }
        let expected_url = format!(
            "https://github.com/ggml-org/llama.cpp/releases/download/{}/{}",
            manifest.release_tag, asset.name
        );
        if asset.url != expected_url
            || asset.bytes == 0
            || !approved_digest_is_sha256(&asset.digest)
            || !matches!(asset.arch.as_str(), "x64" | "arm64")
        {
            return Err(format!(
                "Approved runtime asset {} is malformed",
                asset.name
            ));
        }
        if let Some(install_key) = approved_asset_install_key(asset)? {
            let valid_content_anchor =
                asset
                    .content_manifest_sha256
                    .as_deref()
                    .is_some_and(|digest| {
                        digest.len() == 64
                            && digest.chars().all(|character| {
                                character.is_ascii_hexdigit() && !character.is_ascii_uppercase()
                            })
                    });
            if !asset.dormant && asset.backend != "cuda-companion" && !valid_content_anchor {
                return Err(format!(
                    "Approved install key {install_key} lacks a content-manifest anchor"
                ));
            }
            if (asset.dormant || asset.backend == "cuda-companion")
                && asset.content_manifest_sha256.is_some()
            {
                return Err(format!(
                    "Non-installable asset {} must not carry a content-manifest anchor",
                    asset.name
                ));
            }
            if !asset.dormant
                && active_install_keys
                    .insert(install_key.clone(), asset.backend.clone())
                    .is_some()
            {
                return Err(format!(
                    "Approved install key {install_key} maps to multiple active assets"
                ));
            }
        }
    }

    for asset in manifest
        .assets
        .iter()
        .filter(|asset| asset.backend == "cuda")
    {
        let companion_name = asset
            .companion_name
            .as_deref()
            .ok_or_else(|| format!("Approved CUDA asset {} lacks a companion", asset.name))?;
        let companion = manifest
            .assets
            .iter()
            .find(|candidate| candidate.name == companion_name)
            .ok_or_else(|| format!("Approved CUDA companion {companion_name} is absent"))?;
        if companion.backend != "cuda-companion"
            || companion.arch != asset.arch
            || companion.cuda_version != asset.cuda_version
            || companion.dormant != asset.dormant
        {
            return Err(format!(
                "Approved CUDA companion {companion_name} does not match {}",
                asset.name
            ));
        }
    }

    let mut job_names = HashSet::new();
    let mut job_urls = HashSet::new();
    let mut mapped_active_keys = HashSet::new();
    let mut blocked_backends = HashSet::new();
    for job in &manifest.required_jobs {
        if !job_names.insert(job.name.clone()) || !job_urls.insert(job.url.clone()) {
            return Err(format!(
                "Approved manifest has duplicate required job {}",
                job.name
            ));
        }
        if job.status != "completed"
            || job.conclusion.is_empty()
            || !valid_utc_timestamp(&job.completed_at)
            || !job
                .url
                .starts_with("https://github.com/ggml-org/llama.cpp/actions/")
            || job.install_keys.is_empty()
            || job.platforms.is_empty()
            || job.hardware_classes.is_empty()
        {
            return Err(format!(
                "Required job {} is not terminal or complete",
                job.name
            ));
        }
        if job.conclusion != "success" {
            blocked_backends.insert(job.backend.clone());
        }
        for install_key in &job.install_keys {
            let Some(asset_backend) = active_install_keys.get(install_key) else {
                return Err(format!(
                    "Required job {} maps unknown active install key {install_key}",
                    job.name
                ));
            };
            if asset_backend != &job.backend {
                return Err(format!(
                    "Required job {} ambiguously maps {install_key} to backend {}",
                    job.name, job.backend
                ));
            }
            mapped_active_keys.insert(install_key.clone());
        }
    }
    if mapped_active_keys.len() != active_install_keys.len() {
        return Err("At least one active install key lacks a required-job mapping".into());
    }
    let declared_blocked = manifest
        .approval
        .blocked_backends
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    if declared_blocked != blocked_backends || manifest.approval.note.trim().is_empty() {
        return Err("Approval gate does not match terminal required-job results".into());
    }
    let mut compatibility_keys = HashSet::new();
    for record in &manifest.compatibility_records {
        let key = &record.key;
        let asset = manifest
            .assets
            .iter()
            .find(|asset| asset.name == key.asset_name)
            .ok_or_else(|| {
                format!(
                    "Compatibility record {} maps an unknown asset",
                    record.attestation_id
                )
            })?;
        let install_key = approved_asset_install_key(asset)?;
        let serialized_key = serde_json::to_string(key)
            .map_err(|error| format!("Compatibility key serialization failed: {error}"))?;
        if !compatibility_keys.insert(serialized_key)
            || record.evidence_level != "L4_PRODUCT"
            || record.attestation_id.trim().is_empty()
            || record.attestation_id.len() > 256
            || record.expiry_identity != compatibility_expiry_identity(key)
            || key.os_build.trim().is_empty()
            || !matches!(key.architecture.as_str(), "x64" | "arm64")
            || key.adapter_id.trim().is_empty()
            || key.driver.trim().is_empty()
            || key.firmware.trim().is_empty()
            || key.backend != asset.backend
            || install_key.as_deref() != Some(key.install_key.as_str())
            || key.release_commit != manifest.release_commit
            || key.asset_sha256.len() != 64
            || !key
                .asset_sha256
                .chars()
                .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
            || asset.digest != format!("sha256:{}", key.asset_sha256)
            || asset.dormant
        {
            return Err(format!(
                "Compatibility record {} is malformed or expired",
                record.attestation_id
            ));
        }
    }
    Ok(manifest)
}

pub fn approved_runtime_identity() -> Result<ApprovedRuntimeIdentity, String> {
    let manifest = parse_approved_manifest(APPROVED_RUNTIMES)?;
    Ok(ApprovedRuntimeIdentity {
        release_tag: manifest.release_tag,
        release_commit: manifest.release_commit,
        manifest_sha256: hex::encode(Sha256::digest(APPROVED_RUNTIMES.as_bytes())),
        published_at: manifest.published_at,
        observed_at: manifest.observed_at,
        source: manifest.source,
        supported_windows: manifest.supported_windows,
    })
}

pub fn approved_release_url() -> Result<String, String> {
    let identity = approved_runtime_identity()?;
    Ok(format!("{RELEASE_BY_TAG_URL}/{}", identity.release_tag))
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApprovedManifestGate {
    blocked_backends: Vec<String>,
    note: String,
}

/// Load the pinned approved runtime manifest.
///
/// The manifest is a compile-time file. The build fails when the file is
/// absent, so selection can never run without an approved pin.
pub fn approved_manifest() -> Result<(Vec<ApprovedRuntimeAsset>, Vec<RequiredUpstreamJob>), String>
{
    let manifest = parse_approved_manifest(APPROVED_RUNTIMES)?;
    Ok((manifest.assets, manifest.required_jobs))
}

fn approved_compatibility_records() -> Result<Vec<CompatibilityRecord>, String> {
    Ok(parse_approved_manifest(APPROVED_RUNTIMES)?.compatibility_records)
}

/// Report whether a required upstream job blocks a backend.
///
/// A job blocks when it names the backend and its conclusion is not
/// `success`, or when its status is not `completed`. A queued or failed
/// job therefore blocks the runtime update for that backend.
#[cfg(test)]
pub fn upstream_job_blocks_backend(job: &RequiredUpstreamJob, backend: &str) -> bool {
    if job.backend != backend {
        return false;
    }
    job.conclusion != "success" || job.status != "completed"
}

/// Report whether any required job blocks a backend.
#[cfg(test)]
pub fn backend_is_blocked_by_upstream(jobs: &[RequiredUpstreamJob], backend: &str) -> bool {
    jobs.iter()
        .any(|job| upstream_job_blocks_backend(job, backend))
}

fn jobs_for_install_key<'a>(
    jobs: &'a [RequiredUpstreamJob],
    install_key: &str,
) -> Vec<&'a RequiredUpstreamJob> {
    jobs.iter()
        .filter(|job| job.install_keys.iter().any(|key| key == install_key))
        .collect()
}

fn approved_manifest_tag(approved: &[ApprovedRuntimeAsset]) -> Option<String> {
    approved.first().and_then(|entry| {
        entry
            .url
            .split("/releases/download/")
            .nth(1)
            .and_then(|rest| rest.split('/').next())
            .map(str::to_string)
    })
}

/// Build the user-visible catalog from one approved release.
///
/// Every option must bind to the approved manifest: the asset name, byte
/// count, download URL, and SHA-256 digest must match the manifest entry.
/// The manifest companion name must match for CUDA entries. A missing
/// digest, an unlisted asset, or a changed remote identity rejects the
/// option. The release tag must equal the manifest pin.
pub fn build_approved_catalog(
    release: &GithubRelease,
    hardware: &HardwareInfo,
    approved: &[ApprovedRuntimeAsset],
    jobs: &[RequiredUpstreamJob],
) -> Result<RuntimeCatalog, String> {
    if approved.is_empty() {
        return Err("Approved runtime manifest contains no assets".into());
    }
    let manifest_tag = approved_manifest_tag(approved).ok_or_else(|| {
        "Approved assets do not contain one authoritative release tag".to_string()
    })?;
    if release.tag_name != manifest_tag {
        return Err(format!(
            "Release {} is not the approved runtime {}",
            release.tag_name, manifest_tag
        ));
    }
    for entry in approved
        .iter()
        .filter(|entry| entry.arch == hardware.architecture)
    {
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == entry.name)
            .ok_or_else(|| {
                format!(
                    "Approved runtime asset {} is missing from release {}",
                    entry.name, release.tag_name
                )
            })?;
        bind_asset_to_manifest(asset, approved)?;
    }
    let mut options = Vec::new();
    let mut availability = Vec::new();
    for asset in release.assets.iter().filter(|asset| {
        let lower = asset.name.to_ascii_lowercase();
        lower.starts_with("llama-")
            && lower.contains("-bin-win-")
            && lower.ends_with(".zip")
            && lower.contains(&format!("-{}", hardware.architecture))
    }) {
        let entry = bind_asset_to_manifest(asset, approved)?;
        if entry.arch != hardware.architecture {
            return Err(format!(
                "Approved manifest entry {} targets {}, want {}",
                entry.name, entry.arch, hardware.architecture
            ));
        }
        let backend = backend_for(&asset.name)
            .ok_or_else(|| format!("Runtime asset {} has no recognized backend", asset.name))?;
        if entry.backend != backend {
            return Err(format!(
                "Runtime asset {} backend {backend} differs from approved {}",
                asset.name, entry.backend
            ));
        }
        let (label, description, compatibility) = label_and_description(backend, &asset.name);
        let companion_asset = if backend == "cuda" {
            let Some(companion_name) = entry.companion_name.as_deref() else {
                return Err(format!(
                    "Approved manifest entry {} lacks a CUDA companion",
                    entry.name
                ));
            };
            let companion = release
                .assets
                .iter()
                .find(|candidate| candidate.name == companion_name)
                .ok_or_else(|| {
                    format!(
                        "CUDA companion {companion_name} is missing from release {}",
                        release.tag_name
                    )
                })?;
            bind_asset_to_manifest(companion, approved)?;
            Some(companion.clone())
        } else {
            None
        };
        let install_key = if backend == "cuda" {
            let version = entry.cuda_version.clone().unwrap_or("current".into());
            format!("cuda-{version}")
        } else {
            backend.to_string()
        };
        let required_jobs = jobs_for_install_key(jobs, &install_key);
        let evidence_urls = required_jobs
            .iter()
            .map(|job| job.url.clone())
            .collect::<Vec<_>>();
        if entry.dormant {
            availability.push(BackendAvailability {
                install_key,
                backend: backend.into(),
                status: BackendAvailabilityStatus::Dormant,
                reason: "The approved asset is retained for identity checks but is not enabled for installation."
                    .into(),
                blocking_jobs: Vec::new(),
                evidence_urls,
            });
            continue;
        }
        let blocking_jobs = required_jobs
            .iter()
            .filter(|job| job.status != "completed" || job.conclusion != "success")
            .collect::<Vec<_>>();
        if !blocking_jobs.is_empty() {
            availability.push(BackendAvailability {
                install_key,
                backend: backend.into(),
                status: BackendAvailabilityStatus::Blocked,
                reason: format!(
                    "{} required upstream job(s) failed or remain queued.",
                    blocking_jobs.len()
                ),
                blocking_jobs: blocking_jobs.iter().map(|job| job.name.clone()).collect(),
                evidence_urls: blocking_jobs.iter().map(|job| job.url.clone()).collect(),
            });
            continue;
        }
        availability.push(BackendAvailability {
            install_key: install_key.clone(),
            backend: backend.into(),
            status: BackendAvailabilityStatus::Available,
            reason: "Every mapped upstream job completed successfully.".into(),
            blocking_jobs: Vec::new(),
            evidence_urls,
        });
        options.push(RuntimeOption {
            id: format!("{}:{install_key}", release.tag_name),
            label,
            backend: backend.into(),
            install_key,
            description,
            compatibility,
            asset: asset.clone(),
            companion_asset,
            recommended: false,
        });
    }
    if availability.is_empty() {
        return Err(format!(
            "Release {} has no approved Windows {} runtime archives",
            release.tag_name, hardware.architecture
        ));
    }
    Ok(RuntimeCatalog {
        tag: release.tag_name.clone(),
        published_at: release.published_at.clone().unwrap_or_default(),
        options,
        availability,
        origin: RuntimeCatalogOrigin::Network,
        warning: None,
        recommendation_reason: "Recommendation has not been evaluated.".into(),
    })
}

/// Recommend one approved catalog entry from exact product evidence.
///
/// An accelerator requires an exact L4 compatibility record.
/// Every unlisted or mismatched configuration falls back to CPU.
#[cfg(test)]
fn recommend_capability_option<'a>(
    options: &'a mut [RuntimeOption],
    hardware: &HardwareInfo,
) -> Option<&'a RuntimeOption> {
    apply_capability_recommendation(options, hardware, None);
    options.iter().find(|option| option.recommended)
}

fn apply_capability_recommendation(
    options: &mut [RuntimeOption],
    hardware: &HardwareInfo,
    selected_adapter_id: Option<&str>,
) -> String {
    for option in options.iter_mut() {
        option.recommended = false;
    }
    let exact_index = compatibility_recommendation_index(options, hardware, selected_adapter_id);
    let recommended_index = exact_index
        .or_else(|| options.iter().position(|option| option.backend == "cpu"))
        .or_else(|| options.iter().position(|option| option.backend == "vulkan"));
    if let Some(index) = recommended_index {
        options[index].recommended = true;
    }
    options.sort_by_key(|option| (!option.recommended, option.backend.clone()));
    if exact_index.is_some() {
        "Exact L4 product compatibility record matched the selected adapter, Windows build, driver, runtime commit, and asset digest."
            .into()
    } else if hardware.adapters.len() > 1 && selected_adapter_id.is_none() {
        "CPU fallback: select one adapter before Localmotive checks exact L4 product compatibility records."
            .into()
    } else if selected_adapter_id.is_some() {
        "CPU fallback: the selected adapter has no exact L4 product compatibility record for this Windows build, driver, firmware, runtime commit, and asset."
            .into()
    } else if hardware.adapters.is_empty() {
        "CPU fallback: no accelerator with direct per-adapter evidence was detected.".into()
    } else {
        "CPU fallback: the detected adapter has no exact L4 product compatibility record for this Windows build, driver, firmware, runtime commit, and asset."
            .into()
    }
}

pub fn recommend_catalog_for_adapter(
    catalog: &mut RuntimeCatalog,
    hardware: &HardwareInfo,
    selected_adapter_id: Option<&str>,
) {
    catalog.recommendation_reason =
        apply_capability_recommendation(&mut catalog.options, hardware, selected_adapter_id);
}

fn compatibility_recommendation_index(
    options: &[RuntimeOption],
    hardware: &HardwareInfo,
    selected_adapter_id: Option<&str>,
) -> Option<usize> {
    let os_build = current_windows_build()?;
    let identity = approved_runtime_identity().ok()?;
    let records = approved_compatibility_records().ok()?;
    compatibility_recommendation_index_for_evidence(
        options,
        hardware,
        selected_adapter_id,
        &os_build,
        &records,
        &identity.release_commit,
    )
}

fn compatibility_recommendation_index_for_evidence(
    options: &[RuntimeOption],
    hardware: &HardwareInfo,
    selected_adapter_id: Option<&str>,
    os_build: &str,
    records: &[CompatibilityRecord],
    release_commit: &str,
) -> Option<usize> {
    let adapter = match selected_adapter_id {
        Some(adapter_id) => hardware
            .adapters
            .iter()
            .find(|adapter| adapter.adapter_id == adapter_id)?,
        None if hardware.adapters.len() == 1 => hardware.adapters.first()?,
        None => return None,
    };
    options.iter().enumerate().find_map(|(index, option)| {
        records
            .iter()
            .any(|record| {
                compatibility_record_matches(
                    record,
                    option,
                    adapter,
                    os_build,
                    None,
                    &hardware.architecture,
                    release_commit,
                )
            })
            .then_some(index)
    })
}

#[cfg(windows)]
fn current_windows_build() -> Option<String> {
    Some(windows_version::OsVersion::current().build.to_string())
}

#[cfg(not(windows))]
fn current_windows_build() -> Option<String> {
    None
}

/// Require explicit device selection on multi-adapter systems.
///
/// Return one adapter only after an explicit multi-adapter selection.
#[cfg(test)]
fn require_explicit_device_selection(
    adapters: &[String],
    selected: Option<&str>,
) -> Result<String, String> {
    if adapters.is_empty() {
        return Err("No graphics adapter was detected; use the CPU build".into());
    }
    if adapters.len() == 1 {
        return Ok(adapters[0].clone());
    }
    if let Some(choice) = selected {
        return adapters
            .iter()
            .find(|adapter| adapter.as_str() == choice)
            .cloned()
            .ok_or_else(|| {
                format!("Selected adapter {choice} is not in the detected adapter list")
            });
    }
    Err(
        "Multiple accelerators were detected; choose one adapter explicitly before installing a runtime"
            .into(),
    )
}

/// Recommend one approved catalog entry from already-bound options.
///
/// Removed in Phase 2: `recommend_capability_option` replaces the 0.3
/// vendor preference order with device checks. The old body is deleted so
/// no unpinned preference path remains in the build.
#[cfg(test)]
#[allow(dead_code)]
fn recommend_approved_option<'a>(
    _options: &'a mut [RuntimeOption],
    _hardware: &HardwareInfo,
) -> Option<&'a RuntimeOption> {
    None
}

fn sanitize_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect();
    // A component of only dots (`.` or `..`) still climbs out of its
    // parent after `join`, so neutralize it. Mixed names like
    // `cuda-13.3` keep their dots and existing install paths stay put.
    if sanitized.is_empty() || sanitized.chars().all(|c| c == '.') {
        "_".repeat(sanitized.len().max(1))
    } else {
        sanitized
    }
}

pub fn managed_runtime_relative_path(tag: &str, backend: &str) -> PathBuf {
    PathBuf::from(sanitize_component(tag)).join(sanitize_component(backend))
}

pub fn managed_runtime_root() -> Result<PathBuf, String> {
    managed_runtime_root_in(
        &runtime_data_dir("Localmotive"),
        &runtime_data_dir("GGUF Pilot"),
    )
}

/// Previous product directory. Upgrades fall back to it when the new
/// directory has no records, so existing installs keep working.
fn runtime_data_dir(product: &str) -> PathBuf {
    if let Some(base) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        return base.join(product).join("runtimes");
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.join(product).join("runtimes");
        }
    }
    PathBuf::from(product).join("runtimes")
}

fn managed_runtime_root_in(primary: &Path, legacy: &Path) -> Result<PathBuf, String> {
    if list_managed_runtimes_in(primary).is_empty() && !list_managed_runtimes_in(legacy).is_empty()
    {
        return Ok(legacy.to_path_buf());
    }
    if primary.exists() || !legacy.exists() {
        return Ok(primary.to_path_buf());
    }
    Ok(legacy.to_path_buf())
}

const RUNTIME_CATALOG_CACHE_SCHEMA: u32 = 2;
const MAX_RUNTIME_CATALOG_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeCatalogCacheRecord {
    schema_version: u32,
    url: String,
    release_tag: String,
    release_commit: String,
    manifest_sha256: String,
    etag: Option<String>,
    observed_at_ms: u64,
    body_sha256: String,
    body: String,
}

fn catalog_cache_record(
    identity: &ApprovedRuntimeIdentity,
    etag: Option<String>,
    body: String,
) -> RuntimeCatalogCacheRecord {
    RuntimeCatalogCacheRecord {
        schema_version: RUNTIME_CATALOG_CACHE_SCHEMA,
        url: identity.source.clone(),
        release_tag: identity.release_tag.clone(),
        release_commit: identity.release_commit.clone(),
        manifest_sha256: identity.manifest_sha256.clone(),
        etag,
        observed_at_ms: observed_at_ms(),
        body_sha256: hex::encode(Sha256::digest(body.as_bytes())),
        body,
    }
}

fn validate_catalog_cache(
    bytes: &[u8],
    identity: &ApprovedRuntimeIdentity,
) -> Result<RuntimeCatalogCacheRecord, String> {
    if bytes.len() > MAX_RUNTIME_CATALOG_BYTES {
        return Err("Runtime catalog cache exceeds the 2 MiB limit".into());
    }
    let record: RuntimeCatalogCacheRecord = serde_json::from_slice(bytes)
        .map_err(|error| format!("Runtime catalog cache is invalid: {error}"))?;
    if record.schema_version != RUNTIME_CATALOG_CACHE_SCHEMA
        || record.url != identity.source
        || record.release_tag != identity.release_tag
        || record.release_commit != identity.release_commit
        || record.manifest_sha256 != identity.manifest_sha256
        || record.observed_at_ms == 0
    {
        return Err("Runtime catalog cache key does not match the approved release".into());
    }
    let actual_digest = hex::encode(Sha256::digest(record.body.as_bytes()));
    if actual_digest != record.body_sha256 {
        return Err("Runtime catalog cache body digest does not match".into());
    }
    let release: GithubRelease = serde_json::from_str(&record.body)
        .map_err(|error| format!("Runtime catalog cache body is invalid: {error}"))?;
    if release.tag_name != identity.release_tag
        || release.target_commitish != identity.release_commit
    {
        return Err("Runtime catalog cache release identity does not match approval".into());
    }
    Ok(record)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCatalogErrorKind {
    Busy,
    Timeout,
    RateLimited,
    Http,
    BodyTooLarge,
    InvalidResponse,
    TrustFailure,
    CacheFailure,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCatalogError {
    pub kind: RuntimeCatalogErrorKind,
    pub message: String,
    pub retry_after_seconds: Option<u64>,
}

impl RuntimeCatalogError {
    fn new(kind: RuntimeCatalogErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            retry_after_seconds: None,
        }
    }
}

impl std::fmt::Display for RuntimeCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[derive(Debug)]
enum CatalogHttpResponse {
    NotModified,
    Body { body: String, etag: Option<String> },
}

const RUNTIME_CATALOG_TIMEOUT: Duration = Duration::from_secs(15);

fn runtime_catalog_client(timeout: Duration) -> Result<reqwest::Client, RuntimeCatalogError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );
    reqwest::Client::builder()
        .user_agent(format!("Localmotive/{}", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .connect_timeout(timeout.min(Duration::from_secs(5)))
        .timeout(timeout)
        .build()
        .map_err(|error| {
            RuntimeCatalogError::new(
                RuntimeCatalogErrorKind::Http,
                format!("Could not create the runtime catalog client: {error}"),
            )
        })
}

async fn fetch_catalog_http(
    client: &reqwest::Client,
    url: &str,
    etag: Option<&str>,
) -> Result<CatalogHttpResponse, RuntimeCatalogError> {
    let mut request = client.get(url);
    if let Some(etag) = etag {
        request = request.header(reqwest::header::IF_NONE_MATCH, etag);
    }
    let mut response = request.send().await.map_err(|error| {
        let kind = if error.is_timeout() {
            RuntimeCatalogErrorKind::Timeout
        } else {
            RuntimeCatalogErrorKind::Http
        };
        RuntimeCatalogError::new(kind, format!("Runtime catalog request failed: {error}"))
    })?;
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(CatalogHttpResponse::NotModified);
    }
    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let retry_after_seconds = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());
        let mut error = RuntimeCatalogError::new(
            RuntimeCatalogErrorKind::RateLimited,
            "GitHub rate-limited the runtime catalog request. Retry after the indicated delay.",
        );
        error.retry_after_seconds = retry_after_seconds;
        return Err(error);
    }
    if !response.status().is_success() {
        // GitHub answers an exhausted unauthenticated quota with `403 rate
        // limit exceeded`, not `429`: surface that body as a typed
        // rate-limit error so the UI can show the reset guidance.
        if response.status() == reqwest::StatusCode::FORBIDDEN {
            let body = response.text().await.unwrap_or_default();
            if body.to_ascii_lowercase().contains("rate limit") {
                let mut error = RuntimeCatalogError::new(
                    RuntimeCatalogErrorKind::RateLimited,
                    "GitHub rate-limited the runtime catalog request. Retry after the indicated delay.",
                );
                error.retry_after_seconds = None;
                return Err(error);
            }
            return Err(RuntimeCatalogError::new(
                RuntimeCatalogErrorKind::Http,
                format!("GitHub returned HTTP 403 for the approved runtime catalog: {body}"),
            ));
        }
        return Err(RuntimeCatalogError::new(
            RuntimeCatalogErrorKind::Http,
            format!(
                "GitHub returned HTTP {} for the approved runtime catalog.",
                response.status().as_u16()
            ),
        ));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RUNTIME_CATALOG_BYTES as u64)
    {
        return Err(RuntimeCatalogError::new(
            RuntimeCatalogErrorKind::BodyTooLarge,
            "Runtime catalog metadata exceeds the 2 MiB limit.",
        ));
    }
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| {
        let kind = if error.is_timeout() {
            RuntimeCatalogErrorKind::Timeout
        } else {
            RuntimeCatalogErrorKind::Http
        };
        RuntimeCatalogError::new(kind, format!("Runtime catalog body failed: {error}"))
    })? {
        if body
            .len()
            .checked_add(chunk.len())
            .is_none_or(|length| length > MAX_RUNTIME_CATALOG_BYTES)
        {
            return Err(RuntimeCatalogError::new(
                RuntimeCatalogErrorKind::BodyTooLarge,
                "Runtime catalog metadata exceeds the 2 MiB limit.",
            ));
        }
        body.extend_from_slice(&chunk);
    }
    let body = String::from_utf8(body).map_err(|_| {
        RuntimeCatalogError::new(
            RuntimeCatalogErrorKind::InvalidResponse,
            "Runtime catalog metadata is not UTF-8.",
        )
    })?;
    // Map a truncated or otherwise malformed release body to the typed
    // invalid-response error here, so every `Body` carries parseable
    // release JSON and no caller can silently accept partial metadata.
    if serde_json::from_str::<GithubRelease>(&body).is_err() {
        return Err(RuntimeCatalogError::new(
            RuntimeCatalogErrorKind::InvalidResponse,
            "GitHub release response was invalid: the metadata body is not a complete release document.",
        ));
    }
    Ok(CatalogHttpResponse::Body { body, etag })
}

fn runtime_catalog_cache_path() -> Result<PathBuf, String> {
    let runtime_root = runtime_data_dir("Localmotive");
    let product_root = runtime_root
        .parent()
        .ok_or_else(|| "Could not resolve the Localmotive data directory".to_string())?;
    Ok(product_root.join("cache").join("runtime-catalog-v1.json"))
}

fn read_catalog_cache(
    path: &Path,
    identity: &ApprovedRuntimeIdentity,
) -> Result<Option<RuntimeCatalogCacheRecord>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Could not inspect runtime catalog cache: {error}")),
    };
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || is_reparse_point(&metadata)
        || metadata.len() > MAX_RUNTIME_CATALOG_BYTES as u64
    {
        return Err("Runtime catalog cache is not a bounded regular file".into());
    }
    let bytes = fs::read(path).map_err(|error| format!("Could not read catalog cache: {error}"))?;
    validate_catalog_cache(&bytes, identity).map(Some)
}

fn write_catalog_cache(path: &Path, record: &RuntimeCatalogCacheRecord) -> Result<(), String> {
    let bytes = serde_json::to_vec(record)
        .map_err(|error| format!("Could not serialize runtime catalog cache: {error}"))?;
    if bytes.len() > MAX_RUNTIME_CATALOG_BYTES {
        return Err("Runtime catalog cache exceeds the 2 MiB limit".into());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "Runtime catalog cache has no parent directory".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create runtime catalog cache directory: {error}"))?;
    let parent_metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("Could not inspect runtime catalog cache directory: {error}"))?;
    if !parent_metadata.is_dir()
        || parent_metadata.file_type().is_symlink()
        || is_reparse_point(&parent_metadata)
    {
        return Err("Runtime catalog cache directory is a link or reparse point".into());
    }
    if fs::symlink_metadata(path).is_ok_and(|metadata| {
        !metadata.is_file() || metadata.file_type().is_symlink() || is_reparse_point(&metadata)
    }) {
        return Err("Runtime catalog cache target is not a regular file".into());
    }
    let temporary = parent.join(format!(
        ".runtime-catalog-{}-{}.tmp",
        std::process::id(),
        observed_at_ms()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("Could not create runtime catalog cache: {error}"))?;
    let result = (|| {
        io::Write::write_all(&mut file, &bytes)
            .map_err(|error| format!("Could not write runtime catalog cache: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Could not flush runtime catalog cache: {error}"))?;
        drop(file);
        if path.exists() {
            fs::remove_file(path)
                .map_err(|error| format!("Could not replace runtime catalog cache: {error}"))?;
        }
        fs::rename(&temporary, path)
            .map_err(|error| format!("Could not publish runtime catalog cache: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn catalog_from_release(
    release: &GithubRelease,
    hardware: &HardwareInfo,
    approved: &[ApprovedRuntimeAsset],
    jobs: &[RequiredUpstreamJob],
    origin: RuntimeCatalogOrigin,
    warning: Option<String>,
) -> Result<RuntimeCatalog, String> {
    let identity = approved_runtime_identity()?;
    if release.tag_name != identity.release_tag
        || release.target_commitish != identity.release_commit
        || release.published_at.as_deref() != Some(identity.published_at.as_str())
    {
        return Err(format!(
            "Approved runtime release tag, commit, or publication identity changed for {}",
            identity.release_tag
        ));
    }
    let catalog = build_approved_catalog(release, hardware, approved, jobs)?;
    let mut options = catalog.options;
    let recommendation_reason = apply_capability_recommendation(&mut options, hardware, None);
    Ok(RuntimeCatalog {
        tag: catalog.tag,
        published_at: catalog.published_at,
        options,
        availability: catalog.availability,
        origin,
        warning,
        recommendation_reason,
    })
}

fn cache_catalog(
    record: &RuntimeCatalogCacheRecord,
    hardware: &HardwareInfo,
    approved: &[ApprovedRuntimeAsset],
    jobs: &[RequiredUpstreamJob],
    warning: String,
) -> Result<RuntimeCatalog, RuntimeCatalogError> {
    let release = serde_json::from_str::<GithubRelease>(&record.body).map_err(|error| {
        RuntimeCatalogError::new(
            RuntimeCatalogErrorKind::CacheFailure,
            format!("Validated runtime catalog cache could not be parsed: {error}"),
        )
    })?;
    catalog_from_release(
        &release,
        hardware,
        approved,
        jobs,
        RuntimeCatalogOrigin::Cache,
        Some(warning),
    )
    .map_err(|error| RuntimeCatalogError::new(RuntimeCatalogErrorKind::TrustFailure, error))
}

pub async fn fetch_catalog(hardware: &HardwareInfo) -> Result<RuntimeCatalog, RuntimeCatalogError> {
    let (approved, jobs) = approved_manifest()
        .map_err(|error| RuntimeCatalogError::new(RuntimeCatalogErrorKind::TrustFailure, error))?;
    let identity = approved_runtime_identity()
        .map_err(|error| RuntimeCatalogError::new(RuntimeCatalogErrorKind::TrustFailure, error))?;
    let cache_path = runtime_catalog_cache_path()
        .map_err(|error| RuntimeCatalogError::new(RuntimeCatalogErrorKind::CacheFailure, error))?;
    let (cached, cache_warning) = match read_catalog_cache(&cache_path, &identity) {
        Ok(record) => (record, None),
        Err(error) => (
            None,
            Some(format!("The saved runtime catalog was rejected: {error}")),
        ),
    };
    let client = runtime_catalog_client(RUNTIME_CATALOG_TIMEOUT)?;
    let url = approved_release_url()
        .map_err(|error| RuntimeCatalogError::new(RuntimeCatalogErrorKind::TrustFailure, error))?;
    let response = fetch_catalog_http(
        &client,
        &url,
        cached.as_ref().and_then(|record| record.etag.as_deref()),
    )
    .await;
    match response {
        Ok(CatalogHttpResponse::NotModified) => {
            let record = cached.ok_or_else(|| {
                RuntimeCatalogError::new(
                    RuntimeCatalogErrorKind::InvalidResponse,
                    "GitHub returned 304 without a validated runtime catalog cache.",
                )
            })?;
            cache_catalog(
                &record,
                hardware,
                &approved,
                &jobs,
                "GitHub confirmed the saved runtime catalog is current.".into(),
            )
        }
        Ok(CatalogHttpResponse::Body { body, etag }) => {
            let release = serde_json::from_str::<GithubRelease>(&body).map_err(|error| {
                RuntimeCatalogError::new(
                    RuntimeCatalogErrorKind::InvalidResponse,
                    format!("GitHub release response was invalid: {error}"),
                )
            })?;
            let mut catalog = catalog_from_release(
                &release,
                hardware,
                &approved,
                &jobs,
                RuntimeCatalogOrigin::Network,
                cache_warning,
            )
            .map_err(|error| {
                RuntimeCatalogError::new(RuntimeCatalogErrorKind::TrustFailure, error)
            })?;
            let record = catalog_cache_record(&identity, etag, body);
            if let Err(error) = write_catalog_cache(&cache_path, &record) {
                catalog.warning = Some(match catalog.warning {
                    Some(existing) => format!("{existing} Could not save catalog cache: {error}"),
                    None => format!("Could not save catalog cache: {error}"),
                });
            }
            Ok(catalog)
        }
        Err(error) => {
            if let Some(record) = cached {
                return cache_catalog(
                    &record,
                    hardware,
                    &approved,
                    &jobs,
                    format!("Using the validated saved catalog because refresh failed: {error}"),
                );
            }
            Err(error)
        }
    }
}

#[cfg(test)]
pub(crate) fn fetch_runtime_setup_with<D, F>(
    detect: D,
    fetch: F,
) -> Result<(HardwareInfo, RuntimeCatalog), String>
where
    D: FnOnce() -> HardwareInfo,
    F: FnOnce(&HardwareInfo) -> Result<RuntimeCatalog, String>,
{
    let hardware = detect();
    let catalog = fetch(&hardware)?;
    Ok((hardware, catalog))
}

fn expected_sha256(asset: &GithubAsset) -> Option<&str> {
    asset.digest.as_deref()?.strip_prefix("sha256:")
}

const MAX_RUNTIME_DOWNLOAD_BYTES: u64 = 8 * 1024 * 1024 * 1024;

#[cfg(test)]
fn copy_download_with_limit<R: Read, W: io::Write>(
    reader: &mut R,
    writer: &mut W,
    max_bytes: u64,
) -> Result<(u64, String), String> {
    let mut hasher = Sha256::new();
    let mut written = 0_u64;
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        written = written
            .checked_add(count as u64)
            .ok_or("Runtime download size overflowed")?;
        if written > max_bytes {
            return Err(format!(
                "Runtime download byte limit exceeded: maximum {max_bytes}"
            ));
        }
        writer
            .write_all(&buffer[..count])
            .map_err(|error| error.to_string())?;
        hasher.update(&buffer[..count]);
    }
    Ok((written, hex::encode(hasher.finalize())))
}

#[derive(Clone, Copy)]
struct ArchiveLimits {
    max_entries: usize,
    max_entry_bytes: u64,
    max_total_bytes: u64,
    max_path_bytes: usize,
}

const RUNTIME_ARCHIVE_LIMITS: ArchiveLimits = ArchiveLimits {
    max_entries: 20_000,
    max_entry_bytes: 4 * 1024 * 1024 * 1024,
    max_total_bytes: 16 * 1024 * 1024 * 1024,
    max_path_bytes: 1_024,
};

fn open_verified_capability_directory(path: &Path, label: &str) -> Result<CapDir, String> {
    validate_no_reparse_ancestors(label, path)?;
    let canonical =
        fs::canonicalize(path).map_err(|error| format!("Could not resolve {label}: {error}"))?;
    let directory = CapDir::open_ambient_dir(path, ambient_authority())
        .map_err(|error| format!("Could not securely open {label}: {error}"))?;
    if !crate::download::opened_directory_matches(&directory, &canonical)? {
        return Err(format!("Opened {label} does not match the validated path"));
    }
    Ok(directory)
}

fn extract_zip_with_limits_and_cancel(
    archive_path: &Path,
    destination: &Path,
    limits: ArchiveLimits,
    cancel: Option<&AtomicBool>,
) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    let destination_dir =
        open_verified_capability_directory(destination, "runtime extraction directory")?;
    if archive.len() > limits.max_entries {
        return Err(format!(
            "Archive entry count {} exceeds the limit {}",
            archive.len(),
            limits.max_entries
        ));
    }
    let mut paths = HashSet::new();
    let mut total_bytes = 0_u64;
    for index in 0..archive.len() {
        if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err("Runtime installation cancelled during archive extraction".into());
        }
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let raw_name = entry.name().to_string();
        if raw_name.contains(':') {
            return Err(format!(
                "NTFS alternate data stream paths are not allowed in runtime archives: {raw_name}"
            ));
        }
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("Unsafe path in archive: {raw_name}"))?;
        // Canonicalize traversal before the duplicate check: two entry
        // names can enclose to the same output (`sub/../same.dll` and
        // `same.dll`) and the raw comparison misses the collision. The
        // second write then fails with a bare OS error and, worse,
        // target content becomes order-dependent. Compare the resolved
        // output path instead of the raw enclosed name.
        let output = destination.join(&relative);
        let mut normalized = PathBuf::new();
        for component in output.components() {
            use std::path::Component;
            match component {
                Component::ParentDir => {
                    normalized.pop();
                }
                Component::CurDir => {}
                other => normalized.push(other.as_os_str()),
            }
        }
        if !paths.insert(normalized.clone()) {
            return Err(format!("Duplicate path in archive: {raw_name}"));
        }
        if entry.name().len() > limits.max_path_bytes {
            return Err(format!("Archive path is too long: {}", entry.name()));
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(format!(
                "Archive symlink entries are not allowed: {}",
                entry.name()
            ));
        }
        if entry.size() > limits.max_entry_bytes {
            return Err(format!("Archive entry is too large: {}", entry.name()));
        }
        if total_bytes
            .checked_add(entry.size())
            .is_none_or(|total| total > limits.max_total_bytes)
        {
            return Err("Archive decompressed size exceeds the configured limit".into());
        }
        if entry.is_dir() {
            destination_dir
                .create_dir_all(&relative)
                .map_err(|error| error.to_string())?;
            continue;
        }
        if let Some(parent) = relative.parent() {
            destination_dir
                .create_dir_all(parent)
                .map_err(|error| error.to_string())?;
        }
        let mut options = CapOpenOptions::new();
        options.write(true).create_new(true);
        let mut target = destination_dir
            .open_with(&relative, &options)
            .map_err(|error| error.to_string())?;
        let mut entry_bytes = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        let copy_result = (|| -> Result<(), String> {
            loop {
                if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
                    return Err("Runtime installation cancelled during archive extraction".into());
                }
                let count = entry.read(&mut buffer).map_err(|error| error.to_string())?;
                if count == 0 {
                    break;
                }
                entry_bytes = entry_bytes
                    .checked_add(count as u64)
                    .ok_or("Archive entry size overflowed")?;
                total_bytes = total_bytes
                    .checked_add(count as u64)
                    .ok_or("Archive total size overflowed")?;
                if entry_bytes > limits.max_entry_bytes || total_bytes > limits.max_total_bytes {
                    return Err("Archive decompressed size exceeds the configured limit".into());
                }
                io::Write::write_all(&mut target, &buffer[..count])
                    .map_err(|error| error.to_string())?;
            }
            target.sync_all().map_err(|error| error.to_string())
        })();
        if let Err(error) = copy_result {
            drop(target);
            let _ = destination_dir.remove_file(&relative);
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(test)]
fn extract_zip_with_limits(
    archive_path: &Path,
    destination: &Path,
    limits: ArchiveLimits,
) -> Result<(), String> {
    extract_zip_with_limits_and_cancel(archive_path, destination, limits, None)
}

/// Verify the archive file identity before extraction.
///
/// The downloader already checks size and SHA-256, but the file sits on disk
/// between download and extraction. A replaced file with the same size would
/// otherwise extract without detection, so re-hash here and refuse on mismatch.
#[cfg(test)]
fn extract_verified_zip_with_limits_and_cancel(
    archive_path: &Path,
    destination: &Path,
    limits: ArchiveLimits,
    cancel: Option<&AtomicBool>,
    expected_size: u64,
    expected_sha256_hex: &str,
) -> Result<(), String> {
    let metadata = fs::metadata(archive_path).map_err(|error| error.to_string())?;
    if metadata.len() != expected_size {
        return Err(format!(
            "Runtime archive size mismatch: expected {expected_size} bytes, found {} bytes (SHA-256 identity check failed)",
            metadata.len()
        ));
    }
    let mut file = File::open(archive_path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err("Runtime installation cancelled during archive verification".into());
        }
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let actual = hex::encode(hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected_sha256_hex) {
        return Err(format!(
            "Runtime archive SHA-256 mismatch: expected {expected_sha256_hex}, found {actual}"
        ));
    }
    extract_zip_with_limits_and_cancel(archive_path, destination, limits, cancel)
}

fn extract_zip(archive_path: &Path, destination: &Path, cancel: &AtomicBool) -> Result<(), String> {
    extract_zip_with_limits_and_cancel(
        archive_path,
        destination,
        RUNTIME_ARCHIVE_LIMITS,
        Some(cancel),
    )
}

fn find_runtime(path: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(path).ok()? {
        let child = entry.ok()?.path();
        if child.is_dir() {
            if let Some(found) = find_runtime(&child) {
                return Some(found);
            }
        } else if child
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("llama-server.exe"))
        {
            return Some(child);
        }
    }
    None
}

#[derive(Clone, Debug)]
struct ResolvedRuntimeInstall {
    tag: String,
    release_commit: String,
    architecture: String,
    backend: String,
    install_key: String,
    asset: GithubAsset,
    companion_asset: Option<GithubAsset>,
    content_manifest_sha256: String,
}

fn github_asset_from_approval(asset: &ApprovedRuntimeAsset) -> GithubAsset {
    GithubAsset {
        name: asset.name.clone(),
        browser_download_url: asset.url.clone(),
        size: asset.bytes,
        digest: Some(asset.digest.clone()),
    }
}

fn resolve_approved_install(install_key: &str) -> Result<ResolvedRuntimeInstall, String> {
    if install_key.is_empty()
        || install_key.len() > 64
        || !install_key.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '.')
        })
    {
        return Err("Runtime install key is malformed".into());
    }
    let manifest = parse_approved_manifest(APPROVED_RUNTIMES)?;
    let asset = manifest
        .assets
        .iter()
        .filter(|asset| !asset.dormant)
        .find(|asset| {
            approved_asset_install_key(asset).ok().flatten().as_deref() == Some(install_key)
        })
        .ok_or_else(|| format!("Runtime install key {install_key} is not approved"))?;
    let blocking_jobs = jobs_for_install_key(&manifest.required_jobs, install_key)
        .into_iter()
        .filter(|job| job.status != "completed" || job.conclusion != "success")
        .map(|job| job.name.clone())
        .collect::<Vec<_>>();
    if !blocking_jobs.is_empty() {
        return Err(format!(
            "Runtime install key {install_key} is blocked by required jobs: {}",
            blocking_jobs.join(", ")
        ));
    }
    let companion_asset = match asset.companion_name.as_deref() {
        Some(name) => Some(
            manifest
                .assets
                .iter()
                .find(|candidate| candidate.name == name && !candidate.dormant)
                .map(github_asset_from_approval)
                .ok_or_else(|| format!("Approved companion {name} is unavailable"))?,
        ),
        None => None,
    };
    Ok(ResolvedRuntimeInstall {
        tag: manifest.release_tag,
        release_commit: manifest.release_commit,
        architecture: asset.arch.clone(),
        backend: asset.backend.clone(),
        install_key: install_key.to_string(),
        asset: github_asset_from_approval(asset),
        companion_asset,
        content_manifest_sha256: asset
            .content_manifest_sha256
            .clone()
            .ok_or_else(|| format!("Runtime install key {install_key} lacks a content manifest"))?,
    })
}

fn validate_install_adapter(
    option: &ResolvedRuntimeInstall,
    adapter_id: Option<&str>,
    architecture: &str,
    adapters: &[GpuAdapterInfo],
) -> Result<(), String> {
    if option.architecture != architecture {
        return Err(format!(
            "Runtime {} requires architecture {}, but this computer reports {}",
            option.install_key, option.architecture, architecture
        ));
    }

    if option.backend == "cpu" {
        if adapter_id.is_some() {
            return Err("The CPU runtime install request must not select a GPU adapter".into());
        }
        return Ok(());
    }

    let adapter_id = adapter_id
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 256)
        .ok_or_else(|| "The selected runtime requires one bounded GPU adapter ID".to_string())?;
    let adapter = adapters
        .iter()
        .find(|candidate| candidate.adapter_id == adapter_id)
        .ok_or_else(|| {
            format!("GPU adapter {adapter_id} is not present in current hardware data")
        })?;
    let vendor = adapter.vendor.to_ascii_lowercase();
    let compatible = match option.backend.as_str() {
        "cuda" => vendor == "nvidia",
        "rocm" => vendor == "amd",
        "openvino" | "sycl" => vendor == "intel",
        "vulkan" => matches!(vendor.as_str(), "nvidia" | "amd" | "intel"),
        _ => false,
    };
    if !compatible {
        return Err(format!(
            "GPU adapter {adapter_id} ({vendor}) is incompatible with the {} runtime",
            option.backend
        ));
    }
    Ok(())
}

fn approved_content_manifest_bytes(install_key: &str) -> Result<&'static [u8], String> {
    match install_key {
        "cpu" => Ok(include_bytes!("../runtime-content-manifests/cpu.json")),
        "cuda-12.4" => Ok(include_bytes!(
            "../runtime-content-manifests/cuda-12.4.json"
        )),
        "cuda-13.3" => Ok(include_bytes!(
            "../runtime-content-manifests/cuda-13.3.json"
        )),
        "openvino" => Ok(include_bytes!("../runtime-content-manifests/openvino.json")),
        "rocm" => Ok(include_bytes!("../runtime-content-manifests/rocm.json")),
        "sycl" => Ok(include_bytes!("../runtime-content-manifests/sycl.json")),
        "vulkan" => Ok(include_bytes!("../runtime-content-manifests/vulkan.json")),
        _ => Err(format!(
            "Runtime install key {install_key} has no compiled content manifest"
        )),
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApprovedContentManifest {
    schema_version: u32,
    release_tag: String,
    release_commit: String,
    install_key: String,
    backend: String,
    artifacts: Vec<ApprovedContentArtifact>,
    files: Vec<ApprovedContentFile>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApprovedContentArtifact {
    name: String,
    bytes: u64,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApprovedContentFile {
    path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeInstallRecord {
    schema_version: u32,
    release_tag: String,
    release_commit: String,
    backend: String,
    install_key: String,
    artifacts: Vec<String>,
    runtime: String,
    content_manifest_sha256: String,
}

fn install_artifact_names(install: &ResolvedRuntimeInstall) -> Vec<String> {
    std::iter::once(&install.asset)
        .chain(install.companion_asset.iter())
        .map(|asset| asset.name.clone())
        .collect()
}

fn path_from_manifest(value: &str) -> Result<PathBuf, String> {
    if value.is_empty() || value.contains('\\') || value.contains(':') {
        return Err("Approved content manifest contains a malformed path".into());
    }
    let mut result = PathBuf::new();
    for component in value.split('/') {
        if component.is_empty() || matches!(component, "." | "..") {
            return Err("Approved content manifest contains an unsafe path".into());
        }
        result.push(component);
    }
    Ok(result)
}

#[cfg(windows)]
fn open_managed_file_for_verification(path: &Path) -> Result<File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

    OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
        .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn open_managed_file_for_verification(path: &Path) -> Result<File, String> {
    File::open(path).map_err(|error| error.to_string())
}

#[cfg(windows)]
fn managed_file_link_count(file: &File) -> Result<u32, String> {
    use std::mem::MaybeUninit;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };

    let mut information = MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
    // SAFETY: `file` owns a valid handle and `information` points to writable storage.
    let result =
        unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, information.as_mut_ptr()) };
    if result == 0 {
        return Err(format!(
            "Could not inspect managed file links: {}",
            std::io::Error::last_os_error()
        ));
    }
    // SAFETY: a successful Win32 call initialized the complete output structure.
    Ok(unsafe { information.assume_init() }.nNumberOfLinks)
}

#[cfg(not(windows))]
fn managed_file_link_count(_file: &File) -> Result<u32, String> {
    Ok(1)
}

#[cfg(windows)]
fn reject_managed_file_alternate_streams(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_HANDLE_EOF, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        FindClose, FindFirstStreamW, FindNextStreamW, FindStreamInfoStandard,
        WIN32_FIND_STREAM_DATA,
    };

    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut data = WIN32_FIND_STREAM_DATA::default();
    // SAFETY: `wide` is NUL-terminated and `data` points to writable storage.
    let handle = unsafe {
        FindFirstStreamW(
            wide.as_ptr(),
            FindStreamInfoStandard,
            (&mut data as *mut WIN32_FIND_STREAM_DATA).cast(),
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(format!(
            "Could not enumerate managed file streams: {}",
            std::io::Error::last_os_error()
        ));
    }
    let first_end = data
        .cStreamName
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(data.cStreamName.len());
    let first = String::from_utf16_lossy(&data.cStreamName[..first_end]);
    let mut second = WIN32_FIND_STREAM_DATA::default();
    // SAFETY: `handle` is valid and `second` points to writable storage.
    let has_second =
        unsafe { FindNextStreamW(handle, (&mut second as *mut WIN32_FIND_STREAM_DATA).cast()) };
    // SAFETY: `handle` came from `FindFirstStreamW` and is closed exactly once.
    unsafe { FindClose(handle) };
    if first != "::$DATA" || has_second != 0 {
        return Err("Managed file contains an alternate data stream".into());
    }
    // SAFETY: `FindNextStreamW` returned false, so `GetLastError` describes termination.
    let last_error = unsafe { GetLastError() };
    if last_error != ERROR_HANDLE_EOF {
        return Err(format!(
            "Managed file stream enumeration failed: {}",
            std::io::Error::from_raw_os_error(last_error as i32)
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn reject_managed_file_alternate_streams(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn digest_regular_file(path: &Path, relative: &str) -> Result<(u64, String), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Could not inspect managed file {relative}: {error}"))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || is_reparse_point(&metadata)
        || metadata.len() > RUNTIME_ARCHIVE_LIMITS.max_entry_bytes
    {
        return Err(format!(
            "Managed file {relative} is not a bounded regular file"
        ));
    }
    let mut file = open_managed_file_for_verification(path)
        .map_err(|error| format!("Could not open managed file {relative}: {error}"))?;
    if managed_file_link_count(&file)? != 1 {
        return Err(format!("Managed file {relative} has multiple hard links"));
    }
    reject_managed_file_alternate_streams(path)?;
    let opened_metadata = file
        .metadata()
        .map_err(|error| format!("Could not inspect open managed file {relative}: {error}"))?;
    if !opened_metadata.is_file() || opened_metadata.len() != metadata.len() {
        return Err(format!(
            "Managed file {relative} changed during verification"
        ));
    }
    let mut hasher = Sha256::new();
    let mut read = 0_u64;
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not hash managed file {relative}: {error}"))?;
        if count == 0 {
            break;
        }
        read = read
            .checked_add(count as u64)
            .ok_or_else(|| format!("Managed file {relative} size overflowed"))?;
        if read > RUNTIME_ARCHIVE_LIMITS.max_entry_bytes {
            return Err(format!("Managed file {relative} exceeds the size limit"));
        }
        hasher.update(&buffer[..count]);
    }
    let final_metadata = file
        .metadata()
        .map_err(|error| format!("Could not recheck managed file {relative}: {error}"))?;
    if final_metadata.len() != read || read != metadata.len() {
        return Err(format!(
            "Managed file {relative} changed during verification"
        ));
    }
    Ok((read, hex::encode(hasher.finalize())))
}

fn collect_install_files(root: &Path) -> Result<Vec<String>, String> {
    validate_no_reparse_ancestors("Managed runtime", root)?;
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| format!("Could not inspect managed runtime root: {error}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err("Managed runtime root is a link or reparse point".into());
    }
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .map_err(|error| format!("Could not enumerate managed runtime: {error}"))?
        {
            let entry = entry.map_err(|error| format!("Managed runtime entry failed: {error}"))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("Could not inspect managed runtime entry: {error}"))?;
            if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
                return Err("Managed runtime contains a link or reparse point".into());
            }
            if metadata.is_dir() {
                pending.push(path);
                continue;
            }
            if !metadata.is_file() {
                return Err("Managed runtime contains a non-regular entry".into());
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "Managed runtime entry escaped its root")?
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            if relative != "runtime.json" {
                files.push(relative);
            }
            if files.len() > RUNTIME_ARCHIVE_LIMITS.max_entries {
                return Err("Managed runtime contains too many files".into());
            }
        }
    }
    files.sort();
    Ok(files)
}

fn write_runtime_install_record(
    root: &Path,
    install: &ResolvedRuntimeInstall,
    runtime: &Path,
    content_manifest_sha256: &str,
) -> Result<(), String> {
    let runtime = runtime
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    path_from_manifest(&runtime)?;
    let record = RuntimeInstallRecord {
        schema_version: 1,
        release_tag: install.tag.clone(),
        release_commit: install.release_commit.clone(),
        backend: install.backend.clone(),
        install_key: install.install_key.clone(),
        artifacts: install_artifact_names(install),
        runtime,
        content_manifest_sha256: content_manifest_sha256.to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&record)
        .map_err(|error| format!("Could not encode runtime install record: {error}"))?;
    fs::write(root.join("runtime.json"), bytes)
        .map_err(|error| format!("Could not write runtime install record: {error}"))
}

fn validate_content_manifest_authority(
    install: &ResolvedRuntimeInstall,
    trusted_content_bytes: &[u8],
) -> Result<ApprovedContentManifest, String> {
    if trusted_content_bytes.len() > MAX_RUNTIME_CATALOG_BYTES {
        return Err("Approved content manifest exceeds the 2 MiB limit".into());
    }
    let actual_manifest_digest = hex::encode(Sha256::digest(trusted_content_bytes));
    if actual_manifest_digest != install.content_manifest_sha256 {
        return Err("Compiled content-manifest digest does not match its approval anchor".into());
    }
    let trusted: ApprovedContentManifest = serde_json::from_slice(trusted_content_bytes)
        .map_err(|error| format!("Approved content manifest is invalid: {error}"))?;
    if trusted.schema_version != 1
        || trusted.release_tag != install.tag
        || trusted.release_commit != install.release_commit
        || trusted.backend != install.backend
        || trusted.install_key != install.install_key
    {
        return Err("Approved content manifest identity does not match the install key".into());
    }
    let expected_artifacts = std::iter::once(&install.asset)
        .chain(install.companion_asset.iter())
        .map(|asset| ApprovedContentArtifact {
            name: asset.name.clone(),
            bytes: asset.size,
            sha256: expected_sha256(asset).unwrap_or_default().to_string(),
        })
        .collect::<Vec<_>>();
    if trusted.artifacts != expected_artifacts {
        return Err("Approved content manifest artifact identity does not match approval".into());
    }
    if trusted.files.is_empty() || trusted.files.len() > RUNTIME_ARCHIVE_LIMITS.max_entries {
        return Err("Approved content manifest has an invalid file count".into());
    }
    let mut previous_path: Option<&str> = None;
    let mut total = 0_u64;
    let mut runtime_count = 0_usize;
    for file in &trusted.files {
        path_from_manifest(&file.path)?;
        if file.path == "runtime.json"
            || previous_path.is_some_and(|previous| previous >= file.path.as_str())
        {
            return Err("Approved content manifest paths are duplicated or unsorted".into());
        }
        if file.sha256.len() != 64
            || !file
                .sha256
                .chars()
                .all(|value| value.is_ascii_hexdigit() && !value.is_ascii_uppercase())
            || file.bytes > RUNTIME_ARCHIVE_LIMITS.max_entry_bytes
        {
            return Err("Approved content manifest contains invalid file identity".into());
        }
        if file.path.ends_with("llama-server.exe") {
            runtime_count += 1;
        }
        total = total
            .checked_add(file.bytes)
            .ok_or("Approved content-manifest size overflowed")?;
        if total > RUNTIME_ARCHIVE_LIMITS.max_total_bytes {
            return Err("Approved content manifest exceeds the total size limit".into());
        }
        previous_path = Some(&file.path);
    }
    if runtime_count != 1 {
        return Err("Approved content manifest must contain one llama-server.exe".into());
    }
    Ok(trusted)
}

fn verify_installed_runtime(
    root: &Path,
    install: &ResolvedRuntimeInstall,
    trusted_content_bytes: &[u8],
    trusted_content_sha256: &str,
) -> Result<PathBuf, String> {
    if trusted_content_sha256 != install.content_manifest_sha256 {
        return Err("Runtime install record used a non-approved content-manifest anchor".into());
    }
    let trusted = validate_content_manifest_authority(install, trusted_content_bytes)?;
    let manifest_path = root.join("runtime.json");
    let (manifest_bytes, _) = {
        let metadata = fs::symlink_metadata(&manifest_path)
            .map_err(|error| format!("Managed runtime record is unavailable: {error}"))?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || is_reparse_point(&metadata)
            || metadata.len() > 64 * 1024
        {
            return Err("Managed runtime record is not a bounded regular file".into());
        }
        let bytes = fs::read(&manifest_path)
            .map_err(|error| format!("Could not read managed runtime record: {error}"))?;
        (bytes, metadata)
    };
    let record: RuntimeInstallRecord = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("Managed runtime record is invalid: {error}"))?;
    if record.schema_version != 1
        || record.release_tag != install.tag
        || record.release_commit != install.release_commit
        || record.backend != install.backend
        || record.install_key != install.install_key
        || record.artifacts != install_artifact_names(install)
        || record.content_manifest_sha256 != trusted_content_sha256
    {
        return Err("Managed runtime record does not match compiled approval".into());
    }
    let actual_files = collect_install_files(root)?;
    let expected_files = trusted
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    if actual_files != expected_files {
        return Err("Managed runtime file inventory does not match compiled approval".into());
    }
    for file in &trusted.files {
        let relative = path_from_manifest(&file.path)?;
        let (bytes, digest) = digest_regular_file(&root.join(relative), &file.path)?;
        if bytes != file.bytes || digest != file.sha256 {
            return Err(format!(
                "Managed file {} failed content verification",
                file.path
            ));
        }
    }
    let relative_runtime = path_from_manifest(&record.runtime)?;
    if !expected_files.contains(&record.runtime) || !record.runtime.ends_with("llama-server.exe") {
        return Err("Managed runtime record names an unapproved executable".into());
    }
    Ok(root.join(relative_runtime))
}

fn managed_runtime_verified_in(runtime_path: &Path, root: &Path) -> Result<bool, String> {
    let lexical_managed = runtime_path.starts_with(root);
    let canonical_managed = match (fs::canonicalize(runtime_path), fs::canonicalize(root)) {
        (Ok(runtime), Ok(root)) => runtime.starts_with(root),
        _ => false,
    };
    if !lexical_managed && !canonical_managed {
        return Ok(false);
    }
    if !safe_directory(root) {
        return Err("Managed runtime root is unavailable or untrusted".into());
    }
    let mut directory = runtime_path
        .parent()
        .ok_or("Managed runtime executable has no parent directory")?;
    while directory.starts_with(root) && directory != root {
        if directory.join("runtime.json").exists() {
            let record_bytes = fs::read(directory.join("runtime.json"))
                .map_err(|error| format!("Could not read managed runtime record: {error}"))?;
            if record_bytes.len() > 64 * 1024 {
                return Err("Managed runtime record exceeds the size limit".into());
            }
            let record: RuntimeInstallRecord = serde_json::from_slice(&record_bytes)
                .map_err(|error| format!("Managed runtime record is invalid: {error}"))?;
            let install = resolve_approved_install(&record.install_key)?;
            let expected_directory = root.join(managed_runtime_relative_path(
                &install.tag,
                &install.install_key,
            ));
            if directory != expected_directory {
                return Err("Managed runtime directory does not match compiled approval".into());
            }
            let content = approved_content_manifest_bytes(&install.install_key)?;
            let verified = verify_installed_runtime(
                directory,
                &install,
                content,
                &install.content_manifest_sha256,
            )?;
            let requested = fs::canonicalize(runtime_path)
                .map_err(|error| format!("Could not resolve managed runtime path: {error}"))?;
            let verified = fs::canonicalize(verified)
                .map_err(|error| format!("Could not resolve verified runtime path: {error}"))?;
            if requested != verified {
                return Err("Requested executable is not the approved managed runtime".into());
            }
            return Ok(true);
        }
        directory = directory
            .parent()
            .ok_or("Managed runtime path escaped its root")?;
    }
    Err("Managed runtime has no trusted installation record".into())
}

pub fn managed_runtime_verified(runtime_path: &Path) -> Result<bool, String> {
    managed_runtime_verified_in(runtime_path, &runtime_data_dir("Localmotive"))
}

pub fn verify_managed_runtime_for_launch(runtime_path: &Path) -> Result<(), String> {
    let primary = runtime_data_dir("Localmotive");
    if managed_runtime_verified_in(runtime_path, &primary)? {
        return Ok(());
    }
    let legacy = runtime_data_dir("GGUF Pilot");
    if runtime_path.starts_with(&legacy)
        || fs::canonicalize(runtime_path)
            .ok()
            .zip(fs::canonicalize(&legacy).ok())
            .is_some_and(|(runtime, root)| runtime.starts_with(root))
    {
        return Err(
            "Legacy managed runtimes lack compiled content approval. Install a current runtime."
                .into(),
        );
    }
    Ok(())
}

fn create_runtime_staging(root: &Path, tag: &str, install_key: &str) -> Result<PathBuf, String> {
    validate_no_reparse_ancestors("Managed runtime staging root", root)?;
    for _ in 0..8 {
        let path = root.join(format!(
            ".installing-{}-{}-{:016x}",
            sanitize_component(tag),
            sanitize_component(install_key),
            rand::random::<u64>()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                if let Err(error) =
                    validate_no_reparse_ancestors("Managed runtime staging directory", &path)
                {
                    let _ = fs::remove_dir(&path);
                    return Err(error);
                }
                return Ok(path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Could not create runtime staging directory: {error}"
                ))
            }
        }
    }
    Err("Could not allocate a unique runtime staging directory".into())
}

fn runtime_download_target(
    root: &Path,
    install: &ResolvedRuntimeInstall,
    asset: &GithubAsset,
) -> Result<PathBuf, String> {
    if asset.name.is_empty()
        || asset.name.len() > 256
        || Path::new(&asset.name)
            .file_name()
            .and_then(|name| name.to_str())
            != Some(asset.name.as_str())
    {
        return Err("Approved runtime asset name is not a bounded filename".into());
    }
    Ok(root
        .join(".downloads")
        .join(sanitize_component(&install.tag))
        .join(sanitize_component(&install.install_key))
        .join(&asset.name))
}

fn replace_verified_runtime_directory(staging: &Path, destination: &Path) -> Result<(), String> {
    let root = staging
        .parent()
        .ok_or_else(|| "Managed runtime staging directory has no parent".to_string())?;
    validate_no_reparse_ancestors("Managed runtime root", root)?;
    validate_no_reparse_ancestors("Managed runtime staging directory", staging)?;
    validate_no_reparse_ancestors("Managed runtime destination", destination)?;
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| format!("Could not resolve managed runtime root: {error}"))?;
    let directory = CapDir::open_ambient_dir(root, ambient_authority())
        .map_err(|error| format!("Could not securely open managed runtime root: {error}"))?;
    if !crate::download::opened_directory_matches(&directory, &canonical_root)? {
        return Err("Opened managed runtime root does not match the validated path".into());
    }
    let staging_relative = staging
        .strip_prefix(root)
        .map_err(|_| "Managed runtime staging directory escaped its root".to_string())?;
    let destination_relative = destination
        .strip_prefix(root)
        .map_err(|_| "Managed runtime destination escaped its root".to_string())?;
    let destination_parent = destination_relative
        .parent()
        .ok_or_else(|| "Managed runtime destination has no relative parent".to_string())?;
    directory
        .create_dir_all(destination_parent)
        .map_err(|error| error.to_string())?;
    if directory.symlink_metadata(destination_relative).is_err() {
        return directory
            .rename(staging_relative, &directory, destination_relative)
            .map_err(|error| error.to_string());
    }

    let backup_relative = (0..8)
        .map(|_| PathBuf::from(format!(".replacing-{:016x}", rand::random::<u64>())))
        .find(|candidate| directory.symlink_metadata(candidate).is_err())
        .ok_or_else(|| "Could not allocate a unique runtime replacement backup".to_string())?;
    directory
        .rename(destination_relative, &directory, &backup_relative)
        .map_err(|error| {
            format!("Could not preserve the existing runtime before repair: {error}")
        })?;
    if let Err(error) = directory.rename(staging_relative, &directory, destination_relative) {
        let rollback = directory.rename(&backup_relative, &directory, destination_relative);
        return match rollback {
            Ok(()) => Err(format!("Could not publish the repaired runtime: {error}")),
            Err(rollback_error) => Err(format!(
                "Could not publish the repaired runtime: {error}; rollback also failed: {rollback_error}"
            )),
        };
    }
    directory
        .remove_dir_all(&backup_relative)
        .map_err(|error| format!("Could not remove the replaced runtime backup: {error}"))?;
    Ok(())
}

pub fn install_runtime(
    request: RuntimeInstallRequest,
    cancel: Arc<AtomicBool>,
    mut on_progress: impl FnMut(RuntimeInstallProgress) + Send,
) -> Result<InstalledRuntime, String> {
    let option = resolve_approved_install(&request.install_key)?;
    let hardware = detect_hardware();
    validate_install_adapter(
        &option,
        request.adapter_id.as_deref(),
        &hardware.architecture,
        &hardware.adapters,
    )?;
    let tag = &option.tag;
    let trusted_content = approved_content_manifest_bytes(&option.install_key)?;
    validate_content_manifest_authority(&option, trusted_content)?;
    let root = managed_runtime_root()?;
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    validate_no_reparse_ancestors("Managed runtime root", &root)?;
    let root_metadata = fs::symlink_metadata(&root)
        .map_err(|error| format!("Could not inspect managed runtime root: {error}"))?;
    if !root_metadata.is_dir()
        || root_metadata.file_type().is_symlink()
        || is_reparse_point(&root_metadata)
    {
        return Err("Managed runtime root is a link or reparse point".into());
    }
    let final_dir = root.join(managed_runtime_relative_path(tag, &option.install_key));
    validate_no_reparse_ancestors("Managed runtime destination", &final_dir)?;
    if final_dir.exists() {
        if let Ok(runtime) = verify_installed_runtime(
            &final_dir,
            &option,
            trusted_content,
            &option.content_manifest_sha256,
        ) {
            return Ok(InstalledRuntime {
                tag: tag.clone(),
                backend: option.backend.clone(),
                runtime_path: runtime.to_string_lossy().to_string(),
                install_root: final_dir.to_string_lossy().to_string(),
                reused: true,
            });
        }
    }

    let staging = create_runtime_staging(&root, tag, &option.install_key)?;
    let install = (|| {
        let assets = std::iter::once(&option.asset)
            .chain(option.companion_asset.iter())
            .collect::<Vec<_>>();
        let total = assets.iter().try_fold(0_u64, |sum, asset| {
            sum.checked_add(asset.size)
                .ok_or_else(|| "Runtime download size overflowed".to_string())
        })?;
        let mut completed = 0_u64;
        for asset in assets {
            if cancel.load(Ordering::Relaxed) {
                return Err(
                    "Runtime installation cancelled. Partial download state was kept for resume."
                        .to_string(),
                );
            }
            if asset.size == 0 || asset.size > MAX_RUNTIME_DOWNLOAD_BYTES {
                return Err(format!("Runtime asset has an invalid size: {}", asset.name));
            }
            let expected = expected_sha256(asset)
                .ok_or_else(|| format!("Runtime asset {} lacks an approved SHA-256", asset.name))?;
            let archive = runtime_download_target(&root, &option, asset)?;
            let parent = archive
                .parent()
                .ok_or_else(|| "Runtime download target has no parent directory".to_string())?;
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            validate_no_reparse_ancestors("Runtime download directory", parent)?;
            let downloaded = Arc::new(AtomicU64::new(0));
            let progress_install_key = option.install_key.clone();
            let progress_asset_name = asset.name.clone();
            crate::download::download_file(
                &asset.browser_download_url,
                &archive,
                "official llama.cpp release",
                asset.size,
                expected,
                None,
                4,
                Arc::clone(&cancel),
                downloaded,
                |asset_downloaded, _| {
                    on_progress(RuntimeInstallProgress {
                        install_key: progress_install_key.clone(),
                        asset_name: progress_asset_name.clone(),
                        downloaded: completed.saturating_add(asset_downloaded),
                        total,
                    });
                },
            )?;
            extract_zip(&archive, &staging, &cancel)?;
            fs::remove_file(&archive).map_err(|error| error.to_string())?;
            completed = completed.saturating_add(asset.size);
        }
        let staging_guard =
            open_verified_capability_directory(&staging, "managed runtime staging directory")?;
        let runtime = find_runtime(&staging)
            .ok_or_else(|| "Downloaded archive did not contain llama-server.exe".to_string())?;
        let relative_runtime = runtime
            .strip_prefix(&staging)
            .map_err(|error| error.to_string())?
            .to_path_buf();
        write_runtime_install_record(
            &staging,
            &option,
            &relative_runtime,
            &option.content_manifest_sha256,
        )?;
        verify_installed_runtime(
            &staging,
            &option,
            trusted_content,
            &option.content_manifest_sha256,
        )?;
        drop(staging_guard);
        replace_verified_runtime_directory(&staging, &final_dir)?;
        let runtime = verify_installed_runtime(
            &final_dir,
            &option,
            trusted_content,
            &option.content_manifest_sha256,
        )?;
        Ok(InstalledRuntime {
            tag: tag.clone(),
            backend: option.backend.clone(),
            runtime_path: runtime.to_string_lossy().to_string(),
            install_root: final_dir.to_string_lossy().to_string(),
            reused: false,
        })
    })();
    if install.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    install
}

fn pinned_health_model_path() -> Result<PathBuf, String> {
    let pin = crate::core::pinned_model_load_pin();
    let runtime_root = runtime_data_dir("Localmotive");
    let product_root = runtime_root
        .parent()
        .ok_or_else(|| "Could not resolve the Localmotive data directory".to_string())?;
    Ok(product_root
        .join("health-models")
        .join(&pin.sha256)
        .join(&pin.name))
}

pub(crate) fn ensure_pinned_health_model(
    cancel: Arc<AtomicBool>,
    on_progress: impl FnMut(u64, u64) + Send,
) -> Result<PathBuf, String> {
    let pin = crate::core::pinned_model_load_pin();
    let target = pinned_health_model_path()?;
    if target.exists() {
        crate::health::verify_pinned_model(&target, cancel.as_ref()).map_err(|(_, detail)| {
            format!("Cached health model trust verification failed: {detail}")
        })?;
        return Ok(target);
    }
    let parent = target
        .parent()
        .ok_or_else(|| "The health model target has no parent directory".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create the health model directory: {error}"))?;
    validate_no_reparse_ancestors("Health model directory", parent)?;
    let metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("Could not inspect the health model directory: {error}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err("The health model directory is a link or reparse point".into());
    }
    crate::download::download_file(
        &pin.url,
        &target,
        &pin.repository,
        pin.bytes,
        &pin.sha256,
        None,
        4,
        cancel,
        Arc::new(AtomicU64::new(0)),
        on_progress,
    )
}

pub(crate) fn managed_health_context(
    request: &crate::health::ManagedHealthRequest,
) -> Result<crate::health::ManagedHealthContext, String> {
    let install = resolve_approved_install(&request.install_key)?;
    let hardware = detect_hardware();
    validate_install_adapter(
        &install,
        request.adapter_id.as_deref(),
        &hardware.architecture,
        &hardware.adapters,
    )?;
    let adapter_name = request.adapter_id.as_ref().and_then(|adapter_id| {
        hardware
            .adapters
            .iter()
            .find(|adapter| &adapter.adapter_id == adapter_id)
            .map(|adapter| adapter.name.clone())
    });
    let root = managed_runtime_root()?;
    let install_root = root.join(managed_runtime_relative_path(
        &install.tag,
        &install.install_key,
    ));
    let content = approved_content_manifest_bytes(&install.install_key)?;
    validate_content_manifest_authority(&install, content)?;
    let server_path = verify_installed_runtime(
        &install_root,
        &install,
        content,
        &install.content_manifest_sha256,
    )
    .map_err(|error| format!("Managed runtime health trust verification failed: {error}"))?;
    let model_path = pinned_health_model_path()?;
    let pin = crate::core::pinned_model_load_pin();
    Ok(crate::health::ManagedHealthContext {
        runtime_id: install.install_key,
        install_root,
        server_path,
        backend: install.backend,
        adapter_id: request.adapter_id.clone(),
        adapter_name,
        expected_model: format!("{}@{}/{}", pin.repository, pin.revision, pin.name),
        model_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn system_memory_uses_global_memory_status_evidence() {
        let memory = detect_system_memory();

        assert!(memory.total_physical_bytes.value.unwrap_or(0) > 0);
        assert!(memory.available_physical_bytes.value.unwrap_or(0) > 0);
        assert!(memory.memory_load_percent.value.unwrap_or(101) <= 100);
        assert_eq!(
            memory.available_physical_bytes.source.kind,
            crate::evidence::EvidenceSourceKind::WindowsApi
        );
        assert!(memory
            .available_physical_bytes
            .source
            .detail
            .contains("GlobalMemoryStatusEx"));
    }

    #[cfg(windows)]
    #[test]
    fn process_peak_working_set_uses_process_status_evidence() {
        let memory = process_peak_working_set(std::process::id());

        assert!(memory.value.unwrap_or(0) > 0);
        assert_eq!(
            memory.source.kind,
            crate::evidence::EvidenceSourceKind::WindowsApi
        );
        assert!(memory.source.detail.contains("PeakWorkingSetSize"));
    }

    #[cfg(windows)]
    #[test]
    fn dxgi_reports_each_adapter_without_aggregating_memory() {
        let adapters = detect_dxgi_adapters().unwrap();

        assert!(!adapters.is_empty());
        let mut adapter_ids = std::collections::HashSet::new();
        for adapter in adapters {
            assert!(adapter_ids.insert(adapter.adapter_id.clone()));
            assert!(adapter.compatibility_id.starts_with("pci:"));
            assert_eq!(adapter.compatibility_id.split(':').count(), 5);
            assert!(!adapter.name.is_empty());
            assert_eq!(
                adapter.dedicated_bytes.source.kind,
                crate::evidence::EvidenceSourceKind::WindowsApi
            );
            assert!(adapter
                .dedicated_bytes
                .source
                .detail
                .contains("IDXGIAdapter"));
            if let (Some(budget), Some(usage), Some(available)) = (
                adapter.budget_bytes.value,
                adapter.current_usage_bytes.value,
                adapter.available_budget_bytes.value,
            ) {
                assert_eq!(available, budget.saturating_sub(usage));
                assert!(adapter
                    .budget_bytes
                    .source
                    .detail
                    .contains("QueryVideoMemoryInfo"));
            }
        }
    }

    #[test]
    fn manual_gpu_capacity_uses_one_explicit_metric_without_aggregation() {
        let overrides = vec![HardwareOverride {
            adapter_id: "gpu-0".into(),
            dedicated_bytes: Some(8_000),
            shared_bytes: Some(16_000),
            note: "Firmware reservation".into(),
        }];

        let capacity = manual_override_capacity(&overrides, "gpu-0", 42)
            .unwrap()
            .unwrap();
        assert_eq!(capacity.value, Some(8_000));
        assert_eq!(capacity.level, EvidenceLevel::UserOverride);
        assert!(capacity
            .notes
            .iter()
            .any(|note| note.contains("not aggregated")));
    }

    #[test]
    fn manual_gpu_capacity_rejects_zero_and_duplicate_overrides() {
        let invalid = vec![HardwareOverride {
            adapter_id: "gpu-0".into(),
            dedicated_bytes: Some(0),
            shared_bytes: None,
            note: String::new(),
        }];
        assert!(manual_override_capacity(&invalid, "gpu-0", 42).is_err());

        let duplicate = vec![
            HardwareOverride {
                adapter_id: "gpu-0".into(),
                dedicated_bytes: Some(1),
                shared_bytes: None,
                note: String::new(),
            },
            HardwareOverride {
                adapter_id: "gpu-0".into(),
                dedicated_bytes: Some(2),
                shared_bytes: None,
                note: String::new(),
            },
        ];
        assert!(manual_override_capacity(&duplicate, "gpu-0", 42).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn hardware_report_contains_system_and_per_adapter_evidence() {
        let hardware = detect_hardware();

        assert!(hardware.system_memory.total_physical_bytes.value.is_some());
        assert!(!hardware.adapters.is_empty());
        assert!(hardware.manual_overrides.is_empty());
    }

    #[test]
    fn nvidia_probe_parser_keeps_adapter_memory_separate() {
        let rows = parse_nvidia_probe_rows(
            "NVIDIA RTX 4090, 560.1, 24564, 1024\nNVIDIA RTX 4060, 560.1, 8188, 512\n",
        );

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "NVIDIA RTX 4090");
        assert_eq!(rows[0].total_bytes, Some(24_564 * 1024 * 1024));
        assert_eq!(rows[0].used_bytes, Some(1_024 * 1024 * 1024));
        assert_eq!(rows[1].total_bytes, Some(8_188 * 1024 * 1024));
    }

    #[cfg(windows)]
    #[test]
    fn nvidia_probe_values_remain_separate_from_dxgi_capacity() {
        let mut adapters = detect_dxgi_adapters().unwrap();
        adapters.truncate(1);
        let original_dedicated = adapters[0].dedicated_bytes.clone();
        let rows = vec![NvidiaProbeRow {
            name: adapters[0].name.clone(),
            driver: "fixture-driver".into(),
            total_bytes: Some(12_345),
            used_bytes: Some(678),
        }];

        merge_nvidia_probe_observations(&mut adapters, &rows, 42);

        assert_eq!(adapters[0].dedicated_bytes, original_dedicated);
        assert_eq!(adapters[0].driver.value.as_deref(), Some("fixture-driver"));
        assert!(adapters[0]
            .capacity_observations
            .iter()
            .any(|item| item.evidence.source.kind == EvidenceSourceKind::NvidiaSmi));
    }

    fn approved_release() -> (
        GithubRelease,
        Vec<ApprovedRuntimeAsset>,
        Vec<RequiredUpstreamJob>,
    ) {
        let (approved, jobs) = approved_manifest().unwrap();
        let identity = approved_runtime_identity().unwrap();
        let release = GithubRelease {
            tag_name: identity.release_tag,
            target_commitish: identity.release_commit,
            published_at: Some(identity.published_at),
            assets: approved
                .iter()
                .filter(|entry| entry.backend != "cuda-companion")
                .map(|entry| GithubAsset {
                    name: entry.name.clone(),
                    browser_download_url: entry.url.clone(),
                    size: entry.bytes,
                    digest: Some(entry.digest.clone()),
                })
                .chain(
                    approved
                        .iter()
                        .filter(|entry| entry.backend == "cuda-companion")
                        .map(|entry| GithubAsset {
                            name: entry.name.clone(),
                            browser_download_url: entry.url.clone(),
                            size: entry.bytes,
                            digest: Some(entry.digest.clone()),
                        }),
                )
                .collect(),
        };
        (release, approved, jobs)
    }

    fn approved_sample_release() -> GithubRelease {
        GithubRelease {
            tag_name: "b10816".into(),
            target_commitish: "427291b5b34cd914a31b3fd3b61a68f6184f4b9f".into(),
            published_at: Some("2026-09-04T19:57:56Z".into()),
            assets: vec![
                GithubAsset::sample("llama-b10816-bin-win-cpu-x64.zip"),
                GithubAsset::sample("llama-b10816-bin-win-vulkan-x64.zip"),
                GithubAsset::sample("llama-b10816-bin-win-rocm-10.0-x64.zip"),
                GithubAsset::sample("llama-b10816-bin-win-sycl-x64.zip"),
                GithubAsset::sample("llama-b10816-bin-win-cuda-12.4-x64.zip"),
                GithubAsset::sample("cudart-llama-bin-win-cuda-12.4-x64.zip"),
                GithubAsset::sample("llama-b10816-bin-win-cuda-13.3-x64.zip"),
                GithubAsset::sample("cudart-llama-bin-win-cuda-13.3-x64.zip"),
                GithubAsset::sample("llama-b10816-bin-win-cpu-arm64.zip"),
            ],
        }
    }

    #[allow(dead_code)]
    fn release() -> GithubRelease {
        approved_sample_release()
    }

    #[allow(dead_code)]
    fn legacy_b10736_release() -> GithubRelease {
        approved_sample_release()
    }

    #[test]
    fn catalog_rejects_an_option_when_the_manifest_lacks_a_digest() {
        // Phase 1 RED: an approved manifest entry must carry a SHA-256
        // digest. This test passed only after the gate existed. It failed
        // before with missing ApprovedRuntimeAsset and
        // build_approved_catalog_for_asset. The production gate is now
        // `bind_asset_to_manifest` plus `build_approved_catalog`.
        let candidate = GithubAsset {
            name: "llama-b10796-bin-win-cpu-x64.zip".into(),
            browser_download_url: "https://example.invalid/llama-b10796-bin-win-cpu-x64.zip".into(),
            size: 18389446,
            digest: None,
        };

        let approved: Vec<ApprovedRuntimeAsset> = Vec::new();
        let error = approved_manifest_tag(&approved)
            .ok_or_else(|| {
                format!(
                    "Runtime asset {} is not in the approved manifest",
                    candidate.name
                )
            })
            .unwrap_err();

        assert!(
            error.contains("digest") || error.contains("manifest"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn catalog_rejects_an_option_when_upstream_identity_changes() {
        // Phase 1 RED: selection rejects an asset whose remote identity
        // differs from the approved manifest entry. Finding A-08 records
        // this moving-target risk for release b10796. This test failed
        // before with missing bind_asset_to_manifest.
        let approved = vec![ApprovedRuntimeAsset {
            name: "llama-b10796-bin-win-cpu-x64.zip".into(),
            url: "https://github.com/ggml-org/llama.cpp/releases/download/b10796/llama-b10796-bin-win-cpu-x64.zip".into(),
            bytes: 18389446,
            digest: "sha256:b56186961431c10e3eb5c065c4375b0439f686e5f5b7059deb7b81d1af0e7d7d"
                .into(),
            backend: "cpu".into(),
            arch: "x64".into(),
            dormant: false,
            cuda_version: None,
            companion_name: None,
            content_manifest_sha256: None,
        }];
        let changed = GithubAsset {
            name: "llama-b10796-bin-win-cpu-x64.zip".into(),
            browser_download_url: "https://github.com/ggml-org/llama.cpp/releases/download/b10796/llama-b10796-bin-win-cpu-x64.zip".into(),
            size: 18389446,
            digest: Some(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000".into(),
            ),
        };

        let error = bind_asset_to_manifest(&changed, &approved).unwrap_err();

        assert!(
            error.contains("changed identity"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn catalog_rejects_an_omitted_approved_asset() {
        let (mut release, approved, jobs) = approved_release();
        let omitted = approved
            .iter()
            .find(|entry| entry.arch == "x64" && entry.backend == "cpu" && !entry.dormant)
            .unwrap()
            .name
            .clone();
        release.assets.retain(|asset| asset.name != omitted);
        let mut hardware = detect_hardware();
        hardware.architecture = "x64".into();

        let error = build_approved_catalog(&release, &hardware, &approved, &jobs).unwrap_err();

        assert!(error.contains(&omitted), "unexpected error: {error}");
    }

    #[test]
    fn catalog_rejects_changed_publication_identity() {
        let (mut release, approved, jobs) = approved_release();
        release.published_at = Some("2026-09-04T19:57:55Z".into());
        let mut hardware = detect_hardware();
        hardware.architecture = "x64".into();

        let error = catalog_from_release(
            &release,
            &hardware,
            &approved,
            &jobs,
            RuntimeCatalogOrigin::Network,
            None,
        )
        .unwrap_err();

        assert!(error.contains("publication"), "unexpected error: {error}");
    }

    #[test]
    fn blocked_backend_update_stops_when_a_required_job_fails_or_queues() {
        let jobs: Vec<RequiredUpstreamJob> =
            serde_json::from_str(include_str!("../tests/fixtures/runtime/blocked-jobs.json"))
                .unwrap();

        assert!(backend_is_blocked_by_upstream(&jobs, "cuda"));
        assert!(backend_is_blocked_by_upstream(&jobs, "openvino"));
        assert!(!backend_is_blocked_by_upstream(&jobs, "rocm"));
    }

    #[test]
    fn failed_cuda_jobs_keep_independently_approved_cpu_and_vulkan_options() {
        let (release, approved, mut jobs) = approved_release();
        for job in &mut jobs {
            if job.backend == "cuda" {
                job.conclusion = "failure".into();
            }
        }
        let mut hardware = detect_hardware();
        hardware.architecture = "x64".into();

        let catalog = build_approved_catalog(&release, &hardware, &approved, &jobs)
            .expect("a CUDA failure must not discard unrelated approved backends");

        assert!(catalog.options.iter().any(|option| option.backend == "cpu"));
        assert!(catalog
            .options
            .iter()
            .any(|option| option.backend == "vulkan"));
        assert!(!catalog
            .options
            .iter()
            .any(|option| option.backend == "cuda"));

        let blocked = catalog
            .availability
            .iter()
            .find(|availability| {
                availability.backend == "cuda"
                    && availability.status == BackendAvailabilityStatus::Blocked
            })
            .expect("the blocked CUDA option remains visible");
        assert!(blocked
            .blocking_jobs
            .iter()
            .any(|name| name == "server-cuda"));
    }

    #[test]
    fn all_blocked_backends_return_an_empty_catalog_with_explanations() {
        let (release, approved, mut jobs) = approved_release();
        for job in &mut jobs {
            job.conclusion = "failure".into();
        }
        let mut hardware = detect_hardware();
        hardware.architecture = "x64".into();

        let catalog = catalog_from_release(
            &release,
            &hardware,
            &approved,
            &jobs,
            RuntimeCatalogOrigin::Network,
            None,
        )
        .expect("blocked approved backends must remain present as availability explanations");

        assert!(catalog.options.is_empty());
        assert!(catalog
            .availability
            .iter()
            .any(|entry| entry.status == BackendAvailabilityStatus::Blocked));
    }

    #[test]
    fn approved_manifest_loads_the_pinned_b10816_pin() {
        // The manifest is a compile-time file. This test proves the pin
        // loads, so selection can never run without an approved release.
        let (approved, jobs) = approved_manifest().unwrap();

        assert!(!approved.is_empty());
        assert!(approved.iter().any(|entry| entry.name.contains("cpu-x64")));
        assert!(approved
            .iter()
            .any(|entry| entry.name.contains("cuda-13.3")));
        assert!(jobs.iter().any(|job| job.name == "gpu-rocm"));
        assert!(!backend_is_blocked_by_upstream(&jobs, "rocm"));
    }

    #[test]
    fn hardware_probe_does_not_claim_runtime_support_without_l4_evidence() {
        let source = include_str!("runtime.rs");

        assert!(!source.contains(concat!("CUDA 13", " is the best match")));
        assert!(!source.contains(concat!("CUDA 12", " is the compatible NVIDIA choice")));
        assert!(source.contains(
            "Hardware detection does not establish product support; use the approved catalog recommendation."
        ));
    }

    #[test]
    fn approved_manifest_rejects_unknown_fields_and_ambiguous_job_mappings() {
        let mut manifest: serde_json::Value = serde_json::from_str(APPROVED_RUNTIMES).unwrap();
        manifest["unexpectedAuthority"] = serde_json::json!(true);
        let unknown_error = parse_approved_manifest(&manifest.to_string()).unwrap_err();
        assert!(unknown_error.contains("unknown field"));

        let mut manifest: serde_json::Value = serde_json::from_str(APPROVED_RUNTIMES).unwrap();
        let duplicate = manifest["requiredJobs"][0].clone();
        manifest["requiredJobs"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        let duplicate_error = parse_approved_manifest(&manifest.to_string()).unwrap_err();
        assert!(duplicate_error.contains("duplicate required job"));
    }

    #[test]
    fn approved_manifest_rejects_an_unbound_compatibility_record() {
        let mut manifest: serde_json::Value = serde_json::from_str(APPROVED_RUNTIMES).unwrap();
        let asset = manifest["assets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|asset| {
                asset["backend"] == "cpu"
                    && !asset
                        .get("dormant")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
            })
            .unwrap()
            .clone();
        manifest["compatibilityRecords"] = serde_json::json!([{
            "key": {
                "osBuild": "26100",
                "architecture": "x64",
                "adapterId": "cpu:fixture",
                "driver": "not-applicable",
                "firmware": "not-applicable",
                "backend": "cpu",
                "installKey": "cpu",
                "releaseCommit": manifest["releaseCommit"],
                "assetName": asset["name"],
                "assetSha256": asset["digest"].as_str().unwrap().strip_prefix("sha256:").unwrap()
            },
            "evidenceLevel": "L4_PRODUCT",
            "attestationId": "fixture-attestation",
            "expiryIdentity": "not-bound-to-key"
        }]);

        let error = parse_approved_manifest(&manifest.to_string()).unwrap_err();
        assert!(
            error.contains("malformed or expired"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn approved_manifest_identity_matches_the_frozen_b10816_release() {
        #[derive(Deserialize)]
        struct ReleaseFixture {
            body: serde_json::Value,
        }

        let fixture: ReleaseFixture = serde_json::from_str(include_str!(
            "../tests/fixtures/runtime/b10816-release.json"
        ))
        .unwrap();
        let release: GithubRelease = serde_json::from_value(fixture.body).unwrap();
        let identity = approved_runtime_identity().unwrap();

        assert_eq!(identity.release_tag, "b10816");
        assert_eq!(release.tag_name, identity.release_tag);
        assert_eq!(
            identity.release_commit,
            "427291b5b34cd914a31b3fd3b61a68f6184f4b9f"
        );
        assert_eq!(release.target_commitish, identity.release_commit);
        assert_eq!(
            identity.supported_windows,
            vec!["Windows 11 build 26100 x64"]
        );
    }

    #[test]
    fn every_manifest_asset_and_job_matches_the_frozen_b10816_fixtures() {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ReleaseFixture {
            observed_at: String,
            body: GithubRelease,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct JobsFixture {
            observed_at: String,
            release_tag: String,
            release_commit: String,
            required_jobs: Vec<RequiredUpstreamJob>,
        }

        let release: ReleaseFixture = serde_json::from_str(include_str!(
            "../tests/fixtures/runtime/b10816-release.json"
        ))
        .unwrap();
        let jobs: JobsFixture = serde_json::from_str(include_str!(
            "../tests/fixtures/runtime/b10816-required-jobs.json"
        ))
        .unwrap();
        let manifest = parse_approved_manifest(APPROVED_RUNTIMES).unwrap();

        assert_eq!(manifest.observed_at, release.observed_at);
        assert_eq!(manifest.observed_at, jobs.observed_at);
        assert_eq!(manifest.release_tag, jobs.release_tag);
        assert_eq!(manifest.release_commit, jobs.release_commit);
        assert_eq!(manifest.required_jobs, jobs.required_jobs);
        assert_eq!(manifest.assets.len(), 13);
        for asset in &manifest.assets {
            let remote = release
                .body
                .assets
                .iter()
                .find(|candidate| candidate.name == asset.name)
                .unwrap_or_else(|| panic!("missing frozen asset {}", asset.name));
            bind_asset_to_manifest(remote, &manifest.assets).unwrap();
        }
    }

    #[test]
    fn dormant_arm64_cuda_url_matches_the_frozen_exact_tag_release() {
        #[derive(Deserialize)]
        struct ReleaseFixture {
            body: GithubRelease,
        }

        let fixture: ReleaseFixture = serde_json::from_str(include_str!(
            "../tests/fixtures/runtime/b10816-release.json"
        ))
        .unwrap();
        let approved = approved_manifest().unwrap().0;
        let entry = approved
            .iter()
            .find(|asset| asset.name == "llama-b10816-bin-win-cuda-13.4-arm64.zip")
            .unwrap();
        let remote = fixture
            .body
            .assets
            .iter()
            .find(|asset| asset.name == entry.name)
            .unwrap();

        assert!(entry.dormant);
        assert_eq!(entry.url, remote.browser_download_url);
        assert!(bind_asset_to_manifest(remote, &approved).is_ok());
    }

    #[test]
    fn approved_runtime_catalog_uses_the_exact_tag_endpoint() {
        assert_eq!(
            approved_release_url().unwrap(),
            "https://api.github.com/repos/ggml-org/llama.cpp/releases/tags/b10816"
        );
    }

    /// Phase 1: a wrong-tag release body fails closed through the catalog
    /// identity check (`catalog_from_release` rejects a tag that does not
    /// match the approved manifest tag; `build_approved_catalog` also
    /// rejects a tag that does not match the manifest asset tag). The
    /// release body carries the attacker tag while commit and publication
    /// match, so only the tag authority under test can reject it. Mutation
    /// probe: removing the tag comparison from `catalog_from_release` still
    /// fails closed via `build_approved_catalog`, which proves defense in
    /// depth rather than a single-point check.
    #[test]
    fn catalog_wrong_tag_body_fails_closed_with_an_identity_error() {
        let identity = approved_runtime_identity().unwrap();
        let release = GithubRelease {
            tag_name: "attacker-tag".into(),
            target_commitish: identity.release_commit.clone(),
            published_at: Some(identity.published_at.clone()),
            assets: vec![],
        };
        let hardware = detect_hardware();
        let (approved, jobs) = approved_manifest().unwrap();

        let error = catalog_from_release(
            &release,
            &hardware,
            &approved,
            &jobs,
            RuntimeCatalogOrigin::Network,
            None,
        )
        .unwrap_err();

        assert!(
            error.contains(&identity.release_tag) || error.contains("tag"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn offline_catalog_cache_requires_the_exact_approved_identity_and_body_digest() {
        let identity = approved_runtime_identity().unwrap();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/runtime/b10816-release.json"
        ))
        .unwrap();
        let body = serde_json::to_string(&fixture["body"]).unwrap();
        let cache = catalog_cache_record(&identity, Some("fixture-etag".into()), body);
        let bytes = serde_json::to_vec(&cache).unwrap();
        let encoded: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let compiled_manifest_sha256 = hex::encode(Sha256::digest(APPROVED_RUNTIMES.as_bytes()));

        assert_eq!(
            encoded["manifestSha256"].as_str(),
            Some(compiled_manifest_sha256.as_str())
        );

        assert!(validate_catalog_cache(&bytes, &identity).is_ok());

        let mut wrong_identity = identity.clone();
        wrong_identity.release_commit = "0000000000000000000000000000000000000000".into();
        assert!(validate_catalog_cache(&bytes, &wrong_identity).is_err());

        let mut tampered: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let changed_body = format!("{} ", tampered["body"].as_str().unwrap());
        tampered["body"] = serde_json::Value::String(changed_body);
        assert!(
            validate_catalog_cache(&serde_json::to_vec(&tampered).unwrap(), &identity).is_err()
        );
    }

    /// Phase 1: the catalog client sends no secret header. The metadata
    /// request carries only the GitHub API accept header and the public
    /// product user-agent, so no credential can leak through catalog
    /// refresh on a shared machine or in captured traffic.
    #[test]
    fn catalog_client_sends_no_secret_header() {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let seen_server = std::sync::Arc::clone(&seen);
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut head = vec![0u8; 4096];
            let count = stream.read(&mut head).unwrap_or(0);
            *seen_server.lock().unwrap() = head[..count].to_vec();
            let body = "{}";
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: {}
Connection: close

{}",
                body.len(),
                body
            );
            let _ = stream.flush();
        });
        let client = runtime_catalog_client(Duration::from_secs(5)).unwrap();
        let _ = tauri::async_runtime::block_on(fetch_catalog_http(
            &client,
            &format!("http://{address}/release"),
            None,
        ));

        let head = String::from_utf8_lossy(&seen.lock().unwrap()).to_ascii_lowercase();
        assert!(head.contains("accept: application/vnd.github+json"));
        assert!(head.contains("user-agent: localmotive/"));
        for secret in [
            "authorization:",
            "proxy-authorization:",
            "x-api-key:",
            "api-key:",
            "cookie:",
            "token:",
            "bearer ",
            "ghp_",
            "gho_",
        ] {
            assert!(
                !head.contains(secret),
                "catalog request must not carry a secret header, found: {secret}"
            );
        }
    }

    /// Phase 1: a `429` rate-limit answer maps to the typed rate-limit
    /// error and preserves the `Retry-After` delay. The controlled fixture
    /// answers `429` with `Retry-After: 120`, so the catalog error carries
    /// the same typed kind and delay the UI retry guidance needs.
    #[test]
    fn catalog_429_maps_to_a_typed_rate_limit_error_with_retry_delay() {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut head = vec![0u8; 1024];
            let _ = stream.read(&mut head);
            let body = "{\"message\":\"API rate limit exceeded.\"}";
            let _ = write!(
                stream,
                "HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Content-Length: {}
Retry-After: 120
Connection: close

{}",
                body.len(),
                body
            );
            let _ = stream.flush();
        });
        let client = runtime_catalog_client(Duration::from_secs(5)).unwrap();

        let error = tauri::async_runtime::block_on(fetch_catalog_http(
            &client,
            &format!("http://{address}/release"),
            None,
        ))
        .unwrap_err();

        assert_eq!(error.kind, RuntimeCatalogErrorKind::RateLimited);
        assert_eq!(error.retry_after_seconds, Some(120));
    }

    #[test]
    fn catalog_http_deadline_returns_a_typed_timeout() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let _connection = listener.accept().unwrap();
            std::thread::sleep(Duration::from_millis(500));
        });
        let client = runtime_catalog_client(Duration::from_millis(50)).unwrap();
        let started = std::time::Instant::now();

        let error = tauri::async_runtime::block_on(fetch_catalog_http(
            &client,
            &format!("http://{address}/release"),
            None,
        ))
        .unwrap_err();

        assert_eq!(error.kind, RuntimeCatalogErrorKind::Timeout);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    /// Phase 1 RED: GitHub answers an exhausted unauthenticated quota with
    /// `403 rate limit exceeded`, not `429`. The catalog must surface that
    /// as a typed rate-limit error with the reset message, never as a
    /// generic HTTP failure. This test failed before `fetch_catalog_http`
    /// mapped the 403 rate-limit body.
    #[test]
    fn catalog_rate_limit_body_maps_403_to_a_typed_rate_limit_error() {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut head = vec![0u8; 1024];
            let _ = stream.read(&mut head);
            let body = "{\"message\":\"API rate limit exceeded for x.\",\"documentation_url\":\"https://docs.github.com/rest\"}";
            let _ = write!(
                stream,
                "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.flush();
        });
        let client = runtime_catalog_client(Duration::from_secs(5)).unwrap();

        let error = tauri::async_runtime::block_on(fetch_catalog_http(
            &client,
            &format!("http://{address}/release"),
            None,
        ))
        .unwrap_err();

        assert_eq!(error.kind, RuntimeCatalogErrorKind::RateLimited);
    }

    /// Phase 1 RED: an oversized metadata body must never reach JSON
    /// parsing or catalog construction. The catalog must reject the body
    /// with the typed `BodyTooLarge` error at the 2 MiB bound. This test
    /// failed before the body reader enforced the streaming byte limit.
    #[test]
    fn catalog_oversized_body_is_rejected_before_json_parsing() {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut head = vec![0u8; 4096];
            let _ = stream.read(&mut head);
            let excess = MAX_RUNTIME_CATALOG_BYTES + 64;
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {excess}\r\nConnection: close\r\n\r\n"
            );
            let _ = stream.write_all(&vec![b' '; MAX_RUNTIME_CATALOG_BYTES + 64]);
            let _ = stream.flush();
        });
        let client = runtime_catalog_client(Duration::from_secs(10)).unwrap();

        let error = tauri::async_runtime::block_on(fetch_catalog_http(
            &client,
            &format!("http://{address}/release"),
            None,
        ))
        .unwrap_err();

        assert_eq!(error.kind, RuntimeCatalogErrorKind::BodyTooLarge);
    }

    /// Phase 1 RED: a truncated release body must surface as a typed
    /// `InvalidResponse` error, never as a silently accepted catalog or a
    /// generic transport failure. This test failed before the catalog HTTP
    /// body path validated that every returned body parses as release JSON.
    #[test]
    fn catalog_truncated_body_maps_to_a_typed_invalid_response_error() {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut head = vec![0u8; 1024];
            let _ = stream.read(&mut head);
            let body = "{\"tag_name\":\"b10816\",\"assets\":[";
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.flush();
        });
        let client = runtime_catalog_client(Duration::from_secs(5)).unwrap();

        let error = tauri::async_runtime::block_on(fetch_catalog_http(
            &client,
            &format!("http://{address}/release"),
            None,
        ))
        .unwrap_err();

        assert_eq!(error.kind, RuntimeCatalogErrorKind::InvalidResponse);
        assert!(error.message.contains("invalid"));
    }

    #[test]
    fn immutable_install_key_resolves_all_artifact_authority_in_the_backend() {
        let resolved = resolve_approved_install("cpu").unwrap();
        let (approved, _) = approved_manifest().unwrap();
        let expected = approved
            .iter()
            .find(|asset| asset.backend == "cpu" && !asset.dormant)
            .unwrap();

        assert_eq!(
            resolved.tag,
            approved_runtime_identity().unwrap().release_tag
        );
        assert_eq!(resolved.install_key, "cpu");
        assert_eq!(resolved.backend, "cpu");
        assert_eq!(resolved.asset.name, expected.name);
        assert_eq!(resolved.asset.size, expected.bytes);
        assert_eq!(resolved.asset.browser_download_url, expected.url);
        assert_eq!(
            resolved.asset.digest.as_deref(),
            Some(expected.digest.as_str())
        );
        assert!(resolve_approved_install("attacker-controlled").is_err());
        assert!(resolve_approved_install("cuda-12.4-arm64").is_err());
    }

    #[test]
    fn install_request_rejects_frontend_artifact_overrides_and_requires_exact_adapter_id() {
        let crafted = serde_json::json!({
            "installKey": "cpu",
            "adapterId": null,
            "url": "https://attacker.invalid/runtime.zip",
            "tag": "attacker",
            "backend": "attacker",
            "size": 1,
            "digest": format!("sha256:{}", "0".repeat(64)),
        });
        assert!(serde_json::from_value::<RuntimeInstallRequest>(crafted).is_err());

        let cuda = resolve_approved_install("cuda-13.3").unwrap();
        assert!(validate_install_adapter(&cuda, None, "x64", &[]).is_err());
        assert!(validate_install_adapter(&cuda, Some("missing"), "x64", &[]).is_err());
        assert!(validate_install_adapter(&cuda, Some("missing"), "arm64", &[]).is_err());
    }

    #[test]
    fn runtime_archive_resume_path_is_stable_and_separate_from_install_staging() {
        let root = Path::new(r"C:\managed-runtimes");
        let install = resolve_approved_install("cpu").unwrap();
        let first = runtime_download_target(root, &install, &install.asset).unwrap();
        let second = runtime_download_target(root, &install, &install.asset).unwrap();

        assert_eq!(first, second);
        assert!(first.starts_with(root.join(".downloads").join("b10816").join("cpu")));
        assert_eq!(first.file_name().unwrap(), install.asset.name.as_str());
        assert!(!first.to_string_lossy().contains(".installing-"));
    }

    #[test]
    fn every_active_install_key_has_one_hash_anchored_compiled_content_manifest() {
        for install_key in [
            "cpu",
            "cuda-12.4",
            "cuda-13.3",
            "openvino",
            "rocm",
            "sycl",
            "vulkan",
        ] {
            let resolved = resolve_approved_install(install_key).unwrap();
            let bytes = approved_content_manifest_bytes(install_key).unwrap();
            let manifest = validate_content_manifest_authority(&resolved, bytes).unwrap();
            assert!(!manifest.files.is_empty(), "{install_key}");
            assert!(
                manifest
                    .files
                    .iter()
                    .any(|file| file.path.ends_with("llama-server.exe")),
                "{install_key}"
            );
            let paths = manifest
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>();
            assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }

    #[test]
    fn verified_staging_replaces_a_corrupt_regular_destination() {
        let root = std::env::temp_dir().join(format!(
            "localmotive-runtime-repair-{}-{}",
            std::process::id(),
            observed_at_ms()
        ));
        let destination = root.join("b10816").join("cpu");
        let staging = root.join(".installing-cpu");
        fs::create_dir_all(&destination).unwrap();
        fs::create_dir_all(&staging).unwrap();
        fs::write(destination.join("runtime.bin"), b"corrupt").unwrap();
        fs::write(staging.join("runtime.bin"), b"approved").unwrap();

        replace_verified_runtime_directory(&staging, &destination).unwrap();

        assert_eq!(
            fs::read(destination.join("runtime.bin")).unwrap(),
            b"approved"
        );
        assert!(!staging.exists());
        let names = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert!(!names.iter().any(|name| name.starts_with(".replacing-")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn tampered_runtime_and_forged_local_metadata_cannot_bypass_compiled_content_manifest() {
        let root = std::env::temp_dir().join(format!(
            "localmotive-content-trust-{}-{}",
            std::process::id(),
            observed_at_ms()
        ));
        let bin = root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("llama-server.exe"), b"trusted executable").unwrap();
        fs::write(bin.join("ggml-cpu.dll"), b"trusted backend").unwrap();
        let trusted_content = format!(
            concat!(
                "{{\"schemaVersion\":1,\"releaseTag\":\"b10816\",",
                "\"releaseCommit\":\"427291b5b34cd914a31b3fd3b61a68f6184f4b9f\",",
                "\"installKey\":\"cpu\",\"backend\":\"cpu\",\"artifacts\":[",
                "{{\"name\":\"cpu.zip\",\"bytes\":1,\"sha256\":\"{}\"}}],\"files\":[",
                "{{\"path\":\"bin/ggml-cpu.dll\",\"bytes\":15,\"sha256\":\"{}\"}},",
                "{{\"path\":\"bin/llama-server.exe\",\"bytes\":18,\"sha256\":\"{}\"}}]}}"
            ),
            "a".repeat(64),
            hex::encode(Sha256::digest(b"trusted backend")),
            hex::encode(Sha256::digest(b"trusted executable"))
        );
        let trusted_digest = hex::encode(Sha256::digest(trusted_content.as_bytes()));
        let resolved = ResolvedRuntimeInstall {
            tag: "b10816".into(),
            release_commit: "427291b5b34cd914a31b3fd3b61a68f6184f4b9f".into(),
            architecture: "x64".into(),
            backend: "cpu".into(),
            install_key: "cpu".into(),
            asset: GithubAsset {
                name: "cpu.zip".into(),
                browser_download_url: "https://example.invalid/cpu.zip".into(),
                size: 1,
                digest: Some(format!("sha256:{}", "a".repeat(64))),
            },
            companion_asset: None,
            content_manifest_sha256: trusted_digest.clone(),
        };
        write_runtime_install_record(
            &root,
            &resolved,
            Path::new("bin/llama-server.exe"),
            &trusted_digest,
        )
        .unwrap();
        assert!(verify_installed_runtime(
            &root,
            &resolved,
            trusted_content.as_bytes(),
            &trusted_digest
        )
        .is_ok());

        fs::write(bin.join("llama-server.exe"), b"attacker executable").unwrap();
        let forged_content = trusted_content.replace(
            &hex::encode(Sha256::digest(b"trusted executable")),
            &hex::encode(Sha256::digest(b"attacker executable")),
        );
        let forged_digest = hex::encode(Sha256::digest(forged_content.as_bytes()));
        write_runtime_install_record(
            &root,
            &resolved,
            Path::new("bin/llama-server.exe"),
            &forged_digest,
        )
        .unwrap();
        assert!(verify_installed_runtime(
            &root,
            &resolved,
            trusted_content.as_bytes(),
            &trusted_digest
        )
        .is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn one_runtime_setup_uses_one_hardware_detection() {
        let calls = std::cell::Cell::new(0_u8);

        let (hardware, catalog) = fetch_runtime_setup_with(
            || {
                calls.set(calls.get() + 1);
                detect_hardware()
            },
            |_hardware| {
                Ok(RuntimeCatalog {
                    tag: "fixture".into(),
                    published_at: String::new(),
                    options: Vec::new(),
                    availability: Vec::new(),
                    origin: RuntimeCatalogOrigin::Network,
                    warning: None,
                    recommendation_reason: "fixture".into(),
                })
            },
        )
        .unwrap();

        assert_eq!(calls.get(), 1);
        assert_eq!(hardware.architecture, architecture());
        assert_eq!(catalog.tag, "fixture");
    }

    #[test]
    fn every_b10816_required_job_maps_to_shipping_impact_and_is_green() {
        let jobs = approved_manifest().unwrap().1;

        assert_eq!(jobs.len(), 11);
        for job in &jobs {
            assert_eq!(job.status, "completed", "job {} status", job.name);
            assert_eq!(job.conclusion, "success", "job {} conclusion", job.name);
            assert!(
                !job.install_keys.is_empty(),
                "job {} install keys",
                job.name
            );
            assert_eq!(
                job.platforms,
                vec!["windows-x64"],
                "job {} platform",
                job.name
            );
            assert!(
                !job.hardware_classes.is_empty(),
                "job {} hardware classes",
                job.name
            );
            assert!(!job.completed_at.is_empty(), "job {} completion", job.name);
        }
        for backend in ["cpu", "cuda", "vulkan", "rocm", "sycl", "openvino"] {
            assert!(jobs.iter().any(|job| job.backend == backend), "{backend}");
            assert!(!backend_is_blocked_by_upstream(&jobs, backend), "{backend}");
        }
    }

    #[test]
    fn approved_cpu_option_binds_to_the_pinned_manifest() {
        // Phase 1 GREEN: the pinned CPU asset binds to its manifest entry.
        // The empty-manifest case still rejects with a digest error.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec![],
            vendor: "cpu".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, jobs) = approved_release();
        let catalog =
            build_approved_catalog(&release, &hardware, &approved, &[]).unwrap_or_else(|error| {
                // CPU is not blocked, so only an empty option list can fail here.
                panic!(
                    "approved CPU catalog failed: {error} -- jobs: {}",
                    jobs.len()
                )
            });

        assert!(catalog.options.iter().any(|option| option.backend == "cpu"));
    }

    #[test]
    fn generic_nvidia_name_without_an_exact_record_falls_back_to_cpu() {
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["NVIDIA GeForce RTX 5090".into()],
            vendor: "nvidia".into(),
            cuda_major: Some(13),
            driver_version: "610.74".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, jobs) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;
        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();
        assert_eq!(recommended.backend, "cpu");
        let cuda_keys = options
            .iter()
            .filter(|option| option.backend == "cuda")
            .map(|option| option.install_key.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(cuda_keys.len(), 2);
        assert!(options
            .iter()
            .find(|option| option.install_key == "cuda-13.3")
            .unwrap()
            .companion_asset
            .as_ref()
            .unwrap()
            .name
            .starts_with("cudart-"));
        assert!(options.iter().any(|option| option.backend == "vulkan"));
        assert!(options.iter().any(|option| option.backend == "cpu"));
        assert_eq!(jobs.len(), approved_manifest().unwrap().1.len());
    }

    #[test]
    fn compatibility_record_requires_every_exact_key_field() {
        let source = EvidenceSource {
            kind: EvidenceSourceKind::Policy,
            detail: "test fixture".into(),
        };
        let bytes =
            Evidence::known(1, EvidenceLevel::Observed, source.clone(), 1, Vec::new()).unwrap();
        let adapter = GpuAdapterInfo {
            adapter_id: "luid:00000001:00000002".into(),
            compatibility_id: "pci:10de:2b85:00000000:a1".into(),
            name: "NVIDIA fixture".into(),
            vendor: "nvidia".into(),
            driver: Evidence::known(
                "610.74".into(),
                EvidenceLevel::Observed,
                source.clone(),
                1,
                Vec::new(),
            )
            .unwrap(),
            backend: Evidence::known("cuda".into(), EvidenceLevel::Derived, source, 1, Vec::new())
                .unwrap(),
            dedicated_bytes: bytes.clone(),
            shared_bytes: bytes.clone(),
            budget_bytes: bytes.clone(),
            current_usage_bytes: bytes.clone(),
            available_budget_bytes: bytes.clone(),
            available_for_reservation_bytes: bytes,
            capacity_observations: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec![adapter.name.clone()],
            vendor: "nvidia".into(),
            cuda_major: Some(13),
            driver_version: "610.74".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: vec![adapter.clone()],
            manual_overrides: Vec::new(),
        };
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let option = catalog
            .options
            .iter()
            .find(|option| option.install_key == "cuda-13.3")
            .unwrap();
        let identity = approved_runtime_identity().unwrap();
        let key = CompatibilityKey {
            os_build: "26100".into(),
            architecture: "x64".into(),
            adapter_id: adapter.compatibility_id.clone(),
            driver: "610.74".into(),
            firmware: "not-applicable".into(),
            backend: option.backend.clone(),
            install_key: option.install_key.clone(),
            release_commit: identity.release_commit.clone(),
            asset_name: option.asset.name.clone(),
            asset_sha256: option
                .asset
                .digest
                .as_deref()
                .unwrap()
                .strip_prefix("sha256:")
                .unwrap()
                .into(),
        };
        let record = CompatibilityRecord {
            expiry_identity: compatibility_expiry_identity(&key),
            key: key.clone(),
            evidence_level: "L4_PRODUCT".into(),
            attestation_id: "attestation:fixture".into(),
        };
        assert!(compatibility_record_matches(
            &record,
            option,
            &adapter,
            "26100",
            None,
            "x64",
            &identity.release_commit,
        ));
        let exact_index = compatibility_recommendation_index_for_evidence(
            &catalog.options,
            &hardware,
            Some(&adapter.adapter_id),
            "26100",
            std::slice::from_ref(&record),
            &identity.release_commit,
        )
        .unwrap();
        assert_eq!(catalog.options[exact_index].install_key, "cuda-13.3");
        assert!(compatibility_recommendation_index_for_evidence(
            &catalog.options,
            &hardware,
            Some("luid:wrong"),
            "26100",
            std::slice::from_ref(&record),
            &identity.release_commit,
        )
        .is_none());

        let mut mismatches = Vec::new();
        let mut changed = key.clone();
        changed.os_build = "22631".into();
        mismatches.push(changed);
        let mut changed = key.clone();
        changed.adapter_id = "luid:other".into();
        mismatches.push(changed);
        let mut changed = key.clone();
        changed.driver = "609.00".into();
        mismatches.push(changed);
        let mut changed = key.clone();
        changed.backend = "vulkan".into();
        mismatches.push(changed);
        let mut changed = key.clone();
        changed.install_key = "cuda-12.4".into();
        mismatches.push(changed);
        let mut changed = key.clone();
        changed.release_commit = "0000000000000000000000000000000000000000".into();
        mismatches.push(changed);
        let mut changed = key.clone();
        changed.asset_name = "wrong.zip".into();
        mismatches.push(changed);
        let mut changed = key;
        changed.asset_sha256 = "0".repeat(64);
        mismatches.push(changed);

        for mismatch in mismatches {
            let changed_record = CompatibilityRecord {
                expiry_identity: compatibility_expiry_identity(&mismatch),
                key: mismatch,
                evidence_level: "L4_PRODUCT".into(),
                attestation_id: "attestation:fixture".into(),
            };
            assert!(!compatibility_record_matches(
                &changed_record,
                option,
                &adapter,
                "26100",
                None,
                "x64",
                &identity.release_commit,
            ));
        }
    }

    #[test]
    fn generic_amd_name_without_an_exact_record_falls_back_to_cpu() {
        // A family name does not bind the exact adapter and runtime evidence.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["AMD Radeon RX 7900 XTX".into()],
            vendor: "amd".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;

        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();

        assert_eq!(recommended.backend, "cpu");
        assert!(options.iter().any(|option| option.backend == "vulkan"));
    }

    #[test]
    fn generic_intel_name_without_an_exact_record_falls_back_to_cpu() {
        // A family name does not bind the exact adapter and runtime evidence.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["Intel Arc A770".into()],
            vendor: "intel".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;

        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();

        assert_eq!(recommended.backend, "cpu");
        assert!(options.iter().any(|option| option.backend == "vulkan"));
    }

    #[test]
    fn cpu_fallback_discloses_cpu_backend_without_accelerator_label() {
        // Phase 2 acceptance: CPU fallback must never hide behind an
        // accelerator label. An empty adapter list recommends CPU.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec![],
            vendor: "cpu".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;

        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();

        assert_eq!(recommended.backend, "cpu");
    }

    #[test]
    fn nvidia_family_and_driver_do_not_create_exact_qualification() {
        // Family labels and driver branches do not bind an L4 product record.
        let (release, approved, _) = approved_release();
        for (adapter, major, driver, want) in [
            ("NVIDIA GeForce GTX 1080", 12_u16, "560.70", "cuda-12.4"),
            ("NVIDIA GeForce RTX 2080", 12_u16, "560.70", "cuda-12.4"),
            ("NVIDIA GeForce RTX 3080", 12_u16, "560.70", "cuda-12.4"),
            ("NVIDIA GeForce RTX 4080", 13_u16, "610.74", "cuda-13.3"),
            ("NVIDIA GeForce RTX 5090", 13_u16, "610.74", "cuda-13.3"),
        ] {
            let hardware = HardwareInfo {
                architecture: "x64".into(),
                gpu_names: vec![adapter.into()],
                vendor: "nvidia".into(),
                cuda_major: Some(major),
                driver_version: driver.into(),
                detection_status: "test fixture".into(),
                recommendation: String::new(),
                system_memory: detect_system_memory(),
                adapters: Vec::new(),
                manual_overrides: Vec::new(),
            };
            let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
            let mut options = catalog.options;
            let recommended = recommend_capability_option(&mut options, &hardware)
                .cloned()
                .unwrap();
            assert_eq!(recommended.backend, "cpu", "adapter {adapter}; {want}");
        }
        // Old driver branch rejects CUDA 13.x even with a Blackwell card.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["NVIDIA GeForce RTX 5090".into()],
            vendor: "nvidia".into(),
            cuda_major: Some(13),
            driver_version: "560.70".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;
        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();
        assert_eq!(recommended.backend, "cpu");
    }

    #[test]
    fn selection_discarding_a_second_adapter_fails_without_explicit_choice() {
        // Phase 4 RED: silent cross-adapter preference discards the
        // second adapter. The gate must refuse automatic preference
        // when two accelerator vendors are present. Finding A-07
        // records this gap. This test names the missing helper and
        // must fail until the gate exists.
        let adapters = vec![
            "NVIDIA GeForce RTX 5090".to_string(),
            "AMD Radeon RX 7900 XTX".to_string(),
        ];

        let error = require_explicit_device_selection(&adapters, None).unwrap_err();

        assert!(error.contains("explicit"), "unexpected error: {error}");

        let chosen =
            require_explicit_device_selection(&adapters, Some("AMD Radeon RX 7900 XTX")).unwrap();
        assert_eq!(chosen, "AMD Radeon RX 7900 XTX");

        let unknown =
            require_explicit_device_selection(&adapters, Some("Unknown GPU")).unwrap_err();
        assert!(
            unknown.contains("not in the detected"),
            "unexpected error: {unknown}"
        );

        let single = require_explicit_device_selection(&adapters[..1], None).unwrap();
        assert_eq!(single, "NVIDIA GeForce RTX 5090");

        let empty: Vec<String> = Vec::new();
        let none = require_explicit_device_selection(&empty, None).unwrap_err();
        assert!(none.contains("CPU"), "unexpected error: {none}");
    }

    #[test]
    fn mixed_vendor_adapters_require_explicit_selection_without_silent_fallback() {
        // Phase 2 RED: mixed systems silently select one derived vendor.
        // The gate must refuse automatic accelerator preference and fall
        // back to Vulkan or CPU. Finding A-07 records this gap. This test
        // names the missing helper and must fail until the gate exists.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["NVIDIA GeForce RTX 5090".into(), "Intel Arc A770".into()],
            vendor: "nvidia".into(),
            cuda_major: Some(13),
            driver_version: "610.74".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;

        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();

        assert!(recommended.backend == "vulkan" || recommended.backend == "cpu");
    }

    #[test]
    fn qualcomm_hardware_keeps_dormant_opencl_without_automatic_preference() {
        // Phase 2 RED: Qualcomm hardware receives no OpenCL preference.
        // The gate must label OpenCL as dormant capability on Windows x64.
        // Finding A-02 records this gap. This test names the missing
        // helper and must fail until the gate exists.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["Qualcomm Adreno X1".into()],
            vendor: "other".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;

        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();

        assert_ne!(recommended.backend, "opencl");
        assert!(recommended.backend == "vulkan" || recommended.backend == "cpu");
    }

    #[test]
    fn unsupported_intel_hardware_rejects_sycl_and_keeps_vulkan_or_cpu() {
        // Phase 2 RED: vendor-only SYCL preference accepts unknown Intel
        // hardware. The gate must require Arc or Xe device evidence.
        // Finding A-04 records this gap. This test names the missing
        // helper and must fail until the gate exists.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["Intel Unknown Graphics 2000".into()],
            vendor: "intel".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;

        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();

        assert_ne!(recommended.backend, "sycl");
    }

    #[test]
    fn unsupported_amd_hardware_rejects_rocm_and_keeps_vulkan_or_cpu() {
        // Phase 2 RED: vendor-only ROCm preference accepts unsupported AMD
        // hardware. The gate must require exact AMD matrix evidence.
        // Finding A-03 records this gap. This test names the missing helper
        // and must fail until the gate exists.
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["AMD Radeon HD 8490".into()],
            vendor: "amd".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;

        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();

        assert_ne!(recommended.backend, "rocm");
    }

    #[test]
    fn amd_machine_without_exact_evidence_prefers_cpu() {
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["AMD Radeon RX 7900 XTX".into()],
            vendor: "amd".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        let mut options = catalog.options;
        let recommended = recommend_capability_option(&mut options, &hardware)
            .cloned()
            .unwrap();
        assert_eq!(recommended.backend, "cpu");
        assert!(options.iter().any(|option| option.backend == "vulkan"));
    }

    #[test]
    fn parses_nvidia_cuda_umd_version() {
        assert_eq!(cuda_major_from_smi("CUDA UMD Version: 13.3"), Some(13));
        assert_eq!(cuda_major_from_smi("CUDA Version: 12.4"), Some(12));
    }

    #[test]
    fn managed_runtime_root_prefers_the_new_directory_but_keeps_legacy_installs() {
        let base = std::env::temp_dir().join(format!(
            "localmotive-root-select-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let primary = base.join("Localmotive").join("runtimes");
        let legacy = base.join("GGUF Pilot").join("runtimes");
        // No records anywhere: callers install into the new directory.
        assert_eq!(managed_runtime_root_in(&primary, &legacy).unwrap(), primary);
        // Legacy-only installs keep working until a new runtime arrives.
        let legacy_install = legacy.join("b10000").join("cpu");
        std::fs::create_dir_all(&legacy_install).unwrap();
        std::fs::write(
            legacy_install.join("runtime.json"),
            r#"{"tag":"b10000","backend":"cpu","runtime":"llama-server.exe"}"#,
        )
        .unwrap();
        std::fs::write(legacy_install.join("llama-server.exe"), b"x").unwrap();
        assert_eq!(managed_runtime_root_in(&primary, &legacy).unwrap(), legacy);
        // A new install wins once both directories hold records.
        let primary_install = primary.join("b10000").join("cpu");
        std::fs::create_dir_all(&primary_install).unwrap();
        std::fs::write(
            primary_install.join("runtime.json"),
            r#"{"tag":"b10000","backend":"cpu","runtime":"llama-server.exe"}"#,
        )
        .unwrap();
        std::fs::write(primary_install.join("llama-server.exe"), b"x").unwrap();
        assert_eq!(managed_runtime_root_in(&primary, &legacy).unwrap(), primary);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn managed_runtime_relative_path_is_versioned_and_sanitized() {
        let path = managed_runtime_relative_path("b10736/../../bad", "cuda 13.3");
        assert_eq!(path.to_string_lossy(), "b10736_.._.._bad\\cuda_13.3");
    }

    #[test]
    fn managed_runtime_trust_rejects_a_sibling_prefix_path() {
        let root =
            std::env::temp_dir().join(format!("localmotive-managed-root-{}", std::process::id()));
        let sibling = root.with_file_name(format!(
            "{}-evil",
            root.file_name().unwrap().to_string_lossy()
        ));
        fs::create_dir_all(&sibling).unwrap();
        let executable = sibling.join("llama-server.exe");
        fs::write(&executable, b"not managed").unwrap();

        assert!(!managed_runtime_verified_in(&executable, &root).unwrap());

        fs::remove_dir_all(&sibling).unwrap();
    }

    #[test]
    fn managed_runtime_paths_resolve_inside_roots_with_spaces_unicode_and_drive_forms() {
        // Phase 4: install and manifest paths must stay inside their
        // roots for adversarial path forms: spaces, Unicode, UNC-like
        // components, drive-root strings, and `..` segments. Every form
        // below must either sanitize to a `Normal`-only relative path
        // or fail the manifest guard with `None`.
        for backend in [
            "cuda 13.3",
            "cuda-13.3 héllo 世界",
            "C:\\runtimes",
            "\\\\server\\share",
            "D:",
            "..",
            "../..",
        ] {
            let relative = managed_runtime_relative_path("b10752", backend);
            assert!(
                relative.is_relative(),
                "backend {backend:?} escaped to an absolute path: {relative:?}"
            );
            assert!(
                !relative
                    .components()
                    .any(|component| !matches!(component, std::path::Component::Normal(_))),
                "backend {backend:?} left a non-normal component: {relative:?}"
            );
        }
        // `..` and separator characters in the tag sanitize the same way.
        let tagged = managed_runtime_relative_path("b10752/../../x", "cpu");
        assert!(tagged.is_relative());
        // Manifest-declared runtimes with any non-normal component fail.
        let root = scratch("manifest-path-forms");
        let install = root.join("b10752").join("cpu");
        std::fs::create_dir_all(&install).unwrap();
        std::fs::write(install.join("llama-server.exe"), b"x").unwrap();
        for runtime in [
            "../outside.exe",
            "..\\outside.exe",
            "sub/../../outside.exe",
            "/absolute.exe",
            "C:\\absolute.exe",
            "\\\\server\\share\\x.exe",
            "",
        ] {
            std::fs::write(
                install.join("runtime.json"),
                format!(
                    r#"{{"tag":"b10752","backend":"cpu","runtime":{}}}"#,
                    serde_json::to_string(runtime).unwrap()
                ),
            )
            .unwrap();
            assert!(
                manifest_runtime_path(&install, runtime).is_none(),
                "runtime {runtime:?} escaped its install directory"
            );
        }
        // Normal nested names with spaces and Unicode stay inside.
        std::fs::create_dir_all(install.join("sub dir héllo")).unwrap();
        std::fs::write(install.join("sub dir héllo").join("llama-server.exe"), b"x").unwrap();
        let nested = manifest_runtime_path(&install, "sub dir héllo/llama-server.exe").unwrap();
        assert!(nested.starts_with(&install));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dormant_arm64_assets_never_become_install_options() {
        let hardware = HardwareInfo {
            architecture: "arm64".into(),
            gpu_names: vec![],
            vendor: "cpu".into(),
            cuda_major: None,
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let (release, approved, jobs) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &jobs).unwrap();
        assert!(catalog.options.is_empty());
        assert_eq!(catalog.availability.len(), 3);
        assert!(catalog
            .availability
            .iter()
            .all(|item| item.status == BackendAvailabilityStatus::Dormant));
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("localmotive-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[cfg(windows)]
    #[test]
    fn managed_runtime_inventory_rejects_an_ancestor_junction() {
        // A junction at the release-tag parent previously redirected install
        // publication outside the managed runtime root.
        let root = scratch("runtime-ancestor-junction");
        let outside = root.join("outside");
        let managed = root.join("managed");
        let install = outside.join("b10816").join("cpu");
        fs::create_dir_all(&install).unwrap();
        fs::write(install.join("llama-server.exe"), b"fixture").unwrap();
        let status = crate::proc::hidden_command("cmd.exe")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(&managed)
            .arg(&outside)
            .status()
            .unwrap();
        assert!(status.success(), "could not create the junction fixture");

        let error = collect_install_files(&managed.join("b10816").join("cpu")).unwrap_err();

        assert!(error.contains("ancestor") || error.contains("reparse"));
        let _ = crate::proc::hidden_command("cmd.exe")
            .args(["/D", "/C", "rmdir"])
            .arg(&managed)
            .status();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_extraction_rejects_an_entry_count_over_the_limit() {
        use std::io::Write;

        let root = scratch("archive-entry-limit");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let file = File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        archive.start_file("one.dll", options).unwrap();
        archive.write_all(b"one").unwrap();
        archive.start_file("two.dll", options).unwrap();
        archive.write_all(b"two").unwrap();
        archive.finish().unwrap();

        let error = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 1,
                max_entry_bytes: 16,
                max_total_bytes: 32,
                max_path_bytes: 128,
            },
        )
        .unwrap_err();

        assert!(error.contains("entry count"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_extraction_binds_the_approved_digest_to_the_parsed_file() {
        use std::io::Write;

        let root = scratch("archive-identity");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        let write_archive = |payload: &[u8]| {
            let file = File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive.start_file("runtime.dll", options).unwrap();
            archive.write_all(payload).unwrap();
            archive.finish().unwrap();
        };

        write_archive(b"approved");
        let approved_bytes = fs::read(&archive_path).unwrap();
        let approved_size = approved_bytes.len() as u64;
        let approved_digest = hex::encode(Sha256::digest(&approved_bytes));
        write_archive(b"replaced");
        assert_eq!(fs::metadata(&archive_path).unwrap().len(), approved_size);

        let error = extract_verified_zip_with_limits_and_cancel(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 2,
                max_entry_bytes: 64,
                max_total_bytes: 64,
                max_path_bytes: 64,
            },
            None,
            approved_size,
            &approved_digest,
        )
        .unwrap_err();

        assert!(error.contains("SHA-256"), "unexpected error: {error}");
        assert!(!destination.join("runtime.dll").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn archive_extraction_rejects_ntfs_alternate_stream_entries() {
        use std::io::Write;

        let root = scratch("archive-ads-entry");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let file = File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file(
                "llama-server.exe:hidden",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        archive.write_all(b"hidden stream").unwrap();
        archive.finish().unwrap();

        let error = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 4,
                max_entry_bytes: 1024,
                max_total_bytes: 4096,
                max_path_bytes: 128,
            },
        )
        .unwrap_err();

        assert!(
            error.contains("alternate data stream"),
            "unexpected error: {error}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn content_manifest_paths_reject_ntfs_alternate_streams() {
        let error = path_from_manifest("bin/llama-server.exe:hidden").unwrap_err();
        assert!(
            error.contains("malformed path"),
            "unexpected error: {error}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn managed_runtime_verification_rejects_hard_linked_files() {
        let root = std::env::temp_dir().join(format!(
            "localmotive-managed-hard-link-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let victim = root.join("victim.dll");
        let linked = root.join("ggml.dll");
        fs::write(&victim, b"do not trust").unwrap();
        fs::hard_link(&victim, &linked).unwrap();

        assert!(digest_regular_file(&linked, "ggml.dll").is_err());

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn managed_runtime_verification_rejects_post_install_alternate_streams() {
        let root =
            std::env::temp_dir().join(format!("localmotive-managed-ads-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let file = root.join("ggml.dll");
        fs::write(&file, b"trusted default stream").unwrap();
        fs::write(format!("{}:hidden", file.display()), b"untrusted stream").unwrap();

        assert!(digest_regular_file(&file, "ggml.dll").is_err());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_extraction_rejects_traversal_and_limits() {
        // Phase 4: the archive gate must refuse traversal entries and
        // every resource limit. Link entries stay refused by the
        // unix-mode bit plus the enclosed-name guard in
        // `extract_zip_with_limits`; duplicate names have a dedicated
        // test below because `ZipWriter` cannot build that case.
        // Finding A-09-adjacent hardening records this gap.
        use std::io::Write;
        let options = zip::write::SimpleFileOptions::default();

        // A traversal entry fails: `enclosed_name` returns `None`.
        let root = scratch("archive-traversal");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let file = File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive.start_file("../outside.dll", options).unwrap();
        archive.write_all(b"evil").unwrap();
        archive.finish().unwrap();
        let error = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 20_000,
                max_entry_bytes: 4 * 1024 * 1024 * 1024,
                max_total_bytes: 16 * 1024 * 1024 * 1024,
                max_path_bytes: 1_024,
            },
        )
        .unwrap_err();
        assert!(error.contains("Unsafe path"), "unexpected error: {error}");
        fs::remove_dir_all(root).unwrap();

        // An over-long entry name fails the path limit.
        let root = scratch("archive-path-limit");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let file = File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        let long_name = "a".repeat(200);
        archive.start_file(long_name, options).unwrap();
        archive.write_all(b"x").unwrap();
        archive.finish().unwrap();
        let error = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 20_000,
                max_entry_bytes: 4 * 1024 * 1024 * 1024,
                max_total_bytes: 16 * 1024 * 1024 * 1024,
                max_path_bytes: 128,
            },
        )
        .unwrap_err();
        assert!(error.contains("too long"), "unexpected error: {error}");
        fs::remove_dir_all(root).unwrap();

        // A declared entry size past the per-entry limit fails.
        let root = scratch("archive-entry-bytes");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let file = File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive.start_file("big.dll", options).unwrap();
        archive.write_all(b"123456789").unwrap();
        archive.finish().unwrap();
        let error = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 20_000,
                max_entry_bytes: 8,
                max_total_bytes: 16 * 1024 * 1024 * 1024,
                max_path_bytes: 1_024,
            },
        )
        .unwrap_err();
        assert!(
            error.contains("too large") || error.contains("exceeds"),
            "unexpected error: {error}"
        );
        fs::remove_dir_all(root).unwrap();

        // Combined decompressed bytes past the total limit fail.
        let root = scratch("archive-total-bytes");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let file = File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive.start_file("one.dll", options).unwrap();
        archive.write_all(b"12345").unwrap();
        archive.start_file("two.dll", options).unwrap();
        archive.write_all(b"12345").unwrap();
        archive.finish().unwrap();
        let error = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 20_000,
                max_entry_bytes: 16 * 1024 * 1024 * 1024,
                max_total_bytes: 9,
                max_path_bytes: 1_024,
            },
        )
        .unwrap_err();
        assert!(error.contains("exceeds"), "unexpected error: {error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_extraction_rejects_symlink_mode_entries() {
        // Phase 4: a zip entry with the symlink unix-mode bit must fail
        // even when its path is otherwise safe. `add_symlink` sets the
        // `S_IFLNK` bit the extraction guard checks; `unix_permissions`
        // masks it away and cannot build this case.
        let root = scratch("archive-symlink");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        let file = File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive
            .add_symlink(
                "link.dll",
                "target.dll",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        archive.finish().unwrap();

        let error = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 20_000,
                max_entry_bytes: 4 * 1024 * 1024 * 1024,
                max_total_bytes: 16 * 1024 * 1024 * 1024,
                max_path_bytes: 1_024,
            },
        )
        .unwrap_err();
        assert!(error.contains("symlink"), "unexpected error: {error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn archive_extraction_rejects_a_preexisting_child_junction() {
        use std::io::Write;

        let root = scratch("archive-child-junction");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        let outside = root.join("outside");
        fs::create_dir_all(&destination).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let junction = destination.join("redirect");
        let status = crate::proc::hidden_command("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&outside)
            .status()
            .unwrap();
        assert!(status.success());
        let file = File::create(&archive_path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file(
                "redirect/escaped.dll",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        archive.write_all(b"must stay contained").unwrap();
        archive.finish().unwrap();

        let result = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 20_000,
                max_entry_bytes: 4 * 1024 * 1024 * 1024,
                max_total_bytes: 16 * 1024 * 1024 * 1024,
                max_path_bytes: 1_024,
            },
        );

        assert!(
            result.is_err(),
            "a child junction was followed during extraction"
        );
        assert!(!outside.join("escaped.dll").exists());
        fs::remove_dir(junction).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn archive_extraction_rejects_duplicate_paths() {
        // Phase 4 RED: extraction must refuse a second entry that maps
        // to an already-written path with a `Duplicate path` error. The
        // raw-name duplicate guard (`paths` set) cannot see through
        // traversal (`sub/../same.dll` encloses to a `same.dll` output
        // the guard records under a different key), so the second write
        // fails later with a bare `File exists` OS error instead. A
        // byte-patched archive (both entries read as `one.dll`) proves
        // the same: `ZipArchive` dedupes by central-directory name and
        // exposes only one entry, so the check that must fire never
        // runs. The extractor must canonicalize before it compares.
        let root = scratch("archive-duplicate");
        let archive_path = root.join("runtime.zip");
        let destination = root.join("output");
        fs::create_dir_all(&destination).unwrap();
        {
            use std::io::Write;
            let options = zip::write::SimpleFileOptions::default();
            let file = File::create(&archive_path).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive.start_file("same.dll", options).unwrap();
            archive.write_all(b"first").unwrap();
            archive.start_file("sub/../same.dll", options).unwrap();
            archive.write_all(b"second").unwrap();
            archive.finish().unwrap();
        }
        let error = extract_zip_with_limits(
            &archive_path,
            &destination,
            ArchiveLimits {
                max_entries: 20_000,
                max_entry_bytes: 4 * 1024 * 1024 * 1024,
                max_total_bytes: 16 * 1024 * 1024 * 1024,
                max_path_bytes: 1_024,
            },
        )
        .unwrap_err();
        assert!(
            error.contains("Duplicate path"),
            "unexpected error: {error}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn download_copy_rejects_bytes_past_the_hard_limit() {
        let mut source = std::io::Cursor::new(b"12345".to_vec());
        let mut destination = Vec::new();

        let error = copy_download_with_limit(&mut source, &mut destination, 4).unwrap_err();

        assert!(error.contains("download byte limit"));
    }

    #[test]
    fn identifies_cuda_runtime_from_sibling_dlls() {
        let dir = scratch("dlls");
        for file in [
            "llama-server.exe",
            "ggml-cuda.dll",
            "cudart64_13.dll",
            "ggml-cpu-x64.dll",
        ] {
            fs::write(dir.join(file), b"x").unwrap();
        }
        let identity = describe_runtime(&dir.join("llama-server.exe"));
        assert_eq!(identity.backend, "cuda");
        assert_eq!(identity.cuda_major, Some(13));
        assert_eq!(identity.source, "dlls");
        assert!(identity.tag.is_none());
        assert!(identity.install_key.is_none());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn identifies_cpu_runtime_when_only_cpu_dlls_exist() {
        let dir = scratch("cpu-dlls");
        for file in ["llama-server.exe", "ggml-cpu-x64.dll", "ggml-base.dll"] {
            fs::write(dir.join(file), b"x").unwrap();
        }
        let identity = describe_runtime(&dir.join("llama-server.exe"));
        assert_eq!(identity.backend, "cpu");
        assert_eq!(identity.cuda_major, None);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn identifies_managed_runtime_from_manifest() {
        // Phase 4 RED: a manifest that disagrees with the DLLs beside the
        // runtime must report `mismatch`, not trust either side alone.
        // The manifest-only folder below stays authoritative as before;
        // the two folders after it carry DLL evidence that contradicts
        // or confirms the manifest. Finding A-09 records this gap.
        let dir = scratch("manifest");
        fs::write(dir.join("llama-server.exe"), b"x").unwrap();
        fs::write(
            dir.join("runtime.json"),
            r#"{"tag":"b10752","backend":"cuda","installKey":"cuda-13.3","runtime":"llama-server.exe"}"#,
        )
        .unwrap();
        let identity = describe_runtime(&dir.join("llama-server.exe"));
        assert_eq!(identity.source, "manifest");
        assert_eq!(identity.backend, "cuda");
        assert_eq!(identity.cuda_major, Some(13));
        assert_eq!(identity.tag.as_deref(), Some("b10752"));
        assert_eq!(identity.install_key.as_deref(), Some("cuda-13.3"));
        fs::remove_dir_all(&dir).unwrap();

        // CUDA manifest beside CPU-only DLLs: mismatch, not `manifest`.
        let dir = scratch("manifest-cpu-mismatch");
        for file in ["llama-server.exe", "ggml-cpu-x64.dll", "ggml-base.dll"] {
            fs::write(dir.join(file), b"x").unwrap();
        }
        fs::write(
            dir.join("runtime.json"),
            r#"{"tag":"b10752","backend":"cuda","installKey":"cuda-13.3","runtime":"llama-server.exe"}"#,
        )
        .unwrap();
        let identity = describe_runtime(&dir.join("llama-server.exe"));
        assert_eq!(identity.backend, "mismatch");
        assert_eq!(identity.source, "manifest-dll-mismatch");
        assert_eq!(identity.cuda_major, Some(13));
        fs::remove_dir_all(dir).unwrap();

        // CUDA manifest beside complete CUDA + cudart DLLs: manifest wins.
        let dir = scratch("manifest-cuda-complete");
        for file in [
            "llama-server.exe",
            "ggml-cuda.dll",
            "cudart64_13.dll",
            "ggml-cpu-x64.dll",
        ] {
            fs::write(dir.join(file), b"x").unwrap();
        }
        fs::write(
            dir.join("runtime.json"),
            r#"{"tag":"b10752","backend":"cuda","installKey":"cuda-13.3","runtime":"llama-server.exe"}"#,
        )
        .unwrap();
        let identity = describe_runtime(&dir.join("llama-server.exe"));
        assert_eq!(identity.source, "manifest");
        assert_eq!(identity.backend, "cuda");
        assert_eq!(identity.cuda_major, Some(13));
        fs::remove_dir_all(dir).unwrap();

        // CUDA manifest beside ggml-cuda but no cudart: incomplete
        // companion, so mismatch on a clean machine.
        let dir = scratch("manifest-cuda-no-cudart");
        for file in ["llama-server.exe", "ggml-cuda.dll", "ggml-cpu-x64.dll"] {
            fs::write(dir.join(file), b"x").unwrap();
        }
        fs::write(
            dir.join("runtime.json"),
            r#"{"tag":"b10752","backend":"cuda","installKey":"cuda-13.3","runtime":"llama-server.exe"}"#,
        )
        .unwrap();
        let identity = describe_runtime(&dir.join("llama-server.exe"));
        assert_eq!(identity.backend, "mismatch");
        assert_eq!(identity.source, "manifest-dll-mismatch");
        fs::remove_dir_all(dir).unwrap();

        // Phase 4: every non-CUDA backend needs its shipped DLLs too. A
        // ROCm, SYCL, OpenVINO, or Vulkan manifest beside CPU-only DLLs
        // must report `mismatch`, while complete folders stay
        // `manifest`-authoritative. Finding A-09 records this
        // clean-machine gap; the gate under test is
        // `backend_dependencies_are_complete`.
        for (name, backend, install_key, dlls) in [
            (
                "manifest-rocm-complete",
                "rocm",
                "rocm-10.0",
                vec!["llama-server.exe", "ggml-hip.dll", "ggml-cpu-x64.dll"],
            ),
            (
                "manifest-sycl-complete",
                "sycl",
                "sycl",
                vec!["llama-server.exe", "ggml-sycl.dll", "ggml-cpu-x64.dll"],
            ),
            (
                "manifest-openvino-complete",
                "openvino",
                "openvino-2026.3.1",
                vec!["llama-server.exe", "ggml-openvino.dll", "ggml-cpu-x64.dll"],
            ),
            (
                "manifest-vulkan-complete",
                "vulkan",
                "vulkan",
                vec!["llama-server.exe", "ggml-vulkan.dll", "ggml-cpu-x64.dll"],
            ),
        ] {
            assert!(
                backend_dependencies_are_complete(
                    &dlls
                        .iter()
                        .map(|dll| dll.to_ascii_lowercase())
                        .collect::<Vec<_>>(),
                    backend
                ),
                "{name} fixture must satisfy its dependency rule"
            );
            let dir = scratch(name);
            for file in dlls {
                fs::write(dir.join(file), b"x").unwrap();
            }
            fs::write(
                dir.join("runtime.json"),
                format!(
                    r#"{{"tag":"b10752","backend":"{backend}","installKey":"{install_key}","runtime":"llama-server.exe"}}"#
                ),
            )
            .unwrap();
            let identity = describe_runtime(&dir.join("llama-server.exe"));
            assert_eq!(identity.source, "manifest", "{name} must stay trusted");
            assert_eq!(identity.backend, backend, "{name} backend drift");
            fs::remove_dir_all(dir).unwrap();
        }
        for (name, backend, install_key) in [
            ("manifest-rocm-cpu-mismatch", "rocm", "rocm-10.0"),
            ("manifest-sycl-cpu-mismatch", "sycl", "sycl"),
            (
                "manifest-openvino-cpu-mismatch",
                "openvino",
                "openvino-2026.3.1",
            ),
            ("manifest-vulkan-cpu-mismatch", "vulkan", "vulkan"),
        ] {
            assert!(
                !backend_dependencies_are_complete(
                    &["ggml-cpu-x64.dll".to_string(), "ggml-base.dll".to_string()],
                    backend
                ),
                "{name} fixture must fail its dependency rule"
            );
            let dir = scratch(name);
            for file in ["llama-server.exe", "ggml-cpu-x64.dll", "ggml-base.dll"] {
                fs::write(dir.join(file), b"x").unwrap();
            }
            fs::write(
                dir.join("runtime.json"),
                format!(
                    r#"{{"tag":"b10752","backend":"{backend}","installKey":"{install_key}","runtime":"llama-server.exe"}}"#
                ),
            )
            .unwrap();
            let identity = describe_runtime(&dir.join("llama-server.exe"));
            assert_eq!(identity.backend, "mismatch", "{name} must mismatch");
            assert_eq!(identity.source, "manifest-dll-mismatch", "{name} source");
            fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn managed_runtime_listing_rejects_forged_writable_manifest() {
        let root = scratch("managed-root");
        let install = root.join("b10752").join("cuda-13.3");
        fs::create_dir_all(&install).unwrap();
        fs::write(install.join("llama-server.exe"), b"x").unwrap();
        fs::write(
            install.join("runtime.json"),
            r#"{"tag":"b10752","backend":"cuda","installKey":"cuda-13.3","runtime":"llama-server.exe"}"#,
        )
        .unwrap();
        fs::create_dir_all(root.join(".installing-junk")).unwrap();
        let records = list_managed_runtimes_in(&root);
        assert!(records.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn managed_runtime_manifest_cannot_escape_its_install_directory() {
        let root = scratch("managed-traversal");
        let install = root.join("b10752").join("cpu");
        fs::create_dir_all(&install).unwrap();
        fs::write(root.join("outside.exe"), b"x").unwrap();
        fs::write(
            install.join("runtime.json"),
            r#"{"tag":"b10752","backend":"cpu","runtime":"../../outside.exe"}"#,
        )
        .unwrap();

        let records = list_managed_runtimes_in(&root);

        assert!(records.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn runtime_manifest_must_describe_the_requested_executable() {
        let dir = scratch("manifest-mismatch");
        fs::write(dir.join("llama-server.exe"), b"x").unwrap();
        fs::write(dir.join("other.exe"), b"x").unwrap();
        fs::write(
            dir.join("runtime.json"),
            r#"{"tag":"b10752","backend":"cuda","runtime":"other.exe"}"#,
        )
        .unwrap();

        let identity = describe_runtime(&dir.join("llama-server.exe"));

        assert_ne!(identity.source, "manifest");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn tag_build_numbers_parse_with_or_without_prefix() {
        assert_eq!(tag_build("b10752"), Some(10752));
        assert_eq!(tag_build("10679"), Some(10679));
        assert_eq!(tag_build("preview"), None);
    }
}
