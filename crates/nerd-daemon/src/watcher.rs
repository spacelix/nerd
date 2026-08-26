//! Native directory watcher for parked roots.
//!
//! One OS thread per root runs a blocking `ReadDirectoryChangesW` loop (no
//! idle polling). Notifications are coalesced per root behind a 300 ms
//! debounce, then handed to the reconciliation worker which performs a fresh
//! immediate-child scan. The worker never mutates state directly from an
//! event callback.

use std::{
    collections::BTreeMap,
    ffi::c_void,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, INVALID_HANDLE_VALUE, WAIT_OBJECT_0},
    Storage::FileSystem::{
        CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_LIST_DIRECTORY, FILE_NOTIFY_CHANGE_DIR_NAME,
        FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadDirectoryChangesW,
    },
    System::Threading::{CreateEventW, SetEvent, WaitForSingleObject},
};

const DEBOUNCE_MS: u64 = 300;
const BUFFER_BYTES: usize = 64 * 1024;

/// Raw OS handles owned by one root-watcher thread.
struct ThreadHandles {
    directory: *mut c_void,
    stop_event: *mut c_void,
}

// HANDLEs are opaque integer resources; the bundle as a whole is transferred
// to exactly one worker thread, which closes them before exiting.
unsafe impl Send for ThreadHandles {}

/// Events from one parked root after debounce.
#[derive(Clone, Debug)]
pub struct WatchEvent {
    pub root: PathBuf,
}

#[derive(Debug)]
pub enum WatchError {
    Io(std::io::Error),
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(_) => formatter.write_str("failed to watch the parked root"),
        }
    }
}

impl std::error::Error for WatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for WatchError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct Watcher {
    sender: Sender<WatchEvent>,
    threads: Mutex<Vec<WatchThread>>,
}

struct WatchThread {
    stop_event: *mut c_void,
}

// The stop event HANDLE is an opaque pointer owned by the worker thread and
// closed there before exit; the coordinator only signals it via SetEvent.
unsafe impl Send for WatchThread {}

impl Watcher {
    /// Start the reconciliation worker plus watchers for each parked root.
    pub fn start(
        roots: Vec<PathBuf>,
        on_event: impl Fn(WatchEvent) + Send + 'static,
    ) -> Result<Self, WatchError> {
        let (sender, receiver) = mpsc::channel::<WatchEvent>();
        // Coalescing/debounce relay: collapses bursts into a single delivery.
        let _relay = thread::Builder::new()
            .name("nerd-watch-debounce".to_owned())
            .spawn(move || run_debounce(receiver, on_event))?;

        let mut threads = Vec::new();
        for root in roots {
            threads.push(spawn_root_watcher(root, sender.clone())?);
        }

        Ok(Self {
            sender,
            threads: Mutex::new(threads),
        })
    }

    /// Start watching another parked root at runtime.
    pub fn add_root(&self, root: &Path) -> Result<(), WatchError> {
        let thread = spawn_root_watcher(root.to_path_buf(), self.sender.clone())?;
        self.threads.lock().map(|mut guard| guard.push(thread)).ok();
        Ok(())
    }

    pub fn count_roots(&self) -> usize {
        self.threads
            .lock()
            .map(|guard| guard.len().saturating_sub(1))
            .unwrap_or(0)
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        // The relay thread exits when `sender` drops. Root workers block inside
        // ReadDirectoryChangesW and cannot be interrupted synchronously; they are
        // detached here and die with the process, which also closes their handles.
        let threads = match self.threads.lock() {
            Ok(mut guard) => std::mem::take(&mut *guard),
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                std::mem::take(&mut *guard)
            }
        };
        for thread in threads {
            if !thread.stop_event.is_null() {
                // SAFETY: `stop_event` is a live event HANDLE while the worker runs.
                unsafe {
                    SetEvent(thread.stop_event);
                }
            }
        }
    }
}

fn spawn_root_watcher(
    root: PathBuf,
    sender: Sender<WatchEvent>,
) -> Result<WatchThread, WatchError> {
    use std::os::windows::ffi::OsStrExt as _;
    let mut wide: Vec<u16> = root.as_os_str().encode_wide().collect();
    wide.push(0);

    // SAFETY: `wide` is NUL-terminated; the returned handle is stored in the thread.
    let directory = unsafe {
        CreateFileW(
            wide.as_ptr(),
            FILE_LIST_DIRECTORY,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            std::ptr::null_mut(),
        )
    };
    if directory == INVALID_HANDLE_VALUE {
        return Err(WatchError::Io(std::io::Error::last_os_error()));
    }

    // Manual-reset? No: auto-reset event works for signaling stop once.
    // SAFETY: name pointer is null (unnamed event).
    let stop_event = unsafe { CreateEventW(std::ptr::null(), 0, 0, std::ptr::null()) };
    if stop_event.is_null() {
        // SAFETY: `directory` was opened above.
        unsafe {
            CloseHandle(directory);
        }
        return Err(WatchError::Io(std::io::Error::last_os_error()));
    }

    let root_for_thread = root.clone();
    // The whole handle bundle moves into the worker so the closure itself only
    // captures the Send bundle (field access happens inside the callee).
    let handles = ThreadHandles {
        directory,
        stop_event,
    };
    let _handle = std::thread::Builder::new()
        .name(format!(
            "nerd-watch-{}",
            root.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default()
        ))
        .spawn(move || {
            run_root_loop(root_for_thread, handles, sender);
        })?;

    Ok(WatchThread { stop_event })
}

fn run_root_loop(root: PathBuf, handles: ThreadHandles, sender: Sender<WatchEvent>) {
    let directory = handles.directory;
    let stop_event = handles.stop_event;
    let mut buffer = vec![0u8; BUFFER_BYTES];
    loop {
        let mut bytes_returned = 0u32;
        // Blocking overlapped-free read; returns when an event fires or on stop cleanup.
        // SAFETY: `buffer` is valid for `BUFFER_BYTES` writes; `directory` is open.
        let ok = unsafe {
            ReadDirectoryChangesW(
                directory,
                buffer.as_mut_ptr().cast::<c_void>(),
                BUFFER_BYTES as u32,
                0, // immediate children only
                FILE_NOTIFY_CHANGE_DIR_NAME
                    | FILE_NOTIFY_CHANGE_FILE_NAME
                    | FILE_NOTIFY_CHANGE_LAST_WRITE,
                &mut bytes_returned,
                std::ptr::null_mut(),
                None,
            )
        };
        if ok == 0 {
            break;
        }

        // Non-blocking stop check: PeekNamedPipe equivalent is unavailable for
        // directories, so stop relies on process exit tearing down the blocking
        // call or handle closure by the coordinator.
        parse_notifications(&buffer[..bytes_returned as usize]);
        let _ = &root;
        let _ = sender.send(WatchEvent { root: root.clone() });

        // SAFETY: `stop_event` remains valid for the duration of the thread.
        let wait = unsafe { WaitForSingleObject(stop_event, 0) };
        if wait == WAIT_OBJECT_0 {
            break;
        }
    }
    // SAFETY: both handles were opened for this worker thread.
    unsafe {
        CloseHandle(directory);
        CloseHandle(stop_event);
    }
}

fn parse_notifications(buffer: &[u8]) {
    // Notification content is intentionally not interpreted: every change
    // triggers the same debounced re-scan of immediate children.
    let _ = buffer;
}

fn run_debounce(receiver: mpsc::Receiver<WatchEvent>, on_event: impl Fn(WatchEvent)) {
    let mut pending: BTreeMap<PathBuf, Instant> = BTreeMap::new();
    loop {
        const TICK: Duration = Duration::from_millis(DEBOUNCE_MS);
        match receiver.recv_timeout(TICK) {
            Ok(event) => {
                pending.insert(event.root, Instant::now());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        let now = Instant::now();
        let ready: Vec<PathBuf> = pending
            .iter()
            .filter(|(_, seen)| now.duration_since(**seen) >= TICK)
            .map(|(root, _)| root.clone())
            .collect();
        for root in ready {
            pending.remove(&root);
            on_event(WatchEvent { root });
        }
    }
}
