//! Bounded per-run launch logs with explicit retention.
//!
//! Output-overflow policy: crossing the quota truncates the
//! **retained** output only. The child is never terminated for emitting too
//! much, and the drain keeps reading (and discarding) so the child can never
//! block on a full pipe. The truncation marker is written once when the
//! quota is first crossed.
//!
//! Each launch creates a uniquely named log under the Localmotive temp
//! directory. A drain thread copies the child's output into the file up to a
//! fixed quota and keeps reading (and discarding) afterwards, so a chatty
//! child never blocks on a full pipe. Old runs are pruned on the next launch
//! with the newest failure evidence preserved.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Per-run log quota: output beyond this is discarded, never written.
pub const LOG_QUOTA_BYTES: u64 = 8 * 1024 * 1024;
/// How many whole log runs are kept per prefix before pruning.
pub const LOG_RETENTION_PER_PREFIX: usize = 10;
/// Total bytes the managed log directory may occupy across all runs.
pub const LOG_DIRECTORY_QUOTA_BYTES: u64 = 64 * 1024 * 1024;
/// Failure-evidence files that survive even when their log is pruned.
pub const FAILURE_EVIDENCE_KEPT: usize = 3;

static RUN_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A unique, filename-safe identity for one launch: millisecond timestamp
/// plus a process-local sequence, so two instances or two quick runs can
/// never collide on the same file.
pub fn new_run_id() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    let sequence = RUN_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{millis}-{sequence}")
}

/// Counters shared by the stdout and stderr writers of one run.
#[derive(Clone)]
struct LogQuota {
    quota_bytes: u64,
    written: Arc<AtomicU64>,
    truncated: Arc<AtomicBool>,
}

pub struct LogSink {
    path: PathBuf,
    file: fs::File,
    quota: LogQuota,
}

impl LogSink {
    /// Create the run log in `directory`. Creation fails when the target
    /// name already exists in any form (a collision or a planted link) and
    /// the directory must be a real directory, not a reparse point
    ///.
    pub fn create(directory: &Path, prefix: &str, run_id: &str) -> Result<Self, String> {
        Self::create_with_quota(directory, prefix, run_id, LOG_QUOTA_BYTES)
    }

    pub fn create_with_quota(
        directory: &Path,
        prefix: &str,
        run_id: &str,
        quota_bytes: u64,
    ) -> Result<Self, String> {
        fs::create_dir_all(directory)
            .map_err(|error| format!("Could not create the launch log directory: {error}"))?;
        let metadata = fs::symlink_metadata(directory)
            .map_err(|error| format!("Could not inspect the launch log directory: {error}"))?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || crate::artifact::is_reparse_point(&metadata)
        {
            return Err("The launch log directory cannot be a link or reparse point".into());
        }
        let path = directory.join(format!("{prefix}-{run_id}.log"));
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|error| format!("Could not create the launch log: {error}"))?;
        Ok(Self {
            path,
            file,
            quota: LogQuota {
                quota_bytes,
                written: Arc::new(AtomicU64::new(0)),
                truncated: Arc::new(AtomicBool::new(false)),
            },
        })
    }

    /// A second writer for the other stream: a separate append handle with
    /// its own file position, sharing the one quota with the primary writer
    /// so both streams stay inside the same budget.
    pub fn second_writer(&self) -> Result<LogWriter, String> {
        // the path is refused as a link/reparse point before the
        // reopen, so a planted link is never followed. Append mode is kept:
        // the second stream needs its own end-of-file position while sharing
        // the quota. The no-open variant applies: the sink holds this file
        // open (a second open for link-counting fails with a sharing
        // violation), and an extra hard-link name cannot divert a pinned
        // handle's bytes.
        crate::download::ensure_no_link_or_reparse(&self.path)
            .map_err(|error| format!("The launch log is not safe to reopen: {error}"))?;
        let file = OpenOptions::new()
            .append(true)
            .open(&self.path)
            .map_err(|error| format!("Could not open the launch log for append: {error}"))?;
        Ok(LogWriter {
            file,
            quota: self.quota.clone(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Copy `reader` into the log until EOF, never exceeding the quota and
    /// never blocking the writer: bytes past the quota are drained and
    /// discarded with a single truncation marker.
    pub fn drain<R: Read>(&mut self, reader: R) -> Result<u64, String> {
        let mut writer = LogWriter {
            file: self
                .file
                .try_clone()
                .map_err(|error| format!("Could not prepare the launch log: {error}"))?,
            quota: self.quota.clone(),
        };
        writer.drain(reader)
    }

    #[cfg(test)]
    pub fn written(&self) -> u64 {
        self.quota.written.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub fn truncated(&self) -> bool {
        self.quota.truncated.load(Ordering::Relaxed)
    }
}

/// One stream's writer for a run: appends to the shared run log while the
/// shared quota has room, then keeps reading and discarding.
pub struct LogWriter {
    file: fs::File,
    quota: LogQuota,
}

impl LogWriter {
    pub fn drain<R: Read>(&mut self, mut reader: R) -> Result<u64, String> {
        let mut buffer = [0_u8; 16 * 1024];
        let mut consumed = 0_u64;
        loop {
            let read = reader
                .read(&mut buffer)
                .map_err(|error| format!("Launch log drain failed: {error}"))?;
            if read == 0 {
                break;
            }
            consumed += read as u64;
            let keep = self.reserve(read as u64);
            if keep > 0 {
                self.file
                    .write_all(&buffer[..keep as usize])
                    .map_err(|error| format!("Launch log write failed: {error}"))?;
            }
            if keep < read as u64 && !self.quota.truncated.swap(true, Ordering::Relaxed) {
                let marker = format!(
                    "\n[localmotive: log truncated at {} bytes; further output discarded]\n",
                    self.quota.quota_bytes
                );
                let _ = self.file.write_all(marker.as_bytes());
            }
        }
        self.file
            .flush()
            .map_err(|error| format!("Launch log flush failed: {error}"))?;
        Ok(consumed)
    }

    /// CAS-reserve up to `wanted` bytes from the shared quota.
    fn reserve(&self, wanted: u64) -> u64 {
        let mut current = self.quota.written.load(Ordering::Relaxed);
        loop {
            if current >= self.quota.quota_bytes {
                return 0;
            }
            let keep = wanted.min(self.quota.quota_bytes - current);
            match self.quota.written.compare_exchange_weak(
                current,
                current + keep,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return keep,
                Err(observed) => current = observed,
            }
        }
    }
}

/// Persist the bounded failure tail beside the log so diagnostics survive
/// retention cleanup, and prune old runs. Keeps the newest
/// [`FAILURE_EVIDENCE_KEPT`] failure files plus [`LOG_RETENTION_PER_PREFIX`]
/// logs per prefix within [`LOG_DIRECTORY_QUOTA_BYTES`].
pub fn write_failure_evidence(log_path: &str, evidence_json: &str) -> Option<PathBuf> {
    let path = Path::new(log_path);
    if log_path.is_empty() {
        return None;
    }
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let evidence_path = path.with_file_name(format!("{file_name}.failure.json"));
    // refuse a planted link before the write: `fs::write` truncates,
    // so following a link here would destroy another file's bytes.
    crate::download::ensure_safe_write_entry(&evidence_path).ok()?;
    let bounded = evidence_json.as_bytes();
    let keep = bounded.len().min(64 * 1024);
    fs::write(&evidence_path, &bounded[..keep]).ok()?;
    Some(evidence_path)
}

fn is_managed_log(name: &str) -> bool {
    name.ends_with(".log") && !name.ends_with(".failure.json")
}

fn is_failure_evidence(name: &str) -> bool {
    name.ends_with(".log.failure.json")
}

/// Prune the managed log directory: keep the newest runs per prefix, keep
/// the newest failure-evidence files, and hold the directory under its total
/// quota. Never deletes the just-created run (call it BEFORE creating it).
pub fn prune_log_directory(directory: &Path) -> Result<usize, String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        // A missing directory holds nothing to prune. Any other read
        // failure (denied, I/O) must surface, never read as a clean zero.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(format!("Could not read the log directory: {error}")),
    };
    let mut logs: Vec<(std::time::SystemTime, PathBuf, u64)> = Vec::new();
    let mut failures: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let metadata = match entry.metadata() {
            Ok(metadata) if metadata.is_file() => metadata,
            _ => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
        if is_failure_evidence(&name) {
            failures.push((modified, entry.path()));
        } else if is_managed_log(&name) {
            logs.push((modified, entry.path(), metadata.len()));
        }
    }
    logs.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    failures.sort_by_key(|entry| std::cmp::Reverse(entry.0));

    let mut removed = 0_usize;
    // Per-prefix retention: the prefix is everything before the final
    // `-<run_id>` pair; the run id itself contains one dash.
    let mut per_prefix: std::collections::BTreeMap<String, usize> = Default::default();
    let mut total_bytes: u64 = logs.iter().map(|(_, _, size)| *size).sum();
    for (_, path, size) in &logs {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let stem = name.trim_end_matches(".log");
        // Run names are `{prefix}-{millis}-{sequence}`: a prefix bucket is
        // the stable part before the two run-id segments ("server-<port>" or
        // "tuning"), never the unique run identity itself.
        let prefix = stem.rsplitn(3, '-').nth(2).unwrap_or(stem).to_string();
        let seen = per_prefix.entry(prefix).or_insert(0);
        *seen += 1;
        // Logs prune by retention; failure evidence is preserved by its own
        // independent budget below, not by pinning the log file.
        let over_count = *seen > LOG_RETENTION_PER_PREFIX;
        let over_quota = total_bytes > LOG_DIRECTORY_QUOTA_BYTES;
        if (over_count || over_quota) && fs::remove_file(path).is_ok() {
            total_bytes = total_bytes.saturating_sub(*size);
            removed += 1;
        }
    }
    for (_, path) in failures.iter().skip(FAILURE_EVIDENCE_KEPT) {
        if fs::remove_file(path).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "localmotive-log-sink-{label}-{}-{}",
            std::process::id(),
            new_run_id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn prune_reports_an_unreadable_directory_instead_of_zero() {
        // Retention failures must not read as a normal zero-removal result:
        // a missing directory prunes nothing, but an unreadable one errors.
        let root = scratch("unreadable");
        assert_eq!(prune_log_directory(&root.join("absent")).unwrap(), 0);
        let file = root.join("not-a-directory.log");
        std::fs::write(&file, b"x").unwrap();
        assert!(prune_log_directory(&file).is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn drain_caps_the_file_and_keeps_the_newest_lines() {
        let root = scratch("quota");
        let mut sink = LogSink::create_with_quota(&root, "server-8080", "run-1", 4 * 1024).unwrap();
        let mut payload = Vec::new();
        for index in 0..2_000 {
            payload.extend_from_slice(format!("line-{index:06}\n").as_bytes());
        }
        let expected_lines = payload.len() / 13;
        let consumed = sink.drain(std::io::Cursor::new(&payload)).unwrap();
        assert_eq!(
            consumed as usize,
            payload.len(),
            "the drain must consume the whole stream so the child never blocks"
        );
        assert_eq!(sink.written(), 4 * 1024, "the file stays at the quota");
        assert!(sink.truncated());
        let written = fs::read(sink.path()).unwrap();
        assert!(
            written.len() as u64 <= 4 * 1024 + 128,
            "quota plus the truncation marker only"
        );
        let text = String::from_utf8_lossy(&written);
        assert!(text.contains("truncated at 4096 bytes"), "{text}");
        // The newest retained lines are the ones nearest the quota boundary.
        let last_written = (0..expected_lines)
            .rev()
            .map(|index| format!("line-{index:06}\n"))
            .find(|line| text.contains(line.as_str()));
        assert!(last_written.is_some());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn failure_evidence_survives_log_pruning() {
        let root = scratch("failure-survives");
        // An old failed run whose log is long past retention.
        let old_log = root.join("server-8080-100-0.log");
        fs::write(&old_log, b"old failure output").unwrap();
        let evidence = write_failure_evidence(
            old_log.to_str().unwrap(),
            "{\"schema\":1,\"phase\":\"spawn\",\"message\":\"fixture\"}",
        )
        .expect("evidence written beside the log");
        assert!(evidence.exists());
        // Ten newer runs push the old log past per-prefix retention.
        for index in 1..=15 {
            fs::write(
                root.join(format!("server-8080-200-{index}.log")),
                vec![b'x'; 64],
            )
            .unwrap();
        }
        prune_log_directory(&root).unwrap();
        assert!(!old_log.exists(), "the pruned log is gone");
        assert!(
            evidence.exists(),
            "the failure evidence from the pruned run survives"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn existing_links_are_refused_before_any_reopen() {
        // the reopen checks refuse symlinks, reparse
        // points, and (for truncating writes) extra hard links, while missing
        // and regular paths pass. Both reopen sites go through them (see the
        // guard below).
        let root = scratch("proc08-guard");
        let regular = root.join("regular.log");
        fs::write(&regular, b"x").unwrap();
        assert!(crate::download::ensure_no_link_or_reparse(&regular).is_ok());
        assert!(crate::download::ensure_no_link_or_reparse(&root.join("missing.log")).is_ok());
        assert!(crate::download::ensure_safe_write_entry(&regular).is_ok());
        let victim = root.join("victim.log");
        fs::write(&victim, b"victim").unwrap();
        let hard = root.join("hard.log");
        fs::hard_link(&victim, &hard).unwrap();
        let error = crate::download::ensure_safe_write_entry(&hard).unwrap_err();
        assert!(error.contains("hard link"), "{error}");
        #[cfg(windows)]
        {
            let outside = root.join("outside");
            fs::create_dir_all(&outside).unwrap();
            let junction = root.join("junction.log");
            let status = crate::proc::hidden_command("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(&junction)
                .arg(&outside)
                .status()
                .unwrap();
            assert!(status.success());
            let error = crate::download::ensure_no_link_or_reparse(&junction).unwrap_err();
            assert!(error.contains("reparse"), "{error}");
            use std::os::windows::fs::symlink_file;
            let target = root.join("target.log");
            fs::write(&target, b"t").unwrap();
            let link = root.join("link.log");
            if symlink_file(&target, &link).is_ok() {
                let error = crate::download::ensure_no_link_or_reparse(&link).unwrap_err();
                assert!(error.contains("link"), "{error}");
            }
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn failure_evidence_never_writes_through_a_planted_link() {
        // a hard link planted at the evidence path must be
        // refused: the victim keeps its bytes and no evidence path is
        // reported. `fs::hard_link` needs no privilege, so this is
        // deterministic on every machine.
        let root = scratch("proc08-evidence");
        let victim = root.join("victim.log");
        fs::write(&victim, b"victim-bytes").unwrap();
        let log = root.join("server-1.log");
        fs::write(&log, b"log").unwrap();
        let evidence = root.join("server-1.log.failure.json");
        fs::hard_link(&victim, &evidence).unwrap();
        let result = write_failure_evidence(&log.to_string_lossy(), "{\"schema\":1}");
        assert!(result.is_none(), "a planted link must be refused");
        assert_eq!(
            fs::read(&victim).unwrap(),
            b"victim-bytes",
            "the victim file must keep its bytes"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn log_reopens_go_through_the_link_check() {
        // Both reopen sites refuse planted links before any open or write:
        // `second_writer` uses the no-open variant (the sink holds the file
        // open), `write_failure_evidence` the full truncating-write check.
        let source = include_str!("log_sink.rs");
        for (site, check) in [
            ("fn second_writer(&self)", "ensure_no_link_or_reparse"),
            ("pub fn write_failure_evidence(", "ensure_safe_write_entry"),
        ] {
            let start = source.find(site).expect("the reopen site present");
            let body = &source[start..(start + 1_200).min(source.len())];
            assert!(
                body.contains(check),
                "{site} must refuse existing links before reopening"
            );
        }
    }

    #[test]
    fn two_streams_share_one_quota() {
        let root = scratch("two-streams");
        let mut sink = LogSink::create_with_quota(&root, "server-8080", "run-2", 3 * 1024).unwrap();
        let mut second = sink.second_writer().unwrap();
        let chunk = vec![b'a'; 4 * 1024];
        let first_thread = std::thread::spawn({
            let chunk = chunk.clone();
            move || second.drain(std::io::Cursor::new(chunk)).unwrap()
        });
        let consumed = sink.drain(std::io::Cursor::new(chunk)).unwrap();
        let other = first_thread.join().unwrap();
        assert_eq!(consumed, 4 * 1024);
        assert_eq!(other, 4 * 1024);
        let bytes = fs::read(sink.path()).unwrap();
        assert!(
            bytes.len() as u64 <= 3 * 1024 + 128,
            "both streams together stay inside the quota: {}",
            bytes.len()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn unique_run_ids_and_creation_refuse_collisions_and_links() {
        let root = scratch("unique");
        let first = LogSink::create(&root, "tuning", "run-a").unwrap();
        assert!(
            LogSink::create(&root, "tuning", "run-a").is_err(),
            "create_new"
        );
        let second = LogSink::create(&root, "tuning", "run-b").unwrap();
        assert_ne!(first.path(), second.path());
        assert_ne!(new_run_id(), new_run_id());

        // A planted link target is refused, never followed.
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            let target = root.join("elsewhere.log");
            fs::write(&target, b"x").unwrap();
            let link = root.join("tuning-run-c.log");
            if symlink_file(&target, &link).is_ok() {
                assert!(LogSink::create(&root, "tuning", "run-c").is_err());
            }
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pruning_bounds_the_directory_and_keeps_newest_failure_evidence() {
        let root = scratch("prune");
        // 25 runs of 3 MiB each: well past both retention caps.
        for index in 0..25 {
            let name = format!("server-8080-100-{index}.log");
            fs::write(root.join(&name), vec![b'x'; 3 * 1024 * 1024]).unwrap();
            let failure = root.join(format!("{name}.failure.json"));
            let should_keep_failure = index >= 23;
            if should_keep_failure || index % 7 == 0 {
                fs::write(&failure, b"{\"schema\":1}").unwrap();
            }
        }
        let removed = prune_log_directory(&root).unwrap();
        assert!(removed > 0);
        let mut remaining_logs = 0_u64;
        let mut remaining_bytes = 0_u64;
        let mut remaining_failures = 0_u64;
        for entry in fs::read_dir(&root).unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let len = entry.metadata().unwrap().len();
            if is_failure_evidence(&name) {
                remaining_failures += 1;
            } else if is_managed_log(&name) {
                remaining_logs += 1;
                remaining_bytes += len;
            }
        }
        assert!(
            remaining_logs <= LOG_RETENTION_PER_PREFIX as u64,
            "per-prefix retention bound: {remaining_logs}"
        );
        assert!(
            remaining_bytes <= LOG_DIRECTORY_QUOTA_BYTES,
            "directory quota bound: {remaining_bytes}"
        );
        assert!(remaining_failures <= FAILURE_EVIDENCE_KEPT as u64);
        let _ = fs::remove_dir_all(root);
    }
}
