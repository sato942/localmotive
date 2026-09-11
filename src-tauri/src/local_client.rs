//! The one Rust-owned HTTP client for the locally launched `llama-server`
//! (audit MT-06). It is built from the validated launch profile, honors the
//! profile's TLS certificate as an explicit trusted root (never disabling
//! verification) and its API-key file (read only in Rust), and applies
//! bounded request/response sizes with whole-operation deadlines that cover
//! connection, writes and reads.
//!
//! Secrets never leave this module: the API key is stored without a Debug
//! representation, and it is never placed in events, logs, or manifest
//! command arguments.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const MAX_LOCAL_REQUEST_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_LOCAL_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;
/// Cancellable calls retry in short slices so a cancel flag is observed
/// during connection and response waits rather than only between them.
pub const CANCEL_ATTEMPT_SLICE: Duration = Duration::from_millis(500);
const MAX_KEY_FILE_BYTES: u64 = 64 * 1024;
const MAX_CERT_FILE_BYTES: u64 = 1024 * 1024;

struct Inner {
    scheme: &'static str,
    host: String,
    connect_host: String,
    port: u16,
    api_key: Option<String>,
    client: reqwest::blocking::Client,
}

/// Cloneable handle to the centralized local-server client.
#[derive(Clone)]
pub struct LocalHttpClient {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for LocalHttpClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalHttpClient")
            .field("scheme", &self.inner.scheme)
            .field("host", &self.inner.host)
            .field("port", &self.inner.port)
            .field(
                "api_key",
                &self.inner.api_key.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

fn normalized_connect_host(host: &str) -> String {
    match host {
        "0.0.0.0" => "127.0.0.1".into(),
        "::" | "[::]" => "::1".into(),
        value => value.to_string(),
    }
}

fn validate_host(host: &str) -> Result<(), String> {
    if host.contains(['\r', '\n']) {
        return Err("The local server host contains a control character".into());
    }
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return Err("The local server host is empty".into());
    }
    if trimmed.contains(['/', ' ']) || trimmed.contains("://") {
        return Err(format!(
            "The local server host is not a plain host name: {trimmed}"
        ));
    }
    Ok(())
}

impl LocalHttpClient {
    /// A plain, unauthenticated client for callers that only hold host/port.
    pub fn plain(host: &str, port: u16) -> Result<Self, String> {
        Self::build(host, port, None, None)
    }

    /// Pre-launch transport validation (audit MT-06 I4): the host/port are
    /// usable, the API-key file exists with a single key line, and the TLS
    /// certificate/key pair is readable PEM. A failure rejects the
    /// combination with a precise message before any server starts.
    pub fn validate_transport_files(profile: &crate::core::LaunchProfile) -> Result<(), String> {
        validate_host(&profile.host)?;
        if profile.port == 0 {
            return Err("The local server port is not set".into());
        }
        if !profile.api_key_file.trim().is_empty() {
            let raw = read_bounded_file(
                profile.api_key_file.trim(),
                MAX_KEY_FILE_BYTES,
                "API key file",
            )?;
            let raw = String::from_utf8(raw)
                .map_err(|_| "The API key file is not valid UTF-8".to_string())?;
            let key = raw.trim();
            if key.is_empty() {
                return Err("The API key file is empty".into());
            }
            if key.contains(['\r', '\n']) {
                return Err(
                    "The API key file must contain a single key without line breaks".into(),
                );
            }
        }
        let certificates = [
            (
                profile.ssl_cert_file.trim(),
                "SSL certificate",
                "BEGIN CERTIFICATE",
            ),
            (profile.ssl_key_file.trim(), "SSL private key", "BEGIN"),
        ];
        for (path, label, marker) in certificates {
            if path.is_empty() {
                continue;
            }
            let bytes = read_bounded_file(path, MAX_CERT_FILE_BYTES, label)?;
            let text = String::from_utf8(bytes)
                .map_err(|_| format!("The {label} is not valid UTF-8 PEM"))?;
            if !text.contains(marker) {
                return Err(format!(
                    "The {label} does not look like PEM data ({path}); expected a {marker} block"
                ));
            }
        }
        if !profile.ssl_cert_file.trim().is_empty() && !profile.ssl_key_file.trim().is_empty() {
            // The pair must parse together; a mismatched pair would otherwise
            // fail after launch as a plaintext health timeout.
            let cert = read_bounded_file(
                profile.ssl_cert_file.trim(),
                MAX_CERT_FILE_BYTES,
                "SSL certificate",
            )?;
            let key = read_bounded_file(
                profile.ssl_key_file.trim(),
                MAX_CERT_FILE_BYTES,
                "SSL private key",
            )?;
            let cert_text = String::from_utf8_lossy(&cert);
            let key_text = String::from_utf8_lossy(&key);
            if !cert_text.contains("BEGIN CERTIFICATE") {
                return Err("The SSL certificate is not a PEM certificate".into());
            }
            if !key_text.contains("PRIVATE KEY") {
                return Err("The SSL private key is not a PEM private key".into());
            }
        }
        Ok(())
    }

    /// Build from the validated launch profile: certificate pair (explicit
    /// trust) and API-key file are read here, in Rust, with size bounds.
    pub fn from_profile(profile: &crate::core::LaunchProfile) -> Result<Self, String> {
        let cert_pem = if profile.ssl_cert_file.trim().is_empty() {
            None
        } else {
            Some(read_bounded_file(
                profile.ssl_cert_file.trim(),
                MAX_CERT_FILE_BYTES,
                "SSL certificate",
            )?)
        };
        let api_key = if profile.api_key_file.trim().is_empty() {
            None
        } else {
            let raw = read_bounded_file(
                profile.api_key_file.trim(),
                MAX_KEY_FILE_BYTES,
                "API key file",
            )?;
            let raw = String::from_utf8(raw)
                .map_err(|_| "The API key file is not valid UTF-8".to_string())?;
            let key = raw.trim();
            if key.is_empty() {
                return Err("The API key file is empty".into());
            }
            if key.contains(['\r', '\n']) {
                return Err(
                    "The API key file must contain a single key without line breaks".into(),
                );
            }
            Some(key.to_string())
        };
        Self::build(&profile.host, profile.port, api_key, cert_pem)
    }

    fn build(
        host: &str,
        port: u16,
        api_key: Option<String>,
        cert_pem: Option<Vec<u8>>,
    ) -> Result<Self, String> {
        validate_host(host)?;
        if port == 0 {
            return Err("The local server port is not set".into());
        }
        let scheme = if cert_pem.is_some() { "https" } else { "http" };
        let mut builder = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(0);
        if let Some(pem) = cert_pem.as_deref() {
            // Explicit local trust policy: the profile's certificate is the
            // root for this client. Certificate verification stays enabled.
            let certificate = reqwest::Certificate::from_pem(pem)
                .map_err(|error| format!("The SSL certificate could not be parsed: {error}"))?;
            builder = builder.add_root_certificate(certificate);
        }
        let client = builder
            .build()
            .map_err(|error| format!("The local HTTP client could not be built: {error}"))?;
        Ok(Self {
            inner: Arc::new(Inner {
                scheme,
                host: host.trim().to_string(),
                connect_host: normalized_connect_host(host.trim()),
                port,
                api_key,
                client,
            }),
        })
    }

    pub fn host(&self) -> &str {
        &self.inner.host
    }

    pub fn port(&self) -> u16 {
        self.inner.port
    }

    pub fn scheme(&self) -> &'static str {
        self.inner.scheme
    }

    pub fn api_key_configured(&self) -> bool {
        self.inner.api_key.is_some()
    }

    /// The request URL, with IPv6 hosts bracketed.
    pub fn url(&self, path: &str) -> String {
        let host = self.inner.connect_host.as_str();
        let display_host = if host.contains(':') && !host.starts_with('[') {
            format!("[{host}]")
        } else {
            host.to_string()
        };
        format!(
            "{}://{display_host}:{}{}",
            self.inner.scheme, self.inner.port, path
        )
    }

    pub fn get_bytes(&self, path: &str, budget: Duration) -> Result<(u16, Vec<u8>), String> {
        self.execute("GET", path, None, budget, None)
    }

    pub fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
        budget: Duration,
    ) -> Result<(u16, Vec<u8>), String> {
        self.execute("POST", path, Some(body), budget, None)
    }

    pub fn post_json_cancellable(
        &self,
        path: &str,
        body: &serde_json::Value,
        budget: Duration,
        cancelled: &AtomicBool,
    ) -> Result<(u16, Vec<u8>), String> {
        self.execute("POST", path, Some(body), budget, Some(cancelled))
    }

    pub fn get_bytes_cancellable(
        &self,
        path: &str,
        budget: Duration,
        cancelled: &AtomicBool,
    ) -> Result<(u16, Vec<u8>), String> {
        self.execute("GET", path, None, budget, Some(cancelled))
    }

    fn execute(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
        budget: Duration,
        cancelled: Option<&AtomicBool>,
    ) -> Result<(u16, Vec<u8>), String> {
        if !path.starts_with('/') || path.contains(['\r', '\n']) {
            return Err(format!("Invalid local request path: {path}"));
        }
        let body_bytes = match body {
            Some(value) => serde_json::to_vec(value)
                .map_err(|error| format!("The request body could not be serialized: {error}"))?,
            None => Vec::new(),
        };
        if body_bytes.len() > MAX_LOCAL_REQUEST_BYTES {
            return Err(format!(
                "The local request exceeds the {MAX_LOCAL_REQUEST_BYTES}-byte limit"
            ));
        }
        let url = self.url(path);
        let started = Instant::now();
        loop {
            if let Some(flag) = cancelled {
                if flag.load(Ordering::Relaxed) {
                    return Err("The local request was cancelled".into());
                }
            }
            let remaining = budget.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(format!(
                    "llama-server did not answer within {} seconds",
                    budget.as_secs().max(1)
                ));
            }
            let attempt_budget = if cancelled.is_some() {
                remaining.min(CANCEL_ATTEMPT_SLICE)
            } else {
                remaining
            };
            let mut request = self
                .inner
                .client
                .request(
                    reqwest::Method::from_bytes(method.as_bytes())
                        .map_err(|error| error.to_string())?,
                    &url,
                )
                .timeout(attempt_budget);
            if body.is_some() {
                request = request
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body_bytes.clone());
            }
            if let Some(key) = self.inner.api_key.as_deref() {
                // The key travels only here: never into logs or events.
                request = request.header(reqwest::header::AUTHORIZATION, format!("Bearer {key}"));
            }
            match request.send() {
                Ok(response) => {
                    let status = response.status().as_u16();
                    if response
                        .content_length()
                        .is_some_and(|length| length > MAX_LOCAL_RESPONSE_BYTES)
                    {
                        return Err(format!(
                            "The local response exceeds the {MAX_LOCAL_RESPONSE_BYTES}-byte limit"
                        ));
                    }
                    use std::io::Read;
                    let mut bytes = Vec::new();
                    let mut reader = response.take(MAX_LOCAL_RESPONSE_BYTES + 1);
                    reader.read_to_end(&mut bytes).map_err(|error| {
                        format!("The local response could not be read: {error}")
                    })?;
                    if bytes.len() as u64 > MAX_LOCAL_RESPONSE_BYTES {
                        return Err(format!(
                            "The local response exceeds the {MAX_LOCAL_RESPONSE_BYTES}-byte limit"
                        ));
                    }
                    return Ok((status, bytes));
                }
                Err(error) => {
                    if error.is_timeout() && cancelled.is_some() && started.elapsed() < budget {
                        continue;
                    }
                    if error.is_timeout() {
                        return Err(format!(
                            "llama-server did not answer within {} seconds",
                            budget.as_secs().max(1)
                        ));
                    }
                    return Err(format!(
                        "The local request failed: {error}{}",
                        error_chain(&error)
                    ));
                }
            }
        }
    }
}

/// Actionable guidance for certificate failures reported through reqwest.
///
/// A certificate marked CA:TRUE is refused as a server certificate by webpki
/// ("CaUsedAsEndEntity"). `openssl req -x509` marks the certificate CA:TRUE
/// unless `basicConstraints` is overridden, so this is the common accidental
/// shape. Such a launch can never become healthy by waiting, and the raw
/// verification text does not say what to change.
pub(crate) fn certificate_guidance(message: &str) -> Option<&'static str> {
    if message.contains("CaUsedAsEndEntity") {
        return Some(
            " The SSL certificate that llama-server presents is marked as a CA (basicConstraints CA:TRUE). llama-server must present an end-entity certificate. Regenerate the pair with a subjectAltName and -addext basicConstraints=critical,CA:FALSE, then select both files again.",
        );
    }
    if message.contains("NotValidForName") || message.contains("InvalidServerName") {
        return Some(
            " The SSL certificate does not cover the host that llama-server listens on. Regenerate it with a subjectAltName for that host (for loopback: -addext subjectAltName=IP:127.0.0.1,DNS:localhost).",
        );
    }
    if message.contains("UnknownIssuer") {
        return Some(
            " The certificate llama-server presented is not the certificate configured in the profile. Select the certificate file that matches the server's --ssl-cert-file.",
        );
    }
    if message.contains("invalid peer certificate") {
        return Some(
            " The SSL certificate could not be verified. Check that the certificate file matches the key file and covers the configured host.",
        );
    }
    None
}

fn error_chain(error: &dyn std::error::Error) -> String {
    let mut parts = Vec::new();
    let mut current = error.source();
    while let Some(source) = current {
        parts.push(source.to_string());
        current = source.source();
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join("; "))
    }
}

fn read_bounded_file(path: &str, limit: u64, label: &str) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("The {label} could not be read ({path}): {error}"))?;
    if !metadata.is_file() {
        return Err(format!("The {label} is not a file: {path}"));
    }
    if metadata.len() > limit {
        return Err(format!("The {label} exceeds the {limit}-byte limit"));
    }
    std::fs::read(path).map_err(|error| format!("The {label} could not be read ({path}): {error}"))
}

#[cfg(test)]
pub(crate) mod tests {

    // Manual diagnostic (ignored): probe a live packaged TLS server with the
    // profile's own certificate as the trust root. Set LOCALMOTIVE_DIAG_CERT,
    // LOCALMOTIVE_DIAG_KEY_FILE and optionally LOCALMOTIVE_DIAG_URL_PORT.
    #[test]
    #[ignore = "manual packaged TLS probe against a live local server"]
    fn manual_tls_probe() {
        let Ok(cert) = std::env::var("LOCALMOTIVE_DIAG_CERT") else {
            eprintln!("LOCALMOTIVE_DIAG_CERT not set; skipping");
            return;
        };
        let key_file = std::env::var("LOCALMOTIVE_DIAG_KEY_FILE").unwrap_or_default();
        let port = std::env::var("LOCALMOTIVE_DIAG_URL_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let profile = crate::core::LaunchProfile {
            host: "127.0.0.1".into(),
            port,
            ssl_cert_file: cert,
            api_key_file: key_file,
            ..Default::default()
        };
        let client = match LocalHttpClient::from_profile(&profile) {
            Ok(client) => client,
            Err(error) => {
                eprintln!("BUILD ERROR: {error}");
                return;
            }
        };
        match client.get_bytes("/health", Duration::from_secs(6)) {
            Ok((status, body)) => {
                eprintln!(
                    "PROBE OK: status {status} body {}",
                    String::from_utf8_lossy(&body[..body.len().min(80)])
                );
            }
            Err(error) => eprintln!("PROBE ERROR: {error}"),
        }
    }
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    // Self-signed test pairs, generated for this suite only (CN=localhost,
    // SAN DNS:localhost,IP:127.0.0.1). The "trusted" pair is embedded as the
    // client's explicit root; the "other" pair must be rejected.
    const TRUSTED_CERT: &str = r##"-----BEGIN CERTIFICATE-----
MIIDSTCCAjGgAwIBAgIUZIjkBxN1+npmV3ts7MDTFKaOW00wDQYJKoZIhvcNAQEL
BQAwFDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTI2MDkxMTAwMTk1M1oXDTM2MDkw
ODAwMTk1M1owFDESMBAGA1UEAwwJbG9jYWxob3N0MIIBIjANBgkqhkiG9w0BAQEF
AAOCAQ8AMIIBCgKCAQEAk9pz3sphf3HAkWEmDqDWTEdXvwBzvofyWC3ic/b6nM+S
7N05XUr82M/aspzff5+cHqJOTUyWioc6jqLRPeLTMjsV3TgkjSEgOtfPG/PQJDbx
dwZHc6ZJc9oI2ITVcVKtMTtQySxSZpQBECCU2f02KOcaOj+1I67mNodiYePfZAJB
OKEbe4bcwbTimpEz2YQmlJU3c6eHQoAKYJDWFxVzoCnGuIIKInlaZyXcaqg0qVML
yBkDh07lP9vkx+TrMFnW+cL87yoTRDG6yEG8r0YMB/MaaDto5GCoPVjjWXOvlBWp
L4ifhJHJTfs0oN7QIx0pbzkruJvnmRnW0mps2FF8eQIDAQABo4GSMIGPMB0GA1Ud
DgQWBBRwjoEz3janjTX7exrl2BmiZQxIjDAfBgNVHSMEGDAWgBRwjoEz3janjTX7
exrl2BmiZQxIjDAaBgNVHREEEzARgglsb2NhbGhvc3SHBH8AAAEwDAYDVR0TAQH/
BAIwADAOBgNVHQ8BAf8EBAMCBaAwEwYDVR0lBAwwCgYIKwYBBQUHAwEwDQYJKoZI
hvcNAQELBQADggEBAHtQR/Uf67tYmulrLxbJWDPNULA8UVdwX84MwlO5895+xb+J
uks62iFHbAsMiCFmuopPud2GEVpl4R4/UHnf8gp52v0RtHi0ocEBkwnPhJ1R+eG8
20fpQqmgou3rc0RBFp3iS18ko1mBEBeW3gkdZb7olFYN0PNbvbdl27CrQOYez+wR
Ml7LjB1aaAWH6r/b1jPahYhaQ2fhgoloNs1oXgYMdirjiwOTp+J+FKmG5MOSzW/Z
Qq0mGYxHB6mJ65kPpyEMN6c5ftlUj1jpAMgrICcIuVS7ex/14LCjq2ByN0n+GkCk
zYS/Oztdd1C23CD/Ua+XD+d/NmBNDKpt1dhc/Ks=
-----END CERTIFICATE-----"##;
    const TRUSTED_KEY: &str = r##"-----BEGIN PRIVATE KEY-----
MIIEugIBADANBgkqhkiG9w0BAQEFAASCBKQwggSgAgEAAoIBAQCT2nPeymF/ccCR
YSYOoNZMR1e/AHO+h/JYLeJz9vqcz5Ls3TldSvzYz9qynN9/n5weok5NTJaKhzqO
otE94tMyOxXdOCSNISA6188b89AkNvF3Bkdzpklz2gjYhNVxUq0xO1DJLFJmlAEQ
IJTZ/TYo5xo6P7UjruY2h2Jh499kAkE4oRt7htzBtOKakTPZhCaUlTdzp4dCgApg
kNYXFXOgKca4ggoieVpnJdxqqDSpUwvIGQOHTuU/2+TH5OswWdb5wvzvKhNEMbrI
QbyvRgwH8xpoO2jkYKg9WONZc6+UFakviJ+EkclN+zSg3tAjHSlvOSu4m+eZGdbS
amzYUXx5AgMBAAECgf8DpcCelsAtTOZUQMKvltB4K/Mi/NLjNMghU3VHwIOaN2Zt
jZwjWFqn7JcQwGBeK1DxuKMmvARZMSmN0M5vA2eD5CO9vYltFqcuw8lQRyEiDpPM
bqY4etVx5nxvqqv59HJC7QtMSmXx7IXsCcFHdCl1tvfcpLfdXLE63IKOqO3anS8O
Qei+V3BQwyOma5Ps74od1QnAO68/b2mMKMtOLjz87moccM96zeycxNcbXwvCi9a9
QBLn/1olo6eyIFnQG9Rv56GHi2D5pCiQWgV8r1HLyo9BK2z5fStvp7FsqeNGgSU5
rpx10uJgVe3gjTy4VrhFoS8WGPs4yB3ZuSFHrOECgYEAzzG3242k1pMqVALJo5Qe
Nf8OY3wQiHb9Xop+/z+2x/N25jiIT6Md5XuqOTU6BgvPPV1niG4+/sgPS54wogj+
LmHt498cRlu6iZq6Vnnu2B5GXFY1A3UFWDcl7sPN2WIBzs3A++58fnGdyPRib3/5
awxk+HdrrVYMjGpgJL0rgL0CgYEAtq5YLUcPY6uIOV5GUWKYK44cMV6qOYQWKKe3
8Y/iqx6Ewllt1e6nr0XoYE5tApaGI8kqNmicmy/ecL89A628lIXQgqnT+SifkFBY
YVQWTwGd3pUfWL0mpoddyffGSBw9QdLZdPB4e502UTwL1sMsSg3e1g/7eQTzIJEE
yvKDHG0CgYB5iKZaKKmqG8TWZpQ0WWunLKcZ/+oKwhE45XA89PiYLN1viXWbkQi2
VWWyDOCuLzsuuZ1DT7ev51Xhezb3tOKz/kl2QqbpNmEi2hm3I+rP5mJLQ13xWzD5
X8/mUABtJKn+zn4GyQtJeAefmooq8RwyiKCphhMpJ7JQow0mP7hG2QKBgH7rCeAL
Kqn4lqAk233XnhlEllnGh0WGe06rl6SAbt6sSVgtgZ3MPTwMubGPSzUtFuzt6iH6
9DLtQwHaG63emtIlVgQxsU+95X4CFCUqooUpfmESAcFJSP0RtxuGxX97/yMoNmSE
XXCkfQRbT25aEv8wO81FNVWTFsddeZL2ghjRAoGAKyM375BlPfSp5k4Gaip6zKUE
cz5Z4j60sHk98Ux/xJ9gLEulOoxue3ZvAJWt8k2IoOW/Wa02Q+JPmrwbMt9pLch/
1VtDm/IeXHVYQADh/8npODrQiA77otLYSx4U2BOzK2IPSg4NJDNtNHWCB9+U06Ga
d4VIuUiHXUqjCSUi+0E=
-----END PRIVATE KEY-----"##;

    // A certificate exactly as `openssl req -x509` produces it by default: a
    // self-signed CA-marked certificate (basicConstraints critical CA:TRUE)
    // with a subjectAltName. webpki refuses it as a server certificate
    // ("CaUsedAsEndEntity"), which is why the health wait must fail fast with
    // actionable guidance instead of timing out after ten minutes.
    pub(crate) const CA_TRUE_CERT: &str = r##"-----BEGIN CERTIFICATE-----
MIIDGjCCAgKgAwIBAgIUYlcPXMKGRtkz63lXu+dcd52xjUwwDQYJKoZIhvcNAQEL
BQAwFDESMBAGA1UEAwwJMTI3LjAuMC4xMB4XDTI2MDkxMTE1NDU0MVoXDTM2MDkw
ODE1NDU0MVowFDESMBAGA1UEAwwJMTI3LjAuMC4xMIIBIjANBgkqhkiG9w0BAQEF
AAOCAQ8AMIIBCgKCAQEAtKbvlfhyWioCef6wP1cOahkjU8ISerg6aUgok8cg8Co4
Az85WuD/fN4rSaWa5BTl4cdU/i8hVvzkdeKeQwnYybl2oZgFAp8FH8x0u3uiBrc7
DKG9ESRrtuBZcmJcM6cJYThS/6y66tLLlYx5jAyujdw3Ea+ArI+2CSOErdnKVIwZ
DifbhVin7L6VI2sqff61NLBhgdxIiEeH2xhg03jA0UIibi+6H+nnjPLYz5o1LLio
uDDIgNM2X7Lw6w18/IDsGq+PwqUmGnEnk0Jfour9MuEwI3W0QqwFBmN6kdSKTVcU
kYYPKkUWQkoPLf59FDiw3lah+sYrik0hyh8HjOddEwIDAQABo2QwYjAdBgNVHQ4E
FgQUOmOt8v081fr8F4ltUvXRfDAdKz8wHwYDVR0jBBgwFoAUOmOt8v081fr8F4lt
UvXRfDAdKz8wDwYDVR0TAQH/BAUwAwEB/zAPBgNVHREECDAGhwR/AAABMA0GCSqG
SIb3DQEBCwUAA4IBAQCOt/0kalctL2FIOZJ2LbOAMuDA0XB7CGZq2P2/a3yvNdoV
IriuAkrDKBTU77l0DxNLzQZ30NTiqzytoQujmsNgYLpcJ7Al97bbZGPR0a1e3F/Z
XdAwn4lMc0Q1ZPCuv90HBd37ERw7UdU/sG4tPiLMTMzoStu6SyA44AROXDpnkQQT
SYxZa1gdvXvvEGVB0lbR5crfmqzCycM4Yd1rxBUHeG4QqojP8XUdG3nLIeqk0qYl
3dCUMVF42UjKKOqqfCs8bcMl6M50Y3vAJ4wQCIUV49Y4gYKueTHCctLkNTbJ7DxE
Lm4OI42L8fBHU9OjG+mhIUL1ib8EJrxSIIuN32RM
-----END CERTIFICATE-----"##;
    pub(crate) const CA_TRUE_KEY: &str = r##"-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQC0pu+V+HJaKgJ5
/rA/Vw5qGSNTwhJ6uDppSCiTxyDwKjgDPzla4P983itJpZrkFOXhx1T+LyFW/OR1
4p5DCdjJuXahmAUCnwUfzHS7e6IGtzsMob0RJGu24FlyYlwzpwlhOFL/rLrq0suV
jHmMDK6N3DcRr4Csj7YJI4St2cpUjBkOJ9uFWKfsvpUjayp9/rU0sGGB3EiIR4fb
GGDTeMDRQiJuL7of6eeM8tjPmjUsuKi4MMiA0zZfsvDrDXz8gOwar4/CpSYacSeT
Ql+i6v0y4TAjdbRCrAUGY3qR1IpNVxSRhg8qRRZCSg8t/n0UOLDeVqH6xiuKTSHK
HweM510TAgMBAAECggEADKCinKQKMj0/gRGJdlP6gPYS3xbwvb1E7/kIRRQlPERn
N+ricnTJxwuskPBPfGPtkbOiQEZBGViCC690ipEUoz0girkamI1PCWL8QeKpd7i1
GvPvSFR4ZwcVmYZAlae2YyJRwudrBWEItAJmuKBmTyo2ezj+UJGXEtp1usU/fFtU
PFsFTqPlFLKtAfGiKyK4/f53GqBCKcFXLN5uSiTbbf5xPQiycwitBZ1/aIe3ui+Q
XgBiPEEfo/4zPBr6J0h+rT68DFcSH/leHmaOQLfGInvaBQKYDPlgzbpd0WwjusAd
xZhuqRFicQnCnZvHqAaUYWvkpDpQXwEHM44nLtUvWQKBgQD5TK1NX9L7sizqQjMQ
mVjlNOaTOLOxirmjNVfnF44iwbi13iZIXJgc18DqrNuFkN6mM31n/HZ/RguVU8RS
r4izbKoop5/te4XU6/8uPDC5o9HppDQIGQ0YRTeSquCt9ayFsJE78BItoeYBX7jS
OlvOp2rCF7/qGsaLGoHm19uxFQKBgQC5gezpJJ3ivMo8z79xh9UGx8Saa9d1oKYJ
2pVTMtPI/LYeE4h+zHoxLlIBeD0ignbl/IcavUlxBDQdQ6xJhlNBzaC9uFMKSpBz
sXONFwl1Vg9NmhoanIxUBTo8vo0x9hpoHK8RnuXC/l/wHHx5Sy4rs1uGd56rw2JA
0S39uanPhwKBgQDtJGAyGvXykPGiwOgcYRKrrZ+r6aMdPs4Jj2OXotOFAmv3LGOU
L+hOf3m2gkmrizwQMyiWsxPxS6sXGADHesx5iONwGsvJttd+zCMIUx8yZ7/1FUqd
bV8EeEs9zCg/slOzNFti/aH9IGVPZ0PDTtooAR9PlBHt2hyFE+j/stP7ZQKBgQCw
ZhLQ9AfKprkssGQcYgy4wNd7+9ZLPTMGJbte/PMUqPHIkcx2vpvnDmPej+aaXTMQ
qVwTmjEu7c9ckJBQ7hFXfmA+Z/tWyuanjPMTE/fjgq1Unpf5/CkYcEwbnRsIijw8
CiKTf+R90oOKAJyAfnPuDESZDkBsloNknUS9g4ItGwKBgQCU/0IxUY0otQBvQ6VW
o9Ebz1y4NVZJXyDte8SEpw4Ftd3invM3SbnpO/C+LC4tJV+PHH8EEX1pxDFv0Ukv
TYlofGTiQ3pg+Fn/YZY7kvsZeNWrXpsPtKQD1rooPWuApyimkX+G8/2P6d9hCAdY
y/xLb7CbIroYVJeLpVZRb5cajw==
-----END PRIVATE KEY-----"##;

    const OTHER_CERT: &str = r##"-----BEGIN CERTIFICATE-----
MIIDSTCCAjGgAwIBAgIUag6z61lhNaiiCPtmQOLi4yzH1/cwDQYJKoZIhvcNAQEL
BQAwFDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTI2MDkxMTAwMTk1M1oXDTM2MDkw
ODAwMTk1M1owFDESMBAGA1UEAwwJbG9jYWxob3N0MIIBIjANBgkqhkiG9w0BAQEF
AAOCAQ8AMIIBCgKCAQEA6CxJJrszAJKUgz4QKafuuXw+cBzjmkhYCb/mdgJ4a2ip
203KxjYHme6CpEQRtrwcvYG+GMALjYexEXky3GJ5/Ezx/B4299fqORM4UWLJUFJT
m9Msxj7FRQsoRifFuZ3ns5X0qdAhgi8Na9n0jzsGKjsE813QTSIuryQusaORwwYd
arVwZxQX3p7sHTVxv8RAhmrC+2n501WOtk2Z5eDPmaWLrIXcT25nGl+rCotxfJuS
a2Xincy0/5OTfZ/tlAlXVprhGiwbiRvm6tPyuzQp/oCfz9nDtpcSEvSMerFQFCRz
bmE1dBSudhaEF0kE4r3TW0gSJo2MFqK+hqxxuzZjuQIDAQABo4GSMIGPMB0GA1Ud
DgQWBBRA8IB4j6yrpv/VzHOuoA7pPIDL0DAfBgNVHSMEGDAWgBRA8IB4j6yrpv/V
zHOuoA7pPIDL0DAaBgNVHREEEzARgglsb2NhbGhvc3SHBH8AAAEwDAYDVR0TAQH/
BAIwADAOBgNVHQ8BAf8EBAMCBaAwEwYDVR0lBAwwCgYIKwYBBQUHAwEwDQYJKoZI
hvcNAQELBQADggEBAJ+UrMGFJWNmx7a9ya0yHv0rXFqkPRoM2a0egPWlpBYubwJn
Zpk/wC5KrmdUPSVAjXmRq2nfHCLF5zab7QaDbeDgsxflELcfK4+sPK/gnVl8Scb9
6/hOjiDlobBoDIV9gkvt/Yf8pqnVkR8hNHJh2C3DyCiuSS9jzXqcsyJM30e6nkWY
UkDXOzp5NJGM398KMKFEK0eROfMioIvZPrK/mK7q1BHmLBKS8XxYUXWEyyEFwtAs
KP6lD8SnROVhqM8AtrTxC9ytH742zj6SFYLODz/Bdq5k7xbkAPAIQrP/gtolzGdA
pAW0M3hGy198gjSqBi3Xr+sPBREXowBO99/L5dg=
-----END CERTIFICATE-----"##;
    const OTHER_KEY: &str = r##"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDoLEkmuzMAkpSD
PhApp+65fD5wHOOaSFgJv+Z2AnhraKnbTcrGNgeZ7oKkRBG2vBy9gb4YwAuNh7ER
eTLcYnn8TPH8Hjb31+o5EzhRYslQUlOb0yzGPsVFCyhGJ8W5neezlfSp0CGCLw1r
2fSPOwYqOwTzXdBNIi6vJC6xo5HDBh1qtXBnFBfenuwdNXG/xECGasL7afnTVY62
TZnl4M+ZpYushdxPbmcaX6sKi3F8m5JrZeKdzLT/k5N9n+2UCVdWmuEaLBuJG+bq
0/K7NCn+gJ/P2cO2lxIS9Ix6sVAUJHNuYTV0FK52FoQXSQTivdNbSBImjYwWor6G
rHG7NmO5AgMBAAECggEAA5lsKOWODNw44S/9ICgyUz3R2fsYrDOi6cPH1tyC8WdV
+shh9GCyDXjdHZ7Qh3yzFV0FjWyswSzxWcv/Ndtw+LBYsAfn4j++qdPac6iMZmpg
UXIcp5YhiMh7f1rufcos5WPVvywy2MnR83IVkILhvZXcpck2iXuWLbDp6GcYw2U1
UBpG4sZGVNvUapWEq+7rnAuSf98hiq8N8D8BL1EcKbWmHVIXQzUf77wVMmiwKLh5
fX/KWPMdXaO7Nvm2kjUZ7Vi6zKW7xAmluGdD3MlUUcov46uZrv0XD3gVmb8swMZ2
94afrO/UBHKGajxcE23kzeoT+QSmQkKJjxNYCpZAYQKBgQD4MlW9oEkCzh8sJYmp
247Oy8dJtCYRFVA1xteON2LbAxBNzaN8+7rEfM2SOfpkR04ZYXc4iZYGDFmWdsOx
S4C9AeU+sPaBEKJ9qolC7VUEzMUP/jt5MpnZr5sJEkTyVh/9sJjx4GujDoN3197Q
p4BV7CiHnbYPB2gD3nCp7ohJdwKBgQDvePsqctjp8bfemnWLDaJo6zkBe0ONk4o6
4Br59nsQGSZrfA+UaskEKakLFaKL2Gsm51eFm+h+YdD0SCNnpFUq2RRWeTN7ycGI
/IQRjhNGrRHQ3NViDV53vkYYHKMrXHNkGx0gaXRlCTOWIn7Jcy9SJNJ3nNersk3s
pPt/wQQITwKBgGj4sPbAgeCj2N9NCQpECARCf4kWnjr/bqsv7B8EIdVLWGvvm0PT
G8zak/9ScipTVh658DiDSGZKduGCXYXwzwQhdxmqsrcnl/HaXK9nvVuIV5hKCFFc
K2G8Oa/+gBaWgnVDaYxzRFL0YnofXOeW0FqGxSWGeGem1EE+pRvZ/N5FAoGAbp8S
Vz+KWDdi2p+7YKrBtXnDcZ3BTOs01ZGkpIdpvbwAdXJvt/3EMfoUrpwl5Dfq96Oy
WHP26DrDTbTtNflBpnN046VFVQ+UKXWMhJd+7A0Sx8rbf1nxo5rvwj+oWGVyoHGt
+MT+EZY4kEgLDX/6AhYka0C2mAfb208zJobBGB0CgYEA61+4kf8EP+P/oe07qFsA
xIb/Em1qFT86dKqq7zFImPcX3Q7VeikLzdiMJqklVsTCROtUsBa0wynJYG1m+5Xl
UxGnOU83sS7Rbdtjwpp1y3a/CFg4EoFM7GkMeYhKNAjjqmmWCCiTLc+oz53gkySA
ab1VTmVlluUDakDfjhwCcnE=
-----END PRIVATE KEY-----"##;

    fn pem_der(pem: &str) -> Vec<u8> {
        use base64::Engine;
        let body: String = pem
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect();
        base64::engine::general_purpose::STANDARD
            .decode(body.trim())
            .expect("test pem decodes")
    }

    fn tls_pair(
        cert: &str,
        key: &str,
    ) -> (
        Vec<rustls::pki_types::CertificateDer<'static>>,
        rustls::pki_types::PrivateKeyDer<'static>,
    ) {
        let certs = vec![rustls::pki_types::CertificateDer::from(pem_der(cert))];
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(pem_der(key).into());
        (certs, key)
    }

    /// One-shot plain HTTP fixture; returns the port and the captured request.
    pub(crate) fn serve_plain(response: Vec<u8>, capture: bool) -> (u16, Arc<Mutex<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(Mutex::new(String::new()));
        let seen_thread = seen.clone();
        thread::spawn(move || {
            for incoming in listener.incoming() {
                let Ok(mut stream) = incoming else { break };
                stream
                    .set_read_timeout(Some(Duration::from_millis(500)))
                    .ok();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                if capture && seen_thread.lock().unwrap().is_empty() {
                    if let Ok(text) = String::from_utf8(buffer.to_vec()) {
                        *seen_thread.lock().unwrap() = text;
                    }
                }
                if response.is_empty() {
                    // A genuinely silent server: hold the connection open so
                    // the client observes a wait, not a premature close.
                    thread::sleep(Duration::from_millis(3500));
                } else {
                    let _ = stream.write_all(&response);
                    let _ = stream.flush();
                }
            }
        });
        (port, seen)
    }

    /// One-shot TLS fixture speaking HTTP/1.1 through rustls.
    pub(crate) fn serve_tls(cert: &str, key: &str, response: Vec<u8>) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (certs, private_key) = tls_pair(cert, key);
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .unwrap();
        let config = Arc::new(config);
        thread::spawn(move || {
            for incoming in listener.incoming() {
                let Ok(mut stream) = incoming else { break };
                let connection = rustls::ServerConnection::new(config.clone());
                let Ok(mut tls) = connection else { continue };
                stream
                    .set_read_timeout(Some(Duration::from_millis(2000)))
                    .ok();
                let mut handshake_failed = false;
                while tls.is_handshaking() {
                    if tls.wants_write() && tls.write_tls(&mut stream).is_err() {
                        handshake_failed = true;
                        break;
                    }
                    if tls.wants_read() && tls.read_tls(&mut stream).is_err() {
                        handshake_failed = true;
                        break;
                    }
                    if tls.process_new_packets().is_err() {
                        handshake_failed = true;
                        break;
                    }
                }
                if handshake_failed {
                    continue;
                }
                let _ = tls.writer().write_all(&response);
                let _ = tls.write_tls(&mut stream);
                // Drain the client's final handshake/close bytes before
                // dropping the socket; closing with unread data would send
                // an RST and the client would report an aborted connection.
                for _ in 0..8 {
                    match tls.read_tls(&mut stream) {
                        Ok(0) => break,
                        Ok(_) => {
                            let _ = tls.process_new_packets();
                        }
                        Err(_) => break,
                    }
                }
            }
        });
        port
    }

    fn ok_response(body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .into_bytes()
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "localmotive-local-client-{}-{}",
            std::process::id(),
            name
        ));
        dir
    }

    fn profile_with(host: &str, port: u16) -> crate::core::LaunchProfile {
        crate::core::LaunchProfile {
            host: host.to_string(),
            port,
            ..crate::core::LaunchProfile::default()
        }
    }

    #[test]
    fn mt06_plain_client_normalizes_hosts_and_brackets_ipv6() {
        let client = LocalHttpClient::plain("0.0.0.0", 8080).unwrap();
        assert_eq!(client.url("/health"), "http://127.0.0.1:8080/health");
        let client = LocalHttpClient::plain("::1", 8081).unwrap();
        assert_eq!(client.url("/health"), "http://[::1]:8081/health");
        let client = LocalHttpClient::plain("[::1]", 8082).unwrap();
        assert_eq!(client.url("/tokenize"), "http://[::1]:8082/tokenize");
        assert_eq!(
            LocalHttpClient::plain("127.0.0.1", 1).unwrap().scheme(),
            "http"
        );
    }

    #[test]
    fn mt06_invalid_hosts_and_ports_are_rejected() {
        assert!(LocalHttpClient::plain("", 8080).is_err());
        assert!(LocalHttpClient::plain("http://evil", 8080).is_err());
        assert!(LocalHttpClient::plain("bad host", 8080).is_err());
        assert!(LocalHttpClient::plain("evil\r", 8080).is_err());
        assert!(LocalHttpClient::plain("127.0.0.1", 0).is_err());
    }

    #[test]
    fn mt06_api_key_file_is_read_in_rust_and_redacted_in_debug() {
        let key_path = temp_path("key.txt");
        std::fs::write(&key_path, "super-secret-canary\n").unwrap();
        let mut profile = profile_with("127.0.0.1", 8080);
        profile.api_key_file = key_path.to_string_lossy().to_string();
        let client = LocalHttpClient::from_profile(&profile).unwrap();
        assert!(client.api_key_configured());
        let rendered = format!("{client:?}");
        assert!(
            !rendered.contains("super-secret-canary"),
            "key leaked: {rendered}"
        );
        assert!(rendered.contains("REDACTED"));
        let _ = std::fs::remove_file(&key_path);
    }

    #[test]
    fn mt06_missing_or_empty_or_multiline_key_files_are_rejected() {
        let mut profile = profile_with("127.0.0.1", 8080);
        profile.api_key_file = temp_path("missing.txt").to_string_lossy().to_string();
        let error = LocalHttpClient::from_profile(&profile).unwrap_err();
        assert!(error.contains("API key file"), "{error}");

        let empty = temp_path("empty.txt");
        std::fs::write(&empty, "   ").unwrap();
        profile.api_key_file = empty.to_string_lossy().to_string();
        assert!(LocalHttpClient::from_profile(&profile)
            .unwrap_err()
            .contains("empty"));

        let multiline = temp_path("multi.txt");
        std::fs::write(&multiline, "one\ntwo\n").unwrap();
        profile.api_key_file = multiline.to_string_lossy().to_string();
        assert!(LocalHttpClient::from_profile(&profile)
            .unwrap_err()
            .contains("without line breaks"));
    }

    #[test]
    fn mt06_api_key_protected_completion_sends_the_bearer_header() {
        let (port, seen) = serve_plain(ok_response("{}"), true);
        let key_path = temp_path("key2.txt");
        std::fs::write(&key_path, "canary-key-123").unwrap();
        let mut profile = profile_with("127.0.0.1", port);
        profile.api_key_file = key_path.to_string_lossy().to_string();
        let client = LocalHttpClient::from_profile(&profile).unwrap();
        let (status, _) = client
            .post_json(
                "/completion",
                &serde_json::json!({"n_predict": 1}),
                Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(status, 200);
        let request = seen.lock().unwrap().clone();
        assert!(
            request
                .to_lowercase()
                .contains("authorization: bearer canary-key-123"),
            "{request}"
        );
        let _ = std::fs::remove_file(&key_path);
    }

    #[test]
    fn mt06_trusted_local_tls_certificate_is_accepted() {
        let port = serve_tls(TRUSTED_CERT, TRUSTED_KEY, ok_response("{\"ok\":true}"));
        let cert_path = temp_path("trusted.crt");
        std::fs::write(&cert_path, TRUSTED_CERT).unwrap();
        let mut profile = profile_with("127.0.0.1", port);
        profile.ssl_cert_file = cert_path.to_string_lossy().to_string();
        let client = LocalHttpClient::from_profile(&profile).unwrap();
        assert_eq!(client.scheme(), "https");
        let (status, body) = client
            .get_bytes("/health", Duration::from_secs(10))
            .unwrap();
        assert_eq!(status, 200);
        assert!(String::from_utf8_lossy(&body).contains("ok"));
        let _ = std::fs::remove_file(&cert_path);
    }

    #[test]
    fn mt06_untrusted_certificate_is_rejected_with_verification_enabled() {
        // The server presents the OTHER self-signed pair; the client trusts
        // only the trusted pair. Verification must fail.
        let port = serve_tls(OTHER_CERT, OTHER_KEY, ok_response("{}"));
        let cert_path = temp_path("trusted2.crt");
        std::fs::write(&cert_path, TRUSTED_CERT).unwrap();
        let mut profile = profile_with("127.0.0.1", port);
        profile.ssl_cert_file = cert_path.to_string_lossy().to_string();
        let client = LocalHttpClient::from_profile(&profile).unwrap();
        let error = client
            .get_bytes("/health", Duration::from_secs(5))
            .unwrap_err();
        assert!(
            error.to_lowercase().contains("certificate") || error.to_lowercase().contains("tls"),
            "{error}"
        );
        let _ = std::fs::remove_file(&cert_path);
    }

    #[test]
    fn mt06_chunked_json_response_is_decoded() {
        let body = "{\"done\":true,\"n\":3}";
        let chunked = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n{}\r\n{:x}\r\n{}\r\n0\r\n\r\n",
            9,
            &body[..9],
            body.len() - 9,
            &body[9..]
        )
        .into_bytes();
        let (port, _) = serve_plain(chunked, false);
        let client = LocalHttpClient::plain("127.0.0.1", port).unwrap();
        let (status, bytes) = client
            .post_json(
                "/completion",
                &serde_json::json!({}),
                Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(status, 200);
        assert_eq!(String::from_utf8_lossy(&bytes), body);
    }

    #[test]
    fn mt06_oversized_response_is_rejected() {
        let declaration = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            MAX_LOCAL_RESPONSE_BYTES + 10
        )
        .into_bytes();
        let (port, _) = serve_plain(declaration, false);
        let client = LocalHttpClient::plain("127.0.0.1", port).unwrap();
        let error = client
            .get_bytes("/health", Duration::from_secs(5))
            .unwrap_err();
        assert!(error.contains("exceeds"), "{error}");
    }

    #[test]
    fn mt06_whole_operation_deadline_bounds_a_slow_writer() {
        // The fixture accepts the connection but never writes: the whole
        // operation deadline must bound connect + write + read.
        let (port, _) = serve_plain(Vec::new(), false);
        let client = LocalHttpClient::plain("127.0.0.1", port).unwrap();
        let started = std::time::Instant::now();
        let error = client
            .get_bytes("/health", Duration::from_millis(400))
            .unwrap_err();
        assert!(error.contains("did not answer"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn mt06_cancellation_is_observed_during_the_response_wait() {
        let (port, _) = serve_plain(Vec::new(), false);
        let client = LocalHttpClient::plain("127.0.0.1", port).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(300));
            flag.store(true, Ordering::Relaxed);
        });
        let started = std::time::Instant::now();
        let error = client
            .get_bytes_cancellable("/health", Duration::from_secs(20), &cancelled)
            .unwrap_err();
        assert!(error.contains("cancelled"), "{error}");
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "cancel was not observed promptly"
        );
    }

    #[test]
    fn mt06_error_statuses_reach_the_caller_without_panicking() {
        let response =
            b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
        let (port, _) = serve_plain(response, false);
        let client = LocalHttpClient::plain("127.0.0.1", port).unwrap();
        let (status, _) = client
            .post_json(
                "/completion",
                &serde_json::json!({}),
                Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(status, 401);
    }
}
