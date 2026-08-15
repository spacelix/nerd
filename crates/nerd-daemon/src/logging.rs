use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use tracing_subscriber::fmt::MakeWriter;

use crate::windows;

const LOG_FILE_NAME: &str = "nerd-daemon.jsonl";
const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;
const LOG_GENERATIONS: usize = 5;
const LOG_QUEUE_CAPACITY: usize = 1_024;
const LOG_CONTROL_TIMEOUT: Duration = Duration::from_secs(2);

pub struct LoggingGuard {
    sender: Option<SyncSender<LogCommand>>,
    worker: Option<JoinHandle<()>>,
    health: LogHealthHandle,
}

impl LoggingGuard {
    pub fn initialize(log_dir: &Path) -> io::Result<Self> {
        let health = LogHealthHandle::default();
        let sink = match RotatingFile::open(log_dir) {
            Ok(file) => LogSink::File(file),
            Err(error) => {
                health.record_io_error();
                fallback_message(&format!(
                    "Nerd file logging unavailable ({}); using debug output\n",
                    error.kind()
                ));
                LogSink::Fallback
            }
        };

        let (sender, receiver) = mpsc::sync_channel(LOG_QUEUE_CAPACITY);
        let worker_health = health.clone();
        let worker = thread::Builder::new()
            .name("nerd-log".to_owned())
            .spawn(move || run_worker(receiver, sink, &worker_health));

        let (backend, sender, worker) = match worker {
            Ok(worker) => (
                WriterBackend::Worker(sender.clone()),
                Some(sender),
                Some(worker),
            ),
            Err(error) => {
                health.record_io_error();
                fallback_message(&format!(
                    "Nerd logging worker unavailable ({}); using debug output\n",
                    error.kind()
                ));
                (WriterBackend::Fallback, None, None)
            }
        };

        let make_writer = LogMakeWriter {
            backend,
            health: health.clone(),
        };
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_ansi(false)
            .with_writer(make_writer)
            .finish();

        let mut guard = Self {
            sender,
            worker,
            health,
        };
        if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
            guard.stop_worker_before(Instant::now() + LOG_CONTROL_TIMEOUT);
            return Err(io::Error::other(format!(
                "failed to install tracing subscriber: {error}"
            )));
        }

        Ok(guard)
    }

    pub fn health(&self) -> LogHealthSnapshot {
        self.health.snapshot()
    }

    pub fn health_handle(&self) -> LogHealthHandle {
        self.health.clone()
    }

    pub fn flush(&self) -> io::Result<()> {
        self.flush_before(Instant::now() + LOG_CONTROL_TIMEOUT)
    }

    pub fn flush_before(&self, deadline: Instant) -> io::Result<()> {
        let result = self.try_flush_before(deadline);
        if let Err(error) = &result {
            self.health.record_io_error();
            fallback_message(&format!(
                "Nerd logging flush failed ({}); using debug output\n",
                error.kind()
            ));
        }
        result
    }

    fn try_flush_before(&self, deadline: Instant) -> io::Result<()> {
        let Some(sender) = &self.sender else {
            return Ok(());
        };
        let (reply_sender, reply_receiver) = mpsc::sync_channel(1);
        send_before(sender, LogCommand::Flush(reply_sender), deadline)?;
        let remaining = deadline.saturating_duration_since(Instant::now());
        reply_receiver
            .recv_timeout(remaining)
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => {
                    io::Error::new(io::ErrorKind::TimedOut, "logging flush timed out")
                }
                mpsc::RecvTimeoutError::Disconnected => {
                    io::Error::new(io::ErrorKind::BrokenPipe, "logging worker stopped")
                }
            })?
    }

    pub fn shutdown_before(mut self, deadline: Instant) {
        let _ = self.flush_before(deadline);
        self.stop_worker_before(deadline);
    }

    fn stop_worker_before(&mut self, deadline: Instant) {
        if let Some(sender) = self.sender.take()
            && send_before(&sender, LogCommand::Shutdown, deadline).is_err()
        {
            self.health.record_io_error();
            fallback_message("nerd-log worker did not accept shutdown before deadline\n");
        }
        if let Some(worker) = self.worker.take() {
            while !worker.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            if worker.is_finished() {
                let _ = worker.join();
            } else {
                self.health.record_io_error();
                fallback_message("nerd-log worker did not stop before deadline\n");
            }
        }
    }
}

impl Drop for LoggingGuard {
    fn drop(&mut self) {
        self.stop_worker_before(Instant::now() + LOG_CONTROL_TIMEOUT);
    }
}

#[derive(Clone, Default)]
pub struct LogHealthHandle(Arc<LogHealthInner>);

#[derive(Default)]
struct LogHealthInner {
    degraded: AtomicBool,
    dropped_events: AtomicU64,
    io_errors: AtomicU64,
}

impl LogHealthHandle {
    fn record_drop(&self) {
        self.0.degraded.store(true, Ordering::Release);
        let previous = self.0.dropped_events.fetch_add(1, Ordering::Relaxed);
        if previous == 0 {
            fallback_message("Nerd logging queue full; events are being dropped\n");
        }
    }

    fn record_io_error(&self) {
        self.0.degraded.store(true, Ordering::Release);
        self.0.io_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> LogHealthSnapshot {
        LogHealthSnapshot {
            degraded: self.0.degraded.load(Ordering::Acquire),
            dropped_events: self.0.dropped_events.load(Ordering::Relaxed),
            io_errors: self.0.io_errors.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogHealthSnapshot {
    pub degraded: bool,
    pub dropped_events: u64,
    pub io_errors: u64,
}

#[derive(Clone)]
struct LogMakeWriter {
    backend: WriterBackend,
    health: LogHealthHandle,
}

impl<'writer> MakeWriter<'writer> for LogMakeWriter {
    type Writer = EventWriter;

    fn make_writer(&'writer self) -> Self::Writer {
        EventWriter {
            buffer: Vec::with_capacity(512),
            backend: self.backend.clone(),
            health: self.health.clone(),
        }
    }
}

#[derive(Clone)]
enum WriterBackend {
    Worker(SyncSender<LogCommand>),
    Fallback,
}

struct EventWriter {
    buffer: Vec<u8>,
    backend: WriterBackend,
    health: LogHealthHandle,
}

impl Write for EventWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for EventWriter {
    fn drop(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        let bytes = std::mem::take(&mut self.buffer);
        match &self.backend {
            WriterBackend::Worker(sender) => match sender.try_send(LogCommand::Write(bytes)) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => self.health.record_drop(),
                Err(TrySendError::Disconnected(LogCommand::Write(bytes))) => {
                    self.health.record_io_error();
                    fallback_bytes(&bytes);
                }
                Err(TrySendError::Disconnected(_)) => self.health.record_io_error(),
            },
            WriterBackend::Fallback => fallback_bytes(&bytes),
        }
    }
}

enum LogCommand {
    Write(Vec<u8>),
    Flush(SyncSender<io::Result<()>>),
    Shutdown,
}

enum LogSink {
    File(RotatingFile),
    Fallback,
}

impl LogSink {
    fn write(&mut self, bytes: &[u8], health: &LogHealthHandle) {
        if let Self::File(file) = self {
            if let Err(error) = file.write_all(bytes) {
                health.record_io_error();
                fallback_message(&format!(
                    "Nerd file logging failed ({}); using debug output\n",
                    error.kind()
                ));
                fallback_bytes(bytes);
                *self = Self::Fallback;
            }
        } else {
            fallback_bytes(bytes);
        }
    }

    fn flush(&mut self, health: &LogHealthHandle) -> io::Result<()> {
        if let Self::File(file) = self {
            if let Err(error) = file.flush() {
                health.record_io_error();
                fallback_message(&format!(
                    "Nerd file logging flush failed ({}); using debug output\n",
                    error.kind()
                ));
                *self = Self::Fallback;
            } else {
                return Ok(());
            }
        }
        io::stderr().flush()
    }
}

fn run_worker(receiver: Receiver<LogCommand>, mut sink: LogSink, health: &LogHealthHandle) {
    while let Ok(command) = receiver.recv() {
        match command {
            LogCommand::Write(bytes) => sink.write(&bytes, health),
            LogCommand::Flush(reply) => {
                let _ = reply.send(sink.flush(health));
            }
            LogCommand::Shutdown => {
                let _ = sink.flush(health);
                break;
            }
        }
    }
}

fn send_before(
    sender: &SyncSender<LogCommand>,
    mut command: LogCommand,
    deadline: Instant,
) -> io::Result<()> {
    loop {
        match sender.try_send(command) {
            Ok(()) => return Ok(()),
            Err(TrySendError::Full(returned)) if Instant::now() < deadline => {
                command = returned;
                thread::sleep(Duration::from_millis(5));
            }
            Err(TrySendError::Full(_)) => {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "logging control queue timed out",
                ));
            }
            Err(TrySendError::Disconnected(_)) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "logging worker stopped",
                ));
            }
        }
    }
}

struct RotatingFile {
    base_path: PathBuf,
    file: Option<File>,
    size: u64,
    max_bytes: u64,
    generations: usize,
}

impl RotatingFile {
    fn open(log_dir: &Path) -> io::Result<Self> {
        Self::open_with_limits(log_dir, MAX_LOG_BYTES, LOG_GENERATIONS)
    }

    fn open_with_limits(log_dir: &Path, max_bytes: u64, generations: usize) -> io::Result<Self> {
        if max_bytes == 0 || generations == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "log rotation limits must be non-zero",
            ));
        }
        fs::create_dir_all(log_dir)?;
        let base_path = log_dir.join(LOG_FILE_NAME);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&base_path)?;
        let size = file.metadata()?.len();
        Ok(Self {
            base_path,
            file: Some(file),
            size,
            max_bytes,
            generations,
        })
    }

    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        let incoming = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if self.size > 0 && self.size.saturating_add(incoming) > self.max_bytes {
            self.rotate()?;
        }
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("log file was not open"))?;
        file.write_all(bytes)?;
        self.size = self.size.saturating_add(incoming);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("log file was not open"))?
            .flush()
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file.take();

        for generation in (1..=self.generations).rev() {
            let destination = generation_path(&self.base_path, generation);
            remove_if_present(&destination)?;
            let source = if generation == 1 {
                self.base_path.clone()
            } else {
                generation_path(&self.base_path, generation - 1)
            };
            rename_if_present(&source, &destination)?;
        }

        self.file = Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.base_path)?,
        );
        self.size = 0;
        Ok(())
    }
}

fn generation_path(base: &Path, generation: usize) -> PathBuf {
    let mut value = base.as_os_str().to_owned();
    value.push(format!(".{generation}"));
    PathBuf::from(value)
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn rename_if_present(source: &Path, destination: &Path) -> io::Result<()> {
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn fallback_message(message: &str) {
    fallback_bytes(message.as_bytes());
}

fn fallback_bytes(bytes: &[u8]) {
    let _ = io::stderr().write_all(bytes);
    windows::debug_output(&String::from_utf8_lossy(bytes));
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde_json::Value;
    use uuid::Uuid;

    use super::{
        LOG_FILE_NAME, LogHealthHandle, LogSink, LoggingGuard, RotatingFile, generation_path,
    };

    #[test]
    fn rotates_by_size_and_keeps_configured_generations() {
        let fixture = TempFixture::new("log-rotation");
        let mut file = RotatingFile::open_with_limits(&fixture.path, 4, 2).expect("open log");
        file.write_all(b"0000").expect("write generation zero");
        file.write_all(b"1111").expect("write generation one");
        file.write_all(b"2222").expect("write generation two");
        file.write_all(b"3333").expect("write generation three");
        file.flush().expect("flush log");

        let base = fixture.path.join(LOG_FILE_NAME);
        assert_eq!(fs::read(&base).expect("read current"), b"3333");
        assert_eq!(
            fs::read(generation_path(&base, 1)).expect("read generation one"),
            b"2222"
        );
        assert_eq!(
            fs::read(generation_path(&base, 2)).expect("read generation two"),
            b"1111"
        );
        assert!(!generation_path(&base, 3).exists());
    }

    #[test]
    fn tracing_output_is_json_lines() {
        let fixture = TempFixture::new("log-json");
        let logging = LoggingGuard::initialize(&fixture.path).expect("initialize logging");
        tracing::info!(component = "test", operation_id = 7, "contract event");
        logging.flush().expect("flush logging");

        let content = fs::read_to_string(fixture.path.join(LOG_FILE_NAME)).expect("read log");
        let values: Vec<Value> = content
            .lines()
            .map(|line| serde_json::from_str(line).expect("log line must be JSON"))
            .collect();
        let contract_events: Vec<_> = values
            .iter()
            .filter(|value| value["fields"]["message"] == "contract event")
            .collect();
        assert_eq!(contract_events.len(), 1);
        assert_eq!(contract_events[0]["fields"]["component"], "test");
    }

    #[test]
    fn flush_failure_degrades_to_fallback() {
        let health = LogHealthHandle::default();
        let mut sink = LogSink::File(RotatingFile {
            base_path: PathBuf::from("not-open"),
            file: None,
            size: 0,
            max_bytes: 1,
            generations: 1,
        });

        sink.flush(&health).expect("fallback stderr flush");
        assert!(matches!(sink, LogSink::Fallback));
        let snapshot = health.snapshot();
        assert!(snapshot.degraded);
        assert_eq!(snapshot.io_errors, 1);
    }

    struct TempFixture {
        path: PathBuf,
    }

    impl TempFixture {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("nerd-{name}-{}", Uuid::new_v4()));
            fs::create_dir(&path).expect("create fixture directory");
            Self { path }
        }
    }

    impl Drop for TempFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
