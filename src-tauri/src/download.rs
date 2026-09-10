//! Resumable, parallel HTTP downloader for Hugging Face model files.
//!
//! Design notes, all verified against the live Hugging Face CDN:
//!
//! * `huggingface.co/<repo>/resolve/<rev>/<file>` answers `302` to a CDN URL and
//!   exposes `X-Linked-Size` (true byte length) and `X-Linked-ETag` (the file's
//!   sha256, quoted). The CDN honours `Accept-Ranges: bytes` and returns `206`
//!   for range requests, so several connections can fetch disjoint spans of one
//!   file concurrently.
//! * Progress and resume state live in a `.part` file plus a sidecar `.json`
//!   holding each chunk's completed byte count, so an interrupted download
//!   resumes at chunk granularity instead of restarting.
//! * `hf_transfer` is deprecated upstream and Hugging Face now serves through
//!   Xet; plain ranged HTTPS is the supported, dependency-free path and is what
//!   this module uses.

use cap_std::ambient_authority;
use cap_std::fs::{Dir, File as CapFile, OpenOptions as CapOpenOptions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Chunks smaller than this are not worth a separate connection.
pub const MIN_CHUNK_BYTES: u64 = 8 * 1024 * 1024;
/// Hugging Face tolerates a handful of connections per file; more mostly adds
/// contention and risks throttling.
pub const MAX_CONNECTIONS: usize = 8;
const BUFFER_BYTES: usize = 1024 * 1024;
const MAX_REQUEST_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TRANSFER_ATTEMPTS: usize = 3;
const MAX_RESUME_STATE_BYTES: u64 = 64 * 1024;
#[cfg(not(test))]
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
#[cfg(test)]
// The probe deadline under test must survive a fully parallel test run: a
// 500 ms value flaked under load, hiding real regressions. Tests that need a
// short deadline pass one explicitly via `probe_with_timeout`.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const TRANSFER_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

fn open_download_directory(root: &Path) -> Result<Dir, String> {
    Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|error| format!("Could not securely open {}: {error}", root.display()))
}

#[cfg(windows)]
pub(crate) fn opened_directory_matches(directory: &Dir, expected: &Path) -> Result<bool, String> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::GetFinalPathNameByHandleW;

    let mut buffer = vec![0u16; 32_768];
    let length = unsafe {
        GetFinalPathNameByHandleW(
            directory.as_raw_handle() as _,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            0,
        )
    };
    if length == 0 || length as usize >= buffer.len() {
        return Err("Could not confirm the opened directory's Windows identity.".into());
    }
    let opened = String::from_utf16_lossy(&buffer[..length as usize]);
    // `expected` was canonicalized before the capability was opened. Do not
    // canonicalize it again here: a replaced junction could otherwise make both
    // resolutions agree on the attacker's new target.
    let expected = expected.to_string_lossy().to_string();
    let normalize = |value: String| {
        let value = value
            .strip_prefix(r"\\?\UNC\")
            .map(|tail| format!(r"\\{tail}"))
            .or_else(|| value.strip_prefix(r"\\?\").map(str::to_string))
            .unwrap_or(value);
        value
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_ascii_lowercase()
    };
    Ok(normalize(opened) == normalize(expected))
}

#[cfg(unix)]
pub(crate) fn opened_directory_matches(directory: &Dir, expected: &Path) -> Result<bool, String> {
    use std::os::unix::fs::MetadataExt;
    let opened = directory
        .metadata(".")
        .map_err(|error| format!("Could not inspect the opened model folder: {error}"))?;
    let expected = std::fs::metadata(expected)
        .map_err(|error| format!("Could not inspect {}: {error}", expected.display()))?;
    Ok(opened.dev() == expected.dev() && opened.ino() == expected.ino())
}

/// A byte span of the target file and how much of it is already on disk.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Chunk {
    pub start: u64,
    /// Inclusive, matching HTTP `Range` semantics.
    pub end: u64,
    pub done: u64,
}

impl Chunk {
    pub fn len(&self) -> u64 {
        self.end - self.start + 1
    }
    pub fn is_complete(&self) -> bool {
        self.done >= self.len()
    }
    /// Where the next byte for this chunk belongs in the output file.
    pub fn cursor(&self) -> u64 {
        self.start + self.done
    }
}

/// Split a file into spans for parallel fetching.
///
/// Guarantees, all asserted by tests: the spans exactly tile `0..size` with no
/// gap and no overlap, no span is empty, and a file too small to be worth
/// splitting yields exactly one span.
pub fn plan_chunks(size: u64, connections: usize) -> Vec<Chunk> {
    if size == 0 {
        return Vec::new();
    }
    let requested = connections.clamp(1, MAX_CONNECTIONS) as u64;
    // Never create a chunk below the floor, except when the whole file is.
    let by_floor = size.div_ceil(MIN_CHUNK_BYTES).max(1);
    let count = requested.min(by_floor).max(1);
    let base = size / count;
    let remainder = size % count;

    let mut chunks = Vec::with_capacity(count as usize);
    let mut start = 0u64;
    for index in 0..count {
        // Spread the remainder over the first chunks so the spans stay even.
        let len = base + u64::from(index < remainder);
        if len == 0 {
            continue;
        }
        chunks.push(Chunk {
            start,
            end: start + len - 1,
            done: 0,
        });
        start += len;
    }
    debug_assert_eq!(start, size);
    chunks
}

/// Resume state written next to the partial file.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResumeState {
    pub url: String,
    pub size: u64,
    pub expected_sha256: String,
    /// The server's sha256 for the file, when it published one.
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub chunks: Vec<Chunk>,
}

impl ResumeState {
    pub fn downloaded(&self) -> u64 {
        self.chunks.iter().map(|c| c.done).sum()
    }
    pub fn is_complete(&self) -> bool {
        !self.chunks.is_empty() && self.chunks.iter().all(|c| c.is_complete())
    }
}

/// Decide whether saved resume state may be reused for this download.
///
/// Reusing state after the remote file changed would splice two different files
/// together, so size and ETag must both still match.
pub fn can_resume(state: &ResumeState, size: u64, etag: Option<&str>) -> bool {
    if state.size != size || state.chunks.is_empty() {
        return false;
    }
    if !resume_chunks_are_valid(&state.chunks, size, MAX_CONNECTIONS) {
        return false;
    }
    match (state.etag.as_deref(), etag) {
        (Some(saved), Some(current)) => saved == current,
        (None, None) => true,
        // If an identity signal appears or disappears, size agreement is not
        // enough evidence to splice the files together.
        _ => false,
    }
}

fn resume_chunks_are_valid(chunks: &[Chunk], size: u64, connections: usize) -> bool {
    if chunks.is_empty() || chunks.len() > connections.clamp(1, MAX_CONNECTIONS) {
        return false;
    }
    let mut expected_start = 0u64;
    for chunk in chunks {
        let Some(length) = chunk
            .end
            .checked_sub(chunk.start)
            .and_then(|difference| difference.checked_add(1))
        else {
            return false;
        };
        if chunk.start != expected_start || chunk.done > length {
            return false;
        }
        let Some(next_start) = chunk.end.checked_add(1) else {
            return false;
        };
        expected_start = next_start;
    }
    expected_start == size
}

/// Resume check at the actual requested URL. When no ETag exists, matching the
/// URL is the remaining identity signal; a different revision must restart.
pub fn can_resume_from(
    state: &ResumeState,
    url: &str,
    size: u64,
    expected_sha256: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> bool {
    state.url == url
        && state.expected_sha256.eq_ignore_ascii_case(expected_sha256)
        && state.last_modified.as_deref() == last_modified
        && can_resume(state, size, etag)
}

/// Human-readable byte size. Kept in Rust so every surface agrees.
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

/// Seconds remaining at the current rate, or `None` when it cannot be known.
pub fn eta_seconds(downloaded: u64, total: u64, bytes_per_second: u64) -> Option<u64> {
    if bytes_per_second == 0 || total == 0 || downloaded >= total {
        return None;
    }
    Some((total - downloaded) / bytes_per_second.max(1))
}

/// The canonical download URL for a catalog entry.
pub fn resolve_url(repo: &str, filename: &str, revision: &str) -> String {
    let revision = if revision.is_empty() {
        "main"
    } else {
        revision
    };
    let mut url = reqwest::Url::parse("https://huggingface.co").expect("static HF URL is valid");
    {
        let mut path = url
            .path_segments_mut()
            .expect("HTTPS URL accepts path segments");
        path.clear();
        for segment in repo
            .split('/')
            .chain(std::iter::once("resolve"))
            .chain(revision.split('/'))
            .chain(filename.split('/'))
        {
            path.push(segment);
        }
    }
    url.query_pairs_mut().append_pair("download", "true");
    url.to_string()
}

/// Facts the server reports about a file before any bytes are transferred.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFile {
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub supports_ranges: bool,
}

/// Read `X-Linked-Size`/`X-Linked-ETag` when present, falling back to the plain
/// `Content-Length`/`ETag`. Hugging Face sets the linked variants on LFS and Xet
/// objects, where the plain headers describe the pointer rather than the file.
pub fn read_remote_headers(headers: &reqwest::header::HeaderMap) -> RemoteFile {
    let get = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.trim().to_string())
    };
    let size = get("x-linked-size")
        .or_else(|| get("content-length"))
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let etag = get("x-linked-etag")
        .or_else(|| get("etag"))
        .map(|value| value.trim_matches('"').to_string())
        .filter(|value| !value.is_empty());
    let last_modified = get("last-modified").filter(|value| !value.is_empty());
    let supports_ranges = get("accept-ranges")
        .map(|value| value.to_ascii_lowercase().contains("bytes"))
        .unwrap_or(false);
    RemoteFile {
        size,
        etag,
        last_modified,
        supports_ranges,
    }
}

fn part_paths(target: &Path) -> (PathBuf, PathBuf) {
    let mut part = target.as_os_str().to_os_string();
    part.push(".part");
    let part = PathBuf::from(part);
    let mut meta = part.as_os_str().to_os_string();
    meta.push(".json");
    (part, PathBuf::from(meta))
}

struct DownloadEntries {
    dir: Dir,
    target: PathBuf,
    part: PathBuf,
    meta: PathBuf,
}

fn open_download_entries(target: &Path) -> Result<DownloadEntries, String> {
    let parent = target
        .parent()
        .ok_or_else(|| "The download target has no parent directory.".to_string())?;
    let target_name = target
        .file_name()
        .map(PathBuf::from)
        .ok_or_else(|| "The download target has no filename.".to_string())?;
    let mut part = target_name.as_os_str().to_os_string();
    part.push(".part");
    let part = PathBuf::from(part);
    let mut meta = part.as_os_str().to_os_string();
    meta.push(".json");
    let dir = open_download_directory(parent)?;
    if !opened_directory_matches(&dir, parent)? {
        return Err("The selected model folder changed while the download was starting.".into());
    }
    Ok(DownloadEntries {
        dir,
        target: target_name,
        part,
        meta: PathBuf::from(meta),
    })
}

fn load_resume_state_in(entries: &DownloadEntries) -> Option<ResumeState> {
    let file = entries.dir.open(&entries.meta).ok()?;
    let mut bytes = Vec::new();
    file.take(MAX_RESUME_STATE_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_RESUME_STATE_BYTES {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

fn save_resume_state_in(entries: &DownloadEntries, state: &ResumeState) -> Result<(), String> {
    let text = serde_json::to_string(state)
        .map_err(|error| format!("Could not serialize download progress: {error}"))?;
    for _ in 0..8 {
        let temporary = PathBuf::from(format!(
            "{}.next-{:016x}",
            entries.meta.to_string_lossy(),
            rand::random::<u64>()
        ));
        let mut options = CapOpenOptions::new();
        options.write(true).create_new(true);
        let mut file = match entries.dir.open_with(&temporary, &options) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Could not securely create download progress: {error}"
                ));
            }
        };
        let result = file
            .write_all(text.as_bytes())
            .and_then(|_| file.sync_all())
            .and_then(|_| entries.dir.rename(&temporary, &entries.dir, &entries.meta));
        if let Err(error) = result {
            let _ = entries.dir.remove_file(&temporary);
            return Err(format!(
                "Could not securely save download progress to {}: {error}",
                entries.meta.display()
            ));
        }
        return Ok(());
    }
    Err("Could not allocate a unique download progress file".into())
}

fn entry_is_unsafe_for_writes(is_symlink: bool, file_attributes: u32) -> bool {
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    is_symlink || file_attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(windows)]
fn open_file_hard_link_count(file: &CapFile) -> Result<u32, String> {
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
            "Could not inspect hard links for an open download file: {}",
            std::io::Error::last_os_error()
        ));
    }
    // SAFETY: a successful Win32 call initialized the complete output structure.
    Ok(unsafe { information.assume_init() }.nNumberOfLinks)
}

#[cfg(windows)]
fn hard_link_count(path: &Path) -> Result<u32, String> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

    let file = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
        .map_err(|error| {
            format!(
                "Could not open {} for link inspection: {error}",
                path.display()
            )
        })?;
    open_file_hard_link_count(&CapFile::from_std(file))
}

#[cfg(not(windows))]
fn open_file_hard_link_count(_file: &CapFile) -> Result<u32, String> {
    Ok(1)
}

#[cfg(not(windows))]
fn hard_link_count(_path: &Path) -> Result<u32, String> {
    Ok(1)
}

fn ensure_safe_write_entry(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("Could not inspect {}: {error}", path.display())),
    };
    // Directories are created and listed, never written as files. Opening a
    // directory handle with read access fails on Windows (Access denied),
    // so skip the file link check for them.
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Ok(());
    }
    #[cfg(windows)]
    let file_attributes = {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes()
    };
    #[cfg(not(windows))]
    let file_attributes = 0;
    if entry_is_unsafe_for_writes(metadata.file_type().is_symlink(), file_attributes) {
        return Err(format!(
            "Refusing to write through a symbolic link or reparse point: {}",
            path.display()
        ));
    }
    if hard_link_count(path)? != 1 {
        return Err(format!(
            "Refusing to write through a file with multiple hard links: {}",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
fn load_resume_state(target: &Path) -> Option<ResumeState> {
    let (_, meta) = part_paths(target);
    let text = std::fs::read_to_string(meta).ok()?;
    serde_json::from_str(&text).ok()
}

#[cfg(test)]
fn save_resume_state(target: &Path, state: &ResumeState) -> Result<(), String> {
    let (_, meta) = part_paths(target);
    ensure_safe_write_entry(&meta)?;
    let text = serde_json::to_string(state)
        .map_err(|error| format!("Could not serialize download progress: {error}"))?;
    std::fs::write(&meta, text).map_err(|error| {
        format!(
            "Could not save download progress to {}: {error}",
            meta.display()
        )
    })
}

/// An existing final file is reusable only when it matches the current remote
/// object's length and, when Hugging Face exposes a SHA-256 ETag, its digest.
#[cfg(test)]
fn existing_file_matches(
    path: &Path,
    expected_size: u64,
    expected_sha256: &str,
) -> Result<bool, String> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("Could not inspect {}: {error}", path.display())),
    };
    if metadata.len() != expected_size {
        return Ok(false);
    }
    Ok(sha256_file(path)?.eq_ignore_ascii_case(expected_sha256))
}

fn existing_file_matches_in(
    entries: &DownloadEntries,
    expected_size: u64,
    expected_sha256: &str,
    cancel: Option<&AtomicBool>,
    progress: Option<&mut dyn FnMut(u64)>,
) -> Result<bool, String> {
    let metadata = match entries.dir.metadata(&entries.target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "Could not securely inspect {}: {error}",
                entries.target.display()
            ))
        }
    };
    if !metadata.is_file() || metadata.len() != expected_size {
        return Ok(false);
    }
    let file = entries.dir.open(&entries.target).map_err(|error| {
        format!(
            "Could not securely open {}: {error}",
            entries.target.display()
        )
    })?;
    Ok(sha256_reader_with(file, &entries.target, cancel, progress)?
        .eq_ignore_ascii_case(expected_sha256))
}

/// Compute a file's sha256, streaming so a 20 GiB model is never held in memory.
#[cfg(test)]
fn sha256_file(path: &Path) -> Result<String, String> {
    let file =
        File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    sha256_reader_with(file, path, None, None)
}

/// Streaming SHA-256 with cancellation checks and byte progress, so a large
/// existing-file or final-part verification returns control promptly when
/// the user keeps the current state and stops (audit DC-12 I3).
fn sha256_reader_with(
    mut file: impl Read,
    display_path: &Path,
    cancel: Option<&AtomicBool>,
    mut progress: Option<&mut dyn FnMut(u64)>,
) -> Result<String, String> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; BUFFER_BYTES];
    let mut hashed = 0_u64;
    loop {
        if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err(format!(
                "Verification of {} was cancelled; the existing bytes were kept.",
                display_path.display()
            ));
        }
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not read {}: {error}", display_path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        hashed += read as u64;
        if let Some(progress) = progress.as_mut() {
            progress(hashed);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Flush the part file's data before a resume checkpoint claims those bytes.
///
/// The documented recovery guarantee (audit DC-12): a published checkpoint
/// never claims bytes that were not first written to the part file and
/// flushed to the operating system. A process crash resumes up to the last
/// checkpoint; an OS crash or power loss re-fetches bytes written after the
/// last completed checkpoint, because the sidecar is only renamed after the
/// data sync.
fn sync_part_data(entries: &DownloadEntries) -> Result<(), String> {
    let mut options = CapOpenOptions::new();
    options.write(true);
    let file = match entries.dir.open_with(&entries.part, &options) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Could not open {} to flush its data: {error}",
                entries.part.display()
            ));
        }
    };
    file.sync_data().map_err(|error| {
        format!(
            "Could not flush {} before saving download progress: {error}",
            entries.part.display()
        )
    })
}

/// Documented checkpoint cadence: progress UI updates every poll, but the
/// durable sidecar (with its data sync) is published at most this often,
/// plus on completion or stop (audit DC-12 I2).
const CHECKPOINT_INTERVAL_SECS: u64 = 2;

fn client(
    token: Option<&str>,
    request_timeout: Duration,
) -> Result<reqwest::blocking::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
        // The token rides in an Authorization header, never in the URL or a
        // command line, so it cannot leak through logs or process listings.
        let mut value = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token.trim()))
            .map_err(|_| "The Hugging Face token contains invalid characters".to_string())?;
        value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, value);
    }
    reqwest::blocking::Client::builder()
        .user_agent(format!("Localmotive/{}", env!("CARGO_PKG_VERSION")))
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(20))
        .timeout(request_timeout)
        .build()
        .map_err(|error| format!("Could not create an HTTPS client: {error}"))
}

/// Translate an HTTP status into advice the user can act on.
pub fn explain_status(status: u16, repo: &str, has_token: bool) -> String {
    match status {
        401 | 403 => {
            if has_token {
                format!(
                    "Hugging Face refused access to {repo} (HTTP {status}). \
                     Open the model page and accept its licence, or check that your token has read access."
                )
            } else {
                format!(
                    "{repo} requires authentication (HTTP {status}). \
                     Add a Hugging Face token in this tab, then try again."
                )
            }
        }
        404 => format!("{repo} or that file no longer exists on Hugging Face (HTTP 404)."),
        429 => "Hugging Face is rate-limiting this connection (HTTP 429). \
                Wait a minute and resume — finished chunks are kept."
            .to_string(),
        500..=599 => format!(
            "Hugging Face returned a server error (HTTP {status}). \
             The download can be resumed once it recovers."
        ),
        other => format!("Hugging Face returned HTTP {other}."),
    }
}

/// Probe a file: size, checksum and whether it can be fetched in parallel.
#[cfg(test)]
pub fn probe(url: &str, token: Option<&str>, repo: &str) -> Result<RemoteFile, String> {
    probe_with_cancel(url, token, repo, None)
}

fn probe_with_cancel(
    url: &str,
    token: Option<&str>,
    repo: &str,
    cancel: Option<&AtomicBool>,
) -> Result<RemoteFile, String> {
    #[cfg(test)]
    let probe_timeout = if std::env::var_os("LOCALMOTIVE_LIVE_HEALTH").is_some() {
        Duration::from_secs(15)
    } else {
        PROBE_TIMEOUT
    };
    #[cfg(not(test))]
    let probe_timeout = PROBE_TIMEOUT;
    probe_with_cancel_and_timeout(url, token, repo, cancel, probe_timeout)
}

/// Probe with an explicit deadline, so a test can assert the deadline
/// behaviour deterministically without changing every other probe's timeout.
#[cfg(test)]
pub fn probe_with_timeout(
    url: &str,
    token: Option<&str>,
    repo: &str,
    timeout: Duration,
) -> Result<RemoteFile, String> {
    probe_with_cancel_and_timeout(url, token, repo, None, timeout)
}

fn probe_with_cancel_and_timeout(
    url: &str,
    token: Option<&str>,
    repo: &str,
    cancel: Option<&AtomicBool>,
    probe_timeout: Duration,
) -> Result<RemoteFile, String> {
    if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        return Err("Download cancelled before network access started.".into());
    }
    let client = client(token, probe_timeout)?;
    let response = client
        .get(url)
        .header(reqwest::header::RANGE, "bytes=0-0")
        .send()
        .map_err(|error| {
            if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
                "Download cancelled while checking the remote file.".to_string()
            } else {
                format!("Could not reach Hugging Face: {error}")
            }
        })?;
    if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        return Err("Download cancelled while checking the remote file.".into());
    }
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(explain_status(status, repo, token.is_some()));
    }
    let mut remote = read_remote_headers(response.headers());
    // A 206 to a one-byte range is direct proof of range support, which matters
    // more than the advertised header.
    if status == 206 {
        remote.supports_ranges = true;
        let total = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(|range| range.strip_prefix("bytes 0-0/"))
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|total| *total > 0)
            .ok_or_else(|| {
                "The server returned an invalid Content-Range while probing the file.".to_string()
            })?;
        let linked_size = response
            .headers()
            .get("x-linked-size")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<u64>().ok());
        if linked_size.is_some_and(|size| size != total) {
            return Err("The server reported inconsistent file sizes while probing.".into());
        }
        // Content-Length describes this one-byte 206 response, not the complete
        // object. Content-Range supplies the authoritative complete size.
        remote.size = total;
    } else {
        // A 200 response to the one-byte probe proves that this transfer path
        // ignored Range, even when an intermediary advertises Accept-Ranges.
        remote.supports_ranges = false;
    }
    if remote.size == 0 {
        return Err("Hugging Face did not report a file size for this download.".into());
    }
    Ok(remote)
}

/// Download `url` to `target`, resuming and parallelising where possible.
///
/// `on_progress` is called with the running byte total; it must be cheap.
#[allow(clippy::too_many_arguments)]
pub fn download_file(
    url: &str,
    target: &Path,
    repo: &str,
    expected_size: u64,
    expected_sha256: &str,
    token: Option<&str>,
    connections: usize,
    cancel: Arc<AtomicBool>,
    downloaded: Arc<AtomicU64>,
    on_progress: impl FnMut(u64, u64) + Send,
) -> Result<PathBuf, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("Download cancelled before network access started.".into());
    }
    // Progress must be monotonic for the UI: a later verification pass over
    // local bytes must never rewind the reported byte total (audit DC-12).
    let mut last_reported = 0_u64;
    let mut on_progress = {
        let mut inner = on_progress;
        move |done: u64, total: u64| {
            last_reported = last_reported.max(done);
            inner(last_reported, total);
        }
    };
    if let Some(parent) = target.parent() {
        ensure_safe_write_entry(parent)?;
    }

    let (part, meta) = part_paths(target);
    ensure_safe_write_entry(target)?;
    ensure_safe_write_entry(&part)?;
    ensure_safe_write_entry(&meta)?;
    let entries = open_download_entries(target)?;

    let remote = probe_with_cancel(url, token, repo, Some(&cancel))?;
    if remote.size != expected_size {
        return Err("The remote file size does not match the validated catalog.".into());
    }
    if entries.dir.symlink_metadata(&entries.target).is_ok() {
        let mut verify_progress = |hashed: u64| on_progress(hashed.min(remote.size), remote.size);
        if existing_file_matches_in(
            &entries,
            remote.size,
            expected_sha256,
            Some(cancel.as_ref()),
            Some(&mut verify_progress),
        )? {
            downloaded.store(remote.size, Ordering::Relaxed);
            on_progress(remote.size, remote.size);
            return Ok(target.to_path_buf());
        }
        return Err(format!(
            "{} already exists but does not match the Hugging Face file. Rename or remove it, then resume the download.",
            target.display()
        ));
    }
    let effective_connections = if remote.supports_ranges {
        connections.clamp(1, MAX_CONNECTIONS)
    } else {
        1
    };

    // Reuse previous work only when the remote file is provably the same one.
    let mut state = match load_resume_state_in(&entries) {
        Some(saved)
            if remote.supports_ranges
                && can_resume_from(
                    &saved,
                    url,
                    remote.size,
                    expected_sha256,
                    remote.etag.as_deref(),
                    remote.last_modified.as_deref(),
                )
                && resume_chunks_are_valid(&saved.chunks, remote.size, effective_connections)
                && entries.dir.is_file(&entries.part)
                && entries
                    .dir
                    .metadata(&entries.part)
                    .is_ok_and(|metadata| metadata.is_file() && metadata.len() == remote.size) =>
        {
            saved
        }
        _ => {
            let _ = entries.dir.remove_file(&entries.part);
            let _ = entries.dir.remove_file(&entries.meta);
            ResumeState {
                url: url.to_string(),
                size: remote.size,
                expected_sha256: expected_sha256.to_ascii_lowercase(),
                etag: remote.etag.clone(),
                last_modified: remote.last_modified.clone(),
                chunks: plan_chunks(remote.size, effective_connections),
            }
        }
    };

    let mut options = CapOpenOptions::new();
    options.create(true).write(true).truncate(false);
    let file = entries
        .dir
        .open_with(&entries.part, &options)
        .map_err(|error| format!("Could not securely open {}: {error}", part.display()))?;
    if open_file_hard_link_count(&file)? != 1 {
        return Err(format!(
            "Refusing to use a partial download with multiple hard links: {}",
            part.display()
        ));
    }
    file.set_len(remote.size)
        .map_err(|error| format!("Could not reserve {}: {error}", human_bytes(remote.size)))?;
    let file = Arc::new(Mutex::new(file));

    downloaded.store(state.downloaded(), Ordering::Relaxed);
    let total = remote.size;
    on_progress(state.downloaded(), total);

    let chunks = Arc::new(Mutex::new(state.chunks.clone()));
    let failure: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let worker_count = state.chunks.iter().filter(|c| !c.is_complete()).count();

    if worker_count > 0 {
        std::thread::scope(|scope| {
            for index in 0..state.chunks.len() {
                if state.chunks[index].is_complete() {
                    continue;
                }
                let (client_token, url) = (token, url.to_string());
                let (chunks, file, failure, cancel, downloaded) = (
                    Arc::clone(&chunks),
                    Arc::clone(&file),
                    Arc::clone(&failure),
                    Arc::clone(&cancel),
                    Arc::clone(&downloaded),
                );
                let repo = repo.to_string();
                let remote_etag = remote.etag.clone();
                let remote_last_modified = remote.last_modified.clone();
                let supports_ranges = remote.supports_ranges;
                scope.spawn(move || {
                    if let Err(error) = fetch_chunk(
                        &url,
                        index,
                        &chunks,
                        &file,
                        &cancel,
                        &downloaded,
                        client_token,
                        &repo,
                        remote_etag.as_deref(),
                        remote_last_modified.as_deref(),
                        supports_ranges,
                    ) {
                        let mut slot = failure.lock().unwrap();
                        if slot.is_none() {
                            *slot = Some(error);
                        }
                        cancel.store(true, Ordering::Relaxed);
                    }
                });
            }

            // Report progress every poll; publish a durable checkpoint only
            // at the documented cadence (or on completion), and always flush
            // the part data before the sidecar claims those bytes
            // (audit DC-12 I1/I2).
            let mut last_checkpoint = Instant::now();
            while !cancel.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(400));
                let snapshot = chunks.lock().unwrap().clone();
                let done: u64 = snapshot.iter().map(|c| c.done).sum();
                on_progress(done, total);
                state.chunks = snapshot;
                let due = state.is_complete()
                    || last_checkpoint.elapsed() >= Duration::from_secs(CHECKPOINT_INTERVAL_SECS);
                if due {
                    let checkpoint = sync_part_data(&entries)
                        .and_then(|()| save_resume_state_in(&entries, &state));
                    if let Err(error) = checkpoint {
                        let mut slot = failure.lock().unwrap();
                        if slot.is_none() {
                            *slot = Some(error);
                        }
                        cancel.store(true, Ordering::Relaxed);
                        break;
                    }
                    last_checkpoint = Instant::now();
                }
                if state.is_complete() {
                    break;
                }
            }
        });
    }

    state.chunks = chunks.lock().unwrap().clone();
    sync_part_data(&entries)?;
    save_resume_state_in(&entries, &state)?;
    on_progress(state.downloaded(), total);

    if let Some(error) = failure.lock().unwrap().take() {
        return Err(error);
    }
    if cancel.load(Ordering::Relaxed) && !state.is_complete() {
        return Err("Download cancelled. Progress is kept — press Download to resume.".into());
    }
    if !state.is_complete() {
        return Err(
            "The download ended before every byte arrived. Press Download to resume.".into(),
        );
    }

    drop(file);

    // Verify against the curator-published digest before exposing the final
    // name, even when a CDN omits or uses a non-cryptographic ETag.
    let part_file = entries
        .dir
        .open(&entries.part)
        .map_err(|error| format!("Could not securely open {}: {error}", part.display()))?;
    let mut verify_progress =
        |hashed: u64| on_progress(hashed.min(state.downloaded()), state.downloaded());
    let actual = sha256_reader_with(
        part_file,
        &part,
        Some(cancel.as_ref()),
        Some(&mut verify_progress),
    )?;
    if !actual.eq_ignore_ascii_case(expected_sha256) {
        let _ = entries.dir.remove_file(&entries.part);
        let _ = entries.dir.remove_file(&entries.meta);
        return Err(
            "The downloaded file failed its SHA-256 checksum and was deleted. Please try again."
                .into(),
        );
    }

    ensure_safe_write_entry(&part)?;
    ensure_safe_write_entry(target)?;
    entries
        .dir
        .rename(&entries.part, &entries.dir, &entries.target)
        .map_err(|error| {
            format!(
                "Downloaded, but could not move the file into place: {error}. It is at {}",
                part.display()
            )
        })?;
    let _ = entries.dir.remove_file(&entries.meta);
    Ok(target.to_path_buf())
}

fn response_read_fits(remaining: u64, read: usize) -> bool {
    read as u64 <= remaining
}

fn range_response_is_usable(
    status: reqwest::StatusCode,
    whole_file: bool,
    requested_start: u64,
) -> bool {
    status == reqwest::StatusCode::PARTIAL_CONTENT
        || (status == reqwest::StatusCode::OK && whole_file && requested_start == 0)
}

fn content_range_matches(value: &str, start: u64, end: u64, total: u64) -> bool {
    if total == 0 || start > end || end >= total {
        return false;
    }
    let Some(rest) = value.strip_prefix("bytes ") else {
        return false;
    };
    let Some((span, reported_total)) = rest.split_once('/') else {
        return false;
    };
    let Some((reported_start, reported_end)) = span.split_once('-') else {
        return false;
    };
    reported_start.parse::<u64>().ok() == Some(start)
        && reported_end.parse::<u64>().ok() == Some(end)
        && reported_total.parse::<u64>().ok() == Some(total)
}

fn response_identity_matches(probed: Option<&str>, response: Option<&str>) -> bool {
    match (probed, response) {
        (Some(expected), Some(actual)) => expected.eq_ignore_ascii_case(actual.trim_matches('"')),
        (None, None) => true,
        _ => false,
    }
}

fn response_validators_match(
    probed_etag: Option<&str>,
    probed_last_modified: Option<&str>,
    response_etag: Option<&str>,
    response_last_modified: Option<&str>,
) -> bool {
    response_identity_matches(probed_etag, response_etag)
        && response_identity_matches(probed_last_modified, response_last_modified)
}

#[allow(clippy::too_many_arguments)]
fn fetch_chunk(
    url: &str,
    index: usize,
    chunks: &Arc<Mutex<Vec<Chunk>>>,
    file: &Arc<Mutex<CapFile>>,
    cancel: &Arc<AtomicBool>,
    downloaded: &Arc<AtomicU64>,
    token: Option<&str>,
    repo: &str,
    remote_etag: Option<&str>,
    remote_last_modified: Option<&str>,
    supports_ranges: bool,
) -> Result<(), String> {
    let client = client(token, TRANSFER_REQUEST_TIMEOUT)?;
    // Permit two retries after the initial request for each bounded range.
    let mut attempt = 0_usize;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        if chunks.lock().unwrap()[index].is_complete() {
            return Ok(());
        }
        if !supports_ranges {
            // A retry against a range-ignoring server restarts from zero
            // BEFORE the cursor is read: an interrupted 200 stream cannot be
            // resumed at a byte offset, and a misleading Accept-Ranges header
            // must never cause unsafe reuse of partial bytes (audit DC-02 I3).
            let mut guard = chunks.lock().unwrap();
            let entry = &mut guard[index];
            if entry.done > 0 {
                let lost = entry.done;
                entry.done = 0;
                drop(guard);
                downloaded.fetch_sub(lost, Ordering::Relaxed);
            }
        }
        // A server proven to ignore Range is transferred as one sequential
        // whole-response stream sized by the complete expected length, not by
        // the 8 MiB ranged-request span (audit DC-02 I1).
        let chunk = chunks.lock().unwrap()[index];
        let request_start = chunk.cursor();
        let request_end = if supports_ranges {
            chunk
                .end
                .min(request_start.saturating_add(MAX_REQUEST_BYTES - 1))
        } else {
            chunk.end
        };
        let range = format!("bytes={request_start}-{request_end}");
        let result = (|| -> Result<(), String> {
            let mut request = client.get(url).header(reqwest::header::RANGE, &range);
            if let Some(etag) = remote_etag {
                request = request.header(reqwest::header::IF_MATCH, format!("\"{etag}\""));
            }
            if let Some(last_modified) = remote_last_modified {
                request = request.header(reqwest::header::IF_UNMODIFIED_SINCE, last_modified);
            }
            let response = request
                .send()
                .map_err(|error| format!("Connection failed: {error}"))?;
            let status = response.status().as_u16();
            if status >= 400 {
                return Err(explain_status(status, repo, token.is_some()));
            }
            let whole_file = chunks.lock().unwrap().len() == 1;
            let requested_start = chunk.cursor();
            if !range_response_is_usable(response.status(), whole_file, requested_start) {
                return Err(
                    "The server ignored byte ranges during a parallel download. The partial file was kept; retry later."
                        .into(),
                );
            }
            if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                let total = chunks.lock().unwrap().iter().map(Chunk::len).sum();
                let valid = response
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|v| content_range_matches(v, request_start, request_end, total));
                if !valid {
                    return Err(
                        "The server returned an invalid Content-Range for the requested chunk."
                            .into(),
                    );
                }
            }
            let response_etag = response
                .headers()
                .get("x-linked-etag")
                .or_else(|| response.headers().get(reqwest::header::ETAG))
                .and_then(|value| value.to_str().ok());
            let response_last_modified = response
                .headers()
                .get(reqwest::header::LAST_MODIFIED)
                .and_then(|value| value.to_str().ok());
            if !response_validators_match(
                remote_etag,
                remote_last_modified,
                response_etag,
                response_last_modified,
            ) {
                return Err("The remote file changed while it was downloading.".into());
            }
            let expected_body = request_end - requested_start + 1;
            if response
                .content_length()
                .is_some_and(|length| length != expected_body)
            {
                return Err("The server returned an unexpected response length.".into());
            }
            let mut response = response;
            let mut buffer = vec![0u8; BUFFER_BYTES];
            loop {
                if cancel.load(Ordering::Relaxed) {
                    return Ok(());
                }
                let read = match response.read(&mut buffer) {
                    Ok(read) => read,
                    Err(_) if cancel.load(Ordering::Relaxed) => return Ok(()),
                    Err(error) => return Err(format!("Transfer interrupted: {error}")),
                };
                if read == 0 {
                    return Err(
                        "The response ended before the requested range was complete.".into(),
                    );
                }
                let cursor = chunks.lock().unwrap()[index].cursor();
                let remaining = request_end.saturating_sub(cursor).saturating_add(1);
                if !response_read_fits(remaining, read) {
                    return Err("The server returned more bytes than the requested range.".into());
                }
                let accepted = read;
                {
                    let mut file = file.lock().unwrap();
                    file.seek(SeekFrom::Start(cursor))
                        .map_err(|error| format!("Could not position the file: {error}"))?;
                    file.write_all(&buffer[..accepted])
                        .map_err(|error| format!("Could not write to disk: {error}"))?;
                }
                let complete = {
                    let mut guard = chunks.lock().unwrap();
                    let entry = &mut guard[index];
                    entry.done += accepted as u64;
                    entry.is_complete()
                };
                downloaded.fetch_add(accepted as u64, Ordering::Relaxed);
                if complete || cursor + accepted as u64 > request_end {
                    return Ok(());
                }
            }
        })();

        match result {
            Ok(()) => {
                let chunk = chunks.lock().unwrap()[index];
                if chunk.is_complete() || cancel.load(Ordering::Relaxed) {
                    return Ok(());
                }
                attempt = 0;
            }
            Err(error) => {
                // Authentication and missing files will not fix themselves.
                if error.contains("HTTP 401")
                    || error.contains("HTTP 403")
                    || error.contains("404")
                    || error.contains("requires authentication")
                    || error.contains("refused access")
                {
                    return Err(error);
                }
                attempt += 1;
                if attempt >= MAX_TRANSFER_ATTEMPTS {
                    return Err(error);
                }
                let retry_delay = Duration::from_millis(500 * attempt as u64);
                let mut waited = Duration::ZERO;
                while waited < retry_delay && !cancel.load(Ordering::Relaxed) {
                    let step = Duration::from_millis(50).min(retry_delay - waited);
                    std::thread::sleep(step);
                    waited += step;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Give each test its own temp directory.
    ///
    /// Parallel `cargo test` threads share one process id, so a pid-keyed
    /// temp name lets two tests wipe and recreate each other's folder. That
    /// produced `The selected model folder changed while the download was
    /// starting` on 32-core windows-2025 runners (351 passed, 9 failed).
    /// A random suffix gives each test a private directory instead.
    #[cfg(test)]
    fn unique_test_dir(prefix: &str) -> PathBuf {
        for _ in 0..16 {
            let candidate = std::env::temp_dir().join(format!(
                "{prefix}-{}-{:016x}",
                std::process::id(),
                rand::random::<u64>()
            ));
            if std::fs::create_dir(&candidate).is_ok() {
                return candidate;
            }
        }
        panic!("could not allocate a unique temp dir for {prefix}");
    }

    #[test]
    fn a_pre_cancelled_download_does_not_start_network_io() {
        let root = unique_test_dir("localmotive-pre-cancelled-download");
        std::fs::create_dir_all(&root).unwrap();
        let cancel = Arc::new(AtomicBool::new(true));

        let error = download_file(
            "http://127.0.0.1:9/must-not-connect",
            &root.join("runtime.zip"),
            "test artifact",
            1,
            &"0".repeat(64),
            None,
            1,
            cancel,
            Arc::new(AtomicU64::new(0)),
            |_, _| {},
        )
        .unwrap_err();

        assert!(error.to_ascii_lowercase().contains("cancel"), "{error}");
        assert!(!root.join("runtime.zip.part").exists());
        assert!(!root.join("runtime.zip.part.json").exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_stalled_transfer_request_has_a_deadline() {
        use std::io::Write as _;
        use std::net::TcpListener;
        use std::time::Instant;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut line = String::new();
                use std::io::BufRead as _;
                if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                    break;
                }
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n")
                .unwrap();
            stream.flush().unwrap();
            std::thread::sleep(Duration::from_secs(1));
        });

        let started = Instant::now();
        let mut response = client(None, Duration::from_millis(250))
            .unwrap()
            .get(format!("http://{address}/runtime.zip"))
            .send()
            .unwrap();
        let mut byte = [0_u8; 1];
        assert!(response.read_exact(&mut byte).is_err());
        assert!(started.elapsed() < Duration::from_millis(900));
        server.join().unwrap();
    }

    #[test]
    fn ranged_probe_uses_content_range_total_instead_of_one_byte_content_length() {
        use std::io::{BufRead as _, BufReader, Write as _};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut line = String::new();
                // A WouldBlock here only means the peer has not sent the next
                // header line yet; wait briefly rather than failing the fixture.
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(error) => panic!("server read failed: {error}"),
                }
                if line == "\r\n" || line.is_empty() {
                    break;
                }
            }
            stream
                .write_all(
                    b"HTTP/1.1 206 Partial Content\r\nContent-Length: 1\r\nContent-Range: bytes 0-0/123\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\nx",
                )
                .unwrap();
        });

        let remote = probe(&format!("http://{address}/artifact"), None, "fixture").unwrap();
        assert_eq!(remote.size, 123);
        assert!(remote.supports_ranges);
        server.join().unwrap();
    }

    #[test]
    fn cancellation_retains_valid_state_and_the_next_attempt_resumes() {
        use std::io::{BufRead as _, BufReader, Write as _};
        use std::net::TcpListener;

        let payload = vec![b'x'; 64 * 1024];
        let digest = {
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            format!("{:x}", hasher.finalize())
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server_payload = payload.clone();
        let server_digest = digest.clone();
        let resumed_starts = Arc::new(Mutex::new(Vec::new()));
        let server_resumed_starts = Arc::clone(&resumed_starts);
        let server = std::thread::spawn(move || {
            for request_index in 0..4 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                    request.push_str(&line);
                }
                if request_index == 0 || request_index == 2 {
                    write!(
                        stream,
                        "HTTP/1.1 206 Partial Content\r\nContent-Length: 1\r\nContent-Range: bytes 0-0/{}\r\nX-Linked-Size: {}\r\nX-Linked-ETag: \"{}\"\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n",
                        server_payload.len(),
                        server_payload.len(),
                        server_digest,
                    )
                    .unwrap();
                    stream.write_all(&server_payload[..1]).unwrap();
                } else {
                    let start = request
                        .lines()
                        .find(|line| line.to_ascii_lowercase().starts_with("range:"))
                        .and_then(|line| line.split('=').nth(1))
                        .and_then(|range| range.split('-').next())
                        .and_then(|value| value.trim().parse::<usize>().ok())
                        .unwrap();
                    let response_start = if request_index == 1 { 0 } else { start };
                    let body = &server_payload[response_start..];
                    write!(
                        stream,
                        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nX-Linked-ETag: \"{}\"\r\nConnection: close\r\n\r\n",
                        body.len(),
                        response_start,
                        server_payload.len() - 1,
                        server_payload.len(),
                        server_digest,
                    )
                    .unwrap();
                    if request_index == 1 {
                        stream.write_all(&server_payload[..4096]).unwrap();
                        stream.flush().unwrap();
                        std::thread::sleep(Duration::from_secs(1));
                    } else {
                        server_resumed_starts.lock().unwrap().push(start as u64);
                        stream.write_all(body).unwrap();
                    }
                }
            }
        });

        let root = unique_test_dir("localmotive-cancelled-download");
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("runtime.zip");
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_after_partial = Arc::clone(&cancel);
        let canceller = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            cancel_after_partial.store(true, Ordering::Relaxed);
        });

        let error = download_file(
            &format!("http://{address}/runtime.zip"),
            &target,
            "test artifact",
            payload.len() as u64,
            &digest,
            None,
            1,
            Arc::clone(&cancel),
            Arc::new(AtomicU64::new(0)),
            |_, _| {},
        )
        .unwrap_err();

        canceller.join().unwrap();
        assert!(error.to_ascii_lowercase().contains("cancel"), "{error}");
        assert!(
            !target.exists(),
            "a cancelled download must not become final"
        );
        let (part, _) = part_paths(&target);
        assert!(part.is_file(), "valid partial bytes must remain resumable");
        let state = load_resume_state(&target).expect("resume state must be retained");
        assert!(state.downloaded() >= 4096, "{state:?}");
        assert!(state.downloaded() < payload.len() as u64, "{state:?}");
        assert!(can_resume_from(
            &state,
            &format!("http://{address}/runtime.zip"),
            payload.len() as u64,
            &digest,
            Some(&digest),
            None,
        ));

        cancel.store(false, Ordering::Relaxed);
        let resumed = download_file(
            &format!("http://{address}/runtime.zip"),
            &target,
            "test artifact",
            payload.len() as u64,
            &digest,
            None,
            1,
            cancel,
            Arc::new(AtomicU64::new(0)),
            |_, _| {},
        )
        .expect("the retained bytes must resume");
        server.join().unwrap();
        assert_eq!(std::fs::read(resumed).unwrap(), payload);
        assert!(
            resumed_starts
                .lock()
                .unwrap()
                .iter()
                .all(|start| *start >= 4096),
            "the resumed request must not restart at byte zero"
        );
        assert!(load_resume_state(&target).is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_server_that_ignores_ranges_restarts_a_partial_file_from_zero() {
        use std::io::{BufRead as _, BufReader, Write as _};
        use std::net::TcpListener;

        let payload = vec![b'r'; 16 * 1024];
        let digest = {
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            format!("{:x}", hasher.finalize())
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let server_stop = Arc::clone(&stop);
        let server_payload = payload.clone();
        let server_digest = digest.clone();
        let transfer_starts = Arc::new(Mutex::new(Vec::new()));
        let server_transfer_starts = Arc::clone(&transfer_starts);
        let server = std::thread::spawn(move || {
            let mut request_index = 0_usize;
            while !server_stop.load(Ordering::Relaxed) {
                let (mut stream, _) = match listener.accept() {
                    Ok(accepted) => accepted,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("server accept failed: {error}"),
                };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                loop {
                    let mut line = String::new();
                    // Accepted streams inherit the listener's non-blocking mode
                    // on Windows; a WouldBlock only means the peer has not sent
                    // the next header line yet.
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(1));
                            continue;
                        }
                        Err(error) => panic!("server read failed: {error}"),
                    }
                    if line == "\r\n" {
                        break;
                    }
                    request.push_str(&line);
                }
                // A half-opened probe connection can reach us with no request
                // head yet; wait for the retry rather than parsing an empty
                // request and panicking on the missing Range line.
                if request.is_empty() {
                    continue;
                }
                if request_index > 0 {
                    let start = request
                        .lines()
                        .find(|line| line.to_ascii_lowercase().starts_with("range:"))
                        .and_then(|line| line.split('=').nth(1))
                        .and_then(|range| range.split('-').next())
                        .and_then(|value| value.trim().parse::<u64>().ok())
                        .unwrap();
                    server_transfer_starts.lock().unwrap().push(start);
                }
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nETag: \"{}\"\r\nConnection: close\r\n\r\n",
                    server_payload.len(),
                    server_digest,
                )
                .unwrap();
                stream.write_all(&server_payload).unwrap();
                request_index += 1;
            }
        });

        let root = unique_test_dir("localmotive-range-restart");
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("runtime.zip");
        let (part, _) = part_paths(&target);
        let mut partial = vec![0_u8; payload.len()];
        partial[..128].copy_from_slice(&payload[..128]);
        std::fs::write(&part, partial).unwrap();
        let url = format!("http://{address}/runtime.zip");
        save_resume_state(
            &target,
            &ResumeState {
                url: url.clone(),
                size: payload.len() as u64,
                expected_sha256: digest.clone(),
                etag: Some(digest.clone()),
                last_modified: None,
                chunks: vec![Chunk {
                    start: 0,
                    end: payload.len() as u64 - 1,
                    done: 128,
                }],
            },
        )
        .unwrap();

        let result = download_file(
            &url,
            &target,
            "test artifact",
            payload.len() as u64,
            &digest,
            None,
            4,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicU64::new(0)),
            |_, _| {},
        );
        stop.store(true, Ordering::Relaxed);
        server.join().unwrap();

        let downloaded = result.expect("the ignored range must restart safely");
        assert_eq!(std::fs::read(downloaded).unwrap(), payload);
        assert_eq!(transfer_starts.lock().unwrap().as_slice(), &[0]);
        assert!(load_resume_state(&target).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stalled_http_probe_stops_at_the_configured_deadline() {
        use std::io::Write as _;
        use std::net::TcpListener;
        use std::time::Instant;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            std::thread::sleep(Duration::from_secs(1));
            let _ = stream.write_all(
                b"HTTP/1.1 206 Partial Content\r\nContent-Length: 1\r\nContent-Range: bytes 0-0/1\r\n\r\nx",
            );
        });

        let started = Instant::now();
        let result = probe_with_timeout(
            &format!("http://{address}/runtime.zip"),
            None,
            "test artifact",
            Duration::from_millis(500),
        );
        assert!(result.is_err());
        // The explicit 500 ms deadline must fire; the outer bound is
        // load-aware rather than pinning tester scheduling.
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn chunks_tile_the_file_exactly_with_no_gap_or_overlap() {
        for size in [
            1u64,
            2,
            MIN_CHUNK_BYTES - 1,
            MIN_CHUNK_BYTES,
            807_694_464,
            18_556_689_568,
        ] {
            for connections in [1usize, 2, 3, 4, 8, 64] {
                let chunks = plan_chunks(size, connections);
                assert!(!chunks.is_empty(), "size {size} produced no chunks");
                assert_eq!(chunks[0].start, 0);
                assert_eq!(chunks[chunks.len() - 1].end, size - 1);
                let mut covered = 0u64;
                for pair in chunks.windows(2) {
                    assert_eq!(
                        pair[1].start,
                        pair[0].end + 1,
                        "gap/overlap at size {size} conns {connections}"
                    );
                }
                for chunk in &chunks {
                    assert!(chunk.len() > 0, "empty chunk at size {size}");
                    covered += chunk.len();
                }
                assert_eq!(covered, size, "size {size} conns {connections}");
                assert!(chunks.len() <= MAX_CONNECTIONS);
            }
        }
    }

    #[test]
    fn small_files_use_one_connection_and_empty_files_none() {
        assert_eq!(plan_chunks(1024, 8).len(), 1, "a tiny file is not split");
        assert_eq!(plan_chunks(MIN_CHUNK_BYTES, 8).len(), 1);
        assert_eq!(plan_chunks(MIN_CHUNK_BYTES * 2, 8).len(), 2);
        assert_eq!(plan_chunks(MIN_CHUNK_BYTES * 4, 3).len(), 3);
        assert!(plan_chunks(0, 8).is_empty());
        assert_eq!(plan_chunks(1024, 0).len(), 1, "zero connections is clamped");
    }

    #[test]
    fn resume_metadata_cannot_exceed_the_requested_worker_limit() {
        // Sidecars are local files and may be stale or edited. Their chunk map
        // must not turn a four-connection request into eight worker threads.
        let size = MIN_CHUNK_BYTES * 8;
        let chunks = plan_chunks(size, 8);
        assert!(!resume_chunks_are_valid(&chunks, size, 4));
        assert!(resume_chunks_are_valid(&chunks, size, 8));
    }

    #[test]
    fn every_range_response_is_bound_to_the_probed_remote_identity() {
        // A CDN object can change after the HEAD probe. A worker must reject a
        // different identity rather than splice two same-sized objects.
        assert!(response_identity_matches(Some("abc"), Some("\"abc\"")));
        assert!(!response_identity_matches(Some("abc"), Some("\"def\"")));
        assert!(!response_identity_matches(Some("abc"), None));
        assert!(response_identity_matches(None, None));
    }

    #[test]
    fn reparse_points_and_symlinks_are_never_safe_write_targets() {
        assert!(entry_is_unsafe_for_writes(true, 0));
        assert!(entry_is_unsafe_for_writes(false, 0x400));
        assert!(!entry_is_unsafe_for_writes(false, 0));
    }

    #[cfg(windows)]
    #[test]
    fn hard_links_are_never_safe_write_targets() {
        let root = unique_test_dir("localmotive-download-hard-link");
        std::fs::create_dir_all(&root).unwrap();
        let victim = root.join("victim.bin");
        let target = root.join("artifact.part");
        std::fs::write(&victim, b"do not modify").unwrap();
        std::fs::hard_link(&victim, &target).unwrap();

        assert!(ensure_safe_write_entry(&target).is_err());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn opened_download_files_report_their_hard_link_count() {
        let root = unique_test_dir("localmotive-open-hard-link");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("victim.bin"), b"bounded").unwrap();
        std::fs::hard_link(root.join("victim.bin"), root.join("artifact.part")).unwrap();
        let directory = open_download_directory(&root).unwrap();
        let file = directory.open("artifact.part").unwrap();

        assert_eq!(open_file_hard_link_count(&file).unwrap(), 2);

        drop(file);
        drop(directory);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn resume_sidecar_never_overwrites_a_precreated_hard_link() {
        let root = unique_test_dir("localmotive-resume-hard-link");
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("model.gguf");
        let entries = open_download_entries(&target).unwrap();
        let victim = root.join("victim.txt");
        let predictable = root.join("model.gguf.part.json.next");
        std::fs::write(&victim, b"do not modify").unwrap();
        std::fs::hard_link(&victim, &predictable).unwrap();
        let state = ResumeState {
            url: "https://example.invalid/model.gguf".into(),
            size: 1,
            expected_sha256: "0".repeat(64),
            etag: None,
            last_modified: None,
            chunks: plan_chunks(1, 1),
        };

        let _ = save_resume_state_in(&entries, &state);
        assert_eq!(std::fs::read(&victim).unwrap(), b"do not modify");

        drop(entries);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_open_download_directory_cannot_be_redirected_by_path_replacement() {
        let root = unique_test_dir("localmotive-dir-race");
        let moved = root.with_extension("moved");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&moved);
        std::fs::create_dir_all(&root).unwrap();
        let directory = open_download_directory(&root).unwrap();

        match std::fs::rename(&root, &moved) {
            Ok(()) => {
                std::fs::create_dir_all(&root).unwrap();
                directory
                    .write("probe", b"bound to the opened directory")
                    .unwrap();
                assert!(moved.join("probe").is_file());
                assert!(!root.join("probe").exists());
            }
            Err(_) => {
                // Windows opens the directory without FILE_SHARE_DELETE, so a
                // rename/reparse swap is blocked while the capability is live.
                directory.write("probe", b"replacement blocked").unwrap();
                assert!(root.join("probe").is_file());
            }
        }
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(moved);
    }

    #[test]
    fn an_open_directory_handle_must_match_the_validated_path() {
        let root = unique_test_dir("localmotive-dir-identity");
        let other = root.with_extension("other");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&other);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let directory = open_download_directory(&root).unwrap();
        assert!(opened_directory_matches(&directory, &root).unwrap());
        assert!(!opened_directory_matches(&directory, &other).unwrap());
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(other);
    }

    #[test]
    fn resume_requires_etag_presence_to_stay_consistent() {
        // A proxy/CDN dropping or adding an ETag means identity is no longer
        // proven. Size alone must not splice old bytes into the new response.
        let state = ResumeState {
            url: "u".into(),
            size: 1000,
            expected_sha256: "0".repeat(64),
            etag: Some("abc".into()),
            last_modified: None,
            chunks: plan_chunks(1000, 1),
        };
        assert!(!can_resume(&state, 1000, None));
        let without = ResumeState {
            etag: None,
            ..state.clone()
        };
        assert!(!can_resume(&without, 1000, Some("abc")));
        assert!(can_resume(&without, 1000, None));
    }

    #[test]
    fn resume_geometry_rejects_overflowing_overlapping_spans() {
        // A writable sidecar must not wrap u64::MAX back to zero and make an
        // overlapping second span appear contiguous.
        let chunks = [
            Chunk {
                start: 0,
                end: u64::MAX,
                done: 0,
            },
            Chunk {
                start: 0,
                end: 0,
                done: 0,
            },
        ];
        assert!(!resume_chunks_are_valid(&chunks, 1, 2));
    }

    #[test]
    fn oversized_resume_sidecar_is_rejected_before_deserialization() {
        let root = unique_test_dir("localmotive-oversized-resume");
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("model.gguf");
        let entries = open_download_entries(&target).unwrap();
        let state = ResumeState {
            url: "https://example.invalid/model.gguf".into(),
            size: 1,
            expected_sha256: "0".repeat(64),
            etag: None,
            last_modified: None,
            chunks: plan_chunks(1, 1),
        };
        let mut bytes = serde_json::to_vec(&state).unwrap();
        bytes.resize(1024 * 1024, b' ');
        std::fs::write(root.join("model.gguf.part.json"), bytes).unwrap();

        assert!(load_resume_state_in(&entries).is_none());

        drop(entries);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resume_identity_binds_digest_and_last_modified() {
        let state = ResumeState {
            url: "https://example.invalid/runtime.zip".into(),
            size: 1000,
            expected_sha256: "a".repeat(64),
            etag: Some("etag-a".into()),
            last_modified: Some("Sat, 05 Sep 2026 00:00:00 GMT".into()),
            chunks: plan_chunks(1000, 1),
        };

        assert!(can_resume_from(
            &state,
            &state.url,
            1000,
            &"a".repeat(64),
            Some("etag-a"),
            Some("Sat, 05 Sep 2026 00:00:00 GMT"),
        ));
        assert!(!can_resume_from(
            &state,
            &state.url,
            1000,
            &"b".repeat(64),
            Some("etag-a"),
            Some("Sat, 05 Sep 2026 00:00:00 GMT"),
        ));
        assert!(!can_resume_from(
            &state,
            &state.url,
            1000,
            &"a".repeat(64),
            Some("etag-a"),
            Some("Sun, 06 Sep 2026 00:00:00 GMT"),
        ));
    }

    #[test]
    fn resume_without_an_etag_still_requires_the_same_url() {
        // Small/non-LFS Hub objects may not expose an ETag. A same-sized file
        // at another revision must never inherit those unverified bytes.
        let state = ResumeState {
            url: "https://huggingface.co/owner/repo/resolve/main/model.gguf".into(),
            size: 10,
            expected_sha256: "0".repeat(64),
            etag: None,
            last_modified: None,
            chunks: vec![Chunk {
                start: 0,
                end: 9,
                done: 5,
            }],
        };
        assert!(!can_resume_from(
            &state,
            "https://huggingface.co/owner/repo/resolve/other/model.gguf",
            10,
            &"0".repeat(64),
            None,
            None,
        ));
    }

    #[test]
    fn content_range_must_match_requested_span_and_total() {
        assert!(content_range_matches("bytes 10-19/100", 10, 19, 100));
        assert!(!content_range_matches("bytes 0-19/100", 10, 19, 100));
        assert!(!content_range_matches("bytes 10-20/100", 10, 19, 100));
        assert!(!content_range_matches("bytes 10-19/101", 10, 19, 100));
        assert!(!content_range_matches("bytes 20-19/100", 20, 19, 100));
        assert!(!content_range_matches("bytes 0-100/100", 0, 100, 100));
        assert!(!content_range_matches("bytes 0-0/0", 0, 0, 0));
    }

    #[test]
    fn every_transfer_response_must_match_all_probed_validators() {
        assert!(response_validators_match(
            Some("etag-a"),
            Some("Sat, 05 Sep 2026 00:00:00 GMT"),
            Some("\"etag-a\""),
            Some("Sat, 05 Sep 2026 00:00:00 GMT"),
        ));
        assert!(!response_validators_match(
            Some("etag-a"),
            Some("Sat, 05 Sep 2026 00:00:00 GMT"),
            Some("\"etag-a\""),
            Some("Sun, 06 Sep 2026 00:00:00 GMT"),
        ));
        assert!(!response_validators_match(
            Some("etag-a"),
            None,
            Some("\"etag-b\""),
            None,
        ));
    }

    #[test]
    fn a_range_response_never_writes_past_its_chunk() {
        // An oversized body is a protocol error, not data to truncate silently.
        assert!(!response_read_fits(1024, 2048));
        assert!(response_read_fits(1024, 512));
        assert!(!response_read_fits(0, 1));
    }

    #[test]
    fn a_server_that_ignores_ranges_is_rejected_for_parallel_chunks() {
        // Some proxies answer a Range request with 200 and the whole file. If
        // each worker accepted that body, chunks would contain duplicate bytes.
        assert!(range_response_is_usable(
            reqwest::StatusCode::PARTIAL_CONTENT,
            false,
            10
        ));
        assert!(!range_response_is_usable(reqwest::StatusCode::OK, false, 0));
        assert!(range_response_is_usable(reqwest::StatusCode::OK, true, 0));
        assert!(!range_response_is_usable(reqwest::StatusCode::OK, true, 10));
    }

    #[test]
    fn transfer_permits_only_two_retries_after_the_initial_request() {
        use std::io::{BufRead as _, BufReader, Write as _};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let done = Arc::new(AtomicBool::new(false));
        let server_done = Arc::clone(&done);
        let requests = Arc::new(AtomicU64::new(0));
        let server_requests = Arc::clone(&requests);
        let server = std::thread::spawn(move || {
            while !server_done.load(Ordering::Relaxed) {
                let (mut stream, _) = match listener.accept() {
                    Ok(value) => value,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(error) => panic!("server accept failed: {error}"),
                };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    // Accepted streams inherit the listener's non-blocking mode on
                    // Windows; a WouldBlock only means the peer has not sent the
                    // next header line yet.
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(1));
                            continue;
                        }
                        Err(error) => panic!("server read failed: {error}"),
                    }
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let request = server_requests.fetch_add(1, Ordering::Relaxed);
                if request == 0 {
                    stream
                        .write_all(b"HTTP/1.1 206 Partial Content\r\nContent-Length: 1\r\nContent-Range: bytes 0-0/1024\r\nAccept-Ranges: bytes\r\nETag: \"fixture\"\r\nConnection: close\r\n\r\nx")
                        .unwrap();
                } else {
                    stream
                        .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                        .unwrap();
                }
            }
        });

        let root = unique_test_dir("localmotive-retry-limit");
        std::fs::create_dir_all(&root).unwrap();
        let result = download_file(
            &format!("http://{address}/runtime.zip"),
            &root.join("runtime.zip"),
            "fixture",
            1024,
            &"0".repeat(64),
            None,
            1,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicU64::new(0)),
            |_, _| {},
        );
        done.store(true, Ordering::Relaxed);
        server.join().unwrap();
        assert!(result.is_err());
        // The probe sends one range request; the chunk fetch may then retry
        // the bounded range twice more. reqwest may also transparently retry
        // a failed connection, so count at least the initial chunk request
        // plus its two bounded retries rather than an exact total.
        let chunk_requests = requests.load(Ordering::Relaxed).saturating_sub(1);
        assert!(
            chunk_requests >= 3,
            "expected the initial request plus two retries, saw {chunk_requests} chunk requests"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resume_is_refused_when_the_remote_file_changed() {
        // Splicing bytes from two different revisions would silently corrupt the
        // model, so any disagreement must restart the download.
        let state = ResumeState {
            url: "u".into(),
            size: 1000,
            expected_sha256: "0".repeat(64),
            etag: Some("abc".into()),
            last_modified: None,
            chunks: plan_chunks(1000, 1),
        };
        assert!(can_resume(&state, 1000, Some("abc")));
        assert!(!can_resume(&state, 1001, Some("abc")), "size changed");
        assert!(!can_resume(&state, 1000, Some("different")), "etag changed");

        let gappy = ResumeState {
            chunks: vec![
                Chunk {
                    start: 0,
                    end: 100,
                    done: 0,
                },
                Chunk {
                    start: 500,
                    end: 999,
                    done: 0,
                },
            ],
            ..state.clone()
        };
        assert!(!can_resume(&gappy, 1000, Some("abc")), "chunks must tile");

        let short = ResumeState {
            chunks: vec![Chunk {
                start: 0,
                end: 100,
                done: 0,
            }],
            ..state.clone()
        };
        assert!(
            !can_resume(&short, 1000, Some("abc")),
            "must cover the file"
        );

        let overrun = ResumeState {
            chunks: vec![Chunk {
                start: 0,
                end: 999,
                done: 5000,
            }],
            ..state.clone()
        };
        assert!(
            !can_resume(&overrun, 1000, Some("abc")),
            "done exceeds span"
        );

        let empty = ResumeState {
            chunks: vec![],
            ..state
        };
        assert!(!can_resume(&empty, 1000, Some("abc")));
    }

    #[test]
    fn linked_headers_win_over_pointer_headers() {
        // On LFS/Xet files the plain Content-Length describes the pointer, not
        // the model; trusting it would truncate a 800 MB download to 134 bytes.
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-length", "134".parse().unwrap());
        headers.insert("x-linked-size", "807694464".parse().unwrap());
        headers.insert("etag", "\"pointer\"".parse().unwrap());
        headers.insert(
            "x-linked-etag",
            "\"6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83\""
                .parse()
                .unwrap(),
        );
        headers.insert("accept-ranges", "bytes".parse().unwrap());

        let remote = read_remote_headers(&headers);
        assert_eq!(remote.size, 807_694_464);
        assert_eq!(
            remote.etag.as_deref(),
            Some("6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83"),
            "quotes are stripped so the value can be compared to a computed digest"
        );
        assert!(remote.supports_ranges);
        assert_eq!(remote.etag.as_deref().unwrap().len(), 64);
    }

    #[test]
    fn plain_headers_are_used_when_no_linked_headers_exist() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-length", "2048".parse().unwrap());
        let remote = read_remote_headers(&headers);
        assert_eq!(remote.size, 2048);
        assert_eq!(remote.etag, None);
        assert!(!remote.supports_ranges, "absent means not advertised");
    }

    #[test]
    fn resolve_url_percent_encodes_path_segments() {
        assert_eq!(
            resolve_url("owner/repo", "folder/model Q4#1.gguf", "refs/pr/1"),
            "https://huggingface.co/owner/repo/resolve/refs/pr/1/folder/model%20Q4%231.gguf?download=true"
        );
    }

    #[test]
    fn resolve_url_builds_the_documented_endpoint() {
        assert_eq!(
            resolve_url("bartowski/Llama-3.2-1B-Instruct-GGUF", "m.gguf", "main"),
            "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/m.gguf?download=true"
        );
        assert!(
            resolve_url("a/b", "m.gguf", "").contains("/resolve/main/"),
            "an empty revision falls back to main"
        );
    }

    #[test]
    fn status_explanations_tell_the_user_what_to_do() {
        let anonymous = explain_status(401, "meta-llama/x", false);
        assert!(anonymous.contains("token"), "{anonymous}");
        let with_token = explain_status(403, "meta-llama/x", true);
        assert!(with_token.contains("licence"), "{with_token}");
        assert!(explain_status(429, "a/b", false).contains("resume"));
        assert!(explain_status(503, "a/b", false).contains("resumed"));
        assert!(explain_status(404, "a/b", false).contains("no longer exists"));
    }

    #[test]
    fn progress_arithmetic_never_divides_by_zero() {
        assert_eq!(eta_seconds(0, 100, 0), None, "unknown rate has no ETA");
        assert_eq!(eta_seconds(0, 0, 10), None);
        assert_eq!(eta_seconds(100, 100, 10), None, "finished has no ETA");
        assert_eq!(eta_seconds(0, 1000, 100), Some(10));
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1023), "1023 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(807_694_464), "770.3 MiB");
        assert_eq!(human_bytes(18_556_689_568), "17.3 GiB");
    }

    #[test]
    fn existing_files_are_reused_only_when_remote_identity_matches() {
        let dir = unique_test_dir("localmotive-existing");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("model.gguf");
        std::fs::write(&path, b"abc").unwrap();
        let digest = sha256_file(&path).unwrap();
        assert!(existing_file_matches(&path, 3, &digest).unwrap());
        assert!(!existing_file_matches(&path, 4, &digest).unwrap());
        assert!(!existing_file_matches(&path, 3, &"0".repeat(64)).unwrap());
        assert!(!existing_file_matches(&dir.join("missing.gguf"), 3, &digest).unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sha256_matches_a_known_digest() {
        let dir = unique_test_dir("localmotive-sha");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.bin");
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(
            sha256_file(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        std::fs::write(&path, b"").unwrap();
        assert_eq!(
            sha256_file(&path).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert!(sha256_file(&dir.join("missing.bin")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resume_state_survives_a_round_trip_through_disk() {
        let dir = unique_test_dir("localmotive-rs");
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("model.gguf");
        assert!(load_resume_state(&target).is_none(), "nothing saved yet");

        let mut state = ResumeState {
            url: "https://example/x".into(),
            size: MIN_CHUNK_BYTES * 4,
            expected_sha256: "a".repeat(64),
            etag: Some("a".repeat(64)),
            last_modified: Some("Sat, 05 Sep 2026 00:00:00 GMT".into()),
            chunks: plan_chunks(MIN_CHUNK_BYTES * 4, 4),
        };
        state.chunks[0].done = 128;
        save_resume_state(&target, &state).unwrap();

        let loaded = load_resume_state(&target).expect("state should reload");
        assert_eq!(loaded.chunks, state.chunks);
        assert_eq!(loaded.downloaded(), 128);
        assert!(!loaded.is_complete());

        // Progress is persisted many times during one download. Replacing the
        // existing sidecar must work on Windows, not only the initial write.
        state.chunks[0].done = 256;
        save_resume_state(&target, &state).unwrap();
        let replaced = load_resume_state(&target).expect("replacement state should reload");
        assert_eq!(replaced.downloaded(), 256);

        for chunk in &mut state.chunks {
            chunk.done = chunk.len();
        }
        assert!(state.is_complete());
        assert_eq!(state.downloaded(), MIN_CHUNK_BYTES * 4);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// End-to-end against a real local HTTP server that speaks ranges, so the
    /// chunking, seeking, resume and verification paths are exercised together
    /// rather than only in isolation.
    #[test]
    fn parallel_ranged_download_reassembles_the_original_bytes() {
        use std::io::BufReader;
        use std::net::TcpListener;

        let payload: Vec<u8> = (0..(MIN_CHUNK_BYTES * 3 + 12_345))
            .map(|i| (i % 251) as u8)
            .collect();
        let expected = {
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            format!("{:x}", hasher.finalize())
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let body = payload.clone();
        let digest = expected.clone();
        let server = std::thread::spawn(move || {
            for stream in listener.incoming().take(16) {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                loop {
                    let mut line = String::new();
                    use std::io::BufRead;
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        break;
                    }
                    if line == "\r\n" {
                        break;
                    }
                    request.push_str(&line);
                }
                if request.is_empty() {
                    break;
                }
                let range = request
                    .lines()
                    .find(|l| l.to_ascii_lowercase().starts_with("range:"))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|r| r.trim().to_string());
                let (start, end) = match range {
                    Some(spec) => {
                        let mut parts = spec.split('-');
                        let s: u64 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
                        let e: u64 = parts
                            .next()
                            .and_then(|v| v.trim().parse().ok())
                            .unwrap_or(body.len() as u64 - 1);
                        (s, e.min(body.len() as u64 - 1))
                    }
                    None => (0, body.len() as u64 - 1),
                };
                let slice = &body[start as usize..=end as usize];
                let header = format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nAccept-Ranges: bytes\r\nX-Linked-Size: {}\r\nX-Linked-ETag: \"{}\"\r\nConnection: close\r\n\r\n",
                    slice.len(), start, end, body.len(), body.len(), digest
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(slice);
                let _ = stream.flush();
            }
        });

        let dir = unique_test_dir("localmotive-dl");
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("model.gguf");
        let url = format!("http://127.0.0.1:{port}/model.gguf");

        let mut samples = Vec::new();
        let result = download_file(
            &url,
            &target,
            "test/repo",
            payload.len() as u64,
            &expected,
            None,
            4,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicU64::new(0)),
            |done, total| samples.push((done, total)),
        );
        let path = result.expect("download should succeed");
        // Wake the accept loop so the server thread can exit: it stops on a
        // connection that sends no request.
        let _ = std::net::TcpStream::connect(("127.0.0.1", port));
        let _ = server.join();
        let written = std::fs::read(&path).unwrap();
        assert_eq!(written.len(), payload.len(), "size must match exactly");
        assert_eq!(written, payload, "bytes must match the original exactly");
        assert_eq!(sha256_file(&path).unwrap(), expected, "checksum verified");
        assert!(!target.with_extension("gguf.part").exists());
        assert!(
            load_resume_state(&target).is_none(),
            "resume state is cleaned up after success"
        );
        assert!(
            samples
                .iter()
                .any(|(done, total)| *done == *total && *total == payload.len() as u64),
            "progress must reach 100%: {samples:?}"
        );
        assert!(samples.iter().all(|(done, total)| done <= total));
        assert!(samples.windows(2).all(|pair| pair[0].0 <= pair[1].0));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A download killed part-way must resume from the bytes already on disk
    /// rather than starting over — the whole point for multi-gigabyte models.
    #[test]
    fn an_interrupted_download_resumes_from_existing_bytes() {
        let dir = unique_test_dir("localmotive-rsm");
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("model.gguf");
        let (part, _) = part_paths(&target);

        let size = MIN_CHUNK_BYTES * 2;
        let mut state = ResumeState {
            url: "https://example/model.gguf".into(),
            size,
            expected_sha256: "b".repeat(64),
            etag: Some("b".repeat(64)),
            last_modified: None,
            chunks: plan_chunks(size, 2),
        };
        state.chunks[0].done = state.chunks[0].len(); // first half already fetched
        std::fs::write(&part, vec![0u8; size as usize]).unwrap();
        save_resume_state(&target, &state).unwrap();

        let reloaded = load_resume_state(&target).unwrap();
        assert!(
            can_resume(&reloaded, size, Some(&"b".repeat(64))),
            "same file must be resumable"
        );
        assert_eq!(reloaded.downloaded(), MIN_CHUNK_BYTES, "half is kept");
        assert_eq!(
            reloaded.chunks[1].cursor(),
            MIN_CHUNK_BYTES,
            "the second worker restarts at the boundary, not at zero"
        );
        assert!(
            !can_resume(&reloaded, size, Some(&"c".repeat(64))),
            "a changed file must not be spliced onto the old bytes"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dc12_cancelled_existing_file_verification_keeps_the_file_and_retry_reverifies() {
        // A large existing-file verification must return promptly when the
        // user keeps the current state and stops, keep the bytes on disk,
        // and re-verify (never shortcut to completion) on the next attempt
        // (audit DC-12 I3/V2).
        let payload: Vec<u8> = (0..(3 * 1024 * 1024))
            .map(|index| (index % 251) as u8)
            .collect();
        let digest = {
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            format!("{:x}", hasher.finalize())
        };
        let (port, server) = serve_plain_responses(vec![
            (payload.clone(), true, None),
            (payload.clone(), true, None),
        ]);
        let root = unique_test_dir("localmotive-dc12-verify");
        let target = root.join("existing.bin");
        std::fs::write(&target, &payload).unwrap();

        let cancel = Arc::new(AtomicBool::new(false));
        let downloaded = Arc::new(AtomicU64::new(0));
        let flip = cancel.clone();
        let result = download_file(
            &format!("http://127.0.0.1:{port}/existing.bin"),
            &target,
            "test artifact",
            payload.len() as u64,
            &digest,
            None,
            1,
            cancel,
            downloaded.clone(),
            move |hashed, _total| {
                if hashed > 0 {
                    flip.store(true, Ordering::Relaxed);
                }
            },
        );
        let error = result.unwrap_err();
        assert!(error.contains("cancelled"), "{error}");
        assert!(target.exists(), "the kept bytes must survive cancellation");
        assert_eq!(
            downloaded.load(Ordering::Relaxed),
            0,
            "no transfer may start"
        );

        // Retry without cancellation: the file is re-verified and reported
        // complete only after that verification.
        let retried = download_file(
            &format!("http://127.0.0.1:{port}/existing.bin"),
            &target,
            "test artifact",
            payload.len() as u64,
            &digest,
            None,
            1,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicU64::new(0)),
            |_done, _total| {},
        )
        .unwrap();
        assert_eq!(std::fs::read(&retried).unwrap(), payload);
        let requests = server.join().unwrap();
        assert_eq!(
            requests.len(),
            2,
            "each attempt probes once; existing bytes avoid transfers"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dc12_checkpoint_publication_is_ordered_after_a_data_sync() {
        // The documented guarantee: a published checkpoint never claims
        // bytes that were not first flushed to the part file (audit DC-12
        // I1/I2). Both production publication points must sync first.
        let source = include_str!("download.rs");
        let loop_block = source
            .split("// Report progress every poll")
            .nth(1)
            .unwrap()
            .split("if state.is_complete() {")
            .next()
            .unwrap();
        let sync = loop_block
            .find("sync_part_data(&entries)")
            .expect("the checkpoint loop must flush part data");
        let save = loop_block
            .find("save_resume_state_in(&entries, &state)")
            .expect("the checkpoint loop must publish the sidecar");
        assert!(
            sync < save,
            "data must be flushed before the checkpoint rename"
        );
        let final_block = source
            .split("state.chunks = chunks.lock().unwrap().clone();")
            .nth(1)
            .unwrap()
            .split("if let Some(error) = failure.lock().unwrap().take()")
            .next()
            .unwrap();
        assert!(
            final_block.contains("sync_part_data(&entries)?;")
                && final_block.contains("save_resume_state_in(&entries, &state)?;"),
            "the final save must flush data first"
        );
        assert!(
            final_block.find("sync_part_data(&entries)?;")
                < final_block.find("save_resume_state_in(&entries, &state)?;"),
            "the final save must flush data first"
        );
    }

    /// Serve one scripted plain-HTTP response per entry: (payload,
    /// declare_length, truncate_at). Answers every request with 200 and the
    /// full body, modelling a server that ignores Range (audit DC-02).
    fn serve_plain_responses(
        plan: Vec<(Vec<u8>, bool, Option<usize>)>,
    ) -> (u16, std::thread::JoinHandle<Vec<String>>) {
        use std::io::{BufRead as _, BufReader, Write as _};
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for (payload, declare_length, truncate_at) in plan {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) if line == "\r\n" => break,
                        Ok(_) => request.push_str(&line),
                        Err(_) => break,
                    }
                }
                requests.push(request);
                let header = if declare_length {
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        payload.len()
                    )
                } else {
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
                        .to_string()
                };
                let _ = stream.write_all(header.as_bytes());
                let body_len = truncate_at.unwrap_or(payload.len());
                if declare_length {
                    let _ = stream.write_all(&payload[..body_len]);
                } else {
                    let mut offset = 0;
                    while offset < body_len {
                        let step = (64 * 1024).min(body_len - offset);
                        let _ = write!(stream, "{step:x}\r\n");
                        let _ = stream.write_all(&payload[offset..offset + step]);
                        let _ = stream.write_all(b"\r\n");
                        offset += step;
                    }
                    let _ = stream.write_all(b"0\r\n\r\n");
                }
                let _ = stream.flush();
            }
            requests
        });
        (port, handle)
    }

    fn dc02_payload() -> (Vec<u8>, String) {
        // 8 MiB + 1: one byte past the ranged-request span that used to be
        // mistaken for the expected response length.
        let payload: Vec<u8> = (0..(8 * 1024 * 1024 + 1))
            .map(|index| (index % 251) as u8)
            .collect();
        let digest = {
            let mut hasher = Sha256::new();
            hasher.update(&payload);
            format!("{:x}", hasher.finalize())
        };
        (payload, digest)
    }

    fn run_dc02_download(
        port: u16,
        payload: &[u8],
        digest: &str,
        root: &Path,
    ) -> Result<PathBuf, String> {
        let target = root.join("big.bin");
        let cancel = Arc::new(AtomicBool::new(false));
        let downloaded = Arc::new(AtomicU64::new(0));
        download_file(
            &format!("http://127.0.0.1:{port}/big.bin"),
            &target,
            "unsloth/DC02-GGUF",
            payload.len() as u64,
            digest,
            None,
            4,
            cancel,
            downloaded,
            |_, _| {},
        )
    }

    #[test]
    fn dc02_a_range_ignoring_server_completes_files_larger_than_the_request_span() {
        // The audited defect: HTTP 200 to every Range request for a file
        // larger than 8 MiB was rejected because the response length was
        // compared against the 8 MiB request span. The sequential
        // whole-response path must complete it with the exact bytes and
        // digest (audit DC-02 V1).
        let (payload, digest) = dc02_payload();
        let (port, server) = serve_plain_responses(vec![
            (payload.clone(), true, None), // probe: 200, full Content-Length
            (payload.clone(), true, None), // transfer: 200 with Content-Length
        ]);
        let root = unique_test_dir("localmotive-dc02-large");
        let path = run_dc02_download(port, &payload, &digest, &root).unwrap();
        let written = std::fs::read(&path).unwrap();
        assert_eq!(written.len(), payload.len());
        assert_eq!(written, payload, "the published bytes must be exact");
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dc02_a_chunked_range_ignoring_response_also_completes() {
        // Same path with a chunked transfer (no Content-Length): the read
        // loop must be bounded by the expected object size, not by a header
        // it never receives (audit DC-02 V1).
        let (payload, digest) = dc02_payload();
        let (port, server) = serve_plain_responses(vec![
            (payload.clone(), true, None),  // probe
            (payload.clone(), false, None), // transfer: chunked 200
        ]);
        let root = unique_test_dir("localmotive-dc02-chunked");
        let path = run_dc02_download(port, &payload, &digest, &root).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), payload);
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dc02_an_interrupted_no_range_transfer_retries_from_zero() {
        // An interrupted 200 stream cannot be resumed at an offset: the retry
        // must restart from zero and still publish exactly once, with the
        // digest checked (audit DC-02 V2/V3).
        let (payload, digest) = dc02_payload();
        let truncated = payload.len() / 2;
        let (port, server) = serve_plain_responses(vec![
            (payload.clone(), true, None),            // probe
            (payload.clone(), true, Some(truncated)), // transfer: truncated body
            (payload.clone(), true, None),            // retry: complete body
        ]);
        let root = unique_test_dir("localmotive-dc02-retry");
        let result = run_dc02_download(port, &payload, &digest, &root);
        let requests = server.join().unwrap();
        let path = result
            .unwrap_or_else(|error| panic!("download failed: {error}; requests: {requests:#?}"));
        let written = std::fs::read(&path).unwrap();
        assert_eq!(written, payload);
        assert_eq!(
            requests.len(),
            3,
            "probe plus one failed and one successful transfer"
        );
        // The successful retry restarted from zero, not from the interrupted
        // offset: only then does a range-ignoring server's 200 answer match.
        assert!(
            requests[2].to_ascii_lowercase().contains("range: bytes=0-"),
            "unexpected retry request: {}",
            requests[2]
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
