//! Windows Job Object process-tree management and graceful/forced shutdown.

use std::{ffi::c_void, fmt, io, mem::size_of, ptr::null_mut, time::Duration};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::{
        Console::{CTRL_BREAK_EVENT, GenerateConsoleCtrlEvent},
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JobObjectExtendedLimitInformation, SetInformationJobObject, TerminateJobObject,
        },
        Threading::WaitForSingleObject,
    },
};

const GRACEFUL_GRACE_MS: u32 = 5000;
const WAIT_OBJECT_0: u32 = 0;

#[derive(Debug)]
pub enum JobError {
    Create(io::Error),
    Configure(io::Error),
    Assign(io::Error),
}

impl fmt::Display for JobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create(_) => formatter.write_str("failed to create the process job"),
            Self::Configure(_) => formatter.write_str("failed to configure the process job"),
            Self::Assign(_) => formatter.write_str("failed to assign the process to its job"),
        }
    }
}

impl std::error::Error for JobError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Create(error) | Self::Configure(error) | Self::Assign(error) => Some(error),
        }
    }
}

/// A Job Object that kills its descendants when closed. Owned by the
/// supervisor; dropped exactly once.
pub struct ProcessJob {
    handle: HANDLE,
}

// HANDLE is an opaque integer resource; ProcessJob is owned by the supervisor
// and used from a single task at a time.
unsafe impl Send for ProcessJob {}

impl ProcessJob {
    pub fn create() -> Result<Self, JobError> {
        // SAFETY: null attributes/name create an unnamed job object.
        let handle = unsafe { CreateJobObjectW(null_mut(), null_mut()) };
        if handle.is_null() {
            return Err(JobError::Create(io::Error::last_os_error()));
        }
        let job = Self { handle };

        // KILL_ON_JOB_CLOSE guarantees no orphans survive job shutdown.
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `info` describes a valid JOBOBJECT_EXTENDED_LIMIT_INFORMATION.
        let ok = unsafe {
            SetInformationJobObject(
                job.handle,
                JobObjectExtendedLimitInformation,
                (&raw const info).cast::<c_void>(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == 0 {
            return Err(JobError::Configure(io::Error::last_os_error()));
        }
        Ok(job)
    }

    /// Assign a freshly spawned process to the job.
    ///
    /// # Safety
    /// `process` must be an open handle owned by the caller and must outlive
    /// this call.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub unsafe fn assign(&self, process: HANDLE) -> Result<(), JobError> {
        // SAFETY: `process` is an open handle owned by the caller.
        let ok = unsafe { AssignProcessToJobObject(self.handle, process) };
        if ok == 0 {
            return Err(JobError::Assign(io::Error::last_os_error()));
        }
        Ok(())
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn contains(&self, process: HANDLE) -> io::Result<bool> {
        let mut result = 0i32;
        // SAFETY: `process` is open and `result` is a writable BOOL.
        let ok = unsafe { IsProcessInJob(process, self.handle, &mut result) };
        if ok == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(result != 0)
        }
    }

    /// Stop the tree: graceful CTRL_BREAK to the process group (when the
    /// daemon shares a console), then force-terminate the job after the grace
    /// window regardless.
    pub fn stop(&self, process_group_id: u32) -> Result<(), io::Error> {
        // SAFETY: CTRL_BREAK targets the created process group; a group with no
        // shared console fails benignly and we fall through to force.
        unsafe {
            GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, process_group_id);
        }
        let deadline = std::time::Instant::now() + Duration::from_millis(GRACEFUL_GRACE_MS.into());
        loop {
            // SAFETY: `process_group_id` is only a group id, not a HANDLE;
            // polling the job completion is done via TerminateJobObject below.
            if std::time::Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        // SAFETY: the job handle is valid and force-kills the whole tree.
        let ok = unsafe { TerminateJobObject(self.handle, 1) };
        if ok == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for ProcessJob {
    fn drop(&mut self) {
        // SAFETY: `self.handle` is the owning handle and is closed exactly once.
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

/// Open a process handle with the rights needed to assign it to a job.
pub fn open_process(pid: u32) -> Option<HANDLE> {
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
    };
    // SAFETY: OpenProcess returns an owned handle or null.
    let handle = unsafe { OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid) };
    if handle.is_null() { None } else { Some(handle) }
}

/// Block until a process handle signals (process exit) with a timeout.
///
/// # Safety
/// `process` must be an open handle owned by the caller.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn wait_exit(process: HANDLE, timeout: Duration) -> io::Result<()> {
    let millis = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    // SAFETY: `process` is an open process handle.
    let wait = unsafe { WaitForSingleObject(process, millis) };
    if wait == WAIT_OBJECT_0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

// Re-exported for the supervisor to pass through creation flags.
pub use windows_sys::Win32::System::Threading::{
    CREATE_NEW_PROCESS_GROUP as NEW_PROCESS_GROUP_FLAG, PROCESS_CREATION_FLAGS,
};
