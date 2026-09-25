//! Child-process construction that never flashes a console window.
//!
//! Localmotive is a windowed application, so it owns no console. Every
//! `std::process::Command` it spawns on Windows would therefore allocate a
//! fresh console window: `nvidia-smi`, the PowerShell adapter query, the
//! `llama-server --version` / `--help` capability probes, and the served
//! model itself. Those windows appear and disappear as visible flashes.
//!
//! All child processes must be created through [`hidden_command`], which sets
//! `CREATE_NO_WINDOW` on Windows and behaves like a plain `Command` elsewhere.

use std::ffi::OsStr;
use std::io::Read;
#[cfg(not(windows))]
use std::process::Child;
use std::process::{ChildStderr, ChildStdout, Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// `CREATE_NO_WINDOW`: run the child without allocating a console window.
#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Build a `Command` that does not create a console window.
pub fn hidden_command<S: AsRef<OsStr>>(program: S) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

fn read_bounded<R: Read>(mut reader: R, limit: usize) -> Result<(Vec<u8>, bool), String> {
    let mut retained = Vec::with_capacity(limit.min(64 * 1024));
    let mut exceeded = false;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            return Ok((retained, exceeded));
        }
        let available = limit.saturating_sub(retained.len());
        let keep = count.min(available);
        retained.extend_from_slice(&buffer[..keep]);
        exceeded |= keep < count;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessFailureKind {
    Spawn,
    Timeout,
    Cancelled,
    OutputLimit,
    Io,
    /// The operation ended but the process tree survived termination: the
    /// caller must not treat this like a finished Timeout/Cancelled run
    /// (audit PROC-03).
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessFailure {
    pub kind: ProcessFailureKind,
    pub message: String,
}

impl ProcessFailure {
    fn new(kind: ProcessFailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ProcessFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[cfg(not(windows))]
type ContainedChild = Child;

pub struct ContainedProcess {
    child: ContainedChild,
}

impl ContainedProcess {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    pub fn terminate_and_wait(&mut self) -> bool {
        terminate_and_wait(&mut self.child)
    }

    /// Take the child's piped stdout/stderr so a drain thread can copy them
    /// into a bounded sink without the child ever blocking (audit OPS-01).
    /// Both are `None` when the command did not request pipes.
    pub fn take_pipes(
        &mut self,
    ) -> (
        Option<std::process::ChildStdout>,
        Option<std::process::ChildStderr>,
    ) {
        (self.child.stdout().take(), self.child.stderr().take())
    }
}

impl Drop for ContainedProcess {
    fn drop(&mut self) {
        let _ = terminate_and_wait(&mut self.child);
    }
}

#[cfg(windows)]
mod containment {
    //! Per-child job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
    //!
    //! The packaged G-05 test found the gap this closes: `process-wrap`'s std
    //! `JobObject` wrapper creates its job with `kill_on_drop = false`, so a
    //! forced app exit left the managed `llama-server` alive and serving.
    //! Each contained child now gets its own job. Killing one child terminates
    //! only that child's tree (`TerminateJobObject`); when the app's handles
    //! close - normal exit, forced kill, or crash - every job closes and
    //! Windows kills the remaining children.
    use std::os::windows::io::{AsRawHandle, BorrowedHandle};
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    /// An owned job handle; closing it kills its members on Windows.
    pub struct JobHandle(isize);

    // The handle is process-wide; the app shares it across the threads that
    // own, watch and reap the same child.
    unsafe impl Send for JobHandle {}
    unsafe impl Sync for JobHandle {}

    impl JobHandle {
        pub fn create() -> Result<JobHandle, String> {
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return Err(format!(
                        "process containment is unavailable on this system: {}",
                        std::io::Error::last_os_error()
                    ));
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                let configured = SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of_val(&info) as u32,
                );
                if configured == 0 {
                    let message = format!(
                        "process containment could not be configured: {}",
                        std::io::Error::last_os_error()
                    );
                    let _ = CloseHandle(job);
                    return Err(message);
                }
                Ok(JobHandle(job as isize))
            }
        }

        pub fn handle(&self) -> HANDLE {
            self.0 as HANDLE
        }

        pub fn assign(&self, process: &BorrowedHandle<'_>) -> Result<(), String> {
            let assigned = unsafe {
                AssignProcessToJobObject(self.handle(), process.as_raw_handle() as HANDLE)
            };
            if assigned == 0 {
                return Err(format!(
                    "process containment failed: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok(())
        }

        /// Terminate every process in this job (the child and its descendants).
        pub fn terminate(&self) {
            unsafe {
                let _ = TerminateJobObject(self.handle(), 1);
            }
        }
    }

    impl Drop for JobHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.handle());
            }
        }
    }

    /// Resume every thread of a process that was created suspended.
    /// P1-6 (PROC-01): children start suspended so the job assignment
    /// happens before they run; this releases them afterwards. Returns
    /// an error when no thread could be resumed, so the caller refuses
    /// to leave a permanently suspended child behind.
    pub fn resume_process(pid: u32) -> Result<(), String> {
        use windows_sys::Win32::Foundation::{
            CloseHandle, GetLastError, ERROR_NO_MORE_FILES, HANDLE, INVALID_HANDLE_VALUE,
        };
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
        };
        use windows_sys::Win32::System::Threading::{
            OpenThread, ResumeThread, THREAD_SUSPEND_RESUME,
        };
        struct SnapshotGuard(HANDLE);
        impl Drop for SnapshotGuard {
            fn drop(&mut self) {
                unsafe {
                    let _ = CloseHandle(self.0);
                }
            }
        }
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err(format!(
                    "process containment could not list threads: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let _guard = SnapshotGuard(snapshot);
            let mut entry: THREADENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
            if Thread32First(snapshot, &mut entry) == 0 {
                return Err(format!(
                    "process containment could not list threads: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let mut resumed = false;
            loop {
                if entry.th32OwnerProcessID == pid {
                    let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                    if thread.is_null() {
                        return Err(format!(
                            "process containment could not open a thread: {}",
                            std::io::Error::last_os_error()
                        ));
                    }
                    let previous = ResumeThread(thread);
                    let _ = CloseHandle(thread);
                    if previous == u32::MAX {
                        return Err(format!(
                            "process containment could not resume a thread: {}",
                            std::io::Error::last_os_error()
                        ));
                    }
                    if previous > 0 {
                        resumed = true;
                    }
                }
                if Thread32Next(snapshot, &mut entry) == 0 {
                    if GetLastError() != ERROR_NO_MORE_FILES {
                        return Err(format!(
                            "process containment could not list threads: {}",
                            std::io::Error::last_os_error()
                        ));
                    }
                    break;
                }
            }
            if resumed {
                Ok(())
            } else {
                Err("process containment resumed no thread; refusing a suspended child".into())
            }
        }
    }

    #[cfg(test)]
    pub fn process_in_job(process: std::os::windows::io::RawHandle, job: HANDLE) -> bool {
        use windows_sys::Win32::System::JobObjects::IsProcessInJob;
        let mut result = 0;
        let ok = unsafe { IsProcessInJob(process as HANDLE, job, &mut result) };
        ok != 0 && result != 0
    }

    #[cfg(test)]
    pub fn kill_on_close_set(job: HANDLE) -> bool {
        use windows_sys::Win32::System::JobObjects::QueryInformationJobObject;
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        let ok = unsafe {
            QueryInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &mut info as *mut _ as *mut core::ffi::c_void,
                std::mem::size_of_val(&info) as u32,
                std::ptr::null_mut(),
            )
        };
        ok != 0 && (info.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE) != 0
    }
}

/// A contained child on Windows: the wrapped process plus its kill-on-close
/// job. Killing the child terminates its whole tree; dropping the job handle
/// (any app exit path) does the same for anything still running.
#[cfg(windows)]
pub struct ContainedChild {
    inner: Box<dyn process_wrap::std::ChildWrapper>,
    job: containment::JobHandle,
}

#[cfg(windows)]
impl ContainedChild {
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    pub fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.inner.try_wait()
    }

    pub fn start_kill(&mut self) -> std::io::Result<()> {
        self.job.terminate();
        self.inner.start_kill()
    }

    pub fn stdout(&mut self) -> &mut Option<ChildStdout> {
        self.inner.stdout()
    }

    pub fn stderr(&mut self) -> &mut Option<ChildStderr> {
        self.inner.stderr()
    }

    /// Test-only view: the child's raw process handle, its job handle as a
    /// raw value, and whether that job carries kill-on-close.
    #[cfg(test)]
    pub fn containment_probe(&self) -> Option<(std::os::windows::io::RawHandle, usize, bool)> {
        use std::os::windows::io::AsRawHandle;
        let process = self.inner.process_handle().map(|h| h.as_raw_handle())?;
        Some((
            process,
            self.job.handle() as usize,
            containment::kill_on_close_set(self.job.handle()),
        ))
    }
}

#[cfg(windows)]
fn spawn_contained(command: &mut Command) -> Result<ContainedChild, ProcessFailure> {
    use process_wrap::std::{CommandWrap, CreationFlags};
    use windows::Win32::System::Threading::{
        CREATE_NO_WINDOW as WINDOWS_CREATE_NO_WINDOW, CREATE_SUSPENDED,
    };

    let job = containment::JobHandle::create()
        .map_err(|message| ProcessFailure::new(ProcessFailureKind::Spawn, message))?;
    let command = std::mem::replace(command, Command::new(""));
    let mut wrapped = CommandWrap::from(command);
    // P1-6 (PROC-01): the child starts suspended, so the job assignment
    // below happens before it runs. A child that spawns descendants before
    // the assignment would strand them outside the job.
    wrapped.wrap(CreationFlags(WINDOWS_CREATE_NO_WINDOW | CREATE_SUSPENDED));
    let mut child = wrapped.spawn().map_err(|error| {
        ProcessFailure::new(
            ProcessFailureKind::Spawn,
            format!("Contained child process could not start: {error}"),
        )
    })?;
    match child.process_handle() {
        Some(handle) => {
            if let Err(message) = job.assign(&handle) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ProcessFailure::new(ProcessFailureKind::Spawn, message));
            }
        }
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ProcessFailure::new(
                ProcessFailureKind::Spawn,
                "Child process handle was unavailable; refusing an uncontained child",
            ));
        }
    }
    // Assigned while suspended: release it now. A child we cannot resume is
    // terminated, not left suspended (fail closed).
    if let Err(message) = containment::resume_process(child.id()) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(ProcessFailure::new(ProcessFailureKind::Spawn, message));
    }
    Ok(ContainedChild { inner: child, job })
}

#[cfg(not(windows))]
fn spawn_contained(command: &mut Command) -> Result<ContainedChild, ProcessFailure> {
    command.spawn().map_err(|error| {
        ProcessFailure::new(
            ProcessFailureKind::Spawn,
            format!("Child process could not start: {error}"),
        )
    })
}

pub fn spawn_contained_process(command: &mut Command) -> Result<ContainedProcess, ProcessFailure> {
    spawn_contained(command).map(|child| ContainedProcess { child })
}

#[cfg(windows)]
fn take_stdout(child: &mut ContainedChild) -> Option<ChildStdout> {
    child.stdout().take()
}

#[cfg(not(windows))]
fn take_stdout(child: &mut ContainedChild) -> Option<ChildStdout> {
    child.stdout.take()
}

#[cfg(windows)]
fn take_stderr(child: &mut ContainedChild) -> Option<ChildStderr> {
    child.stderr().take()
}

#[cfg(not(windows))]
fn take_stderr(child: &mut ContainedChild) -> Option<ChildStderr> {
    child.stderr.take()
}

/// How long cleanup waits for an exit confirmation before reporting the
/// outcome as unresolved (audit S-01). A blocking `wait()` would hide an
/// unkillable process forever and make every caller's deadline a lie.
pub const TERMINATION_DEADLINE: Duration = Duration::from_secs(10);
const TERMINATION_POLL: Duration = Duration::from_millis(50);

/// Terminate the child and confirm the exit within a monotonic deadline.
/// Returns `true` only when the exit was observed; `false` means the caller
/// must report an unresolved cleanup outcome instead of assuming success.
fn terminate_and_wait(child: &mut ContainedChild) -> bool {
    terminate_and_wait_with_deadline(child, TERMINATION_DEADLINE)
}

fn terminate_and_wait_with_deadline(child: &mut ContainedChild, deadline: Duration) -> bool {
    #[cfg(windows)]
    {
        let _ = child.start_kill();
    }
    #[cfg(not(windows))]
    {
        let _ = child.kill();
    }
    let limit = Instant::now() + deadline;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => {
                if Instant::now() >= limit {
                    return false;
                }
                std::thread::sleep(TERMINATION_POLL);
            }
            Err(_) => return false,
        }
    }
}

/// Run one child process with finite time, cancellation, and output limits.
pub fn output_with_timeout_and_cancel(
    command: &mut Command,
    timeout: Duration,
    max_stream_bytes: usize,
    cancel: &AtomicBool,
) -> Result<Output, ProcessFailure> {
    if cancel.load(Ordering::Relaxed) {
        return Err(ProcessFailure::new(
            ProcessFailureKind::Cancelled,
            "Child process was cancelled before start",
        ));
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = spawn_contained(command)?;
    let Some(stdout) = take_stdout(&mut child) else {
        terminate_and_wait(&mut child);
        return Err(ProcessFailure::new(
            ProcessFailureKind::Io,
            "Child process did not expose stdout",
        ));
    };
    let Some(stderr) = take_stderr(&mut child) else {
        terminate_and_wait(&mut child);
        return Err(ProcessFailure::new(
            ProcessFailureKind::Io,
            "Child process did not expose stderr",
        ));
    };
    let stdout_reader = std::thread::spawn(move || read_bounded(stdout, max_stream_bytes));
    let stderr_reader = std::thread::spawn(move || read_bounded(stderr, max_stream_bytes));
    let started = Instant::now();

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if cancel.load(Ordering::Relaxed) => {
                let terminated = terminate_and_wait(&mut child);
                let _ = join_reader_with_deadline(stdout_reader, READER_JOIN_GRACE);
                let _ = join_reader_with_deadline(stderr_reader, READER_JOIN_GRACE);
                return Err(cleanup_outcome(
                    ProcessFailureKind::Cancelled,
                    "Child process was cancelled and terminated",
                    terminated,
                ));
            }
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let terminated = terminate_and_wait(&mut child);
                let _ = join_reader_with_deadline(stdout_reader, READER_JOIN_GRACE);
                let _ = join_reader_with_deadline(stderr_reader, READER_JOIN_GRACE);
                return Err(cleanup_outcome(
                    ProcessFailureKind::Timeout,
                    &format!(
                        "Child process timed out after {} milliseconds",
                        timeout.as_millis()
                    ),
                    terminated,
                ));
            }
            Err(error) => {
                terminate_and_wait(&mut child);
                let _ = join_reader_with_deadline(stdout_reader, READER_JOIN_GRACE);
                let _ = join_reader_with_deadline(stderr_reader, READER_JOIN_GRACE);
                return Err(ProcessFailure::new(
                    ProcessFailureKind::Io,
                    format!("Child process status failed: {error}"),
                ));
            }
        }
    };
    if !terminate_and_wait(&mut child) {
        let _ = join_reader_with_deadline(stdout_reader, READER_JOIN_GRACE);
        let _ = join_reader_with_deadline(stderr_reader, READER_JOIN_GRACE);
        return Err(ProcessFailure::new(
            ProcessFailureKind::Unresolved,
            "Child process tree did not stop within the cleanup limit",
        ));
    }
    let (stdout, stdout_exceeded) = join_reader_with_deadline(stdout_reader, READER_JOIN_GRACE)
        .ok_or_else(|| {
            ProcessFailure::new(
                ProcessFailureKind::Io,
                "Child stdout reader did not finish within the grace period",
            )
        })?
        .map_err(|error| ProcessFailure::new(ProcessFailureKind::Io, error))?;
    let (stderr, stderr_exceeded) = join_reader_with_deadline(stderr_reader, READER_JOIN_GRACE)
        .ok_or_else(|| {
            ProcessFailure::new(
                ProcessFailureKind::Io,
                "Child stderr reader did not finish within the grace period",
            )
        })?
        .map_err(|error| ProcessFailure::new(ProcessFailureKind::Io, error))?;
    if stdout_exceeded || stderr_exceeded {
        return Err(ProcessFailure::new(
            ProcessFailureKind::OutputLimit,
            format!("Child process output exceeded {max_stream_bytes} bytes per stream"),
        ));
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

/// Classify a cleanup exit by whether termination actually stopped the tree.
///
/// A timeout or cancel whose tree survives `terminate_and_wait` returns
/// `Unresolved` instead of the success-shaped Timeout/Cancelled kind, so
/// callers can distinguish "the operation ended" from "the operation ended
/// and its process is gone".
fn cleanup_outcome(kind: ProcessFailureKind, detail: &str, terminated: bool) -> ProcessFailure {
    if terminated {
        ProcessFailure::new(kind, detail)
    } else {
        ProcessFailure::new(
            ProcessFailureKind::Unresolved,
            format!("{detail}, but the process tree survived cleanup and may still be running"),
        )
    }
}

/// Grace period for pipe-reader threads to observe EOF after the child is
/// gone. A reader that is still blocked past the deadline belongs to a child
/// that survived termination; its handle is detached instead of joined
/// forever, so one unkillable child cannot hang a bounded operation
/// (audit PROC-02). The detached thread exits on its own when the child
/// finally releases the pipes.
const READER_JOIN_GRACE: Duration = Duration::from_secs(5);

/// Join a pipe-reader thread, giving up after `deadline`.
///
/// Returns `None` when the thread is still blocked past the deadline (its
/// handle is detached) or when it panicked.
pub(crate) fn join_reader_with_deadline<T>(
    handle: std::thread::JoinHandle<T>,
    deadline: Duration,
) -> Option<T> {
    let give_up = Instant::now() + deadline;
    while !handle.is_finished() {
        if Instant::now() >= give_up {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    handle.join().ok()
}

/// Run one child process with finite time and per-stream output limits.
pub fn output_with_timeout(
    command: &mut Command,
    timeout: Duration,
    max_stream_bytes: usize,
) -> Result<Output, String> {
    output_with_timeout_and_cancel(command, timeout, max_stream_bytes, &AtomicBool::new(false))
        .map_err(|error| error.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_join_returns_finished_threads() {
        // P1-18 (PROC-02): cleanup paths must not join pipe readers forever.
        let handle = std::thread::spawn(|| 42);
        let started = Instant::now();
        let result = join_reader_with_deadline(handle, Duration::from_millis(100));
        assert_eq!(result, Some(42));
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "a finished reader must join promptly"
        );
    }

    #[test]
    fn reader_join_gives_up_on_a_blocked_thread() {
        // A thread parked forever (the analogue of a reader blocked on the
        // pipes of a child that survived termination) detaches after the
        // deadline instead of hanging the bounded operation.
        let handle = std::thread::spawn(|| {
            std::thread::park();
            42
        });
        let started = Instant::now();
        let result = join_reader_with_deadline(handle, Duration::from_millis(100));
        assert_eq!(result, None);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "a blocked reader must detach after the deadline"
        );
    }

    #[test]
    fn cleanup_outcome_reports_unresolved_termination() {
        // P1-19 (PROC-03): a timeout or cancel whose tree survives cleanup
        // must not return the success-shaped Timeout/Cancelled kind.
        let failure = cleanup_outcome(
            ProcessFailureKind::Timeout,
            "Child process timed out after 100 milliseconds",
            false,
        );
        assert_eq!(failure.kind, ProcessFailureKind::Unresolved);
        assert!(
            failure.message.contains("survived"),
            "the message must say the tree survived: {}",
            failure.message
        );
        let finished = cleanup_outcome(ProcessFailureKind::Timeout, "detail", true);
        assert_eq!(finished.kind, ProcessFailureKind::Timeout);
        let cancelled = cleanup_outcome(ProcessFailureKind::Cancelled, "detail", true);
        assert_eq!(cancelled.kind, ProcessFailureKind::Cancelled);
    }

    #[test]
    fn failed_tree_cleanup_names_the_unresolved_kind() {
        // The post-loop branch (the tree ignored termination) must surface
        // Unresolved instead of the generic Io kind (audit PROC-03).
        let source = include_str!("proc.rs");
        let runner = source
            .find("pub fn output_with_timeout_and_cancel(")
            .expect("the bounded runner present");
        let body = &source[runner..runner + 6_000];
        assert!(
            body.contains("ProcessFailureKind::Unresolved"),
            "the bounded runner must surface unresolved termination"
        );
    }

    #[test]
    fn cleanup_paths_never_join_readers_without_a_deadline() {
        // The bounded runner must not contain a bare reader join: every
        // cleanup path (cancel, timeout, status error, failed termination)
        // goes through the deadline helper (audit PROC-02).
        let source = include_str!("proc.rs");
        let runner = source
            .find("pub fn output_with_timeout_and_cancel(")
            .expect("the bounded runner present");
        let body = &source[runner..runner + 6_000];
        assert!(
            !body.contains("stdout_reader.join()") && !body.contains("stderr_reader.join()"),
            "reader joins must go through join_reader_with_deadline"
        );
        assert!(
            body.contains("join_reader_with_deadline"),
            "the bounded runner must join readers with a deadline"
        );
    }

    #[cfg(windows)]
    #[test]
    fn contained_children_are_members_of_the_kill_on_close_job() {
        // G-05 packaged finding: the crate's std JobObject wrapper creates
        // its job with kill_on_drop = false, so a killed app used to leave
        // the managed server running. Every contained child now owns a job
        // with that limit and is a member of it.
        let mut command = super::hidden_command("powershell.exe");
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Start-Sleep -Seconds 30",
        ]);
        let mut process = super::spawn_contained_process(&mut command).expect("spawn");
        let (raw_process, raw_job, kill_on_close) =
            process.child.containment_probe().expect("probe");
        use windows_sys::Win32::Foundation::HANDLE;
        assert!(
            kill_on_close,
            "the child's job must carry JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE"
        );
        assert!(
            super::containment::process_in_job(raw_process, raw_job as HANDLE),
            "contained child must belong to its kill-on-close job"
        );
        assert!(process.terminate_and_wait(), "cleanup must succeed");
    }

    #[cfg(windows)]
    #[test]
    fn suspended_children_are_assigned_before_they_run() {
        // P1-6 (PROC-01): the spawn-assign gap let a fast child spawn
        // descendants outside the job. The child starts suspended (it must
        // not exit while held), joins the job, then resumes to its exit.
        use std::os::windows::io::{AsRawHandle, BorrowedHandle};
        use std::os::windows::process::CommandExt;
        use windows::Win32::System::Threading::{CREATE_NO_WINDOW, CREATE_SUSPENDED};
        let mut command = super::hidden_command("cmd.exe");
        command.args(["/C", "exit 42"]);
        command.creation_flags((CREATE_NO_WINDOW | CREATE_SUSPENDED).0);
        let mut child = command.spawn().expect("suspended spawn");
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(
            child.try_wait().expect("poll").is_none(),
            "a suspended child must not run before assignment"
        );
        let job = super::containment::JobHandle::create().expect("job");
        let borrowed = unsafe { BorrowedHandle::borrow_raw(child.as_raw_handle()) };
        job.assign(&borrowed).expect("assign while suspended");
        assert!(
            super::containment::process_in_job(child.as_raw_handle(), job.handle()),
            "assigned-while-suspended child must belong to the job"
        );
        super::containment::resume_process(child.id()).expect("resume");
        assert_eq!(child.wait().expect("wait").code(), Some(42));
    }

    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Give each test its own temp directory: parallel `cargo test` threads
    /// share one process id, so a pid-keyed name lets two tests wipe each
    /// other's marker files and the descendant fixture never appears to start.
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

    /// Source-level invariant: no module may bypass `hidden_command`, because a
    /// single direct `Command::new` reintroduces the console flash.
    #[test]
    fn no_module_constructs_a_raw_command() {
        // Discover every production module instead of trusting a fixed list
        // (audit S-01.I3): a new module cannot silently bypass the invariant.
        // proc.rs itself is the one module allowed to build commands, and only
        // inside `hidden_command`; assert that exception stays small.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut checked = 0_usize;
        for entry in std::fs::read_dir(&dir).expect("src directory") {
            let path = entry.expect("src entry").path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            if name == "proc.rs" {
                // Production code may construct commands only inside
                // `hidden_command`; test fixtures below `mod tests` spawn real
                // child processes on purpose.
                let production = text.split("mod tests").next().unwrap_or(&text);
                // Two production occurrences are allowed and both are known:
                // `hidden_command` builds the real command, and
                // `spawn_contained` swaps in an empty placeholder (which
                // constructs no process).
                assert_eq!(
                    production.matches("Command::new(").count(),
                    2,
                    "proc.rs may construct commands only inside hidden_command"
                );
                assert!(production.contains("let mut command = Command::new(program);"));
                assert!(production.contains("Command::new(\"\")"));
                continue;
            }
            assert!(
                !text.contains("Command::new("),
                "{name} constructs a raw Command; use proc::hidden_command instead"
            );
            checked += 1;
        }
        assert!(
            checked >= 15,
            "expected the production module set, saw {checked}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn child_tree_is_assigned_before_user_code_can_run() {
        let source = include_str!("proc.rs");
        assert!(source.contains("containment::JobHandle::create()"));
        assert!(source.contains("JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE"));
        assert_eq!(source.matches("ProcessTree::assign").count(), 1);
        // P1-6 (PROC-01): assignment after a running start still leaves a
        // gap where the child spawns descendants outside the job. The spawn
        // must order suspended create, job assignment, then resume.
        let start = source
            .find("fn spawn_contained(command: &mut Command)")
            .expect("spawn_contained");
        let after = &source[start..];
        let body = &after[..after
            .find("#[cfg(not(windows))]")
            .expect("windows spawn end")];
        let suspended = body
            .find("WINDOWS_CREATE_NO_WINDOW | CREATE_SUSPENDED")
            .expect("suspended create");
        let assigned = body.find("job.assign").expect("job assignment");
        let resumed = body
            .find("resume_process")
            .expect("resume after assignment");
        assert!(
            suspended < assigned && assigned < resumed,
            "spawn must order suspended create, job assignment, then resume"
        );
    }

    #[cfg(windows)]
    #[test]
    fn hidden_command_keeps_the_requested_program() {
        let command = super::hidden_command("nvidia-smi.exe");
        assert_eq!(command.get_program(), "nvidia-smi.exe");
    }

    #[cfg(windows)]
    #[test]
    fn bounded_output_terminates_a_stalled_hardware_probe() {
        let mut command = super::hidden_command("powershell.exe");
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Start-Sleep -Seconds 2",
        ]);
        let started = Instant::now();

        let error =
            super::output_with_timeout(&mut command, Duration::from_millis(50), 1024).unwrap_err();

        assert!(error.contains("timed out"), "{error}");
        // PowerShell spawn alone costs seconds on a loaded box (measured
        // ~3.2 s nested here), so allow a generous spawn allowance on top of
        // the 50 ms kill timeout instead of a 1 s wall clock.
        assert!(started.elapsed() < Duration::from_secs(30));
    }

    #[cfg(windows)]
    #[test]
    fn cancellation_terminates_a_running_health_child() {
        let mut command = super::hidden_command("powershell.exe");
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Start-Sleep -Seconds 10",
        ]);
        let cancel = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancel);
        let canceller = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            signal.store(true, Ordering::Relaxed);
        });
        let started = Instant::now();

        let error = super::output_with_timeout_and_cancel(
            &mut command,
            Duration::from_secs(5),
            1024,
            &cancel,
        )
        .unwrap_err();

        canceller.join().unwrap();
        assert_eq!(error.kind, super::ProcessFailureKind::Cancelled);
        // As above, cancelling at 50 ms must still beat the 10 s sleep, but
        // process spawn under load costs seconds, so allow 30 s total.
        assert!(started.elapsed() < Duration::from_secs(30));
    }

    #[cfg(windows)]
    #[test]
    fn cancellation_terminates_descendant_processes() {
        use base64::Engine as _;

        let root = unique_test_dir("localmotive-process-tree-cancel");
        std::fs::create_dir_all(&root).unwrap();
        let started_marker = root.join("descendant-started.txt");
        let marker = root.join("descendant-survived.txt");
        let started_literal = started_marker.to_string_lossy().replace('\'', "''");
        let marker_literal = marker.to_string_lossy().replace('\'', "''");
        let descendant_script = format!(
            "[System.IO.File]::WriteAllText('{started_literal}', 'started'); Start-Sleep -Milliseconds 800; [System.IO.File]::WriteAllText('{marker_literal}', 'survived')"
        );
        let encoded_bytes = descendant_script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        let encoded = base64::engine::general_purpose::STANDARD.encode(encoded_bytes);
        let parent_script = format!(
            "Start-Process -FilePath 'powershell.exe' -ArgumentList '-NoProfile','-NonInteractive','-EncodedCommand','{encoded}'; Start-Sleep -Seconds 10"
        );
        let mut command = super::hidden_command("powershell.exe");
        command.args(["-NoProfile", "-NonInteractive", "-Command", &parent_script]);
        let cancel = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancel);
        let signal_marker = started_marker.clone();
        let canceller = std::thread::spawn(move || {
            let started = Instant::now();
            while !signal_marker.exists() && started.elapsed() < Duration::from_secs(3) {
                std::thread::sleep(Duration::from_millis(10));
            }
            signal.store(true, Ordering::Relaxed);
        });

        let error = super::output_with_timeout_and_cancel(
            &mut command,
            Duration::from_secs(5),
            1024,
            &cancel,
        )
        .unwrap_err();

        canceller.join().unwrap();
        assert_eq!(error.kind, super::ProcessFailureKind::Cancelled);
        // PowerShell spawn under full-suite load can exceed the 3 s
        // start-signal window plus the 5 s kill timeout (measured ~3.2 s
        // for one nested spawn here): retry the whole fixture once before
        // calling it a product failure.
        if !started_marker.exists() {
            let cancel = Arc::new(AtomicBool::new(false));
            let signal = Arc::clone(&cancel);
            let signal_marker = started_marker.clone();
            let canceller = std::thread::spawn(move || {
                let started = Instant::now();
                while !signal_marker.exists() && started.elapsed() < Duration::from_secs(8) {
                    std::thread::sleep(Duration::from_millis(10));
                }
                signal.store(true, Ordering::Relaxed);
            });
            let mut retry = super::hidden_command("powershell.exe");
            retry.args(["-NoProfile", "-NonInteractive", "-Command", &parent_script]);
            let retry_error = super::output_with_timeout_and_cancel(
                &mut retry,
                Duration::from_secs(12),
                1024,
                &cancel,
            )
            .unwrap_err();
            canceller.join().unwrap();
            assert_eq!(retry_error.kind, super::ProcessFailureKind::Cancelled);
        }
        assert!(
            started_marker.exists(),
            "the descendant fixture did not start"
        );
        std::thread::sleep(Duration::from_millis(1_000));
        assert!(
            !marker.exists(),
            "cancellation left a descendant process running"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn dropping_a_contained_process_terminates_descendants() {
        use base64::Engine as _;

        let root = unique_test_dir("localmotive-process-tree-drop");
        std::fs::create_dir_all(&root).unwrap();
        let started_marker = root.join("descendant-started.txt");
        let marker = root.join("descendant-survived.txt");
        let spawned_marker = root.join("descendant-spawned.txt");
        let started_literal = started_marker.to_string_lossy().replace('\'', "''");
        let marker_literal = marker.to_string_lossy().replace('\'', "''");
        let spawned_literal = spawned_marker.to_string_lossy().replace('\'', "''");
        let descendant_script = format!(
            "[System.IO.File]::WriteAllText('{started_literal}', 'started'); Start-Sleep -Milliseconds 800; [System.IO.File]::WriteAllText('{marker_literal}', 'survived')"
        );
        let encoded_bytes = descendant_script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        let encoded = base64::engine::general_purpose::STANDARD.encode(encoded_bytes);
        let parent_script = format!(
            "Start-Process -FilePath 'powershell.exe' -ArgumentList '-NoProfile','-NonInteractive','-EncodedCommand','{encoded}' -PassThru | ForEach-Object {{ [System.IO.File]::WriteAllText('{spawned_literal}', [string]$_.Id) }}; Start-Sleep -Seconds 10"
        );
        let mut command = super::hidden_command("powershell.exe");
        command.args(["-NoProfile", "-NonInteractive", "-Command", &parent_script]);
        let mut process = super::spawn_contained_process(&mut command).unwrap();
        // PowerShell's cold start under a fully parallel test run (or a
        // loaded machine: nested startup plus antivirus scanning) can far
        // exceed fifteen seconds. The parent exits on its own schedule
        // (about ten seconds after spawning) whether or not the descendant
        // has booted yet, so parent exit alone must NOT end the wait: the
        // parent records the spawned child id when Start-Process succeeded,
        // and only an exited parent WITHOUT that record is a fixture
        // failure. The descendant may take the full bound.
        let started = Instant::now();
        let mut parent_exit: Option<std::process::ExitStatus> = None;
        while !started_marker.exists() && started.elapsed() < Duration::from_secs(45) {
            parent_exit = process.try_wait().ok().flatten();
            if parent_exit.is_some() && !spawned_marker.exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }

        assert!(
            started_marker.exists(),
            "the descendant fixture did not start (spawned={}, parent_exit={parent_exit:?}, elapsed={:?})",
            spawned_marker.exists(),
            started.elapsed()
        );
        drop(process);
        std::thread::sleep(Duration::from_millis(1_000));
        assert!(!marker.exists(), "drop left a descendant process running");
        std::fs::remove_dir_all(root).unwrap();
    }
}
