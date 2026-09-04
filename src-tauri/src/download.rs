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
use std::time::Duration;

/// Chunks smaller than this are not worth a separate connection.
pub const MIN_CHUNK_BYTES: u64 = 8 * 1024 * 1024;
/// Hugging Face tolerates a handful of connections per file; more mostly adds
/// contention and risks throttling.
pub const MAX_CONNECTIONS: usize = 8;
const BUFFER_BYTES: usize = 1024 * 1024;

fn open_download_directory(root: &Path) -> Result<Dir, String> {
    Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|error| format!("Could not securely open {}: {error}", root.display()))
}

#[cfg(windows)]
fn opened_directory_matches(directory: &Dir, expected: &Path) -> Result<bool, String> {
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
        return Err("Could not confirm the opened model folder's Windows identity.".into());
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
fn opened_directory_matches(directory: &Dir, expected: &Path) -> Result<bool, String> {
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
#[serde(rename_all = "camelCase")]
pub struct ResumeState {
    pub url: String,
    pub size: u64,
    /// The server's sha256 for the file, when it published one.
    pub etag: Option<String>,
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
        if chunk.start != expected_start || chunk.end < chunk.start || chunk.done > chunk.len() {
            return false;
        }
        expected_start = chunk.end + 1;
    }
    expected_start == size
}

/// Resume check at the actual requested URL. When no ETag exists, matching the
/// URL is the remaining identity signal; a different revision must restart.
pub fn can_resume_from(state: &ResumeState, url: &str, size: u64, etag: Option<&str>) -> bool {
    state.url == url && can_resume(state, size, etag)
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
    let supports_ranges = get("accept-ranges")
        .map(|value| value.to_ascii_lowercase().contains("bytes"))
        .unwrap_or(false);
    RemoteFile {
        size,
        etag,
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
    let text = entries.dir.read_to_string(&entries.meta).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_resume_state_in(entries: &DownloadEntries, state: &ResumeState) -> Result<(), String> {
    let text = serde_json::to_string(state)
        .map_err(|error| format!("Could not serialize download progress: {error}"))?;
    let temporary = PathBuf::from(format!("{}.next", entries.meta.to_string_lossy()));
    entries
        .dir
        .write(&temporary, text)
        .and_then(|_| entries.dir.rename(&temporary, &entries.dir, &entries.meta))
        .map_err(|error| {
            format!(
                "Could not securely save download progress to {}: {error}",
                entries.meta.display()
            )
        })
}

fn entry_is_unsafe_for_writes(is_symlink: bool, file_attributes: u32) -> bool {
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    is_symlink || file_attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

fn ensure_safe_write_entry(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("Could not inspect {}: {error}", path.display())),
    };
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
    Ok(sha256_reader(file, &entries.target)?.eq_ignore_ascii_case(expected_sha256))
}

/// Compute a file's sha256, streaming so a 20 GiB model is never held in memory.
#[cfg(test)]
fn sha256_file(path: &Path) -> Result<String, String> {
    let file =
        File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    sha256_reader(file, path)
}

fn sha256_reader(mut file: impl Read, display_path: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; BUFFER_BYTES];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not read {}: {error}", display_path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn client(token: Option<&str>) -> Result<reqwest::blocking::Client, String> {
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
        .timeout(None)
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
pub fn probe(url: &str, token: Option<&str>, repo: &str) -> Result<RemoteFile, String> {
    let client = client(token)?;
    let response = client
        .get(url)
        .header(reqwest::header::RANGE, "bytes=0-0")
        .send()
        .map_err(|error| format!("Could not reach Hugging Face: {error}"))?;
    let status = response.status().as_u16();
    if status >= 400 {
        return Err(explain_status(status, repo, token.is_some()));
    }
    let mut remote = read_remote_headers(response.headers());
    // A 206 to a one-byte range is direct proof of range support, which matters
    // more than the advertised header.
    if status == 206 {
        remote.supports_ranges = true;
        if let Some(range) = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(total) = range.rsplit('/').next().and_then(|t| t.parse::<u64>().ok()) {
                remote.size = total;
            }
        }
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
    mut on_progress: impl FnMut(u64, u64) + Send,
) -> Result<PathBuf, String> {
    if let Some(parent) = target.parent() {
        ensure_safe_write_entry(parent)?;
    }

    let (part, meta) = part_paths(target);
    ensure_safe_write_entry(target)?;
    ensure_safe_write_entry(&part)?;
    ensure_safe_write_entry(&meta)?;
    let entries = open_download_entries(target)?;

    let remote = probe(url, token, repo)?;
    if remote.size != expected_size {
        return Err("The remote file size does not match the validated catalog.".into());
    }
    if entries.dir.symlink_metadata(&entries.target).is_ok() {
        if existing_file_matches_in(&entries, remote.size, expected_sha256)? {
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
            if can_resume_from(&saved, url, remote.size, remote.etag.as_deref())
                && resume_chunks_are_valid(&saved.chunks, remote.size, effective_connections)
                && entries.dir.is_file(&entries.part) =>
        {
            saved
        }
        _ => {
            let _ = entries.dir.remove_file(&entries.part);
            ResumeState {
                url: url.to_string(),
                size: remote.size,
                etag: remote.etag.clone(),
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
                    ) {
                        let mut slot = failure.lock().unwrap();
                        if slot.is_none() {
                            *slot = Some(error);
                        }
                        cancel.store(true, Ordering::Relaxed);
                    }
                });
            }

            // Report progress and persist resume state while the workers run.
            while !cancel.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(400));
                let snapshot = chunks.lock().unwrap().clone();
                let done: u64 = snapshot.iter().map(|c| c.done).sum();
                on_progress(done, total);
                state.chunks = snapshot;
                if let Err(error) = save_resume_state_in(&entries, &state) {
                    let mut slot = failure.lock().unwrap();
                    if slot.is_none() {
                        *slot = Some(error);
                    }
                    cancel.store(true, Ordering::Relaxed);
                    break;
                }
                if state.is_complete() {
                    break;
                }
            }
        });
    }

    state.chunks = chunks.lock().unwrap().clone();
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
    let actual = sha256_reader(part_file, &part)?;
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
) -> Result<(), String> {
    let client = client(token)?;
    // Retry a stalled connection a few times; a long download crossing a brief
    // network blip should not lose the whole file.
    let mut attempt = 0;
    loop {
        let chunk = chunks.lock().unwrap()[index];
        if chunk.is_complete() || cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        let range = format!("bytes={}-{}", chunk.cursor(), chunk.end);
        let result = (|| -> Result<(), String> {
            let response = client
                .get(url)
                .header(reqwest::header::RANGE, &range)
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
                let expected_start = chunk.cursor();
                let valid = response
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|v| content_range_matches(v, expected_start, chunk.end, total));
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
            if !response_identity_matches(remote_etag, response_etag) {
                return Err("The remote file changed while it was downloading.".into());
            }
            let expected_body = chunk.end - requested_start + 1;
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
                let read = response
                    .read(&mut buffer)
                    .map_err(|error| format!("Transfer interrupted: {error}"))?;
                if read == 0 {
                    return Ok(());
                }
                let cursor = chunks.lock().unwrap()[index].cursor();
                let remaining = {
                    let guard = chunks.lock().unwrap();
                    guard[index].len() - guard[index].done
                };
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
                if complete {
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
                attempt += 1;
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
                if attempt >= 5 {
                    return Err(error);
                }
                std::thread::sleep(Duration::from_millis(500 * attempt as u64));
            }
        }
        if attempt >= 5 {
            return Err("The connection kept dropping. Press Download to resume.".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn an_open_download_directory_cannot_be_redirected_by_path_replacement() {
        let root =
            std::env::temp_dir().join(format!("localmotive-dir-race-{}", std::process::id()));
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
        let root =
            std::env::temp_dir().join(format!("localmotive-dir-identity-{}", std::process::id()));
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
            etag: Some("abc".into()),
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
    fn resume_without_an_etag_still_requires_the_same_url() {
        // Small/non-LFS Hub objects may not expose an ETag. A same-sized file
        // at another revision must never inherit those unverified bytes.
        let state = ResumeState {
            url: "https://huggingface.co/owner/repo/resolve/main/model.gguf".into(),
            size: 10,
            etag: None,
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
            None,
        ));
    }

    #[test]
    fn content_range_must_match_requested_span_and_total() {
        assert!(content_range_matches("bytes 10-19/100", 10, 19, 100));
        assert!(!content_range_matches("bytes 0-19/100", 10, 19, 100));
        assert!(!content_range_matches("bytes 10-20/100", 10, 19, 100));
        assert!(!content_range_matches("bytes 10-19/101", 10, 19, 100));
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
    fn resume_is_refused_when_the_remote_file_changed() {
        // Splicing bytes from two different revisions would silently corrupt the
        // model, so any disagreement must restart the download.
        let state = ResumeState {
            url: "u".into(),
            size: 1000,
            etag: Some("abc".into()),
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
        let dir = std::env::temp_dir().join(format!("localmotive-existing-{}", std::process::id()));
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
        let dir = std::env::temp_dir().join(format!("localmotive-sha-{}", std::process::id()));
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
        let dir = std::env::temp_dir().join(format!("localmotive-rs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("model.gguf");
        assert!(load_resume_state(&target).is_none(), "nothing saved yet");

        let mut state = ResumeState {
            url: "https://example/x".into(),
            size: MIN_CHUNK_BYTES * 4,
            etag: Some("a".repeat(64)),
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

        let dir = std::env::temp_dir().join(format!("localmotive-dl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
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
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A download killed part-way must resume from the bytes already on disk
    /// rather than starting over — the whole point for multi-gigabyte models.
    #[test]
    fn an_interrupted_download_resumes_from_existing_bytes() {
        let dir = std::env::temp_dir().join(format!("localmotive-rsm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("model.gguf");
        let (part, _) = part_paths(&target);

        let size = MIN_CHUNK_BYTES * 2;
        let mut state = ResumeState {
            url: "https://example/model.gguf".into(),
            size,
            etag: Some("b".repeat(64)),
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
}
