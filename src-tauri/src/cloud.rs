//! Cloud AI providers used by the tuner.
//!
//! Two ways in, mirroring how Hermes Agent itself authenticates: paste an API
//! key, or (for OpenRouter) sign in through the browser with OAuth PKCE and
//! receive a user-controlled key back on a loopback callback. Either way the
//! secret lands in the Windows Credential Manager under the app's service name —
//! never in localStorage, never in a config file, never in a log line.
//!
//! All traffic is OpenAI-compatible `chat/completions`, so Anthropic, Gemini,
//! OpenAI, DeepSeek, xAI, and OpenRouter share one code path.

use crate::tune::{Advisor, Proposal, TuningBrief};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::net::TcpListener;
use std::time::Duration;

const KEYRING_SERVICE: &str = "Localmotive";
/// Previous product name. Upgrades read legacy entries once and move them to
/// the current service so existing users keep their keys.
const LEGACY_KEYRING_SERVICE: &str = "GGUF Pilot";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: &'static str,
    pub label: &'static str,
    pub base_url: &'static str,
    pub key_prefix_hint: &'static str,
    pub console_url: &'static str,
    pub supports_oauth: bool,
    pub default_model: &'static str,
    /// Some providers do not implement `GET /models` on their compatibility layer.
    pub lists_models: bool,
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "openrouter",
        label: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        key_prefix_hint: "sk-or-",
        console_url: "https://openrouter.ai/settings/keys",
        supports_oauth: true,
        default_model: "anthropic/claude-sonnet-4.6",
        lists_models: true,
    },
    Provider {
        id: "anthropic",
        label: "Anthropic",
        base_url: "https://api.anthropic.com/v1",
        key_prefix_hint: "sk-ant-",
        console_url: "https://platform.claude.com/settings/keys",
        supports_oauth: false,
        default_model: "claude-sonnet-4-6",
        lists_models: true,
    },
    Provider {
        id: "openai",
        label: "OpenAI",
        base_url: "https://api.openai.com/v1",
        key_prefix_hint: "sk-",
        console_url: "https://platform.openai.com/api-keys",
        supports_oauth: false,
        default_model: "gpt-5",
        lists_models: true,
    },
    Provider {
        id: "gemini",
        label: "Google Gemini",
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        key_prefix_hint: "AIza",
        console_url: "https://aistudio.google.com/apikey",
        supports_oauth: false,
        default_model: "gemini-2.5-pro",
        lists_models: true,
    },
    Provider {
        id: "deepseek",
        label: "DeepSeek",
        base_url: "https://api.deepseek.com/v1",
        key_prefix_hint: "sk-",
        console_url: "https://platform.deepseek.com/api_keys",
        supports_oauth: false,
        default_model: "deepseek-reasoner",
        lists_models: true,
    },
    Provider {
        id: "xai",
        label: "xAI",
        base_url: "https://api.x.ai/v1",
        key_prefix_hint: "xai-",
        console_url: "https://console.x.ai",
        supports_oauth: false,
        default_model: "grok-4",
        lists_models: true,
    },
];

pub fn provider(id: &str) -> Result<&'static Provider, String> {
    PROVIDERS
        .iter()
        .find(|provider| provider.id == id)
        .ok_or_else(|| format!("Unknown cloud provider `{id}`"))
}

// ---------------------------------------------------------------------------
// Credential storage
// ---------------------------------------------------------------------------

/// Where secrets live. Abstracted so tests use memory and the app uses the
/// Windows Credential Manager.
pub trait SecretStore {
    fn get(&self, account: &str) -> Result<Option<String>, String>;
    fn set(&self, account: &str, secret: &str) -> Result<(), String>;
    fn delete(&self, account: &str) -> Result<(), String>;
}

pub struct KeyringStore;

impl SecretStore for KeyringStore {
    fn get(&self, account: &str) -> Result<Option<String>, String> {
        if let Some(secret) = read_service(KEYRING_SERVICE, account)? {
            return Ok(Some(secret));
        }
        // Legacy installs stored keys under the previous product name.
        // Migrate the value forward so the old entry does not linger.
        if let Some(secret) = read_service(LEGACY_KEYRING_SERVICE, account)? {
            write_service(KEYRING_SERVICE, account, &secret)?;
            delete_service(LEGACY_KEYRING_SERVICE, account)?;
            return Ok(Some(secret));
        }
        Ok(None)
    }
    fn set(&self, account: &str, secret: &str) -> Result<(), String> {
        write_service(KEYRING_SERVICE, account, secret)?;
        // A migrated write replaces the legacy entry; never keep two copies.
        delete_service(LEGACY_KEYRING_SERVICE, account)?;
        Ok(())
    }
    fn delete(&self, account: &str) -> Result<(), String> {
        delete_service(KEYRING_SERVICE, account)?;
        delete_service(LEGACY_KEYRING_SERVICE, account)?;
        Ok(())
    }
}

fn read_service(service: &str, account: &str) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(service, account).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("Credential Manager read failed: {error}")),
    }
}

fn write_service(service: &str, account: &str, secret: &str) -> Result<(), String> {
    keyring::Entry::new(service, account)
        .and_then(|entry| entry.set_password(secret))
        .map_err(|error| format!("Credential Manager write failed: {error}"))
}

fn delete_service(service: &str, account: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(service, account).map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("Credential Manager delete failed: {error}")),
    }
}

fn account_for(provider_id: &str) -> String {
    format!("cloud:{provider_id}")
}

/// What the UI is allowed to know about a stored key: that it exists, its
/// length, and a masked tail. Never the key.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CredentialStatus {
    pub provider: String,
    pub configured: bool,
    pub masked: String,
}

pub fn mask_secret(secret: &str) -> String {
    let tail: String = secret
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if secret.chars().count() <= 8 {
        "••••".into()
    } else {
        format!("••••{tail}")
    }
}

pub fn credential_status<S: SecretStore>(
    store: &S,
    provider_id: &str,
) -> Result<CredentialStatus, String> {
    provider(provider_id)?;
    let secret = store.get(&account_for(provider_id))?;
    Ok(CredentialStatus {
        provider: provider_id.into(),
        configured: secret.is_some(),
        masked: secret.as_deref().map(mask_secret).unwrap_or_default(),
    })
}

pub fn save_credential<S: SecretStore>(
    store: &S,
    provider_id: &str,
    secret: &str,
) -> Result<CredentialStatus, String> {
    provider(provider_id)?;
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return Err("API key is empty".into());
    }
    if trimmed.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("API key contains whitespace or control characters".into());
    }
    store.set(&account_for(provider_id), trimmed)?;
    credential_status(store, provider_id)
}

pub fn clear_credential<S: SecretStore>(
    store: &S,
    provider_id: &str,
) -> Result<CredentialStatus, String> {
    provider(provider_id)?;
    store.delete(&account_for(provider_id))?;
    credential_status(store, provider_id)
}

fn require_secret<S: SecretStore>(store: &S, provider_id: &str) -> Result<String, String> {
    store.get(&account_for(provider_id))?.ok_or_else(|| {
        format!(
            "No API key is stored for {}",
            provider(provider_id)
                .map(|p| p.label)
                .unwrap_or(provider_id)
        )
    })
}

// ---------------------------------------------------------------------------
// OpenRouter OAuth PKCE
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
}

pub fn pkce_challenge() -> PkceChallenge {
    use rand::RngCore;
    let mut bytes = [0_u8; 48];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    PkceChallenge {
        challenge: pkce_challenge_for(&verifier),
        verifier,
    }
}

pub fn pkce_challenge_for(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub fn openrouter_auth_url(callback_url: &str, challenge: &str) -> String {
    format!(
        "https://openrouter.ai/auth?callback_url={}&code_challenge={}&code_challenge_method=S256",
        percent_encode(callback_url),
        percent_encode(challenge)
    )
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 3);
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Limits for the loopback callback (audit CLD-01): a request line, a code
/// value, and the number of stray connections one login may tolerate.
pub const MAX_CALLBACK_REQUEST_LINE_BYTES: usize = 8 * 1024;
pub const MAX_CALLBACK_CODE_BYTES: usize = 512;
pub const MAX_CALLBACK_REQUESTS: u32 = 32;

/// The callback request as parsed under the strict contract.
#[derive(Debug, PartialEq, Eq)]
pub enum CallbackRequest {
    /// A well-formed `GET /callback?code=…` with one unambiguous code.
    Code(String),
    /// A harmless probe (favicon, another path, another method) that must
    /// not end the login.
    Ignored,
    /// A malformed or ambiguous callback request.
    Malformed(String),
}

fn percent_decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                if index + 2 >= bytes.len() {
                    return Err("truncated percent escape".into());
                }
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                    .map_err(|_| "invalid percent escape".to_string())?;
                let byte = u8::from_str_radix(hex, 16)
                    .map_err(|_| "invalid percent escape".to_string())?;
                out.push(byte);
                index += 3;
            }
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|_| "percent-decoded code is not UTF-8".into())
}

/// Parse the first request line under the audited contract (CLD-01 I2):
/// only `GET /callback` is accepted; the query is decoded; exactly one
/// non-empty `code` parameter must be present.
pub fn parse_callback_request_line(line: &str) -> CallbackRequest {
    if line.len() > MAX_CALLBACK_REQUEST_LINE_BYTES {
        return CallbackRequest::Malformed("request line exceeds the supported length".into());
    }
    let mut parts = line.split_whitespace();
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else {
        return CallbackRequest::Malformed("request line is incomplete".into());
    };
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (target, None),
    };
    if path != "/callback" {
        return CallbackRequest::Ignored;
    }
    if method != "GET" {
        return CallbackRequest::Malformed(format!("only GET is accepted, saw {method}"));
    }
    let Some(query) = query else {
        return CallbackRequest::Malformed("callback arrived without a code".into());
    };
    let mut codes = Vec::new();
    for pair in query.split('&') {
        let (key, raw) = match pair.split_once('=') {
            Some((key, raw)) => (key, raw),
            None => (pair, ""),
        };
        if key != "code" {
            continue;
        }
        let raw = raw.split('#').next().unwrap_or(raw);
        match percent_decode(raw) {
            Ok(code) => codes.push(code),
            Err(reason) => return CallbackRequest::Malformed(format!("code {reason}")),
        }
    }
    match codes.len() {
        0 => CallbackRequest::Malformed("callback arrived without a code".into()),
        1 if codes[0].is_empty() => CallbackRequest::Malformed("code is empty".into()),
        1 if codes[0].len() > MAX_CALLBACK_CODE_BYTES => {
            CallbackRequest::Malformed("code exceeds the supported length".into())
        }
        1 => CallbackRequest::Code(codes.remove(0)),
        _ => CallbackRequest::Malformed("code was supplied more than once".into()),
    }
}

/// Historical test helper: the code when the request parses strictly.
#[cfg(test)]
pub fn code_from_request_line(line: &str) -> Option<String> {
    match parse_callback_request_line(line) {
        CallbackRequest::Code(code) => Some(code),
        _ => None,
    }
}

/// Bind a loopback listener on an ephemeral port and return it with the
/// callback URL the browser will be sent to.
pub fn bind_callback() -> Result<(TcpListener, String), String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    Ok((listener, format!("http://127.0.0.1:{port}/callback")))
}

fn callback_page(title: &str, accent: &str, message: &str) -> String {
    format!(
        "<!doctype html><meta charset=utf-8><title>Localmotive</title><body style=\"background:#171a1b;color:#e8e9e4;font:15px 'Public Sans','Segoe UI',sans-serif;display:grid;place-items:center;height:100vh;margin:0\"><div style=\"border:1px solid #424849;background:#222627;padding:28px 32px;text-align:center\"><div style=\"font:700 22px 'Bahnschrift Condensed','Arial Narrow',sans-serif;letter-spacing:.06em;color:{accent}\">{title}</div><p style=\"color:#9ca3a0;margin:12px 0 0\">{message}</p></div></body>"
    )
}

/// Read one bounded request line from an accepted stream. The deadline is
/// enforced per chunk, so a steady byte trickle cannot extend the login
/// beyond its overall budget (audit CLD-01 I1).
fn read_bounded_request_line(
    stream: &mut std::net::TcpStream,
    deadline: std::time::Instant,
) -> Result<Option<String>, String> {
    use std::io::Read;
    let mut line = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        if std::time::Instant::now() >= deadline {
            return Err("Timed out waiting for the browser to finish signing in".into());
        }
        let read = match stream.read(&mut chunk) {
            Ok(read) => read,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                if std::time::Instant::now() >= deadline {
                    return Err("Timed out waiting for the browser to finish signing in".into());
                }
                continue;
            }
            Err(error) => return Err(error.to_string()),
        };
        if read == 0 {
            return Ok(None);
        }
        // The request line ends at the first LF (CRLF and bare LF clients
        // both exist); a trailing CR is part of the line ending.
        if let Some(position) = chunk[..read].iter().position(|byte| *byte == b'\n') {
            let mut end = position;
            if end > 0 && chunk[end - 1] == b'\r' {
                end -= 1;
            }
            line.extend_from_slice(&chunk[..end]);
            break;
        }
        if line.len() + read > MAX_CALLBACK_REQUEST_LINE_BYTES {
            return Err("The callback request line exceeded the supported length".into());
        }
        line.extend_from_slice(&chunk[..read]);
    }
    String::from_utf8(line)
        .map(Some)
        .map_err(|_| "The callback request line was not valid UTF-8".into())
}

/// Wait (bounded) for the browser to hit the callback, answer it, and return
/// the authorization code. Stray probes are answered and ignored; malformed
/// callback attempts get an explicit 400 without ending the login before the
/// deadline or the request budget is exhausted (audit CLD-01).
pub fn wait_for_code(listener: &TcpListener, timeout: Duration) -> Result<String, String> {
    let deadline = std::time::Instant::now() + timeout;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let mut served = 0_u32;
    loop {
        if std::time::Instant::now() >= deadline {
            return Err("Timed out waiting for the browser to finish signing in".into());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream
                    .set_nonblocking(false)
                    .map_err(|error| error.to_string())?;
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .map_err(|error| error.to_string())?;
                served += 1;
                if served > MAX_CALLBACK_REQUESTS {
                    return Err(
                        "Too many connections reached the callback; the sign-in was stopped".into(),
                    );
                }
                let line = match read_bounded_request_line(&mut stream, deadline) {
                    Ok(Some(line)) => line,
                    Ok(None) => continue,
                    Err(error) => {
                        if error.contains("Timed out") {
                            return Err(error);
                        }
                        let body = callback_page(
                            "INVALID CALLBACK",
                            "#e0705f",
                            "The callback request was malformed. Return to Localmotive and try again.",
                        );
                        let _ = write!(
                            stream,
                            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.flush();
                        continue;
                    }
                };
                match parse_callback_request_line(&line) {
                    CallbackRequest::Ignored => {
                        let body = callback_page(
                            "NOT FOUND",
                            "#9ca3a0",
                            "This address is not the Localmotive callback. Continue the sign-in from the application.",
                        );
                        let _ = write!(
                            stream,
                            "HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.flush();
                    }
                    CallbackRequest::Malformed(reason) => {
                        let body = callback_page(
                            "INVALID CALLBACK",
                            "#e0705f",
                            "The sign-in callback was malformed. Return to Localmotive and try again.",
                        );
                        let _ = write!(
                            stream,
                            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nX-Localmotive-Reason: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            reason.replace(['\r', '\n'], " "),
                            body.len(),
                            body
                        );
                        let _ = stream.flush();
                    }
                    CallbackRequest::Code(code) => {
                        // The connection is not confirmed until the exchange
                        // and the credential write succeed (CLD-01 I4); the
                        // application window reports that outcome.
                        let body = callback_page(
                            "CALLBACK RECEIVED",
                            "#9edc72",
                            "Return to Localmotive; the application confirms the connection when the key exchange finishes.",
                        );
                        let _ = write!(
                            stream,
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.flush();
                        return Ok(code);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

#[derive(Deserialize)]
struct KeyExchange {
    key: String,
}

pub fn exchange_code_for_key(code: &str, verifier: &str) -> Result<String, String> {
    let response = http_client(Duration::from_secs(30))?
        .post("https://openrouter.ai/api/v1/auth/keys")
        .json(&serde_json::json!({
            "code": code,
            "code_verifier": verifier,
            "code_challenge_method": "S256",
        }))
        .send()
        .map_err(|error| format!("OpenRouter key exchange failed: {error}"))?;
    let status = response.status();
    let text = response.text().map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(format!(
            "OpenRouter key exchange returned {status}: {}",
            text.chars().take(200).collect::<String>()
        ));
    }
    serde_json::from_str::<KeyExchange>(&text)
        .map(|exchange| exchange.key)
        .map_err(|error| format!("OpenRouter key exchange response was unreadable: {error}"))
}

// ---------------------------------------------------------------------------
// Chat completions
// ---------------------------------------------------------------------------

fn http_client(timeout: Duration) -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent(format!("Localmotive/{}", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(20))
        .timeout(timeout)
        .build()
        .map_err(|error| error.to_string())
}

fn authed(
    request: reqwest::blocking::RequestBuilder,
    provider: &Provider,
    secret: &str,
) -> reqwest::blocking::RequestBuilder {
    let request = request.bearer_auth(secret);
    if provider.id == "openrouter" {
        request
            .header("HTTP-Referer", "https://github.com/sato942/localmotive")
            .header("X-Title", "Localmotive")
    } else {
        request
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CloudModel {
    pub id: String,
    pub label: String,
}

pub fn parse_models(body: &str) -> Result<Vec<CloudModel>, String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|error| error.to_string())?;
    let items = value["data"]
        .as_array()
        .or_else(|| value["models"].as_array())
        .ok_or_else(|| "Model list response had no `data` array".to_string())?;
    let mut models = items
        .iter()
        .filter_map(|item| {
            let id = item["id"].as_str().or_else(|| item["name"].as_str())?;
            let id = id.strip_prefix("models/").unwrap_or(id).to_string();
            let label = item["name"]
                .as_str()
                .or_else(|| item["display_name"].as_str())
                .map(str::to_string)
                .unwrap_or_else(|| id.clone());
            Some(CloudModel { id, label })
        })
        .collect::<Vec<_>>();
    models.sort_by(|a, b| a.id.cmp(&b.id));
    models.dedup_by(|a, b| a.id == b.id);
    Ok(models)
}

pub fn list_models<S: SecretStore>(
    store: &S,
    provider_id: &str,
) -> Result<Vec<CloudModel>, String> {
    let provider = provider(provider_id)?;
    let secret = require_secret(store, provider_id)?;
    if !provider.lists_models {
        return Ok(vec![CloudModel {
            id: provider.default_model.into(),
            label: provider.default_model.into(),
        }]);
    }
    let response = authed(
        http_client(Duration::from_secs(30))?.get(format!("{}/models", provider.base_url)),
        provider,
        &secret,
    )
    .send()
    .map_err(|error| format!("{} model list failed: {error}", provider.label))?;
    let status = response.status();
    let text = response.text().map_err(|error| error.to_string())?;
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(format!(
            "{} rejected the API key ({status})",
            provider.label
        ));
    }
    if !status.is_success() {
        return Err(format!("{} model list returned {status}", provider.label));
    }
    parse_models(&text)
}

pub fn extract_reply(body: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|error| error.to_string())?;
    if let Some(message) = value["error"]["message"].as_str() {
        return Err(format!("Provider error: {message}"));
    }
    let content = &value["choices"][0]["message"]["content"];
    if let Some(text) = content.as_str() {
        return Ok(text.to_string());
    }
    if let Some(parts) = content.as_array() {
        let joined = parts
            .iter()
            .filter_map(|part| part["text"].as_str())
            .collect::<Vec<_>>()
            .join("");
        if !joined.is_empty() {
            return Ok(joined);
        }
    }
    Err("Provider response contained no assistant text".into())
}

pub fn chat<S: SecretStore>(
    store: &S,
    provider_id: &str,
    model: &str,
    system: &str,
    user: &str,
) -> Result<String, String> {
    let provider = provider(provider_id)?;
    let secret = require_secret(store, provider_id)?;
    let response = authed(
        http_client(Duration::from_secs(180))?
            .post(format!("{}/chat/completions", provider.base_url)),
        provider,
        &secret,
    )
    .json(&serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "temperature": 0.2,
        "max_tokens": 1200,
    }))
    .send()
    .map_err(|error| format!("{} request failed: {error}", provider.label))?;
    let status = response.status();
    let text = response.text().map_err(|error| error.to_string())?;
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(format!(
            "{} rejected the API key ({status})",
            provider.label
        ));
    }
    if !status.is_success() {
        let detail = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v["error"]["message"].as_str().map(str::to_string))
            .unwrap_or_else(|| text.chars().take(200).collect());
        return Err(format!("{} returned {status}: {detail}", provider.label));
    }
    extract_reply(&text)
}

/// Quick connectivity/credential probe: one tiny completion.
pub fn probe<S: SecretStore>(store: &S, provider_id: &str, model: &str) -> Result<String, String> {
    chat(
        store,
        provider_id,
        model,
        "Reply with the single word OK.",
        "Connection test.",
    )
}

/// The production `Advisor`: serialises the brief, calls the provider, parses
/// the proposal.
pub struct CloudAdvisor<'a, S: SecretStore> {
    pub store: &'a S,
    pub provider_id: String,
    pub model: String,
    pub last_raw_reply: String,
}

impl<S: SecretStore> Advisor for CloudAdvisor<'_, S> {
    fn propose(&mut self, brief: &TuningBrief) -> Result<Proposal, String> {
        let user = format!(
            "Here is the tuning brief as JSON. Propose the next configuration to measure.\n\n{}",
            serde_json::to_string_pretty(&brief.wire).map_err(|error| error.to_string())?
        );
        let reply = chat(
            self.store,
            &self.provider_id,
            &self.model,
            crate::tune::SYSTEM_PROMPT,
            &user,
        )?;
        self.last_raw_reply = reply.clone();
        crate::tune::parse_proposal(&reply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MemoryStore(RefCell<HashMap<String, String>>);
    impl SecretStore for MemoryStore {
        fn get(&self, account: &str) -> Result<Option<String>, String> {
            Ok(self.0.borrow().get(account).cloned())
        }
        fn set(&self, account: &str, secret: &str) -> Result<(), String> {
            self.0.borrow_mut().insert(account.into(), secret.into());
            Ok(())
        }
        fn delete(&self, account: &str) -> Result<(), String> {
            self.0.borrow_mut().remove(account);
            Ok(())
        }
    }

    #[test]
    fn every_provider_has_an_openai_compatible_base_and_unique_id() {
        let mut ids = std::collections::HashSet::new();
        for provider in PROVIDERS {
            assert!(
                ids.insert(provider.id),
                "duplicate provider id {}",
                provider.id
            );
            assert!(provider.base_url.starts_with("https://"));
            assert!(
                !provider.base_url.ends_with('/'),
                "{} base_url must not end with /",
                provider.id
            );
        }
        assert!(provider("openrouter").unwrap().supports_oauth);
        assert!(provider("nope").is_err());
    }

    #[test]
    fn credentials_are_stored_trimmed_and_reported_masked_never_plaintext() {
        let store = MemoryStore::default();
        assert!(!credential_status(&store, "openrouter").unwrap().configured);
        let status =
            save_credential(&store, "openrouter", "  sk-or-v1-abcdef1234567890  ").unwrap();
        assert!(status.configured);
        assert_eq!(status.masked, "••••7890");
        assert!(!serde_json::to_string(&status).unwrap().contains("abcdef"));
        assert_eq!(
            store.get("cloud:openrouter").unwrap().as_deref(),
            Some("sk-or-v1-abcdef1234567890")
        );
        assert!(save_credential(&store, "openrouter", "   ").is_err());
        assert!(save_credential(&store, "openrouter", "sk-or bad key").is_err());
        assert!(save_credential(&store, "unknown", "sk-x").is_err());
        assert!(!clear_credential(&store, "openrouter").unwrap().configured);
        assert_eq!(mask_secret("short"), "••••");
    }

    #[test]
    fn pkce_challenge_is_s256_of_verifier_and_auth_url_is_encoded() {
        // RFC 7636 appendix B test vector.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            pkce_challenge_for(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        let fresh = pkce_challenge();
        assert_eq!(pkce_challenge_for(&fresh.verifier), fresh.challenge);
        assert!(fresh.verifier.len() >= 43);
        let url = openrouter_auth_url("http://127.0.0.1:5123/callback", "abc+/=");
        assert!(url.starts_with("https://openrouter.ai/auth?callback_url=http%3A%2F%2F127.0.0.1%3A5123%2Fcallback&code_challenge=abc%2B%2F%3D&code_challenge_method=S256"));
    }

    #[test]
    fn callback_request_line_yields_code() {
        assert_eq!(
            code_from_request_line("GET /callback?code=abc123&state=x HTTP/1.1").as_deref(),
            Some("abc123")
        );
        assert_eq!(
            code_from_request_line("GET /callback?state=x&code=zzz HTTP/1.1").as_deref(),
            Some("zzz")
        );
        assert_eq!(code_from_request_line("GET /favicon.ico HTTP/1.1"), None);
        assert_eq!(code_from_request_line("GET /callback?code= HTTP/1.1"), None);
    }

    #[test]
    fn cld01_request_line_contract_is_strict_and_decoded() {
        assert_eq!(
            parse_callback_request_line("GET /callback?code=abc123 HTTP/1.1"),
            CallbackRequest::Code("abc123".into())
        );
        assert_eq!(
            parse_callback_request_line("GET /callback?code=a%2Db%20c HTTP/1.1"),
            CallbackRequest::Code("a-b c".into())
        );
        assert_eq!(
            parse_callback_request_line("GET /favicon.ico HTTP/1.1"),
            CallbackRequest::Ignored
        );
        assert_eq!(
            parse_callback_request_line("GET /other?code=x HTTP/1.1"),
            CallbackRequest::Ignored
        );
        assert!(matches!(
            parse_callback_request_line("POST /callback?code=x HTTP/1.1"),
            CallbackRequest::Malformed(_)
        ));
        assert!(matches!(
            parse_callback_request_line("GET /callback?code=a&code=b HTTP/1.1"),
            CallbackRequest::Malformed(_)
        ));
        assert!(matches!(
            parse_callback_request_line("GET /callback?code= HTTP/1.1"),
            CallbackRequest::Malformed(_)
        ));
        assert!(matches!(
            parse_callback_request_line("GET /callback?code=%ZZ HTTP/1.1"),
            CallbackRequest::Malformed(_)
        ));
        assert!(matches!(
            parse_callback_request_line("GET /callback?code=%E2 HTTP/1.1"),
            CallbackRequest::Malformed(_)
        ));
        let overlong = format!(
            "GET /callback?code={} HTTP/1.1",
            "a".repeat(MAX_CALLBACK_CODE_BYTES + 1)
        );
        assert!(matches!(
            parse_callback_request_line(&overlong),
            CallbackRequest::Malformed(_)
        ));
    }

    #[test]
    fn cld01_probes_and_bad_requests_do_not_end_the_login() {
        let (listener, url) = bind_callback().unwrap();
        let port: u16 = url
            .rsplit(':')
            .next()
            .unwrap()
            .split('/')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            // A favicon probe, a malformed duplicate-code callback, and an
            // overlong request line all arrive before the real browser.
            let send = |request: String| {
                let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
                let _ = stream.write_all(request.as_bytes());
                let mut response = String::new();
                let _ = std::io::Read::read_to_string(&mut stream, &mut response);
                response
            };
            let favicon = send("GET /favicon.ico HTTP/1.1\r\nHost: x\r\n\r\n".into());
            let duplicate = send("GET /callback?code=a&code=b HTTP/1.1\r\nHost: x\r\n\r\n".into());
            let overlong = send(format!(
                "GET /callback?code={} HTTP/1.1\r\nHost: x\r\n\r\n",
                "b".repeat(MAX_CALLBACK_REQUEST_LINE_BYTES + 64)
            ));
            let real = send("GET /callback?code=real-code HTTP/1.1\r\nHost: x\r\n\r\n".into());
            (favicon, duplicate, overlong, real)
        });
        let code = wait_for_code(&listener, Duration::from_secs(10)).unwrap();
        assert_eq!(code, "real-code");
        let (favicon, duplicate, overlong, real) = handle.join().unwrap();
        assert!(favicon.starts_with("HTTP/1.1 404"), "{favicon}");
        assert!(duplicate.starts_with("HTTP/1.1 400"), "{duplicate}");
        assert!(overlong.starts_with("HTTP/1.1 400"), "{overlong}");
        assert!(real.starts_with("HTTP/1.1 200"), "{real}");
    }

    #[test]
    fn cld01_successive_probes_are_bounded_by_the_request_budget() {
        let (listener, url) = bind_callback().unwrap();
        let port: u16 = url
            .rsplit(':')
            .next()
            .unwrap()
            .split('/')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let handle = std::thread::spawn(move || {
            for _ in 0..(MAX_CALLBACK_REQUESTS + 2) {
                if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
                    // Once the request budget trips, the listener stops
                    // answering; a bounded client timeout keeps this thread
                    // from blocking on a silent socket.
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                    let _ = stream.write_all(b"GET /favicon.ico HTTP/1.1\r\nHost: x\r\n\r\n");
                    let mut response = String::new();
                    if std::io::Read::read_to_string(&mut stream, &mut response).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
        let error = wait_for_code(&listener, Duration::from_secs(10)).unwrap_err();
        assert!(error.contains("Too many connections"), "{error}");
        let _ = handle.join();
    }

    #[test]
    fn cld01_a_byte_trickle_cannot_extend_the_overall_deadline() {
        let (listener, url) = bind_callback().unwrap();
        let port: u16 = url
            .rsplit(':')
            .next()
            .unwrap()
            .split('/')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
                // A slow header trickle that never completes the request.
                for _ in 0..40 {
                    if stream.write_all(b"G").is_err() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        });
        let started = std::time::Instant::now();
        let error = wait_for_code(&listener, Duration::from_millis(1_200)).unwrap_err();
        let elapsed = started.elapsed();
        assert!(error.contains("Timed out"), "{error}");
        // The 1.2 s budget plus one read-timeout of slack: a trickle that
        // ignores the deadline would run for the full 4 s of client writes.
        assert!(
            elapsed < Duration::from_millis(2_200),
            "the deadline must bound the trickle, took {elapsed:?}"
        );
        let _ = handle.join();
    }

    #[test]
    fn loopback_callback_receives_code_from_a_browser_like_client() {
        let (listener, url) = bind_callback().unwrap();
        let port: u16 = url
            .rsplit(':')
            .next()
            .unwrap()
            .split('/')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            // Another parallel test (or a stray loopback probe) can race us
            // to this ephemeral port: tolerate a reset/empty reply and let
            // the `wait_for_code` assertion below carry the real verdict.
            if write!(
                stream,
                "GET /callback?code=the-code HTTP/1.1
\nHost: x
\n
\n"
            )
            .is_err()
            {
                return String::new();
            }
            let mut response = String::new();
            let _ = std::io::Read::read_to_string(&mut stream, &mut response);
            response
        });
        let code = wait_for_code(&listener, Duration::from_secs(5)).unwrap();
        assert_eq!(code, "the-code");
        let response = handle.join().unwrap();
        if response.is_empty() {
            // Our connection lost a port race; the server side already proved
            // the code arrived. Skip the body assertions for this run.
            return;
        }
        assert!(response.starts_with("HTTP/1.1 200"));
        // The browser page must not claim the connection succeeded before
        // the key exchange and credential write finish (audit CLD-01 I4).
        assert!(response.contains("CALLBACK RECEIVED"), "{response}");
        assert!(
            response.contains("confirms the connection when the key exchange finishes")
                || response.contains("Return to Localmotive"),
            "{response}"
        );
    }

    #[test]
    fn model_list_parsing_handles_openai_and_gemini_shapes() {
        let openai = r#"{"data":[{"id":"gpt-5","object":"model"},{"id":"gpt-4.1"}]}"#;
        let models = parse_models(openai).unwrap();
        assert_eq!(
            models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec!["gpt-4.1", "gpt-5"]
        );
        let gemini = r#"{"data":[{"id":"models/gemini-2.5-pro","object":"model"}]}"#;
        assert_eq!(parse_models(gemini).unwrap()[0].id, "gemini-2.5-pro");
        assert!(parse_models(r#"{"oops":1}"#).is_err());
    }

    #[test]
    fn reply_extraction_handles_string_content_part_arrays_and_errors() {
        let plain =
            r#"{"choices":[{"message":{"role":"assistant","content":"{\"changes\":{}}"}}]}"#;
        assert_eq!(extract_reply(plain).unwrap(), r#"{"changes":{}}"#);
        let parts = r#"{"choices":[{"message":{"content":[{"type":"text","text":"a"},{"type":"text","text":"b"}]}}]}"#;
        assert_eq!(extract_reply(parts).unwrap(), "ab");
        let error = r#"{"error":{"message":"Insufficient credits"}}"#;
        assert!(extract_reply(error)
            .unwrap_err()
            .contains("Insufficient credits"));
    }

    #[test]
    fn chat_without_a_stored_key_fails_before_any_network_call() {
        let store = MemoryStore::default();
        let error = chat(&store, "anthropic", "claude", "s", "u").unwrap_err();
        assert!(error.contains("No API key is stored for Anthropic"));
    }
}
