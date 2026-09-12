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
use std::time::Duration;

pub const MAX_LOCAL_REQUEST_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_LOCAL_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;
/// Cancellable calls run the request on a worker thread and observe the
/// cancel flag in short slices, so a cancel is noticed during connection and
/// response waits. The slice never bounds the response itself: a healthy
/// completion can legitimately outlive one slice (a ~560 ms completion on the
/// approved managed runtime livelocked the v2 benchmark when the slice
/// re-issued it), so the worker keeps the whole-operation deadline and a
/// cancelled call abandons it.
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
    /// Cancellable requests run on worker threads; this counts the workers
    /// that have not exited yet. A cancelled call abandons its worker, which
    /// exits by itself at the whole-operation deadline - the counter is the
    /// packaged/unit-observable bound on that ownership.
    active_workers: std::sync::atomic::AtomicUsize,
}

/// A `Write` sink that accepts at most `limit` bytes and then errors, so a
/// serialized request body is never fully materialized past its limit (R15).
struct BoundedVec {
    buffer: Vec<u8>,
    limit: usize,
}

impl BoundedVec {
    fn new(limit: usize) -> Self {
        Self {
            buffer: Vec::new(),
            limit,
        }
    }

    fn into_inner(self) -> Vec<u8> {
        self.buffer
    }
}

impl std::io::Write for BoundedVec {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        if self.buffer.len() + data.len() > self.limit {
            return Err(std::io::Error::other("request body exceeds the limit"));
        }
        self.buffer.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Decrements the worker counter when the worker thread exits, including on
/// unwind, so an abandoned worker is never double-counted or leaked.
struct WorkerLease(Arc<Inner>);

impl Drop for WorkerLease {
    fn drop(&mut self) {
        self.0.active_workers.fetch_sub(1, Ordering::Relaxed);
    }
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

/// One DER TLV frame: tag byte, value bytes, and the rest of the input after
/// the frame (R10). Length parsing accepts short and long form with bounds
/// checks; the walker is deliberately strict so malformed data fails closed.
struct DerFrame<'a> {
    tag: u8,
    value: &'a [u8],
    rest: &'a [u8],
}

fn der_frame(input: &[u8]) -> Result<DerFrame<'_>, String> {
    if input.len() < 2 {
        return Err("DER data ended before a complete element".into());
    }
    let tag = input[0];
    let first = input[1];
    let mut index = 2;
    let length = if first & 0x80 == 0 {
        first as usize
    } else {
        let count = (first & 0x7f) as usize;
        if count == 0 || count > 4 || input.len() < index + count {
            return Err("DER length is malformed".into());
        }
        let mut value = 0_usize;
        for byte in &input[index..index + count] {
            value = (value << 8) | *byte as usize;
        }
        index += count;
        value
    };
    if input.len() < index + length {
        return Err("DER element extends past the available data".into());
    }
    Ok(DerFrame {
        tag,
        value: &input[index..index + length],
        rest: &input[index + length..],
    })
}

fn der_length(length: usize) -> Vec<u8> {
    if length < 0x80 {
        return vec![length as u8];
    }
    let bytes = length.to_be_bytes();
    let first = bytes
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(bytes.len() - 1);
    let mut out = vec![0x80 | (bytes.len() - first) as u8];
    out.extend_from_slice(&bytes[first..]);
    out
}

fn der_tlv(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    out.extend_from_slice(&der_length(value.len()));
    out.extend_from_slice(value);
    out
}

fn der_children(value: &[u8]) -> Result<Vec<DerFrame<'_>>, String> {
    let mut frames = Vec::new();
    let mut cursor = value;
    while !cursor.is_empty() {
        let frame = der_frame(cursor)?;
        cursor = frame.rest;
        frames.push(frame);
    }
    Ok(frames)
}

/// The SubjectPublicKeyInfo TLV, extracted from a certificate's DER (R10).
/// The walk is structural: Certificate SEQUENCE, tbsCertificate SEQUENCE,
/// optional [0] version, then serialNumber, signature, issuer, validity,
/// subject, and the SPKI. Semantic X.509 validity is still enforced by the
/// webpki parse at the call site.
fn subject_public_key_info_from_certificate(der: &[u8]) -> Result<Vec<u8>, String> {
    let outer = der_frame(der)?;
    if outer.tag != 0x30 {
        return Err("The certificate is not a DER sequence".into());
    }
    let tbs = der_frame(outer.value)?;
    if tbs.tag != 0x30 {
        return Err("The certificate has no tbsCertificate".into());
    }
    let mut cursor = tbs.value;
    let first = der_frame(cursor)?;
    if first.tag == 0xa0 {
        cursor = first.rest;
    }
    for _ in 0..5 {
        let frame = der_frame(cursor)?;
        cursor = frame.rest;
    }
    let spki = der_frame(cursor)?;
    if spki.tag != 0x30 {
        return Err("The certificate has no SubjectPublicKeyInfo".into());
    }
    Ok(der_tlv(0x30, spki.value))
}

const OID_RSA_ENCRYPTION: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01];
const OID_EC_PUBLIC_KEY: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];

/// Build a canonical RSA SPKI from PKCS#1 modulus/exponent INTEGER frames.
fn rsa_spki(modulus: &DerFrame<'_>, exponent: &DerFrame<'_>) -> Vec<u8> {
    let mut rsa_key = Vec::new();
    rsa_key.extend_from_slice(&der_tlv(0x02, modulus.value));
    rsa_key.extend_from_slice(&der_tlv(0x02, exponent.value));
    let mut inner = Vec::new();
    inner.extend_from_slice(&der_tlv(0x06, OID_RSA_ENCRYPTION));
    inner.extend_from_slice(&[0x05, 0x00]);
    let mut bit_string = vec![0x00];
    bit_string.extend_from_slice(&der_tlv(0x30, &rsa_key));
    let mut spki = der_tlv(0x30, &inner);
    spki.extend_from_slice(&der_tlv(0x03, &bit_string));
    der_tlv(0x30, &spki)
}

fn ec_spki(curve_oid: &DerFrame<'_>, public_key: &DerFrame<'_>) -> Result<Vec<u8>, String> {
    if public_key.value.first() != Some(&0x00) {
        return Err("The EC public key bit string is malformed".into());
    }
    let mut inner = Vec::new();
    inner.extend_from_slice(&der_tlv(0x06, OID_EC_PUBLIC_KEY));
    inner.extend_from_slice(&der_tlv(0x06, curve_oid.value));
    let mut spki = der_tlv(0x30, &inner);
    spki.extend_from_slice(&der_tlv(0x03, public_key.value));
    Ok(der_tlv(0x30, &spki))
}

/// The SPKI a private key would present in a certificate (R10): PKCS#8
/// wrapped, raw PKCS#1 (RSA), or raw SEC1 (EC). The returned bytes are
/// canonical DER, so byte equality against the certificate's SPKI is a real
/// pair match.
fn subject_public_key_info_from_private_key(der: &[u8]) -> Result<Vec<u8>, String> {
    let outer = der_frame(der)?;
    if outer.tag != 0x30 {
        return Err("The private key is not a DER sequence".into());
    }
    let children = der_children(outer.value)?;
    if children.is_empty() {
        return Err("The private key is empty".into());
    }
    // PKCS#8: SEQUENCE { INTEGER 0, SEQUENCE algorithm, OCTET STRING key }.
    // PKCS#1 / SEC1: SEQUENCE { INTEGER version, ... } with the key material
    // as INTEGER (RSA) or OCTET STRING (EC) directly.
    let algorithm = if children.len() >= 3 && children[1].tag == 0x30 && children[2].tag == 0x04 {
        &children[1]
    } else if children.len() >= 2 && children[1].tag == 0x04 {
        // Raw SEC1 (EC PRIVATE KEY). The parsed structure spans the whole
        // document - version INTEGER, private OCTET STRING, curve [0] and
        // public key [1] - so the complete DER is what the SPKI extractor
        // must read. Passing children[1] (the private scalar) made every
        // complete SEC1 key fail as malformed.
        return raw_sec1_spki(der);
    } else if children.len() >= 3 && children[1].tag == 0x02 && children[2].tag == 0x02 {
        // Raw PKCS#1 RSA: SEQUENCE { version, modulus, publicExponent }.
        // The first INTEGER is the version, not the modulus.
        return Ok(rsa_spki(&children[1], &children[2]));
    } else {
        return Err("The private key is not in a supported PKCS#8, PKCS#1 or SEC1 form".into());
    };
    let key_octets = children[2].value;
    let algorithm_children = der_children(algorithm.value)?;
    let oid = algorithm_children
        .first()
        .filter(|frame| frame.tag == 0x06)
        .ok_or("The private key algorithm is malformed")?;
    if oid.value == OID_RSA_ENCRYPTION {
        let inner = der_frame(key_octets)?;
        if inner.tag != 0x30 {
            return Err("The RSA key body is malformed".into());
        }
        let rsa = der_children(inner.value)?;
        // PKCS#1: SEQUENCE { version, modulus, publicExponent, ... }.
        if rsa.len() < 3 || rsa[0].tag != 0x02 || rsa[1].tag != 0x02 || rsa[2].tag != 0x02 {
            return Err("The RSA key body is malformed".into());
        }
        return Ok(rsa_spki(&rsa[1], &rsa[2]));
    }
    if oid.value == OID_EC_PUBLIC_KEY {
        let curve = algorithm_children
            .get(1)
            .filter(|frame| frame.tag == 0x06)
            .ok_or("The EC key names no curve")?;
        return raw_sec1_spki_with_curve(key_octets, curve);
    }
    Err("The private key algorithm is not RSA or EC".into())
}

fn raw_sec1_spki(sec1: &[u8]) -> Result<Vec<u8>, String> {
    let outer = der_frame(sec1)?;
    if outer.tag != 0x30 {
        return Err("The EC key body is malformed".into());
    }
    let children = der_children(outer.value)?;
    let curve = children
        .iter()
        .find(|frame| frame.tag == 0xa0)
        .ok_or("The EC key names no curve")?;
    let curve_inner = der_frame(curve.value)?;
    if curve_inner.tag != 0x06 {
        return Err("The EC key curve is malformed".into());
    }
    let public = children
        .iter()
        .find(|frame| frame.tag == 0xa1)
        .ok_or("The EC key embeds no public key; regenerate it with one (openssl ec -pubout)")?;
    let public_inner = der_frame(public.value)?;
    if public_inner.tag != 0x03 {
        return Err("The EC key public part is malformed".into());
    }
    ec_spki(&curve_inner, &public_inner)
}

fn raw_sec1_spki_with_curve(sec1: &[u8], curve: &DerFrame<'_>) -> Result<Vec<u8>, String> {
    let outer = der_frame(sec1)?;
    if outer.tag != 0x30 {
        return Err("The EC key body is malformed".into());
    }
    let children = der_children(outer.value)?;
    let public = children
        .iter()
        .find(|frame| frame.tag == 0xa1)
        .ok_or("The EC key embeds no public key; regenerate it with one (openssl ec -pubout)")?;
    let public_inner = der_frame(public.value)?;
    if public_inner.tag != 0x03 {
        return Err("The EC key public part is malformed".into());
    }
    ec_spki(curve, &public_inner)
}

/// Decode the file as PEM certificates; returns at least one DER certificate
/// (R10: strict PEM framing and base64, not substring markers).
fn pem_certificate_der(bytes: &[u8], label: &str) -> Result<Vec<u8>, String> {
    let mut reader = bytes;
    let mut certificates = rustls_pemfile::certs(&mut reader);
    let first = certificates
        .next()
        .ok_or_else(|| format!("The {label} contains no PEM CERTIFICATE block"))?
        .map_err(|error| format!("The {label} is not valid PEM: {error}"))?;
    Ok(first.as_ref().to_vec())
}

fn pem_private_key_der(bytes: &[u8], label: &str) -> Result<Vec<u8>, String> {
    let mut reader = bytes;
    let key = rustls_pemfile::private_key(&mut reader)
        .map_err(|error| format!("The {label} is not valid PEM: {error}"))?
        .ok_or_else(|| format!("The {label} contains no PEM private-key block"))?;
    Ok(key.secret_der().to_vec())
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
        // R10 (follow-up review db548c8): the checks decode the real PEM and
        // DER structures. Marker substrings ("MIIB" inside BEGIN/END lines)
        // passed the old check; now the certificate must parse as X.509, the
        // key must decode as a supported PKCS#8/PKCS#1/SEC1 form, and the
        // pair must present the same SubjectPublicKeyInfo.
        let cert_path = profile.ssl_cert_file.trim();
        let key_path = profile.ssl_key_file.trim();
        let certificates = if cert_path.is_empty() {
            None
        } else {
            let bytes = read_bounded_file(cert_path, MAX_CERT_FILE_BYTES, "SSL certificate")?;
            let der = pem_certificate_der(&bytes, "SSL certificate")?;
            let certificate = rustls_pki_types::CertificateDer::from(der.clone());
            webpki::EndEntityCert::try_from(&certificate).map_err(|error| {
                format!(
                    "The SSL certificate is not a valid X.509 certificate ({cert_path}): {error}"
                )
            })?;
            Some(der)
        };
        let key = if key_path.is_empty() {
            None
        } else {
            let bytes = read_bounded_file(key_path, MAX_CERT_FILE_BYTES, "SSL private key")?;
            Some(pem_private_key_der(&bytes, "SSL private key")?)
        };
        if let (Some(certificate), Some(key)) = (certificates.as_deref(), key.as_deref()) {
            let cert_spki =
                subject_public_key_info_from_certificate(certificate).map_err(|error| {
                    format!("The SSL certificate has no readable public key ({cert_path}): {error}")
                })?;
            let key_spki = subject_public_key_info_from_private_key(key).map_err(|error| {
                format!("The SSL private key could not be read ({key_path}): {error}")
            })?;
            if cert_spki != key_spki {
                return Err(format!(
                    "The SSL certificate ({cert_path}) and private key ({key_path}) do not belong together: their public keys differ"
                ));
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
            .pool_max_idle_per_host(0)
            // R15 (follow-up review db548c8): the client speaks to exactly
            // one loopback server. Redirects are never followed - a 3xx is a
            // failure, not a hop - and environment proxies are ignored so a
            // stray HTTP_PROXY can never reroute local-server traffic.
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy();
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
                active_workers: std::sync::atomic::AtomicUsize::new(0),
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

    /// Test-only view of the abandoned-worker bound (MT-06 extension).
    /// How many cancellable workers of this client have not exited yet
    /// (R04, follow-up review). A cancelled call abandons its worker; the
    /// worker exits by itself at the whole-operation deadline, and this count
    /// is the ownership bound the run drains before releasing its slot.
    pub(crate) fn active_cancellable_workers(&self) -> usize {
        self.inner.active_workers.load(Ordering::Relaxed)
    }

    /// Waits until every cancellable worker of this client has exited, or
    /// until the ceiling elapses. Returns true when the client is drained.
    /// The wait is a poll on the worker count; a worker never outlives its
    /// own request deadline, so the caller passes a ceiling above that
    /// deadline and treats expiry as unresolved ownership (R04).
    pub(crate) fn wait_for_worker_drain(&self, ceiling: Duration) -> bool {
        let started = std::time::Instant::now();
        loop {
            if self.active_cancellable_workers() == 0 {
                return true;
            }
            if started.elapsed() >= ceiling {
                return false;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
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
            Some(value) => {
                // R15: serialize through a bounded writer. The pre-check
                // after serialization could only reject an oversized body
                // AFTER the whole body had been allocated; the writer stops
                // the copy at the limit instead.
                let mut bounded = BoundedVec::new(MAX_LOCAL_REQUEST_BYTES);
                match serde_json::to_writer(&mut bounded, value) {
                    Ok(()) => bounded.into_inner(),
                    Err(error) if error.is_io() => {
                        return Err(format!(
                            "The local request exceeds the {MAX_LOCAL_REQUEST_BYTES}-byte limit"
                        ));
                    }
                    Err(error) => {
                        return Err(format!("The request body could not be serialized: {error}"));
                    }
                }
            }
            None => Vec::new(),
        };
        if body_bytes.len() > MAX_LOCAL_REQUEST_BYTES {
            return Err(format!(
                "The local request exceeds the {MAX_LOCAL_REQUEST_BYTES}-byte limit"
            ));
        }
        let url = self.url(path);
        match cancelled {
            None => run_local_request(
                &self.inner,
                method,
                &url,
                &body_bytes,
                body.is_some(),
                budget,
            ),
            Some(flag) => {
                // Cancellable requests run on a worker thread with the full
                // deadline; this thread observes the cancel flag in short
                // slices. The slice must never discard a response that
                // legitimately outlives it: the old slice-retry killed every
                // completion slower than 500 ms and re-issued it until the
                // deadline, which livelocked the v2 benchmark on the approved
                // managed runtime (~476 tok/s, ~560 ms per completion) with
                // 500+ duplicate generations. A cancelled call abandons the
                // worker, which exits by itself at the deadline.
                if flag.load(Ordering::Relaxed) {
                    return Err("The local request was cancelled".into());
                }
                let (sender, receiver) = std::sync::mpsc::channel();
                let inner = Arc::clone(&self.inner);
                let worker_method = method.to_string();
                let worker_url = url.clone();
                let worker_body = body_bytes.clone();
                let has_body = body.is_some();
                self.inner.active_workers.fetch_add(1, Ordering::Relaxed);
                let lease_inner = Arc::clone(&self.inner);
                std::thread::Builder::new()
                    .name("localmotive-local-request".into())
                    .spawn(move || {
                        let _lease = WorkerLease(lease_inner);
                        let _ = sender.send(run_local_request(
                            &inner,
                            &worker_method,
                            &worker_url,
                            &worker_body,
                            has_body,
                            budget,
                        ));
                    })
                    .map_err(|error| {
                        self.inner.active_workers.fetch_sub(1, Ordering::Relaxed);
                        format!("The local request worker could not start: {error}")
                    })?;
                loop {
                    match receiver.recv_timeout(CANCEL_ATTEMPT_SLICE) {
                        Ok(result) => {
                            // R03 (follow-up review): cancellation is defined
                            // at RESULT ACCEPTANCE. A flag set after dispatch
                            // but before the response is accepted must not be
                            // lost to the polling slice; the received result
                            // is discarded rather than misreported as success.
                            if flag.load(Ordering::Relaxed) {
                                return Err("The local request was cancelled".into());
                            }
                            return result;
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            if flag.load(Ordering::Relaxed) {
                                return Err("The local request was cancelled".into());
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            return Err("The local request worker stopped unexpectedly".into());
                        }
                    }
                }
            }
        }
    }
}

/// One bounded attempt of a local request: connect, write and read inside the
/// whole-operation deadline. Shared by the plain path and the cancellable
/// worker thread.
fn run_local_request(
    inner: &Inner,
    method: &str,
    url: &str,
    body_bytes: &[u8],
    has_body: bool,
    budget: Duration,
) -> Result<(u16, Vec<u8>), String> {
    let mut request = inner
        .client
        .request(
            reqwest::Method::from_bytes(method.as_bytes()).map_err(|error| error.to_string())?,
            url,
        )
        .timeout(budget);
    if has_body {
        request = request
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body_bytes.to_vec());
    }
    if let Some(key) = inner.api_key.as_deref() {
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
            reader
                .read_to_end(&mut bytes)
                .map_err(|error| format!("The local response could not be read: {error}"))?;
            if bytes.len() as u64 > MAX_LOCAL_RESPONSE_BYTES {
                return Err(format!(
                    "The local response exceeds the {MAX_LOCAL_RESPONSE_BYTES}-byte limit"
                ));
            }
            Ok((status, bytes))
        }
        Err(error) => {
            if error.is_timeout() {
                return Err(format!(
                    "llama-server did not answer within {} seconds",
                    budget.as_secs().max(1)
                ));
            }
            Err(format!(
                "The local request failed: {error}{}",
                error_chain(&error)
            ))
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
    // R15 (follow-up review db548c8): the read itself is bounded by the
    // handle, not by a metadata pre-check. A file that grows between the
    // check and the read - or a non-regular file that lies about its size -
    // can never copy more than the limit plus one byte, and the overflow is
    // detected and refused.
    use std::io::Read as _;
    let file = std::fs::File::open(path)
        .map_err(|error| format!("The {label} could not be read ({path}): {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("The {label} could not be read ({path}): {error}"))?;
    if !metadata.is_file() {
        return Err(format!("The {label} is not a file: {path}"));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("The {label} could not be read ({path}): {error}"))?;
    if bytes.len() as u64 > limit {
        return Err(format!("The {label} exceeds the {limit}-byte limit"));
    }
    Ok(bytes)
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

    /// R10 EC-path fixture: a throwaway P-256 PKCS#8 pair generated with
    /// openssl 3.2.4 for the SPKI pair-match tests. Test-only bytes; no
    /// operational trust.
    const EC_CERT: &str = r##"-----BEGIN CERTIFICATE-----
MIIBjzCCATWgAwIBAgIUZ7nMCMIXaF6SRs5RxaJeL+IBNNswCgYIKoZIzj0EAwIw
HTEbMBkGA1UEAwwSbG9jYWxtb3RpdmUtcjEwLWVjMB4XDTI2MDkxMjE1NDcyMVoX
DTM2MDkwOTE1NDcyMVowHTEbMBkGA1UEAwwSbG9jYWxtb3RpdmUtcjEwLWVjMFkw
EwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEfw2csAv/vFgELK6WQ+PaH+iIymxFzc7i
2ygcei5aMDvDMIN+vFG+t+QS1VkNXd+a5PxJbu+4dvbx1gCNpEt2WaNTMFEwHQYD
VR0OBBYEFIbWCGFhRTipnap5jTdnXF+aHXFLMB8GA1UdIwQYMBaAFIbWCGFhRTip
nap5jTdnXF+aHXFLMA8GA1UdEwEB/wQFMAMBAf8wCgYIKoZIzj0EAwIDSAAwRQIg
Lx8HQfPVUsiWjG73iCnMFuo4w8NF88IVvsfclDjFKNMCIQC36ow/MH5LgUmVLmuB
I6EbfRyZwLstdf7ZthlFNpXKEQ==
-----END CERTIFICATE-----"##;
    const EC_KEY: &str = r##"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgYXOhc8MY1PcNmJop
RdhymZjDnfWm6bAYG6WuzSr+KEihRANCAAR/DZywC/+8WAQsrpZD49of6IjKbEXN
zuLbKBx6LlowO8Mwg368Ub635BLVWQ1d35rk/Elu77h29vHWAI2kS3ZZ
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

    /// HTTP fixture that answers every request after `delay` and counts the
    /// requests it saw. Used by the cancellable-slice regression below: a
    /// healthy completion can legitimately take longer than one cancel slice.
    pub(crate) fn serve_delayed(delay: Duration) -> (u16, Arc<std::sync::atomic::AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = requests.clone();
        thread::spawn(move || {
            for incoming in listener.incoming() {
                let Ok(mut stream) = incoming else { break };
                counter.fetch_add(1, Ordering::Relaxed);
                stream
                    .set_read_timeout(Some(Duration::from_millis(500)))
                    .ok();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                thread::sleep(delay);
                let _ = stream.write_all(&ok_response("{\"content\":\"delayed\"}"));
                let _ = stream.flush();
            }
        });
        (port, requests)
    }

    /// HTTP fixture that answers headers immediately and dribbles the body
    /// over `total`, counting requests, live connections, completed and
    /// aborted responses. The MT-06 extension uses it so cancellation is
    /// judged by resources (workers, sockets, duplicate requests) instead of
    /// caller-return latency alone.
    pub(crate) struct SlowBodyFixture {
        pub port: u16,
        pub requests: Arc<std::sync::atomic::AtomicUsize>,
        pub active: Arc<std::sync::atomic::AtomicUsize>,
        pub completed: Arc<std::sync::atomic::AtomicUsize>,
        pub aborted: Arc<std::sync::atomic::AtomicUsize>,
    }

    pub(crate) fn serve_slow_body(total: Duration, chunks: usize) -> SlowBodyFixture {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let active = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let aborted = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (r, a, c, ab) = (
            requests.clone(),
            active.clone(),
            completed.clone(),
            aborted.clone(),
        );
        thread::spawn(move || {
            for incoming in listener.incoming() {
                let Ok(mut stream) = incoming else { break };
                r.fetch_add(1, Ordering::Relaxed);
                a.fetch_add(1, Ordering::Relaxed);
                stream
                    .set_read_timeout(Some(Duration::from_millis(500)))
                    .ok();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                let body = b"{\"content\":\"dribbled response body for the MT-06 extension\"}";
                let headers = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let mut wrote_ok = stream.write_all(headers.as_bytes()).is_ok();
                let pause = total / chunks as u32;
                let size = body.len().div_ceil(chunks);
                for start in (0..body.len()).step_by(size) {
                    if !wrote_ok {
                        break;
                    }
                    let end = (start + size).min(body.len());
                    wrote_ok = stream.write_all(&body[start..end]).is_ok();
                    thread::sleep(pause);
                }
                let _ = stream.flush();
                if wrote_ok {
                    c.fetch_add(1, Ordering::Relaxed);
                } else {
                    ab.fetch_add(1, Ordering::Relaxed);
                }
                a.fetch_sub(1, Ordering::Relaxed);
            }
        });
        SlowBodyFixture {
            port,
            requests,
            active,
            completed,
            aborted,
        }
    }

    fn settle_until(mut predicate: impl FnMut() -> bool, timeout: Duration) -> bool {
        let started = std::time::Instant::now();
        while started.elapsed() < timeout {
            if predicate() {
                return true;
            }
            thread::sleep(Duration::from_millis(50));
        }
        predicate()
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
    fn a_cancellable_response_that_outlives_the_cancel_slice_still_completes() {
        // Regression from the 0.6.0 re-bind probes: the approved managed CUDA
        // runtime serves the default v2 workload at ~476 tok/s, so one
        // completion takes ~560 ms — longer than CANCEL_ATTEMPT_SLICE. The
        // old slice-retry killed that healthy response and re-issued the
        // request every 500 ms until the whole-operation deadline: the
        // benchmark livelocked and the server generated 500+ duplicate
        // completions. A response slower than the slice must be observed
        // exactly once, on the caller's own thread budget.
        let (port, requests) = serve_delayed(Duration::from_millis(900));
        let client = LocalHttpClient::plain("127.0.0.1", port).unwrap();
        let cancelled = AtomicBool::new(false);
        let started = std::time::Instant::now();
        let (status, bytes) = client
            .post_json_cancellable(
                "/completion",
                &serde_json::json!({"prompt": [1], "n_predict": 4}),
                Duration::from_secs(6),
                &cancelled,
            )
            .expect("a response slower than the cancel slice must complete");
        assert_eq!(status, 200);
        assert!(String::from_utf8(bytes).unwrap().contains("delayed"));
        assert!(started.elapsed() < Duration::from_secs(6));
        assert_eq!(
            requests.load(Ordering::Relaxed),
            1,
            "the request must reach the server exactly once"
        );
        thread::sleep(Duration::from_millis(600));
        assert_eq!(
            requests.load(Ordering::Relaxed),
            1,
            "no duplicate request may follow the completed response"
        );
    }

    /// A server that tells the test when the request has been accepted, then
    /// waits for the test's release flag before answering with a fast 200.
    /// It coordinates the R03 window: cancel set after dispatch, response
    /// accepted afterwards.
    pub(crate) fn serve_gated(accepted: Arc<AtomicBool>, release: Arc<AtomicBool>) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut buffer = [0_u8; 8192];
                let _ = stream.read(&mut buffer);
                accepted.store(true, Ordering::SeqCst);
                while !release.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(5));
                }
                let body = b"{\"ok\":true}";
                let headers = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(headers.as_bytes());
                let _ = stream.write_all(body);
            }
        });
        port
    }

    #[test]
    fn r04_worker_drain_waits_for_the_abandoned_slow_request_to_exit() {
        // R04 (follow-up review db548c8): after a cancellation the abandoned
        // worker keeps its slow request alive until its own deadline, and the
        // run must not report its ownership resolved before that worker
        // exits - otherwise replacement work can overlap the slow request.
        let fixture = serve_slow_body(Duration::from_millis(1200), 12);
        let client = LocalHttpClient::plain("127.0.0.1", fixture.port).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let caller_client = client.clone();
        let call = thread::spawn(move || {
            caller_client.post_json_cancellable(
                "/completion",
                &serde_json::json!({"prompt": [1]}),
                Duration::from_secs(30),
                &flag,
            )
        });
        assert!(
            settle_until(
                || client.active_cancellable_workers() > 0,
                Duration::from_secs(5)
            ),
            "the worker must be counted while its request runs"
        );
        cancelled.store(true, Ordering::Relaxed);
        assert!(
            call.join().unwrap().is_err(),
            "the cancelled call reports the cancellation"
        );
        assert!(
            client.active_cancellable_workers() > 0,
            "right after the cancel the slow worker is still in flight"
        );
        let started = std::time::Instant::now();
        assert!(
            client.wait_for_worker_drain(Duration::from_secs(10)),
            "the worker exits by itself at its deadline"
        );
        let waited = started.elapsed();
        assert!(
            waited >= Duration::from_millis(600),
            "the drain waited for the slow worker to exit, not just observed zero ({waited:?})"
        );
        assert_eq!(client.active_cancellable_workers(), 0);
    }

    #[test]
    fn r10_transport_validation_parses_real_x509_and_matches_the_pair() {
        // R10 (follow-up review db548c8): the validator decodes the real PEM
        // and DER. Marker-wrapped junk must fail; a mismatched pair must fail
        // with the pair message; a real matching pair passes.
        let dir = std::env::temp_dir().join(format!("localmotive-r10-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let write = |name: &str, content: &str| {
            let path = dir.join(name);
            std::fs::write(&path, content).unwrap();
            path.to_string_lossy().to_string()
        };
        let cert = write("ok.crt", TRUSTED_CERT);
        let key = write("ok.key", TRUSTED_KEY);
        let other_key = write("other.key", OTHER_KEY);
        let junk_cert = write(
            "junk.crt",
            "-----BEGIN CERTIFICATE-----
MIIB
-----END CERTIFICATE-----
",
        );
        let junk_key = write(
            "junk.key",
            "-----BEGIN PRIVATE KEY-----
MIIB
-----END PRIVATE KEY-----
",
        );
        let mut profile = crate::core::LaunchProfile {
            model: "C:/models/model.gguf".into(),
            alias: "test-model".into(),
            host: "127.0.0.1".into(),
            port: 8080,
            ..crate::core::LaunchProfile::default()
        };

        profile.ssl_cert_file = cert.clone();
        profile.ssl_key_file = key.clone();
        if let Err(error) = LocalHttpClient::validate_transport_files(&profile) {
            panic!("a real matching pair must validate: {error}");
        }

        profile.ssl_cert_file = junk_cert.clone();
        profile.ssl_key_file = junk_key.clone();
        let error = LocalHttpClient::validate_transport_files(&profile).unwrap_err();
        assert!(
            error.contains("X.509") || error.contains("PEM"),
            "marker-wrapped junk must fail real parsing: {error}"
        );

        profile.ssl_cert_file = cert.clone();
        profile.ssl_key_file = junk_key;
        let error = LocalHttpClient::validate_transport_files(&profile).unwrap_err();
        assert!(
            error.contains("private-key block")
                || error.contains("PEM")
                || error.contains("malformed"),
            "a junk private key must fail real decoding: {error}"
        );

        profile.ssl_cert_file = cert.clone();
        profile.ssl_key_file = other_key;
        let error = LocalHttpClient::validate_transport_files(&profile).unwrap_err();
        assert!(
            error.contains("do not belong together"),
            "an unrelated pair must fail the public-key match: {error}"
        );

        // A real EC (P-256, PKCS#8) pair exercises the EC SPKI path.
        let ec_cert = write("ec.crt", EC_CERT);
        let ec_key_path = write("ec.key", EC_KEY);
        profile.ssl_cert_file = ec_cert.clone();
        profile.ssl_key_file = ec_key_path.clone();
        LocalHttpClient::validate_transport_files(&profile)
            .expect("a real matching EC pair must validate");

        // Cross-family mismatch: the RSA certificate with the EC private key.
        profile.ssl_cert_file = cert;
        profile.ssl_key_file = ec_key_path;
        let error = LocalHttpClient::validate_transport_files(&profile).unwrap_err();
        assert!(
            error.contains("do not belong together"),
            "cross-family material must fail the public-key match: {error}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The complete raw SEC1 form of the EC fixture, byte-identical to
    /// `openssl pkey -in ec.key -traditional`: the PKCS#8 privateKey OCTET
    /// STRING holds the ECPrivateKey structure (version, private scalar,
    /// public point) and the curve parameters [0] are re-inserted from the
    /// PKCS#8 algorithm identifier, exactly as the traditional encoding does.
    fn ec_sec1_der() -> Vec<u8> {
        let der = pem_der(EC_KEY);
        let outer = der_frame(&der).expect("PKCS#8 outer");
        let children = der_children(outer.value).expect("PKCS#8 children");
        let algorithm = der_children(children[1].value).expect("PKCS#8 algorithm");
        let curve = algorithm.get(1).expect("curve OID");
        let inner = der_frame(children[2].value).expect("ECPrivateKey outer");
        let fields = der_children(inner.value).expect("ECPrivateKey children");
        assert_eq!(fields[0].tag, 0x02, "ECPrivateKey version");
        assert_eq!(fields[1].tag, 0x04, "ECPrivateKey private scalar");
        let mut body = Vec::new();
        body.extend_from_slice(&der_tlv(fields[0].tag, fields[0].value));
        body.extend_from_slice(&der_tlv(fields[1].tag, fields[1].value));
        body.extend_from_slice(&der_tlv(0xa0, &der_tlv(0x06, curve.value)));
        for field in fields.iter().skip(2) {
            body.extend_from_slice(&der_tlv(field.tag, field.value));
        }
        der_tlv(0x30, &body)
    }

    fn pem_from_der(label: &str, der: &[u8]) -> String {
        use base64::Engine;
        let body = base64::engine::general_purpose::STANDARD.encode(der);
        let wrapped = body
            .as_bytes()
            .chunks(64)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        format!("-----BEGIN {label}-----\n{wrapped}\n-----END {label}-----\n")
    }

    fn ec_sec1_pem() -> String {
        pem_from_der("EC PRIVATE KEY", &ec_sec1_der())
    }

    /// A SEC1 structure with one context-tagged field removed, to prove each
    /// required part is enforced rather than assumed.
    fn ec_sec1_pem_without(tag: u8) -> String {
        let der = ec_sec1_der();
        let outer = der_frame(&der).expect("SEC1 outer");
        let children = der_children(outer.value).expect("SEC1 children");
        let mut body = Vec::new();
        let mut removed = false;
        for child in &children {
            if child.tag == tag {
                removed = true;
                continue;
            }
            body.extend_from_slice(&der_tlv(child.tag, child.value));
        }
        assert!(removed, "fixture did not contain tag {tag:#x}");
        pem_from_der("EC PRIVATE KEY", &der_tlv(0x30, &body))
    }

    #[test]
    fn r16_a_raw_sec1_private_key_is_parsed_and_matched() {
        // Follow-up review: the raw SEC1 branch passed the private scalar
        // (children[1]) to the SEC1 parser instead of the complete SEC1
        // structure, so every "EC PRIVATE KEY" file failed as malformed even
        // though rustls-pemfile decodes it. A complete, valid SEC1 structure
        // must validate against its certificate, and incomplete structures
        // must fail closed with a precise message.
        let dir = std::env::temp_dir().join(format!("localmotive-r16-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let write = |name: &str, content: &str| {
            let path = dir.join(name);
            std::fs::write(&path, content).unwrap();
            path.to_string_lossy().to_string()
        };
        let cert = write("ec.crt", EC_CERT);
        let sec1 = write("ec-sec1.key", &ec_sec1_pem());
        let mut profile = crate::core::LaunchProfile {
            model: "C:/models/model.gguf".into(),
            alias: "test-model".into(),
            host: "127.0.0.1".into(),
            port: 8080,
            ..crate::core::LaunchProfile::default()
        };

        profile.ssl_cert_file = cert.clone();
        profile.ssl_key_file = sec1.clone();
        LocalHttpClient::validate_transport_files(&profile)
            .expect("a complete raw SEC1 pair must validate");

        // The pair match still applies to the SEC1 path.
        profile.ssl_cert_file = write("other.crt", OTHER_CERT);
        profile.ssl_key_file = sec1.clone();
        let error = LocalHttpClient::validate_transport_files(&profile).unwrap_err();
        assert!(
            error.contains("do not belong together"),
            "an unrelated certificate must fail the SEC1 pair match: {error}"
        );

        profile.ssl_cert_file = cert.clone();
        profile.ssl_key_file = write("ec-sec1-no-public.key", &ec_sec1_pem_without(0xa1));
        let error = LocalHttpClient::validate_transport_files(&profile).unwrap_err();
        assert!(
            error.contains("embeds no public key"),
            "a SEC1 key without its public part must fail with guidance: {error}"
        );

        profile.ssl_key_file = write("ec-sec1-no-curve.key", &ec_sec1_pem_without(0xa0));
        let error = LocalHttpClient::validate_transport_files(&profile).unwrap_err();
        assert!(
            error.contains("names no curve"),
            "a SEC1 key without its curve must fail: {error}"
        );

        let mut truncated = ec_sec1_der();
        truncated.truncate(truncated.len() - 3);
        profile.ssl_key_file = write(
            "ec-sec1-truncated.key",
            &pem_from_der("EC PRIVATE KEY", &truncated),
        );
        let error = LocalHttpClient::validate_transport_files(&profile).unwrap_err();
        assert!(
            error.contains("could not be read"),
            "a truncated SEC1 structure must fail closed: {error}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn r15_a_redirect_is_not_followed_and_the_client_fails_the_call() {
        // R15 (follow-up review db548c8): the loopback client speaks to one
        // server. A 3xx is a failure, never a hop: the redirect target must
        // never receive the request.
        let reached = Arc::new(AtomicBool::new(false));
        let second = TcpListener::bind("127.0.0.1:0").unwrap();
        let second_port = second.local_addr().unwrap().port();
        let hit = Arc::clone(&reached);
        thread::spawn(move || {
            if let Ok((_stream, _)) = second.accept() {
                hit.store(true, Ordering::SeqCst);
            }
        });
        let first = TcpListener::bind("127.0.0.1:0").unwrap();
        let first_port = first.local_addr().unwrap().port();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = first.accept() {
                let mut buffer = [0_u8; 8192];
                let _ = stream.read(&mut buffer);
                let redirect = format!(
                    "HTTP/1.1 302 Found
location: http://127.0.0.1:{second_port}/steal
content-length: 0
connection: close

"
                );
                let _ = stream.write_all(redirect.as_bytes());
            }
        });
        let client = LocalHttpClient::plain("127.0.0.1", first_port).unwrap();
        let (status, _body) = client
            .post_json(
                "/completion",
                &serde_json::json!({"prompt": [1]}),
                Duration::from_secs(10),
            )
            .unwrap();
        // The raw status is observable and the hop never happened: callers
        // see 302, not a 200 from somewhere else.
        assert_eq!(status, 302, "the redirect status is returned, not followed");
        thread::sleep(Duration::from_millis(250));
        assert!(
            !reached.load(Ordering::SeqCst),
            "the redirect target must never receive the request"
        );
    }

    #[test]
    fn r15_the_request_body_serializes_through_a_bounded_writer() {
        // R15: the writer refuses bytes past the cap instead of accepting a
        // full oversized allocation that is checked only afterwards.
        use std::io::Write as _;
        let mut bounded = BoundedVec::new(16);
        assert!(bounded.write_all(b"0123456789abcdef").is_ok());
        assert!(
            bounded.write_all(b"x").is_err(),
            "bytes past the limit are refused"
        );
        assert_eq!(bounded.into_inner().len(), 16);
        let mut whole = BoundedVec::new(4);
        assert!(
            whole.write(b"12345").is_err(),
            "a single oversized write is refused whole, never truncated into a valid body"
        );
    }

    #[test]
    fn r15_bounded_file_reads_are_enforced_by_the_read_not_a_metadata_precheck() {
        // R15: the read is bounded by the handle (take(limit + 1)), so a file
        // that lies about or grows past its size can never copy more than the
        // limit plus one byte.
        let dir = std::env::temp_dir().join(format!("localmotive-r15-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let at_limit = dir.join("at-limit.bin");
        std::fs::write(&at_limit, vec![7_u8; 64]).unwrap();
        let over = dir.join("over.bin");
        std::fs::write(&over, vec![7_u8; 65]).unwrap();
        assert_eq!(
            read_bounded_file(&at_limit.to_string_lossy(), 64, "fixture")
                .unwrap()
                .len(),
            64
        );
        let error = read_bounded_file(&over.to_string_lossy(), 64, "fixture").unwrap_err();
        assert!(error.contains("exceeds the 64-byte limit"), "{error}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_cancel_set_before_response_acceptance_discards_the_received_result() {
        // R03 (follow-up review db548c8): cancellation is defined at result
        // acceptance. The request is dispatched, the server accepts it, the
        // user cancels, and only then is the (fast, successful) response
        // released. Accepting it would mislabel the cancelled attempt as
        // success; the received result must be discarded.
        let accepted = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let port = serve_gated(Arc::clone(&accepted), Arc::clone(&release));
        let client = LocalHttpClient::plain("127.0.0.1", port).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let call_client = client.clone();
        let call = thread::spawn(move || {
            call_client.post_json_cancellable(
                "/completion",
                &serde_json::json!({"prompt": [1]}),
                Duration::from_secs(30),
                &flag,
            )
        });
        assert!(
            settle_until(|| accepted.load(Ordering::SeqCst), Duration::from_secs(5)),
            "the fixture must accept the request"
        );
        cancelled.store(true, Ordering::Relaxed);
        release.store(true, Ordering::SeqCst);
        let result = call.join().unwrap();
        assert!(
            result.is_err(),
            "a response accepted after the cancel was set must be discarded, got {result:?}"
        );
        assert!(result.unwrap_err().contains("cancelled"));
    }

    #[test]
    fn a_cancelled_body_read_resolves_worker_ownership_without_duplicate_requests() {
        // MT-06 extension: a cancelled call returns on the caller thread, but
        // the abandoned worker must still exit - at the latest when its own
        // whole-operation deadline expires - and the half-read connection must
        // be torn down exactly once. The fixture streams far longer than the
        // budget so the worker's exit is deadline-driven, and any retry would
        // show up as a second request.
        let fixture = serve_slow_body(Duration::from_millis(4000), 40);
        let client = LocalHttpClient::plain("127.0.0.1", fixture.port).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let caller_client = client.clone();
        let caller = thread::spawn(move || {
            caller_client.post_json_cancellable(
                "/completion",
                &serde_json::json!({"prompt": [1], "n_predict": 4}),
                Duration::from_millis(1200),
                &flag,
            )
        });
        assert!(settle_until(
            || fixture.requests.load(Ordering::Relaxed) == 1,
            Duration::from_secs(2)
        ));
        thread::sleep(Duration::from_millis(250));
        cancelled.store(true, Ordering::Relaxed);
        let cancel_seen = std::time::Instant::now();
        let outcome = caller.join().unwrap();
        assert!(
            outcome.is_err(),
            "a cancelled call must not report a response"
        );
        assert!(
            cancel_seen.elapsed() < Duration::from_millis(700),
            "the caller returns within one cancel slice"
        );
        assert!(
            settle_until(
                || client.active_cancellable_workers() == 0,
                Duration::from_secs(4)
            ),
            "the abandoned worker exits by its deadline"
        );
        assert!(
            settle_until(
                || fixture.aborted.load(Ordering::Relaxed) == 1,
                Duration::from_secs(2)
            ),
            "the deadline aborts the half-read connection exactly once"
        );
        assert_eq!(
            fixture.requests.load(Ordering::Relaxed),
            1,
            "a cancelled request is never re-issued"
        );
        assert_eq!(fixture.completed.load(Ordering::Relaxed), 0);
        assert!(
            settle_until(
                || fixture.active.load(Ordering::Relaxed) == 0,
                Duration::from_secs(2)
            ),
            "no connection may stay open after the abort"
        );
    }

    #[test]
    fn a_cancelled_call_lets_a_short_body_finish_on_the_owned_connection_once() {
        // The abandoned worker owns its in-flight request: a body that
        // finishes inside the worker's deadline completes normally (no abort,
        // no retry) and the worker still exits. The body must outlast the
        // caller's cancel-return (one 500 ms slice), so it streams for about
        // 1.8 s while the caller cancels after ~100 ms.
        let fixture = serve_slow_body(Duration::from_millis(1800), 18);
        let client = LocalHttpClient::plain("127.0.0.1", fixture.port).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let caller_client = client.clone();
        let caller = thread::spawn(move || {
            caller_client.post_json_cancellable(
                "/completion",
                &serde_json::json!({"prompt": [2]}),
                Duration::from_secs(5),
                &flag,
            )
        });
        assert!(settle_until(
            || fixture.requests.load(Ordering::Relaxed) == 1,
            Duration::from_secs(2)
        ));
        thread::sleep(Duration::from_millis(100));
        cancelled.store(true, Ordering::Relaxed);
        assert!(caller.join().unwrap().is_err());
        assert!(
            settle_until(
                || fixture.completed.load(Ordering::Relaxed) == 1,
                Duration::from_secs(3)
            ),
            "the owned connection finishes its short body"
        );
        assert!(
            settle_until(
                || client.active_cancellable_workers() == 0,
                Duration::from_secs(2)
            ),
            "the worker exits once its request resolves"
        );
        assert_eq!(fixture.requests.load(Ordering::Relaxed), 1);
        assert_eq!(fixture.aborted.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn repeated_cancel_restart_cycles_keep_workers_bounded_and_requests_exact() {
        // Six cancel/restart cycles against one fixture: every cycle issues
        // exactly one request, the abandoned worker is reaped by its deadline,
        // and the live-worker count never exceeds the one in-flight call. Any
        // duplicate request would also break the exact request count.
        let fixture = serve_slow_body(Duration::from_millis(3000), 30);
        let client = LocalHttpClient::plain("127.0.0.1", fixture.port).unwrap();
        for cycle in 0..6usize {
            let cancelled = Arc::new(AtomicBool::new(false));
            let flag = Arc::clone(&cancelled);
            let cycle_client = client.clone();
            let caller = thread::spawn(move || {
                cycle_client.post_json_cancellable(
                    "/completion",
                    &serde_json::json!({"prompt": [cycle]}),
                    Duration::from_millis(1000),
                    &flag,
                )
            });
            assert!(
                settle_until(
                    || fixture.requests.load(Ordering::Relaxed) == cycle + 1,
                    Duration::from_secs(2)
                ),
                "cycle {cycle} reaches the server"
            );
            thread::sleep(Duration::from_millis(150));
            cancelled.store(true, Ordering::Relaxed);
            assert!(caller.join().unwrap().is_err());
            assert!(
                client.active_cancellable_workers() <= 1,
                "cycle {cycle} must never stack workers beyond its own call"
            );
            assert!(
                settle_until(
                    || client.active_cancellable_workers() == 0,
                    Duration::from_secs(3)
                ),
                "cycle {cycle} reaps its worker by the deadline"
            );
        }
        assert_eq!(
            fixture.requests.load(Ordering::Relaxed),
            6,
            "one request per cycle, never duplicated"
        );
        assert_eq!(fixture.completed.load(Ordering::Relaxed), 0);
        assert_eq!(client.active_cancellable_workers(), 0);
        assert!(
            settle_until(
                || fixture.active.load(Ordering::Relaxed) == 0,
                Duration::from_secs(2)
            ),
            "no connection survives the cycles"
        );
    }

    #[test]
    fn a_cancellable_request_is_still_cancelled_during_a_long_slow_response() {
        // The cancel slice exists for responsiveness: a request whose server
        // will not answer for 10 s must still return cancelled within about a
        // second, even though the worker keeps the full deadline.
        let (port, _requests) = serve_delayed(Duration::from_secs(10));
        let client = LocalHttpClient::plain("127.0.0.1", port).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(250));
            flag.store(true, Ordering::Relaxed);
        });
        let started = std::time::Instant::now();
        let error = client
            .post_json_cancellable(
                "/completion",
                &serde_json::json!({}),
                Duration::from_secs(30),
                &cancelled,
            )
            .unwrap_err();
        assert!(error.contains("cancelled"), "{error}");
        assert!(
            started.elapsed() < Duration::from_secs(2),
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
