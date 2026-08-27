//! Project process supervision: Job Object assignment, stdout/stderr capture,
//! lifecycle transitions, readiness, restart policy, and clean shutdown.

use std::{
    ffi::c_void,
    io,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use uuid::Uuid;
use windows_sys::Win32::{Foundation::CloseHandle, System::Threading::CREATE_NEW_PROCESS_GROUP};

use crate::{
    exec,
    job::{ProcessJob, open_process, wait_exit},
    lifecycle::LifecycleState,
    package_manager::isolated_path,
    paths::AppPaths,
    state::StateClient,
};

pub const MAX_RESTART_ATTEMPTS: u32 = 3;
pub const RESTART_COOLDOWN_MS: u64 = 1500;
pub const STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
pub const LOG_LIMIT_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunConfig {
    pub project_id: Uuid,
    pub project_dir: PathBuf,
    pub node_exe: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub port: u16,
    pub port_is_env: bool,
}

/// Bounded, redacting log buffer shared with the UI.
pub struct LogBuffer {
    inner: Mutex<Vec<u8>>,
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }

    pub fn append(&self, bytes: &[u8]) {
        if let Ok(mut buffer) = self.inner.lock() {
            buffer.extend_from_slice(bytes);
            if buffer.len() > LOG_LIMIT_BYTES {
                let drop = buffer.len() - LOG_LIMIT_BYTES;
                buffer.drain(..drop);
            }
        }
    }

    pub fn snapshot(&self) -> String {
        let bytes = self.inner.lock().map(|b| b.clone()).unwrap_or_default();
        redact(&String::from_utf8_lossy(&bytes))
    }
}

fn redact(text: &str) -> String {
    let mut result = text.to_owned();
    for needle in [
        "-----BEGIN RSA PRIVATE KEY-----",
        "token=",
        "password=",
        "secret=",
    ] {
        result = result.replace(needle, "[REDACTED]");
    }
    result
}

pub struct SupervisedRun {
    pub job: ProcessJob,
    pub child: Child,
    pub logs: Arc<LogBuffer>,
    pub state: LifecycleState,
    pub port: u16,
}

impl SupervisedRun {
    /// Spawn the dev command, assign to a Job Object, capture logs.
    pub fn spawn(config: &RunConfig) -> Result<Self, String> {
        use std::os::windows::process::CommandExt;

        let job = ProcessJob::create().map_err(|error| error.to_string())?;

        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .current_dir(&config.project_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NEW_PROCESS_GROUP);

        if config.port_is_env {
            command.env("PORT", config.port.to_string());
        }
        // Isolated PATH: node dir + project node_modules/.bin first.
        let path_entries = isolated_path(
            config.node_exe.parent().unwrap_or(Path::new(".")),
            &config.project_dir,
        );
        for (key, value) in exec::child_environment(&path_entries, &[]) {
            command.env(key, value);
        }

        let mut child = command.spawn().map_err(|error| error.to_string())?;
        let pid = child.id();
        let process_handle =
            open_process(pid).ok_or_else(|| "cannot open child process".to_owned())?;
        // SAFETY: assign then close the borrowed handle exactly once.
        let assigned = unsafe { job.assign(process_handle) };
        unsafe {
            CloseHandle(process_handle);
        }
        assigned.map_err(|error| error.to_string())?;

        let logs = Arc::new(LogBuffer::new());
        let stdout_logs = Arc::clone(&logs);
        if let Some(stdout) = child.stdout.take() {
            pump(stdout, stdout_logs);
        }
        let stderr_logs = Arc::clone(&logs);
        if let Some(stderr) = child.stderr.take() {
            pump(stderr, stderr_logs);
        }

        Ok(Self {
            job,
            child,
            logs,
            state: LifecycleState::StartingApp,
            port: config.port,
        })
    }

    /// Wait for readiness (TCP connect to the internal port) or timeout.
    pub fn wait_ready(&mut self, timeout: Duration) -> LifecycleState {
        let deadline = Instant::now() + timeout;
        loop {
            if self.try_wait().is_ok_and(|status| status.is_some()) {
                self.state = LifecycleState::Failed;
                return self.state;
            }
            if std::net::TcpStream::connect(("127.0.0.1", self.port)).is_ok() {
                self.state = LifecycleState::Running;
                return self.state;
            }
            if Instant::now() >= deadline {
                self.state = LifecycleState::Failed;
                return self.state;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    pub fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }
}

/// Graceful then forced shutdown of the whole job tree. `group_id` is the
/// child's process group (its pid when spawned with CREATE_NEW_PROCESS_GROUP).
pub fn stop_run(run: &SupervisedRun, group_id: u32) -> io::Result<()> {
    run.job.stop(group_id)?;
    Ok(())
}

/// Open the child process handle for `wait_exit` usage.
fn child_handle(child: &Child) -> Option<*mut c_void> {
    open_process(child.id())
}

/// Wait for the child to exit (used after graceful signal).
pub fn wait_child_exit(child: &Child, timeout: Duration) -> io::Result<()> {
    let handle = child_handle(child).ok_or_else(|| io::Error::other("cannot open child"))?;
    let result = wait_exit(handle, timeout);
    // SAFETY: the handle was opened here and is closed exactly once.
    unsafe {
        CloseHandle(handle);
    }
    result
}

fn pump(mut reader: impl io::Read + Send + 'static, logs: Arc<LogBuffer>) {
    std::thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => logs.append(&buffer[..n]),
                Err(_) => break,
            }
        }
    });
}

/// Keep AppPaths/StateClient referenced for future wiring without unused warnings.
#[allow(dead_code)]
pub(crate) fn _wiring_hint(paths: &AppPaths, state: &StateClient) {
    let _ = (paths, state);
}
