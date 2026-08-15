use std::{
    ffi::{OsString, c_void},
    io,
    mem::size_of,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    ptr::null_mut,
};

use windows_sys::{
    Win32::{
        Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, GetLastError, HANDLE, LocalFree},
        Security::{
            Authorization::{
                ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
                SDDL_REVISION_1,
            },
            GetTokenInformation, IsValidSid, IsWellKnownSid, PSECURITY_DESCRIPTOR,
            SECURITY_ATTRIBUTES, TOKEN_ELEVATION, TOKEN_QUERY, TOKEN_USER, TokenElevation,
            TokenUser, WinLocalSystemSid,
        },
        System::{
            Com::CoTaskMemFree,
            Diagnostics::Debug::OutputDebugStringW,
            ProcessStatus::{
                K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
            },
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
        UI::Shell::{FOLDERID_LocalAppData, SHGetKnownFolderPath},
    },
    core::PWSTR,
};

use crate::ProcessSecurityError;

pub(crate) struct OwnedHandle(HANDLE);

impl OwnedHandle {
    pub(crate) fn new(handle: HANDLE) -> io::Result<Self> {
        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(handle))
        }
    }

    pub(crate) fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: `OwnedHandle` accepts one valid owned handle and closes it exactly once.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub(crate) struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

impl SecurityDescriptor {
    pub(crate) fn current_user_and_system() -> io::Result<Self> {
        let sddl = current_user_and_system_sddl()?;
        let wide = to_wide(&sddl);
        let mut descriptor = null_mut();

        // SAFETY: `wide` is NUL-terminated and both output pointers are valid for this call.
        let converted = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                null_mut(),
            )
        };
        if converted == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self(descriptor))
    }

    pub(crate) fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0.cast::<c_void>(),
            bInheritHandle: 0,
        }
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: SDDL conversion allocated this descriptor with LocalAlloc.
        unsafe {
            LocalFree(self.0.cast::<c_void>());
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessMemory {
    pub working_set_bytes: u64,
    pub peak_working_set_bytes: u64,
    pub private_usage_bytes: u64,
}

pub fn process_memory() -> io::Result<ProcessMemory> {
    let mut counters = PROCESS_MEMORY_COUNTERS_EX {
        cb: size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        ..Default::default()
    };

    // SAFETY: The pseudo-handle is always valid for the current process. The output pointer and
    // size describe a writable `PROCESS_MEMORY_COUNTERS_EX`, whose prefix matches the base type.
    let result = unsafe {
        K32GetProcessMemoryInfo(
            GetCurrentProcess(),
            (&raw mut counters).cast::<PROCESS_MEMORY_COUNTERS>(),
            size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(ProcessMemory {
        working_set_bytes: counters.WorkingSetSize as u64,
        peak_working_set_bytes: counters.PeakWorkingSetSize as u64,
        private_usage_bytes: counters.PrivateUsage as u64,
    })
}

pub(crate) fn ensure_non_elevated_current_process() -> Result<(), ProcessSecurityError> {
    let identity = current_process_identity().map_err(ProcessSecurityError::Query)?;
    classify_process_token(identity.elevated, identity.is_local_system())
}

pub(crate) fn local_app_data() -> io::Result<PathBuf> {
    let mut raw: PWSTR = null_mut();
    let folder_id = FOLDERID_LocalAppData;

    // SAFETY: The known-folder ID is valid, no token is supplied, and `raw` is a valid output.
    let result = unsafe { SHGetKnownFolderPath(&raw const folder_id, 0, null_mut(), &mut raw) };
    if result < 0 {
        return Err(io::Error::from_raw_os_error(result));
    }

    let path = if raw.is_null() {
        Err(io::Error::other("Local AppData path was empty"))
    } else {
        // SAFETY: SHGetKnownFolderPath returns a NUL-terminated UTF-16 string.
        let len = unsafe {
            let mut len = 0;
            while *raw.add(len) != 0 {
                len += 1;
            }
            len
        };
        // SAFETY: The preceding scan found the allocation's terminating NUL.
        let value = unsafe { std::slice::from_raw_parts(raw, len) };
        Ok(PathBuf::from(OsString::from_wide(value)))
    };

    // SAFETY: SHGetKnownFolderPath allocates the returned pointer with CoTaskMemAlloc.
    unsafe {
        CoTaskMemFree(raw.cast::<c_void>());
    }
    path
}

pub(crate) fn debug_output(message: &str) {
    let sanitized = message.replace('\0', "?");
    let wide = to_wide(&sanitized);
    // SAFETY: `wide` is NUL-terminated and remains alive for the duration of the call.
    unsafe {
        OutputDebugStringW(wide.as_ptr());
    }
}

fn current_user_sid() -> io::Result<String> {
    let identity = current_process_identity()?;
    let mut sid_string: PWSTR = null_mut();
    // SAFETY: The identity retains the validated SID buffer through this conversion call.
    let converted = unsafe { ConvertSidToStringSidW(identity.sid(), &mut sid_string) };
    if converted == 0 {
        return Err(io::Error::last_os_error());
    }

    // SAFETY: ConvertSidToStringSidW returns a NUL-terminated UTF-16 string.
    let len = unsafe {
        let mut len = 0;
        while *sid_string.add(len) != 0 {
            len += 1;
        }
        len
    };
    // SAFETY: The preceding scan found the allocation's terminating NUL.
    let sid_slice = unsafe { std::slice::from_raw_parts(sid_string, len) };
    let sid = String::from_utf16(sid_slice)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "current user SID was invalid"));

    // SAFETY: ConvertSidToStringSidW allocated the string with LocalAlloc.
    unsafe {
        LocalFree(sid_string.cast::<c_void>());
    }
    sid
}

fn current_process_identity() -> io::Result<TokenIdentity> {
    let mut token = null_mut();
    // SAFETY: The current-process pseudo-handle is valid and `token` is a writable output.
    let opened = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) };
    if opened == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = OwnedHandle::new(token)?;
    query_token_identity(token.as_raw())
}

fn query_token_identity(token: HANDLE) -> io::Result<TokenIdentity> {
    let mut elevation = TOKEN_ELEVATION::default();
    let mut elevation_size = 0;
    // SAFETY: `elevation` and its reported size are valid writable outputs.
    let elevation_loaded = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            (&raw mut elevation).cast::<c_void>(),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut elevation_size,
        )
    };
    if elevation_loaded == 0 {
        return Err(io::Error::last_os_error());
    }
    if elevation_size != size_of::<TOKEN_ELEVATION>() as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "token elevation response had an invalid size",
        ));
    }

    let mut required = 0;
    // SAFETY: A null data pointer with zero length is the documented size-query call.
    let size_query = unsafe { GetTokenInformation(token, TokenUser, null_mut(), 0, &mut required) };
    // SAFETY: This immediately follows the token size query.
    let size_error = unsafe { GetLastError() };
    if size_query != 0
        || size_error != ERROR_INSUFFICIENT_BUFFER
        || required < size_of::<TOKEN_USER>() as u32
    {
        return if size_error == 0 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "token user size query returned invalid metadata",
            ))
        } else {
            Err(io::Error::from_raw_os_error(size_error as i32))
        };
    }

    let word_size = size_of::<usize>();
    let word_count = (required as usize).div_ceil(word_size);
    let mut buffer = vec![0usize; word_count];
    // SAFETY: The usize-backed buffer has sufficient size and alignment for `TOKEN_USER`.
    let loaded = unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast::<c_void>(),
            required,
            &mut required,
        )
    };
    if loaded == 0 {
        return Err(io::Error::last_os_error());
    }

    let identity = TokenIdentity {
        buffer,
        elevated: elevation.TokenIsElevated != 0,
    };
    // SAFETY: The SID pointer is backed by the retained token buffer.
    if identity.sid().is_null() || unsafe { IsValidSid(identity.sid()) } == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "token user SID was invalid",
        ));
    }
    Ok(identity)
}

struct TokenIdentity {
    buffer: Vec<usize>,
    elevated: bool,
}

impl TokenIdentity {
    fn sid(&self) -> windows_sys::Win32::Security::PSID {
        // SAFETY: `query_token_identity` initialized TOKEN_USER at this aligned buffer address.
        unsafe { (*self.buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }

    fn is_local_system(&self) -> bool {
        // SAFETY: `query_token_identity` validated this SID before constructing the identity.
        unsafe { IsWellKnownSid(self.sid(), WinLocalSystemSid) != 0 }
    }
}

fn classify_process_token(elevated: bool, local_system: bool) -> Result<(), ProcessSecurityError> {
    if local_system {
        Err(ProcessSecurityError::LocalSystem)
    } else if elevated {
        Err(ProcessSecurityError::Elevated)
    } else {
        Ok(())
    }
}

fn current_user_and_system_sddl() -> io::Result<String> {
    Ok(format!("D:P(A;;GA;;;SY)(A;;GA;;;{})", current_user_sid()?))
}

pub(crate) fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        classify_process_token, current_user_and_system_sddl, local_app_data, process_memory,
    };
    use crate::ProcessSecurityError;

    #[test]
    fn security_policy_allows_only_system_and_current_user() {
        let sddl = current_user_and_system_sddl().expect("build SDDL");
        assert!(sddl.starts_with("D:P"));
        assert_eq!(sddl.matches("(A;;GA;;;").count(), 2);
        assert!(sddl.contains("(A;;GA;;;SY)"));
        assert!(sddl.contains("(A;;GA;;;S-1-5-"));
        assert!(!sddl.contains(";;;WD)"));
        assert!(!sddl.contains(";;;AN)"));
        assert!(!sddl.contains(";;;BA)"));
    }

    #[test]
    fn known_folder_and_process_metrics_are_available() {
        let path = local_app_data().expect("resolve Local AppData");
        assert!(path.is_absolute());

        let memory = process_memory().expect("read process memory");
        assert!(memory.working_set_bytes > 0);
        assert!(memory.peak_working_set_bytes >= memory.working_set_bytes);
        assert!(memory.private_usage_bytes > 0);
    }

    #[test]
    fn elevated_and_system_tokens_are_rejected() {
        assert!(classify_process_token(false, false).is_ok());
        assert!(matches!(
            classify_process_token(true, false),
            Err(ProcessSecurityError::Elevated)
        ));
        assert!(matches!(
            classify_process_token(false, true),
            Err(ProcessSecurityError::LocalSystem)
        ));
        assert!(matches!(
            classify_process_token(true, true),
            Err(ProcessSecurityError::LocalSystem)
        ));
    }
}
