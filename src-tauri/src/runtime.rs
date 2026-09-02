use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

const RELEASES_URL: &str = "https://api.github.com/repos/ggml-org/llama.cpp/releases?per_page=20";

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
    let text = fs::read_to_string(dir.join("runtime.json")).ok()?;
    serde_json::from_str(&text).ok()
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

    if let Some(manifest) = read_manifest(dir) {
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
        if !tag_dir.is_dir()
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
            let Some(manifest) = read_manifest(&install_dir) else {
                continue;
            };
            let runtime_path = install_dir.join(&manifest.runtime);
            if !runtime_path.is_file() {
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
    Ok(list_managed_runtimes_in(&managed_runtime_root()?))
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

pub fn detect_hardware() -> HardwareInfo {
    let architecture = architecture();
    let nvidia_query = crate::proc::hidden_command("nvidia-smi.exe")
        .args(["--query-gpu=name,driver_version", "--format=csv,noheader"])
        .output();
    if let Ok(output) = nvidia_query {
        if output.status.success() {
            let rows = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| line.split_once(',').unwrap_or((line, "")))
                .map(|(name, driver)| (name.trim().to_string(), driver.trim().to_string()))
                .collect::<Vec<_>>();
            let gpu_names = rows
                .iter()
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>();
            let driver_version = rows
                .first()
                .map(|(_, driver)| driver.clone())
                .unwrap_or_default();
            if !gpu_names.is_empty() {
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
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .ok_or_else(|| "Unable to resolve an application data directory".to_string())?;
    Ok(base.join("GGUF Pilot").join("runtimes"))
}

fn github_client() -> Result<reqwest::blocking::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );
    reqwest::blocking::Client::builder()
        .user_agent(format!("GGUF-Pilot/{}", env!("CARGO_PKG_VERSION")))
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

fn download_asset(
    client: &reqwest::blocking::Client,
    asset: &GithubAsset,
    destination: &Path,
) -> Result<(), String> {
    let mut response = client
        .get(&asset.browser_download_url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("Download failed for {}: {error}", asset.name))?;
    let mut file = File::create(destination).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut written = 0_u64;
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = response
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        io::Write::write_all(&mut file, &buffer[..count]).map_err(|error| error.to_string())?;
        hasher.update(&buffer[..count]);
        written += count as u64;
    }
    if asset.size > 0 && written != asset.size {
        return Err(format!(
            "Download size mismatch for {}: expected {}, received {}",
            asset.name, asset.size, written
        ));
    }
    if let Some(expected) = expected_sha256(asset) {
        let actual = hex::encode(hasher.finalize());
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(format!("SHA-256 mismatch for {}", asset.name));
        }
    }
    Ok(())
}

fn extract_zip(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|error| error.to_string())?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("Unsafe path in archive: {}", entry.name()))?;
        let output = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&output).map_err(|error| error.to_string())?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut target = File::create(&output).map_err(|error| error.to_string())?;
        io::copy(&mut entry, &mut target).map_err(|error| error.to_string())?;
    }
    Ok(())
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
        };
        let catalog = build_catalog(&release(), &hardware);
        assert_eq!(catalog.options.len(), 1);
        assert!(catalog.options[0].asset.name.contains("arm64"));
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gguf-pilot-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
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
    fn tag_build_numbers_parse_with_or_without_prefix() {
        assert_eq!(tag_build("b10752"), Some(10752));
        assert_eq!(tag_build("10679"), Some(10679));
        assert_eq!(tag_build("preview"), None);
    }
}
