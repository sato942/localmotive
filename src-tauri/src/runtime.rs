use crate::artifact::is_reparse_point;
use crate::evidence::{Evidence, EvidenceLevel, EvidenceSource, EvidenceSourceKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RELEASES_URL: &str = "https://api.github.com/repos/ggml-org/llama.cpp/releases?per_page=20";

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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCatalog {
    pub tag: String,
    pub published_at: String,
    pub options: Vec<RuntimeOption>,
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
            };
        }
        return RuntimeIdentity {
            path: display,
            cuda_major,
            backend,
            tag: Some(manifest.tag),
            install_key,
            source: "manifest".into(),
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

pub fn detect_hardware() -> HardwareInfo {
    let architecture = architecture();
    let system_memory = detect_system_memory();
    let mut adapters = detect_dxgi_adapters().unwrap_or_default();
    let hardware_observed_at_ms = observed_at_ms();
    let nvidia_query = crate::proc::hidden_command("nvidia-smi.exe")
        .args([
            "--query-gpu=name,driver_version,memory.total,memory.used",
            "--format=csv,noheader,nounits",
        ])
        .output();
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
                let smi = crate::proc::hidden_command("nvidia-smi.exe")
                    .output()
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
                    recommendation: match cuda_major {
                        Some(major) if major >= 13 => "CUDA 13 is the best match for this NVIDIA driver".into(),
                        _ => "CUDA 12 is the compatible NVIDIA choice; Vulkan remains available as a fallback".into(),
                    },
                    system_memory,
                    adapters,
                    manual_overrides: Vec::new(),
                };
            }
        }
    }

    let script = "Get-CimInstance Win32_VideoController | ForEach-Object { \"$($_.Name)`t$($_.DriverVersion)\" }";
    let adapter_rows = crate::proc::hidden_command("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
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
    let recommendation = match vendor {
        "nvidia" => "CUDA is available when driver branch and CUDA major match; Vulkan remains available as a fallback",
        "amd" => "ROCm needs an exact AMD matrix match; otherwise use Vulkan or CPU",
        "intel" => "SYCL needs Arc or Xe device evidence; otherwise use Vulkan or CPU",
        "qualcomm" => "OpenCL is dormant capability on Windows x64; use Vulkan or CPU",
        "cpu" => "No supported GPU runtime was detected; use the CPU build",
        _ => "Use the Vulkan build for broad Windows GPU compatibility",
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
        recommendation: recommendation.into(),
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
                "Fastest path for supported NVIDIA GPUs".into(),
                "Requires an NVIDIA driver compatible with this CUDA major version".into(),
            )
        }
        "rocm" => (
            "AMD ROCm".into(),
            "Native AMD GPU acceleration for supported Radeon hardware".into(),
            "If the GPU is unsupported, install Vulkan instead".into(),
        ),
        "sycl" => (
            "Intel SYCL".into(),
            "Intel Arc/Xe GPU acceleration".into(),
            "Requires current Intel graphics drivers".into(),
        ),
        "openvino" => (
            "Intel OpenVINO".into(),
            "Alternative Intel CPU/GPU inference backend".into(),
            "Useful when SYCL is unavailable or OpenVINO is preferred".into(),
        ),
        "vulkan" => (
            "Vulkan".into(),
            "Broad Windows GPU compatibility across AMD, Intel, and NVIDIA".into(),
            "Usually slower than a vendor-native backend but easier to run".into(),
        ),
        "opencl" => (
            "OpenCL".into(),
            "Specialized OpenCL Windows build".into(),
            "Choose only for matching hardware such as supported Adreno devices".into(),
        ),
        _ => (
            "CPU".into(),
            "Portable CPU-only Windows build".into(),
            "Works without a supported GPU; performance depends on CPU and memory bandwidth".into(),
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
#[serde(rename_all = "camelCase")]
pub struct ApprovedRuntimeAsset {
    pub name: String,
    pub url: String,
    pub bytes: u64,
    pub digest: String,
    pub backend: String,
    pub arch: String,
    #[serde(default)]
    pub cuda_version: Option<String>,
    #[serde(default)]
    pub companion_name: Option<String>,
}

/// One required upstream hardware job that gates a backend.
///
/// A runtime update blocks while any required job for its backend fails or
/// remains queued. Release `b10796` exposes 95 successes, five failures,
/// and one queued check.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RequiredUpstreamJob {
    pub name: String,
    pub backend: String,
    pub conclusion: String,
    pub status: String,
    pub url: String,
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApprovedRuntimeManifest {
    release_tag: String,
    #[allow(dead_code)]
    #[serde(default)]
    schema_version: u32,
    #[allow(dead_code)]
    #[serde(default)]
    release_commit: String,
    #[allow(dead_code)]
    #[serde(default)]
    published_at: String,
    #[serde(default)]
    assets: Vec<ApprovedRuntimeAsset>,
    #[serde(default)]
    required_jobs: Vec<RequiredUpstreamJob>,
    #[allow(dead_code)]
    #[serde(default)]
    approval: Option<ApprovedManifestGate>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApprovedManifestGate {
    #[allow(dead_code)]
    #[serde(default)]
    release_tag: String,
    #[allow(dead_code)]
    #[serde(default)]
    release_commit: String,
    #[allow(dead_code)]
    #[serde(default)]
    blocked_backends: Vec<String>,
    #[allow(dead_code)]
    #[serde(default)]
    note: String,
}

/// Load the pinned approved runtime manifest.
///
/// The manifest is a compile-time file. The build fails when the file is
/// absent, so selection can never run without an approved pin.
pub fn approved_manifest() -> Result<(Vec<ApprovedRuntimeAsset>, Vec<RequiredUpstreamJob>), String>
{
    let manifest: ApprovedRuntimeManifest = serde_json::from_str(APPROVED_RUNTIMES)
        .map_err(|error| format!("Approved runtime manifest is invalid: {error}"))?;
    if manifest.release_tag != "b10796" {
        return Err(format!(
            "Approved runtime manifest pins {}, want b10796",
            manifest.release_tag
        ));
    }
    if manifest.assets.is_empty() {
        return Err("Approved runtime manifest contains no assets".into());
    }
    Ok((manifest.assets, manifest.required_jobs))
}

/// Report whether a required upstream job blocks a backend.
///
/// A job blocks when it names the backend and its conclusion is not
/// `success`, or when its status is not `completed`. A queued or failed
/// job therefore blocks the runtime update for that backend.
pub fn upstream_job_blocks_backend(job: &RequiredUpstreamJob, backend: &str) -> bool {
    if job.backend != backend {
        return false;
    }
    job.conclusion != "success" || job.status != "completed"
}

/// Report whether any required job blocks a backend.
pub fn backend_is_blocked_by_upstream(jobs: &[RequiredUpstreamJob], backend: &str) -> bool {
    jobs.iter()
        .any(|job| upstream_job_blocks_backend(job, backend))
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
    let manifest_tag = approved_manifest_tag(approved).unwrap_or("b10796".to_string());
    if release.tag_name != manifest_tag {
        return Err(format!(
            "Release {} is not the approved runtime {}",
            release.tag_name, manifest_tag
        ));
    }
    let mut options = Vec::new();
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
        if backend_is_blocked_by_upstream(jobs, backend) {
            return Err(format!(
                "Runtime backend {backend} is blocked: a required upstream job failed or remains queued"
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
    if options.is_empty() {
        return Err(format!(
            "Release {} has no approved Windows {} runtime archives",
            release.tag_name, hardware.architecture
        ));
    }
    Ok(RuntimeCatalog {
        tag: release.tag_name.clone(),
        published_at: release.published_at.clone().unwrap_or_default(),
        options,
    })
}

/// Recommend one approved catalog entry from device evidence.
///
/// Vendor detection is a hint only. ROCm requires exact AMD matrix
/// evidence. SYCL requires Intel Arc or Xe device evidence. CUDA requires
/// a compatible driver branch plus a matching CUDA major version. Unknown
/// combinations fall back to Vulkan or CPU. OpenVINO and OpenCL stay
/// catalog entries without automatic preference. Arm64 assets stay
/// dormant on x64 hosts.
pub fn recommend_capability_option<'a>(
    options: &'a mut [RuntimeOption],
    hardware: &HardwareInfo,
) -> Option<&'a RuntimeOption> {
    let recommended_index = if device_adapters(hardware).is_empty() {
        options.iter().position(|option| option.backend == "cpu")
    } else {
        capability_recommendation_index(options, hardware)
            .or_else(|| options.iter().position(|option| option.backend == "vulkan"))
            .or_else(|| options.iter().position(|option| option.backend == "cpu"))
    };
    if let Some(index) = recommended_index {
        options[index].recommended = true;
    }
    options.sort_by_key(|option| (!option.recommended, option.backend.clone()));
    options.iter().find(|option| option.recommended)
}

fn capability_recommendation_index(
    options: &[RuntimeOption],
    hardware: &HardwareInfo,
) -> Option<usize> {
    if hardware.architecture != "x64" {
        return None;
    }
    let adapters = device_adapters(hardware);
    if require_explicit_device_selection(&adapters, None).is_err() {
        return None;
    }
    let adapter = adapters.first()?;
    match adapter_vendor(adapter, hardware) {
        VendorClass::Nvidia => nvidia_cuda_index(options, hardware),
        VendorClass::Amd => amd_backend_index(options, adapter),
        VendorClass::Intel => intel_backend_index(options, adapter),
        VendorClass::Qualcomm => None,
        VendorClass::Other => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VendorClass {
    Nvidia,
    Amd,
    Intel,
    Qualcomm,
    Other,
}

fn adapter_vendor(adapter: &str, hardware: &HardwareInfo) -> VendorClass {
    let combined = format!("{} {}", adapter, hardware.vendor).to_ascii_lowercase();
    if combined.contains("nvidia") || combined.contains("geforce") || combined.contains("quadro") {
        VendorClass::Nvidia
    } else if combined.contains("qualcomm") || combined.contains("adreno") {
        VendorClass::Qualcomm
    } else if combined.contains("amd") || combined.contains("radeon") {
        VendorClass::Amd
    } else if combined.contains("intel")
        || combined.contains("arc")
        || combined.contains("iris")
        || combined.contains("uhd")
        || combined.contains("hd graphics")
    {
        VendorClass::Intel
    } else {
        VendorClass::Other
    }
}

fn device_adapters(hardware: &HardwareInfo) -> Vec<String> {
    if !hardware.adapters.is_empty() {
        return hardware
            .adapters
            .iter()
            .map(|adapter| adapter.name.clone())
            .collect();
    }
    hardware.gpu_names.clone()
}

fn mixed_adapter_vendors(adapters: &[String]) -> bool {
    let mut kinds = std::collections::HashSet::new();
    for adapter in adapters {
        let lower = adapter.to_ascii_lowercase();
        if lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro") {
            kinds.insert("nvidia");
        } else if lower.contains("amd") || lower.contains("radeon") {
            kinds.insert("amd");
        } else if lower.contains("intel")
            || lower.contains("arc")
            || lower.contains("iris")
            || lower.contains("uhd")
        {
            kinds.insert("intel");
        } else if lower.contains("qualcomm") || lower.contains("adreno") {
            kinds.insert("qualcomm");
        } else {
            kinds.insert("other");
        }
    }
    kinds.len() > 1
}

/// Require explicit device selection on multi-adapter systems.
///
/// Returns the selected adapter name when exactly one accelerator
/// vendor is present, or when the caller names one observed adapter
/// explicitly. Refuses automatic preference when two accelerator
/// vendors are present without an explicit choice, so selection never
/// discards a second adapter silently. Finding A-07 records this gap.
pub fn require_explicit_device_selection(
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
    if mixed_adapter_vendors(adapters) {
        return Err(
            "Multiple accelerator vendors were detected; choose one adapter explicitly before installing a runtime"
                .into(),
        );
    }
    Ok(adapters[0].clone())
}

fn nvidia_cuda_index(options: &[RuntimeOption], hardware: &HardwareInfo) -> Option<usize> {
    let major = hardware.cuda_major?;
    options
        .iter()
        .enumerate()
        .filter(|(_, option)| option.backend == "cuda")
        .filter(|(_, option)| {
            manifest_cuda_major(option).is_some_and(|runtime_major| runtime_major <= major)
        })
        .filter(|(_, option)| cuda_driver_branch_supports(hardware, option))
        .max_by_key(|(_, option)| manifest_cuda_major(option).unwrap_or(0))
        .map(|(index, _)| index)
}

fn manifest_cuda_major(option: &RuntimeOption) -> Option<u16> {
    option
        .install_key
        .strip_prefix("cuda-")?
        .split('.')
        .next()?
        .parse()
        .ok()
}

fn cuda_driver_branch_supports(hardware: &HardwareInfo, option: &RuntimeOption) -> bool {
    let Some(runtime_major) = manifest_cuda_major(option) else {
        return false;
    };
    let Some(driver_branch) = driver_branch(&hardware.driver_version) else {
        return false;
    };
    match runtime_major {
        13 => driver_branch >= 580,
        12 => driver_branch >= 525,
        _ => false,
    }
}

fn driver_branch(version: &str) -> Option<u32> {
    version
        .split(|c: char| !c.is_ascii_digit())
        .find(|part| !part.is_empty())?
        .parse()
        .ok()
}

/// Require exact AMD matrix evidence before ROCm preference.
///
/// Supported families come from the AMD ROCm 7.14 compatibility snapshot
/// in `research/0.4/evidence/vendor/amd-rocm-compatibility.md`: RDNA 2
/// (`gfx1030`), RDNA 3 (`gfx1100`/`gfx1101`/`gfx1102`), RDNA 4
/// (`gfx1200`/`gfx1201`), RDNA 3.5 APUs (`gfx1150`-`gfx1153`, `gfx1103`),
/// and listed Instinct cards on Windows 11. Anything else keeps Vulkan
/// or CPU fallback.
fn amd_backend_index(options: &[RuntimeOption], adapter: &str) -> Option<usize> {
    if amd_matrix_supports_rocm(adapter) {
        options.iter().position(|option| option.backend == "rocm")
    } else {
        None
    }
}

fn amd_matrix_supports_rocm(adapter: &str) -> bool {
    let lower = adapter.to_ascii_lowercase();
    if lower.contains("instinct") {
        return lower.contains("mi350")
            || lower.contains("mi300")
            || lower.contains("mi200")
            || lower.contains("mi100");
    }
    if lower.contains("radeon ai pro")
        || lower.contains("radeon pro")
        || lower.contains("radeon rx")
        || lower.contains("ryzen ai")
    {
        return true;
    }
    if lower.contains("radeon 8")
        || lower.contains("radeon 7")
        || lower.contains("radeon 6")
        || lower.contains("rdna")
    {
        return true;
    }
    lower.contains("gfx1030")
        || lower.contains("gfx1100")
        || lower.contains("gfx1101")
        || lower.contains("gfx1102")
        || lower.contains("gfx1150")
        || lower.contains("gfx1151")
        || lower.contains("gfx1152")
        || lower.contains("gfx1153")
        || lower.contains("gfx1103")
        || lower.contains("gfx1200")
        || lower.contains("gfx1201")
}

/// Require Intel Arc or Xe device evidence before SYCL preference.
///
/// Families come from the OpenVINO 2026 system-requirements snapshot in
/// `research/0.4/evidence/vendor/intel-openvino-requirements.md`. Arc,
/// Iris Xe, UHD, HD Graphics, Flex, and Max cards qualify. Unknown Intel
/// names keep Vulkan or CPU fallback.
fn intel_backend_index(options: &[RuntimeOption], adapter: &str) -> Option<usize> {
    if intel_matrix_supports_sycl(adapter) {
        options.iter().position(|option| option.backend == "sycl")
    } else {
        None
    }
}

fn intel_matrix_supports_sycl(adapter: &str) -> bool {
    let lower = adapter.to_ascii_lowercase();
    lower.contains("arc")
        || lower.contains("iris xe")
        || lower.contains("iris")
        || lower.contains("uhd")
        || lower.contains("hd graphics")
        || lower.contains("flex")
        || lower.contains("max")
        || lower.contains("xe")
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

fn github_client() -> Result<reqwest::blocking::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );
    reqwest::blocking::Client::builder()
        .user_agent(format!("Localmotive/{}", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(900))
        .build()
        .map_err(|error| error.to_string())
}

pub fn fetch_catalog(hardware: &HardwareInfo) -> Result<RuntimeCatalog, String> {
    let (approved, jobs) = approved_manifest()?;
    let body = github_client()?
        .get(RELEASES_URL)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("GitHub release lookup failed: {error}"))?
        .text()
        .map_err(|error| format!("GitHub release body could not be read: {error}"))?;
    let releases = serde_json::from_str::<Vec<GithubRelease>>(&body)
        .map_err(|error| format!("GitHub release response was invalid at {error}"))?;
    let manifest_tag = approved_manifest_tag(&approved).unwrap_or("b10796".to_string());
    let release = releases
        .iter()
        .find(|release| release.tag_name == manifest_tag)
        .ok_or_else(|| {
            format!("Approved runtime {manifest_tag} is not in the recent GitHub releases")
        })?;
    let catalog = build_approved_catalog(release, hardware, &approved, &jobs)?;
    if catalog.options.is_empty() {
        return Err(format!(
            "Release {} has no Windows {} runtime archives",
            catalog.tag, hardware.architecture
        ));
    }
    let mut options = catalog.options;
    recommend_capability_option(&mut options, hardware);
    Ok(RuntimeCatalog {
        tag: catalog.tag,
        published_at: catalog.published_at,
        options,
    })
}

fn expected_sha256(asset: &GithubAsset) -> Option<&str> {
    asset.digest.as_deref()?.strip_prefix("sha256:")
}

const MAX_RUNTIME_DOWNLOAD_BYTES: u64 = 8 * 1024 * 1024 * 1024;

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

fn download_asset(
    client: &reqwest::blocking::Client,
    asset: &GithubAsset,
    destination: &Path,
) -> Result<(), String> {
    if asset.size > MAX_RUNTIME_DOWNLOAD_BYTES {
        return Err(format!("Runtime asset is too large: {}", asset.name));
    }
    let mut response = client
        .get(&asset.browser_download_url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("Download failed for {}: {error}", asset.name))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| error.to_string())?;
    let copy_result = copy_download_with_limit(
        &mut response,
        &mut file,
        if asset.size > 0 {
            asset.size
        } else {
            MAX_RUNTIME_DOWNLOAD_BYTES
        },
    );
    let (written, actual_sha256) = match copy_result {
        Ok(result) => result,
        Err(error) => {
            drop(file);
            let _ = fs::remove_file(destination);
            return Err(error);
        }
    };
    file.sync_all().map_err(|error| error.to_string())?;
    if asset.size > 0 && written != asset.size {
        drop(file);
        let _ = fs::remove_file(destination);
        return Err(format!(
            "Download size mismatch for {}: expected {}, received {}",
            asset.name, asset.size, written
        ));
    }
    if let Some(expected) = expected_sha256(asset) {
        if !actual_sha256.eq_ignore_ascii_case(expected) {
            drop(file);
            let _ = fs::remove_file(destination);
            return Err(format!("SHA-256 mismatch for {}", asset.name));
        }
    }
    Ok(())
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

fn extract_zip_with_limits(
    archive_path: &Path,
    destination: &Path,
    limits: ArchiveLimits,
) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
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
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let raw_name = entry.name().to_string();
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
        let output = destination.join(&relative);
        if entry.is_dir() {
            fs::create_dir_all(&output).map_err(|error| error.to_string())?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut target = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)
            .map_err(|error| error.to_string())?;
        let mut entry_bytes = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        let copy_result = (|| -> Result<(), String> {
            loop {
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
            let _ = fs::remove_file(&output);
            return Err(error);
        }
    }
    Ok(())
}

fn extract_zip(archive_path: &Path, destination: &Path) -> Result<(), String> {
    extract_zip_with_limits(archive_path, destination, RUNTIME_ARCHIVE_LIMITS)
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

pub fn install_runtime(tag: &str, option: &RuntimeOption) -> Result<InstalledRuntime, String> {
    let root = managed_runtime_root()?;
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let final_dir = root.join(managed_runtime_relative_path(tag, &option.install_key));
    if let Some(runtime) = find_runtime(&final_dir) {
        return Ok(InstalledRuntime {
            tag: tag.into(),
            backend: option.backend.clone(),
            runtime_path: runtime.to_string_lossy().to_string(),
            install_root: final_dir.to_string_lossy().to_string(),
            reused: true,
        });
    }

    let staging = root.join(format!(
        ".installing-{}-{}-{}",
        sanitize_component(tag),
        sanitize_component(&option.install_key),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    let client = github_client()?;
    let install = (|| {
        for (index, asset) in std::iter::once(&option.asset)
            .chain(option.companion_asset.iter())
            .enumerate()
        {
            let archive = staging.join(format!("download-{index}.zip"));
            download_asset(&client, asset, &archive)?;
            extract_zip(&archive, &staging)?;
            fs::remove_file(archive).map_err(|error| error.to_string())?;
        }
        let runtime = find_runtime(&staging)
            .ok_or_else(|| "Downloaded archive did not contain llama-server.exe".to_string())?;
        let relative_runtime = runtime
            .strip_prefix(&staging)
            .map_err(|error| error.to_string())?
            .to_path_buf();
        let manifest = serde_json::json!({
            "tag": tag,
            "backend": option.backend,
            "installKey": option.install_key,
            "asset": option.asset.name,
            "companionAsset": option.companion_asset.as_ref().map(|asset| &asset.name),
            "runtime": relative_runtime,
        });
        fs::write(
            staging.join("runtime.json"),
            serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        if final_dir.exists() {
            fs::remove_dir_all(&final_dir).map_err(|error| error.to_string())?;
        }
        if let Some(parent) = final_dir.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::rename(&staging, &final_dir).map_err(|error| error.to_string())?;
        let runtime = final_dir.join(relative_runtime);
        Ok(InstalledRuntime {
            tag: tag.into(),
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
        let release = GithubRelease {
            tag_name: "b10796".into(),
            published_at: Some("2026-09-04T05:31:09Z".into()),
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
            tag_name: "b10796".into(),
            published_at: Some("2026-09-04T05:31:09Z".into()),
            assets: vec![
                GithubAsset::sample("llama-b10796-bin-win-cpu-x64.zip"),
                GithubAsset::sample("llama-b10796-bin-win-vulkan-x64.zip"),
                GithubAsset::sample("llama-b10796-bin-win-rocm-10.0-x64.zip"),
                GithubAsset::sample("llama-b10796-bin-win-sycl-x64.zip"),
                GithubAsset::sample("llama-b10796-bin-win-cuda-12.4-x64.zip"),
                GithubAsset::sample("cudart-llama-bin-win-cuda-12.4-x64.zip"),
                GithubAsset::sample("llama-b10796-bin-win-cuda-13.3-x64.zip"),
                GithubAsset::sample("cudart-llama-bin-win-cuda-13.3-x64.zip"),
                GithubAsset::sample("llama-b10796-bin-win-cpu-arm64.zip"),
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
            cuda_version: None,
            companion_name: None,
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
    fn blocked_backend_update_stops_when_a_required_job_fails_or_queues() {
        // Phase 1 RED: a runtime update blocks while a required upstream
        // hardware job fails or remains queued. Release b10796 exposes
        // five failures and one queued check. This test failed before
        // with missing upstream_job_blocks_backend.
        let jobs = approved_manifest().unwrap().1;

        assert!(backend_is_blocked_by_upstream(&jobs, "cuda"));
        assert!(backend_is_blocked_by_upstream(&jobs, "rocm"));
        assert!(backend_is_blocked_by_upstream(&jobs, "openvino"));
    }

    #[test]
    fn approved_manifest_loads_the_pinned_b10796_pin() {
        // The manifest is a compile-time file. This test proves the pin
        // loads, so selection can never run without an approved release.
        let (approved, jobs) = approved_manifest().unwrap();

        assert!(!approved.is_empty());
        assert!(approved.iter().any(|entry| entry.name.contains("cpu-x64")));
        assert!(approved
            .iter()
            .any(|entry| entry.name.contains("cuda-13.3")));
        assert!(jobs.iter().any(|job| job.name == "gpu-rocm"));
        assert!(backend_is_blocked_by_upstream(&jobs, "rocm"));
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
    fn nvidia_cuda_13_machine_gets_matching_recommendation_and_cudart() {
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
        assert_eq!(recommended.backend, "cuda");
        assert!(recommended.asset.name.contains("cuda-13.3"));
        let cuda_keys = options
            .iter()
            .filter(|option| option.backend == "cuda")
            .map(|option| option.install_key.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(cuda_keys.len(), 2);
        assert!(recommended
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
    fn supported_amd_hardware_keeps_rocm_preference() {
        // Supported AMD hardware keeps ROCm preference through the
        // capability gate. RDNA 3 (`RX 7900 XTX`) is in the AMD matrix.
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

        assert_eq!(recommended.backend, "rocm");
        assert!(options.iter().any(|option| option.backend == "vulkan"));
    }

    #[test]
    fn supported_intel_arc_hardware_keeps_sycl_preference() {
        // Supported Intel hardware keeps SYCL preference through the
        // capability gate. Arc A770 is in the OpenVINO requirements list.
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

        assert_eq!(recommended.backend, "sycl");
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
    fn pascal_through_blackwell_cuda_mapping_uses_driver_branch_gates() {
        // Phase 2 RED: CUDA mapping must enforce driver branch 580 or
        // later for CUDA 13.x and branch 525 or later for CUDA 12.x.
        // NVIDIA minor-version snapshot pins these gates. Pascal keeps
        // CUDA 12.4. Turing through Blackwell keep the highest compatible
        // CUDA major. This test names the missing helper and must fail
        // until the gate exists.
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
            assert_eq!(recommended.backend, "cuda", "adapter {adapter}");
            assert_eq!(recommended.install_key, want, "adapter {adapter}");
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
        assert_ne!(recommended.install_key, "cuda-13.3");
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
    fn amd_machine_prefers_rocm_with_vulkan_fallback() {
        // Retired Phase 1 vendor-only check. `supported_amd_hardware_keeps_rocm_preference`
        // covers the capability gate now.
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
        assert_eq!(recommended.backend, "rocm");
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
    fn arm64_machine_only_receives_arm64_assets() {
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
        let (release, approved, _) = approved_release();
        let catalog = build_approved_catalog(&release, &hardware, &approved, &[]).unwrap();
        assert_eq!(catalog.options.len(), 3);
        assert!(catalog
            .options
            .iter()
            .all(|option| option.asset.name.contains("arm64")));
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("localmotive-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
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
    fn lists_managed_runtimes_from_manifests() {
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
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tag, "b10752");
        assert_eq!(records[0].install_key, "cuda-13.3");
        assert_eq!(records[0].backend, "cuda");
        assert!(records[0].runtime_path.ends_with("llama-server.exe"));
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
