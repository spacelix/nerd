//! Supported-location preflight. Runs before registration, watcher setup,
//! trust binding, or execution so unsupported paths fail early.

use std::{fmt, path::Path};

#[derive(Debug)]
pub enum LocationError {
    RelativePath,
    UncPath,
    RemovableOrNetworkDrive,
    LinkEscape(String),
    CloudSyncFolder,
    UnsupportedPath(String),
}

impl fmt::Display for LocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RelativePath => formatter.write_str("path must be absolute"),
            Self::UncPath => formatter.write_str("UNC and mapped network drives are unsupported"),
            Self::RemovableOrNetworkDrive => {
                formatter.write_str("only local fixed drives are supported")
            }
            Self::LinkEscape(detail) => write!(
                formatter,
                "directory is a link; escape protection requires the real directory: {detail}"
            ),
            Self::CloudSyncFolder => {
                formatter.write_str("OneDrive or cloud-sync-controlled folders are unsupported")
            }
            Self::UnsupportedPath(detail) => write!(formatter, "unsupported location: {detail}"),
        }
    }
}

impl std::error::Error for LocationError {}

/// Validate that `path` is a supported project location.
pub fn preflight(path: &Path) -> Result<(), LocationError> {
    use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
    use windows_sys::Win32::System::WindowsProgramming::{
        DRIVE_FIXED, DRIVE_REMOTE, DRIVE_REMOVABLE,
    };

    if !path.is_absolute() {
        return Err(LocationError::RelativePath);
    }
    let text = path.to_string_lossy();
    // Strip the Windows verbatim prefix ("\\?\") that Path::canonicalize
    // emits so normal and verbatim forms share one code path.
    let mut normalized = text.replace('/', "\\");
    if let Some(stripped) = normalized.clone().strip_prefix(r"\\?\") {
        normalized = stripped.to_owned();
    }
    if normalized.starts_with("\\\\") || normalized.starts_with("//") {
        return Err(LocationError::UncPath);
    }

    // "C:\" form for the API call.
    let wide_drive: Vec<u16> = format!("{}\\", &normalized[..3])
        .encode_utf16()
        .chain([0])
        .collect();

    // SAFETY: `wide_drive` is NUL-terminated for this call.
    let drive_type = unsafe { GetDriveTypeW(wide_drive.as_ptr()) };
    if drive_type == DRIVE_REMOTE || drive_type == DRIVE_REMOVABLE || drive_type == 0 {
        return Err(LocationError::RemovableOrNetworkDrive);
    }
    let _ = DRIVE_FIXED;

    // Reject directories that are reparse points (symlinks/junctions): Nerd
    // registers only real directories to avoid link escapes.
    if is_reparse_point(path) {
        return Err(LocationError::LinkEscape(text.into_owned()));
    }

    // OneDrive / known cloud-sync roots are never supported.
    for variable in ["OneDrive", "OneDriveCommercial", "OneDriveConsumer"] {
        if let Ok(root) = std::env::var(variable)
            && normalized
                .to_ascii_lowercase()
                .starts_with(root.to_ascii_lowercase().as_str())
        {
            return Err(LocationError::CloudSyncFolder);
        }
    }
    if normalized.contains("\\wsl$") || normalized.contains("\\wsl.localhost") {
        return Err(LocationError::UnsupportedPath(normalized));
    }
    Ok(())
}

fn is_reparse_point(path: &Path) -> bool {
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, GetFileAttributesW, OPEN_EXISTING,
    };
    let wide = path_wide(path);
    // SAFETY: `wide` is NUL-terminated for the call.
    let attributes = unsafe { GetFileAttributesW(wide.as_ptr()) };
    if attributes == u32::MAX {
        return false;
    }
    if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return true;
    }
    // SAFETY: `wide` outlives the handle creation below; the handle closes once.
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
    if handle.is_null() || handle == usize::MAX as *mut core::ffi::c_void {
        return false;
    }
    // SAFETY: the opened handle must be closed exactly once.
    unsafe {
        windows_sys::Win32::Foundation::CloseHandle(handle);
    }
    false
}

fn path_wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt as _;
    let mut value: Vec<u16> = path.as_os_str().encode_wide().collect();
    value.push(0);
    value
}
