//! Stable directory identity: (NTFS volume serial, file index). Detects a
//! replaced directory (same path, new identity) versus a rename/move (same
//! identity, different path).

use std::{fmt, io, path::Path};

use windows_sys::Win32::{
    Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DirectoryIdentity {
    pub volume_serial: u64,
    pub file_id: u64,
}

#[derive(Debug)]
pub enum IdentityError {
    Io(io::Error),
}

impl fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => formatter.write_str("failed to read stable directory identity"),
        }
    }
}

impl std::error::Error for IdentityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
        }
    }
}

impl From<io::Error> for IdentityError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Read the stable identity of an existing directory. Fails with a typed
/// error when the filesystem cannot provide one.
pub fn identify(path: &Path) -> Result<DirectoryIdentity, IdentityError> {
    let wide = windows_wide(path);
    // SAFETY: `wide` is NUL-terminated and outlives the creation call; the
    // handle is closed exactly once below.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(IdentityError::Io(io::Error::last_os_error()));
    }

    let mut info = BY_HANDLE_FILE_INFORMATION {
        ..Default::default()
    };
    // SAFETY: `info` is a valid writable output for this call.
    let ok = unsafe { GetFileInformationByHandle(handle, &mut info) };
    // SAFETY: the handle was opened above and must be closed exactly once.
    unsafe {
        CloseHandle(handle);
    }
    if ok == 0 {
        return Err(IdentityError::Io(io::Error::last_os_error()));
    }

    Ok(DirectoryIdentity {
        volume_serial: u64::from(info.dwVolumeSerialNumber),
        file_id: (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
    })
}

fn windows_wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt as _;
    let mut value: Vec<u16> = path.as_os_str().encode_wide().collect();
    value.push(0);
    value
}
