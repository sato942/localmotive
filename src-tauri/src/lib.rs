mod cloud;
mod core;
mod gguf;
mod proc;
mod runtime;
mod tune;

use core::{BenchmarkSummary, LaunchProfile, LogicalModel, RuntimeCapabilities};
use serde::Serialize;
use std::fs::{self, File};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::Emitter;

struct ManagedServer {
    child: Child,
    profile: LaunchProfile,
    command: String,
    log_path: String,
    started_at: u64,
}

#[derive(Default)]
struct AppState {
    server: Mutex<Option<ManagedServer>>,
    tuning: Mutex<Option<Arc<AtomicBool>>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerStatus {
    running: bool,
    pid: Option<u32>,
    profile_name: Option<String>,
    alias: Option<String>,
    port: Option<u16>,
    command: Option<String>,
    log_path: Option<String>,
    started_at: Option<u64>,
    exit_code: Option<i32>,
}

fn status_from(slot: &mut Option<ManagedServer>) -> ServerStatus {
    if let Some(server) = slot.as_mut() {
        match server.child.try_wait() {
            Ok(Some(code)) => {
                let status = ServerStatus {
                    running: false,
                    pid: None,
                    profile_name: Some(server.profile.name.clone()),
                    alias: Some(server.profile.alias.clone()),
                    port: Some(server.profile.port),
                    command: Some(server.command.clone()),
                    log_path: Some(server.log_path.clone()),
                    started_at: Some(server.started_at),
                    exit_code: code.code(),
                };
                *slot = None;
                status
            }
            Ok(None) => ServerStatus {
                running: true,
                pid: Some(server.child.id()),
                profile_name: Some(server.profile.name.clone()),
                alias: Some(server.profile.alias.clone()),
                port: Some(server.profile.port),
                command: Some(server.command.clone()),
                log_path: Some(server.log_path.clone()),
                started_at: Some(server.started_at),
                exit_code: None,
            },
            Err(_) => ServerStatus {
                running: false,
                pid: None,
                profile_name: Some(server.profile.name.clone()),
                alias: Some(server.profile.alias.clone()),
                port: Some(server.profile.port),
                command: Some(server.command.clone()),
                log_path: Some(server.log_path.clone()),
                started_at: Some(server.started_at),
                exit_code: None,
            },
        }
    } else {
        ServerStatus {
            running: false,
            pid: None,
            profile_name: None,
            alias: None,
            port: None,
            command: None,
            log_path: None,
            started_at: None,
            exit_code: None,
        }
    }
}

/// Validate every path the profile references, then spawn llama-server with
/// output redirected to a per-port log file. Shared by the Start button and
/// the tuner so both launch exactly the same way.
fn spawn_server(
    profile: &LaunchProfile,
    log_name: &str,
) -> Result<(Child, String, String), String> {
    if !Path::new(&profile.runtime).is_file() {
        return Err(format!("Runtime does not exist: {}", profile.runtime));
    }
    if !Path::new(&profile.model).is_file() {
        return Err(format!("Model does not exist: {}", profile.model));
    }
    if let Some(path) = profile.draft_model.as_ref().filter(|p| !p.is_empty()) {
        if !Path::new(path).is_file() {
            return Err(format!("Draft model does not exist: {path}"));
        }
    }
    if let Some(path) = profile.mmproj.as_ref().filter(|p| !p.is_empty()) {
        if !Path::new(path).is_file() {
            return Err(format!("Vision projector does not exist: {path}"));
        }
    }
    for (label, path) in [
        ("API key file", profile.api_key_file.as_str()),
        ("SSL private key", profile.ssl_key_file.as_str()),
        ("SSL certificate", profile.ssl_cert_file.as_str()),
        ("Chat template", profile.chat_template_file.as_str()),
    ] {
        if !path.is_empty() && !Path::new(path).is_file() {
            return Err(format!("{label} does not exist: {path}"));
        }
    }
    TcpListener::bind((profile.host.as_str(), profile.port)).map_err(|_| {
        format!(
            "Port {} is already in use on {}",
            profile.port, profile.host
        )
    })?;

    let capabilities = core::inspect_runtime(Path::new(&profile.runtime))?;
    let raw_args = profile.build_args()?;
    let (args, _) = core::filter_supported_args(&raw_args, &capabilities.supported_flags);
    let command = profile.display_command_with_args(&args);
    let log_dir = std::env::temp_dir().join("gguf-pilot");
    fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    let log_path = log_dir.join(format!("{log_name}.log"));
    let stdout = File::create(&log_path).map_err(|e| e.to_string())?;
    let stderr = stdout.try_clone().map_err(|e| e.to_string())?;
    let child = proc::hidden_command(&profile.runtime)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok((child, command, log_path.to_string_lossy().to_string()))
}

fn connect_host(host: &str) -> &str {
    match host {
        "0.0.0.0" => "127.0.0.1",
        "::" | "[::]" => "::1",
        value => value,
    }
}

/// Block until `/health` answers 200, the child exits, or the deadline passes.
fn wait_until_healthy(
    child: &mut Child,
    host: &str,
    port: u16,
    log_path: &str,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let host = connect_host(host);
    loop {
        if let Ok(Some(code)) = child.try_wait() {
            let tail = fs::read_to_string(log_path)
                .map(|text| {
                    text.lines()
                        .rev()
                        .take(12)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            return Err(format!(
                "llama-server exited with {} before becoming healthy\n{tail}",
                code.code()
                    .map(|c| format!("code {c}"))
                    .unwrap_or_else(|| "a signal".into())
            ));
        }
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &format!("{host}:{port}")
                .parse()
                .map_err(|e: std::net::AddrParseError| e.to_string())?,
            Duration::from_millis(500),
        ) {
            use std::io::{Read, Write};
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let request =
                format!("GET /health HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
            if stream.write_all(request.as_bytes()).is_ok() {
                let mut response = String::new();
                let _ = stream.read_to_string(&mut response);
                if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200") {
                    return Ok(());
                }
            }
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "llama-server did not become healthy within {} s",
                timeout.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(400));
    }
}

#[tauri::command]
fn scan_models(root: String) -> Result<Vec<LogicalModel>, String> {
    core::scan_models(Path::new(&root))
}

#[tauri::command]
fn inspect_runtime(path: String) -> Result<RuntimeCapabilities, String> {
    core::inspect_runtime(Path::new(&path))
}

#[tauri::command]
fn describe_runtime(path: String) -> runtime::RuntimeIdentity {
    runtime::describe_runtime(Path::new(&path))
}

#[tauri::command]
fn list_managed_runtimes() -> Result<Vec<runtime::ManagedRuntimeRecord>, String> {
    runtime::list_managed_runtimes()
}

#[tauri::command]
fn read_gguf_summary(path: String) -> Result<gguf::GgufSummary, String> {
    gguf::read_summary(Path::new(&path))
}

#[tauri::command]
fn preview_command(profile: LaunchProfile) -> Result<String, String> {
    let capabilities = core::inspect_runtime(Path::new(&profile.runtime))?;
    let raw_args = profile.build_args()?;
    let (args, _) = core::filter_supported_args(&raw_args, &capabilities.supported_flags);
    Ok(profile.display_command_with_args(&args))
}

#[tauri::command]
fn start_server(
    profile: LaunchProfile,
    state: tauri::State<AppState>,
) -> Result<ServerStatus, String> {
    let mut slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable")?;
    if status_from(&mut slot).running {
        return Err("Stop the running server before starting another profile".into());
    }
    if state
        .tuning
        .lock()
        .map_err(|_| "Tuning state is unavailable")?
        .is_some()
    {
        return Err("A tuning session is running; stop it before starting a server".into());
    }
    let (child, command, log_path) = spawn_server(&profile, &format!("server-{}", profile.port))?;
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    *slot = Some(ManagedServer {
        child,
        profile,
        command,
        log_path,
        started_at,
    });
    Ok(status_from(&mut slot))
}

#[tauri::command]
fn stop_server(state: tauri::State<AppState>) -> Result<ServerStatus, String> {
    let mut slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable")?;
    if let Some(server) = slot.as_mut() {
        server.child.kill().map_err(|e| e.to_string())?;
        let _ = server.child.wait();
    }
    *slot = None;
    Ok(status_from(&mut slot))
}

#[tauri::command]
fn server_status(state: tauri::State<AppState>) -> Result<ServerStatus, String> {
    let mut slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable")?;
    Ok(status_from(&mut slot))
}

#[tauri::command]
fn read_server_log(state: tauri::State<AppState>) -> Result<String, String> {
    let slot = state
        .server
        .lock()
        .map_err(|_| "Server state is unavailable")?;
    let Some(server) = slot.as_ref() else {
        return Ok("No server is running.".into());
    };
    let text = fs::read_to_string(&server.log_path).map_err(|e| e.to_string())?;
    let lines = text.lines().rev().take(250).collect::<Vec<_>>();
    Ok(lines.into_iter().rev().collect::<Vec<_>>().join("\n"))
}

#[tauri::command]
fn benchmark_server(
    host: String,
    port: u16,
    tokens: u32,
    repeats: u16,
) -> Result<BenchmarkSummary, String> {
    core::benchmark_server(&host, port, tokens, repeats)
}

#[tauri::command]
fn detect_hardware() -> runtime::HardwareInfo {
    runtime::detect_hardware()
}

#[tauri::command]
async fn fetch_runtime_catalog() -> Result<runtime::RuntimeCatalog, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let hardware = runtime::detect_hardware();
        runtime::fetch_catalog(&hardware)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn managed_runtime_root() -> Result<String, String> {
    Ok(runtime::managed_runtime_root()?
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
async fn install_managed_runtime(
    tag: String,
    option: runtime::RuntimeOption,
) -> Result<runtime::InstalledRuntime, String> {
    tauri::async_runtime::spawn_blocking(move || runtime::install_runtime(&tag, &option))
        .await
        .map_err(|error| error.to_string())?
}

// ---------------------------------------------------------------------------
// Cloud credentials
// ---------------------------------------------------------------------------

#[tauri::command]
fn cloud_providers() -> Vec<cloud::Provider> {
    cloud::PROVIDERS.to_vec()
}

#[tauri::command]
fn cloud_credential_status(provider: String) -> Result<cloud::CredentialStatus, String> {
    cloud::credential_status(&cloud::KeyringStore, &provider)
}

#[tauri::command]
fn cloud_save_credential(
    provider: String,
    secret: String,
) -> Result<cloud::CredentialStatus, String> {
    cloud::save_credential(&cloud::KeyringStore, &provider, &secret)
}

#[tauri::command]
fn cloud_clear_credential(provider: String) -> Result<cloud::CredentialStatus, String> {
    cloud::clear_credential(&cloud::KeyringStore, &provider)
}

#[tauri::command]
async fn cloud_list_models(provider: String) -> Result<Vec<cloud::CloudModel>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        cloud::list_models(&cloud::KeyringStore, &provider)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn cloud_probe(provider: String, model: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        cloud::probe(&cloud::KeyringStore, &provider, &model)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Full OpenRouter PKCE sign-in: bind a loopback callback, open the browser,
/// wait for the code, exchange it, store the resulting key. The key never
/// crosses the IPC boundary to the frontend.
#[tauri::command]
async fn cloud_openrouter_login(app: tauri::AppHandle) -> Result<cloud::CredentialStatus, String> {
    use tauri_plugin_opener::OpenerExt;
    let (listener, callback_url) = cloud::bind_callback()?;
    let pkce = cloud::pkce_challenge();
    let url = cloud::openrouter_auth_url(&callback_url, &pkce.challenge);
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| format!("Could not open the browser: {error}"))?;
    tauri::async_runtime::spawn_blocking(move || {
        let code = cloud::wait_for_code(&listener, Duration::from_secs(300))?;
        let key = cloud::exchange_code_for_key(&code, &pkce.verifier)?;
        cloud::save_credential(&cloud::KeyringStore, "openrouter", &key)
    })
    .await
    .map_err(|error| error.to_string())?
}

// ---------------------------------------------------------------------------
// AI tuning
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TuningProgress {
    phase: String,
    message: String,
    trial: Option<tune::TuningTrial>,
}

struct LiveBench<'a> {
    app: &'a tauri::AppHandle,
    cancel: Arc<AtomicBool>,
    tokens: u32,
    repeats: u16,
}

impl tune::Bench for LiveBench<'_> {
    fn measure(&mut self, profile: &LaunchProfile) -> Result<(BenchmarkSummary, String), String> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let _ = self.app.emit(
            "tuning-progress",
            TuningProgress {
                phase: "launch".into(),
                message: "Starting llama-server with the candidate configuration…".into(),
                trial: None,
            },
        );
        let (mut child, command, log_path) = spawn_server(profile, "tuning")?;
        let result = (|| {
            wait_until_healthy(
                &mut child,
                &profile.host,
                profile.port,
                &log_path,
                Duration::from_secs(600),
            )?;
            let _ = self.app.emit(
                "tuning-progress",
                TuningProgress {
                    phase: "measure".into(),
                    message: format!(
                        "Healthy. Measuring {} × {} tokens…",
                        self.repeats, self.tokens
                    ),
                    trial: None,
                },
            );
            if self.cancel.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            core::benchmark_server(&profile.host, profile.port, self.tokens, self.repeats)
        })();
        let _ = child.kill();
        let _ = child.wait();
        // Give the OS a moment to release the port before the next launch.
        std::thread::sleep(Duration::from_millis(600));
        result.map(|summary| (summary, command))
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TuningRequest {
    profile: LaunchProfile,
    provider: String,
    model: String,
    target_context: u32,
    max_trials: u32,
    tokens: u32,
    repeats: u16,
    companions: Vec<String>,
}

#[tauri::command]
async fn start_tuning(
    app: tauri::AppHandle,
    request: TuningRequest,
    state: tauri::State<'_, AppState>,
) -> Result<tune::TuningReport, String> {
    {
        let mut slot = state
            .server
            .lock()
            .map_err(|_| "Server state is unavailable")?;
        if status_from(&mut slot).running {
            return Err("Stop the running server before tuning; the tuner launches its own".into());
        }
    }
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut tuning = state
            .tuning
            .lock()
            .map_err(|_| "Tuning state is unavailable")?;
        if tuning.is_some() {
            return Err("A tuning session is already running".into());
        }
        *tuning = Some(cancel.clone());
    }
    if !(1..=12).contains(&request.max_trials) {
        clear_tuning(&state);
        return Err("Trials must be between 1 and 12".into());
    }
    if request.target_context < 512 {
        clear_tuning(&state);
        return Err("Target context must be at least 512 tokens".into());
    }

    let handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let emit = |phase: &str, message: String| {
            let _ = handle.emit(
                "tuning-progress",
                TuningProgress {
                    phase: phase.into(),
                    message,
                    trial: None,
                },
            );
        };
        emit("prepare", "Reading hardware, GGUF metadata, and runtime capabilities…".into());
        let hardware = runtime::detect_hardware();
        let capabilities = core::inspect_runtime(Path::new(&request.profile.runtime))?;
        let gguf = gguf::read_summary(Path::new(&request.profile.model)).ok();
        let system_ram_bytes = system_ram_bytes();
        let inputs = tune::TuningInputs {
            objective: "Maximise measured generation tokens/second at the target context while the server starts and answers.",
            target_context: request.target_context,
            hardware: &hardware,
            system_ram_bytes,
            gguf: gguf.as_ref(),
            capabilities: &capabilities,
            companions: &request.companions,
            max_trials: request.max_trials,
        };
        let mut bench = LiveBench {
            app: &handle,
            cancel: cancel.clone(),
            tokens: request.tokens.clamp(64, 2048),
            repeats: request.repeats.clamp(1, 5),
        };
        let store = cloud::KeyringStore;
        let mut advisor = cloud::CloudAdvisor {
            store: &store,
            provider_id: request.provider.clone(),
            model: request.model.clone(),
            last_raw_reply: String::new(),
        };
        let progress_handle = handle.clone();
        tune::run_tuning(&request.profile, &inputs, &mut bench, &mut advisor, |trial| {
            let _ = progress_handle.emit(
                "tuning-progress",
                TuningProgress {
                    phase: "trial".into(),
                    message: match (&trial.mean_tps, &trial.error) {
                        (Some(tps), _) => format!("Trial {}: {tps:.2} tok/s", trial.index),
                        (None, Some(error)) => format!("Trial {} failed: {}", trial.index, error.lines().next().unwrap_or("")),
                        _ => format!("Trial {}", trial.index),
                    },
                    trial: Some(trial.clone()),
                },
            );
        })
    })
    .await
    .map_err(|error| error.to_string());
    clear_tuning(&state);
    result?
}

fn clear_tuning(state: &tauri::State<'_, AppState>) {
    if let Ok(mut tuning) = state.tuning.lock() {
        *tuning = None;
    }
}

#[tauri::command]
fn cancel_tuning(state: tauri::State<AppState>) -> Result<bool, String> {
    let tuning = state
        .tuning
        .lock()
        .map_err(|_| "Tuning state is unavailable")?;
    match tuning.as_ref() {
        Some(flag) => {
            flag.store(true, Ordering::Relaxed);
            Ok(true)
        }
        None => Ok(false),
    }
}

fn system_ram_bytes() -> Option<u64> {
    let output = proc::hidden_command("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

#[tauri::command]
fn suggest_port(host: String, preferred: u16) -> Result<u16, String> {
    core::pick_free_port(&host, preferred)
}

#[tauri::command]
fn about_info(app: tauri::AppHandle) -> AboutInfo {
    let package = app.package_info();
    AboutInfo {
        name: package.name.clone(),
        version: package.version.to_string(),
        tauri_version: tauri::VERSION.to_string(),
        identifier: app.config().identifier.clone(),
        os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        runtime_root: runtime::managed_runtime_root()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        log_dir: std::env::temp_dir()
            .join("gguf-pilot")
            .to_string_lossy()
            .to_string(),
        repository: env!("CARGO_PKG_REPOSITORY").to_string(),
        license: env!("CARGO_PKG_LICENSE").to_string(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AboutInfo {
    name: String,
    version: String,
    tauri_version: String,
    identifier: String,
    os: String,
    runtime_root: String,
    log_dir: String,
    repository: String,
    license: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_models,
            inspect_runtime,
            describe_runtime,
            list_managed_runtimes,
            read_gguf_summary,
            preview_command,
            start_server,
            stop_server,
            server_status,
            read_server_log,
            benchmark_server,
            detect_hardware,
            fetch_runtime_catalog,
            managed_runtime_root,
            install_managed_runtime,
            cloud_providers,
            cloud_credential_status,
            cloud_save_credential,
            cloud_clear_credential,
            cloud_list_models,
            cloud_probe,
            cloud_openrouter_login,
            start_tuning,
            cancel_tuning,
            suggest_port,
            about_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GGUF Pilot");
}
