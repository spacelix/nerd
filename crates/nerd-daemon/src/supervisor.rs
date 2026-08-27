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
    /// npm script name (e.g. "dev").
    pub script: String,
    /// Arguments passed to the script after `npm run <script>`.
    pub args: Vec<String>,
    pub port: u16,
    pub port_is_env: bool,
    /// Optional HTTP readiness path; when set, readiness is an HTTP GET.
    pub readiness_path: Option<String>,
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
    readiness_path: Option<String>,
}

impl SupervisedRun {
    /// Spawn the dev command, assign to a Job Object, capture logs.
    pub fn spawn(config: &RunConfig) -> Result<Self, String> {
        use std::os::windows::process::CommandExt;

        let job = ProcessJob::create().map_err(|error| error.to_string())?;

        // npm ships with every Node distribution; run the script through it so
        // `npm run <script>` resolves with the isolated PATH.
        let npm_cmd = config
            .node_exe
            .parent()
            .unwrap_or(Path::new("."))
            .join("npm.cmd");
        let mut command = Command::new(npm_cmd);
        let mut full_args = vec!["run".to_owned(), config.script.clone()];
        full_args.extend(config.args.clone());
        command
            .args(&full_args)
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
            readiness_path: config.readiness_path.clone(),
        })
    }

    pub fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    fn config_readiness_path(&self) -> Option<String> {
        self.readiness_path.clone()
    }
}

/// Wait for readiness (HTTP GET to the readiness path when configured, else a
/// TCP connect to the internal port) or timeout. On timeout the whole job tree
/// is terminated so no orphan process survives.
pub fn wait_ready(run: &mut SupervisedRun, timeout: Duration, group_id: u32) -> LifecycleState {
    let deadline = Instant::now() + timeout;
    loop {
        if run.try_wait().is_ok_and(|status| status.is_some()) {
            run.state = LifecycleState::Failed;
            return run.state;
        }
        if readiness_ok(run.port, run.config_readiness_path()) {
            run.state = LifecycleState::Running;
            return run.state;
        }
        if Instant::now() >= deadline {
            // Kill the tree so a failed startup leaves no orphan process.
            let _ = run.job.stop(group_id);
            run.state = LifecycleState::Failed;
            return run.state;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn readiness_ok(port: u16, readiness_path: Option<String>) -> bool {
    match readiness_path {
        Some(path) => http_get(port, &path).is_some(),
        None => std::net::TcpStream::connect(("127.0.0.1", port)).is_ok(),
    }
}

/// Minimal HTTP GET used for readiness probing; returns status on success.
fn http_get(port: u16, path: &str) -> Option<u16> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n",
        if path.is_empty() { "/" } else { path }
    );
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = [0u8; 2048];
    let n = stream.read(&mut response).ok()?;
    let text = String::from_utf8_lossy(&response[..n]);
    text.lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .filter(|code| (200..400).contains(code))
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
