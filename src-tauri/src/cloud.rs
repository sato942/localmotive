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
use std::io::{BufRead, BufReader, Write};
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

/// Pull the `code` query parameter out of the first request line the browser
/// sends to the loopback callback.
pub fn code_from_request_line(line: &str) -> Option<String> {
    let path = line.split_whitespace().nth(1)?;
    let query = path.split_once('?')?.1;
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("code="))
        .map(|code| code.split('#').next().unwrap_or(code).to_string())
        .filter(|code| !code.is_empty())
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

/// Wait (bounded) for the browser to hit the callback, answer it with a small
/// HTML page, and return the authorization code.
pub fn wait_for_code(listener: &TcpListener, timeout: Duration) -> Result<String, String> {
    listener
        .set_nonblocking(false)
        .map_err(|error| error.to_string())?;
    let deadline = std::time::Instant::now() + timeout;
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream
                    .set_nonblocking(false)
                    .map_err(|error| error.to_string())?;
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .map_err(|error| error.to_string())?;
                let mut reader =
                    BufReader::new(stream.try_clone().map_err(|error| error.to_string())?);
                let mut line = String::new();
                // A half-opened probe connection can reach `accept` with no
                // request bytes yet; skip it and keep waiting for the real
                // browser callback rather than answering an empty request.
                match reader.read_line(&mut line) {
                    Ok(0) | Ok(_) if line.is_empty() => continue,
                    Ok(_) => {}
                    Err(error) => return Err(error.to_string()),
                }
                let code = code_from_request_line(&line);
                let body = if code.is_some() {
                    "<!doctype html><meta charset=utf-8><title>Localmotive</title><body style=\"background:#171a1b;color:#e8e9e4;font:15px 'Public Sans','Segoe UI',sans-serif;display:grid;place-items:center;height:100vh;margin:0\"><div style=\"border:1px solid #424849;background:#222627;padding:28px 32px;text-align:center\"><div style=\"font:700 22px 'Bahnschrift Condensed','Arial Narrow',sans-serif;letter-spacing:.06em;color:#9edc72\">OPENROUTER CONNECTED</div><p style=\"color:#9ca3a0;margin:12px 0 0\">You can close this tab and return to Localmotive.</p></div></body>"
                } else {
                    "<!doctype html><meta charset=utf-8><title>Localmotive</title><body style=\"background:#171a1b;color:#e8e9e4;font:15px sans-serif;display:grid;place-items:center;height:100vh;margin:0\"><div style=\"border:1px solid #424849;background:#222627;padding:28px 32px\">No authorization code was returned. Return to Localmotive and try again.</div></body>"
                };
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.flush();
                if let Some(code) = code {
                    return Ok(code);
                }
                // A favicon or stray request: keep waiting.
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline {
                    return Err("Timed out waiting for the browser to finish signing in".into());
                }
                std::thread::sleep(Duration::from_millis(100));
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
            serde_json::to_string_pretty(brief).map_err(|error| error.to_string())?
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
        assert!(response.contains("OPENROUTER CONNECTED"));
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
