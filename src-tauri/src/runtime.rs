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
        return RuntimeIdentity {
            path: display,
            cuda_major: install_key.as_deref().and_then(cuda_major_from_install_key),
            backend: manifest.backend,
            tag: Some(manifest.tag),
            install_key,
            source: "manifest".into(),
        };
    }

    let names = fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let has = |needle: &str| names.iter().any(|name| name.contains(needle));
    let backend = if has("ggml-cuda") {
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
    };
    let cuda_major = names.iter().find_map(|name| {
        name.strip_prefix("cudart64_")?
            .split(['.', '_'])
            .next()?
            .parse()
            .ok()
    });
    RuntimeIdentity {
        path: display,
        backend: backend.into(),
        cuda_major,
        tag: None,
        install_key: None,
        source: if backend == "unknown" { "none" } else { "dlls" }.into(),
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
    let (vendor, recommendation) = if combined.contains("amd") || combined.contains("radeon") {
        (
            "amd",
            "ROCm is recommended for supported AMD Radeon GPUs; Vulkan is the broad fallback",
        )
    } else if combined.contains("intel") {
        (
            "intel",
            "SYCL is recommended for Intel Arc/Xe GPUs; Vulkan and CPU builds remain available",
        )
    } else if gpu_names.is_empty() {
        (
            "cpu",
            "No supported GPU runtime was detected; use the CPU build",
        )
    } else {
        (
            "other",
            "Use the Vulkan build for broad Windows GPU compatibility",
        )
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

fn cuda_version(name: &str) -> u16 {
    name.split("-cuda-")
        .nth(1)
        .and_then(|value| value.split('.').next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

pub fn build_catalog(release: &GithubRelease, hardware: &HardwareInfo) -> RuntimeCatalog {
    let arch_marker = format!("-{}", hardware.architecture);
    let mut options = release
        .assets
        .iter()
        .filter(|asset| {
            let lower = asset.name.to_ascii_lowercase();
            lower.starts_with("llama-")
                && lower.contains("-bin-win-")
                && lower.ends_with(".zip")
                && lower.contains(&arch_marker)
        })
        .filter_map(|asset| {
            let backend = backend_for(&asset.name)?;
            let (label, description, compatibility) = label_and_description(backend, &asset.name);
            let companion_asset = if backend == "cuda" {
                let suffix = asset.name.split("-bin-win-").nth(1)?;
                release
                    .assets
                    .iter()
                    .find(|candidate| {
                        candidate.name.starts_with("cudart-") && candidate.name.ends_with(suffix)
                    })
                    .cloned()
            } else {
                None
            };
            let install_key = if backend == "cuda" {
                let version = asset
                    .name
                    .split("-cuda-")
                    .nth(1)
                    .and_then(|value| {
                        value.strip_suffix(&format!("-{}.zip", hardware.architecture))
                    })
                    .unwrap_or("current");
                format!("cuda-{version}")
            } else {
                backend.to_string()
            };
            Some(RuntimeOption {
                id: format!("{}:{install_key}", release.tag_name),
                label,
                backend: backend.into(),
                install_key,
                description,
                compatibility,
                asset: asset.clone(),
                companion_asset,
                recommended: false,
            })
        })
        .collect::<Vec<_>>();

    let recommended_index = match hardware.vendor.as_str() {
        "nvidia" => options
            .iter()
            .enumerate()
            .filter(|(_, option)| option.backend == "cuda")
            .filter(|(_, option)| {
                hardware
                    .cuda_major
                    .is_none_or(|major| cuda_version(&option.asset.name) <= major)
            })
            .max_by_key(|(_, option)| cuda_version(&option.asset.name))
            .map(|(index, _)| index),
        "amd" => options.iter().position(|option| option.backend == "rocm"),
        "intel" => options.iter().position(|option| option.backend == "sycl"),
        "other" => options.iter().position(|option| option.backend == "vulkan"),
        _ => options.iter().position(|option| option.backend == "cpu"),
    }
    .or_else(|| options.iter().position(|option| option.backend == "vulkan"))
    .or_else(|| options.iter().position(|option| option.backend == "cpu"));
    if let Some(index) = recommended_index {
        options[index].recommended = true;
    }
    options.sort_by_key(|option| (!option.recommended, option.backend.clone()));
    RuntimeCatalog {
        tag: release.tag_name.clone(),
        published_at: release.published_at.clone().unwrap_or_default(),
        options,
    }
}

fn sanitize_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
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
    let body = github_client()?
        .get(RELEASES_URL)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("GitHub release lookup failed: {error}"))?
        .text()
        .map_err(|error| format!("GitHub release body could not be read: {error}"))?;
    let releases = serde_json::from_str::<Vec<GithubRelease>>(&body)
        .map_err(|error| format!("GitHub release response was invalid at {error}"))?;
    let release = releases
        .iter()
        .find(|release| {
            release.assets.iter().any(|asset| {
                let name = asset.name.to_ascii_lowercase();
                name.starts_with("llama-") && name.contains("-bin-win-") && name.ends_with(".zip")
            })
        })
        .ok_or_else(|| {
            "No recent llama.cpp release contains Windows runtime archives".to_string()
        })?;
    let catalog = build_catalog(release, hardware);
    if catalog.options.is_empty() {
        return Err(format!(
            "Release {} has no Windows {} runtime archives",
            catalog.tag, hardware.architecture
        ));
    }
    Ok(catalog)
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
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("Unsafe path in archive: {}", entry.name()))?;
        if entry.name().len() > limits.max_path_bytes {
            return Err(format!("Archive path is too long: {}", entry.name()));
        }
        if !paths.insert(relative.clone()) {
            return Err(format!("Duplicate path in archive: {}", entry.name()));
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
        let output = destination.join(relative);
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

    fn release() -> GithubRelease {
        GithubRelease {
            tag_name: "b10736".into(),
            published_at: Some("2026-09-01T00:00:00Z".into()),
            assets: vec![
                GithubAsset::sample("llama-b10736-bin-win-cpu-x64.zip"),
                GithubAsset::sample("llama-b10736-bin-win-vulkan-x64.zip"),
                GithubAsset::sample("llama-b10736-bin-win-rocm-7.14-x64.zip"),
                GithubAsset::sample("llama-b10736-bin-win-sycl-x64.zip"),
                GithubAsset::sample("llama-b10736-bin-win-cuda-12.4-x64.zip"),
                GithubAsset::sample("cudart-llama-bin-win-cuda-12.4-x64.zip"),
                GithubAsset::sample("llama-b10736-bin-win-cuda-13.3-x64.zip"),
                GithubAsset::sample("cudart-llama-bin-win-cuda-13.3-x64.zip"),
                GithubAsset::sample("llama-b10736-bin-win-cpu-arm64.zip"),
            ],
        }
    }

    #[test]
    fn nvidia_cuda_13_machine_gets_matching_recommendation_and_cudart() {
        let hardware = HardwareInfo {
            architecture: "x64".into(),
            gpu_names: vec!["NVIDIA GeForce RTX 5090".into()],
            vendor: "nvidia".into(),
            cuda_major: Some(13),
            driver_version: "test".into(),
            detection_status: "test fixture".into(),
            recommendation: String::new(),
            system_memory: detect_system_memory(),
            adapters: Vec::new(),
            manual_overrides: Vec::new(),
        };
        let catalog = build_catalog(&release(), &hardware);
        let recommended = catalog
            .options
            .iter()
            .find(|option| option.recommended)
            .unwrap();
        assert_eq!(recommended.backend, "cuda");
        assert!(recommended.asset.name.contains("cuda-13.3"));
        let cuda_keys = catalog
            .options
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
        assert!(catalog
            .options
            .iter()
            .any(|option| option.backend == "vulkan"));
        assert!(catalog.options.iter().any(|option| option.backend == "cpu"));
    }

    #[test]
    fn amd_machine_prefers_rocm_with_vulkan_fallback() {
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
        let catalog = build_catalog(&release(), &hardware);
        assert_eq!(
            catalog
                .options
                .iter()
                .find(|option| option.recommended)
                .unwrap()
                .backend,
            "rocm"
        );
        assert!(catalog
            .options
            .iter()
            .any(|option| option.backend == "vulkan"));
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
        let catalog = build_catalog(&release(), &hardware);
        assert_eq!(catalog.options.len(), 1);
        assert!(catalog.options[0].asset.name.contains("arm64"));
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
        fs::remove_dir_all(dir).unwrap();
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
