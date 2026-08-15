use std::{fmt, io, thread::ThreadId};

use windows_sys::Win32::{
    Foundation::{
        ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_INVALID_HANDLE, GetLastError,
        WAIT_ABANDONED, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    System::Threading::{CreateMutexExW, MUTEX_ALL_ACCESS, ReleaseMutex, WaitForSingleObject},
};

use crate::windows::{OwnedHandle, SecurityDescriptor, to_wide};

const INSTANCE_MUTEX_NAME: &str = r"Global\Nerd.Daemon.6f843fb5-1bc8-4d47-a038-751c7c218fe8";

pub struct InstanceGuard {
    handle: OwnedHandle,
    owner_thread: ThreadId,
    abandoned_predecessor: bool,
}

impl InstanceGuard {
    pub(crate) fn acquire(security: &SecurityDescriptor) -> Result<Self, InstanceGuardError> {
        Self::acquire_named(security, INSTANCE_MUTEX_NAME)
    }

    fn acquire_named(
        security: &SecurityDescriptor,
        mutex_name: &str,
    ) -> Result<Self, InstanceGuardError> {
        let mut attributes = security.attributes();
        let name = to_wide(mutex_name);

        // SAFETY: Attributes and name are valid for the duration of the call; handle inheritance
        // is disabled. Ownership is acquired separately to avoid initial-owner races.
        let raw =
            unsafe { CreateMutexExW(&raw mut attributes, name.as_ptr(), 0, MUTEX_ALL_ACCESS) };
        if raw.is_null() {
            // SAFETY: This immediately follows the failed Win32 call.
            let code = unsafe { GetLastError() };
            return match code {
                ERROR_ACCESS_DENIED | ERROR_ALREADY_EXISTS | ERROR_INVALID_HANDLE => {
                    Err(InstanceGuardError::MachineConflict)
                }
                _ => Err(InstanceGuardError::Os(io::Error::from_raw_os_error(
                    code as i32,
                ))),
            };
        }
        let handle = OwnedHandle::new(raw).map_err(InstanceGuardError::Os)?;

        // SAFETY: The handle refers to a mutex and zero timeout performs a non-blocking acquire.
        let wait_result = unsafe { WaitForSingleObject(handle.as_raw(), 0) };
        let abandoned_predecessor = match wait_result {
            WAIT_OBJECT_0 => false,
            WAIT_ABANDONED => true,
            WAIT_TIMEOUT => return Err(InstanceGuardError::AlreadyRunning),
            WAIT_FAILED => return Err(InstanceGuardError::Os(io::Error::last_os_error())),
            other => {
                return Err(InstanceGuardError::Os(io::Error::other(format!(
                    "unexpected mutex wait result {other}"
                ))));
            }
        };

        Ok(Self {
            handle,
            owner_thread: std::thread::current().id(),
            abandoned_predecessor,
        })
    }

    pub fn abandoned_predecessor(&self) -> bool {
        self.abandoned_predecessor
    }
}

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        debug_assert_eq!(self.owner_thread, std::thread::current().id());
        // SAFETY: The current thread acquired this mutex once and releases it once before close.
        unsafe {
            ReleaseMutex(self.handle.as_raw());
        }
    }
}

#[derive(Debug)]
pub enum InstanceGuardError {
    AlreadyRunning,
    MachineConflict,
    Os(io::Error),
}

impl fmt::Display for InstanceGuardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRunning => formatter.write_str("Nerd daemon is already running"),
            Self::MachineConflict => {
                formatter.write_str("another Windows session owns the Nerd daemon instance")
            }
            Self::Os(error) => write!(formatter, "instance guard failed: {error}"),
        }
    }
}

impl std::error::Error for InstanceGuardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Os(error) => Some(error),
            Self::AlreadyRunning | Self::MachineConflict => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use uuid::Uuid;

    use super::{InstanceGuard, InstanceGuardError};
    use crate::windows::SecurityDescriptor;

    #[test]
    fn guard_blocks_another_thread_and_is_released_on_drop() {
        let name = format!(r"Global\Nerd.Test.{}", Uuid::new_v4());
        let security = SecurityDescriptor::current_user_and_system().expect("security descriptor");
        let first = InstanceGuard::acquire_named(&security, &name).expect("first guard");

        let competing_name = name.clone();
        let competing = thread::spawn(move || {
            let security =
                SecurityDescriptor::current_user_and_system().expect("security descriptor");
            matches!(
                InstanceGuard::acquire_named(&security, &competing_name),
                Err(InstanceGuardError::AlreadyRunning)
            )
        })
        .join()
        .expect("competing thread");
        assert!(competing);

        drop(first);
        let replacement =
            InstanceGuard::acquire_named(&security, &name).expect("replacement guard");
        drop(replacement);
    }
}
