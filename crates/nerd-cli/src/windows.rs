use std::{
    ffi::{OsString, c_void},
    fmt, fs, io,
    mem::size_of,
    os::windows::{ffi::OsStringExt, io::AsRawHandle},
    path::{Path, PathBuf},
    ptr::null_mut,
};

use tokio::net::windows::named_pipe::NamedPipeClient;
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, GetLastError, HANDLE},
    Security::{
        EqualSid, GetTokenInformation, IsValidSid, IsWellKnownSid, TOKEN_ELEVATION, TOKEN_QUERY,
        TOKEN_USER, TokenElevation, TokenUser, WinLocalSystemSid,
    },
    System::{
        Pipes::GetNamedPipeServerProcessId,
        Threading::{
            GetCurrentProcess, GetProcessId, OpenProcess, OpenProcessToken,
            PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
        },
    },
};

const MAX_IMAGE_PATH_UNITS: usize = 32_768;

pub(crate) struct VerifiedServer {
    _process: OwnedHandle,
}

pub(crate) fn verify_server(client: &NamedPipeClient) -> Result<VerifiedServer, PeerIdentityError> {
    let expected_image = expected_daemon_image().map_err(PeerIdentityError::Query)?;
    verify_server_image(client, expected_image)
}

fn verify_server_image(
    client: &NamedPipeClient,
    expected_image: PathBuf,
) -> Result<VerifiedServer, PeerIdentityError> {
    let current = current_process_identity().map_err(PeerIdentityError::Query)?;
    classify_token(current.elevated, current.is_local_system(), "CLI")?;

    let pipe = client.as_raw_handle().cast::<c_void>();
    let first_pid = pipe_server_pid(pipe).map_err(PeerIdentityError::Query)?;
    // SAFETY: PID is nonzero, access is query-only, and handle inheritance is disabled.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, first_pid) };
    let process = OwnedHandle::new(process).map_err(PeerIdentityError::Query)?;

    // SAFETY: `process` is a valid process handle retained for the full IPC exchange.
    if unsafe { GetProcessId(process.as_raw()) } != first_pid {
        return Err(PeerIdentityError::ProcessChanged);
    }
    if pipe_server_pid(pipe).map_err(PeerIdentityError::Query)? != first_pid {
        return Err(PeerIdentityError::ProcessChanged);
    }

    let server = process_identity(process.as_raw()).map_err(PeerIdentityError::Query)?;
    classify_token(server.elevated, server.is_local_system(), "daemon")?;
    // SAFETY: Both identities retain buffers containing SIDs validated by `IsValidSid`.
    if unsafe { EqualSid(current.sid(), server.sid()) } == 0 {
        return Err(PeerIdentityError::WrongUser);
    }

    let observed_image =
        fs::canonicalize(query_process_image(process.as_raw()).map_err(PeerIdentityError::Query)?)
            .map_err(PeerIdentityError::Query)?;
    ensure_expected_image(&observed_image, &expected_image)?;
    if pipe_server_pid(pipe).map_err(PeerIdentityError::Query)? != first_pid {
        return Err(PeerIdentityError::ProcessChanged);
    }

    Ok(VerifiedServer { _process: process })
}

fn ensure_expected_image(observed: &Path, expected: &Path) -> Result<(), PeerIdentityError> {
    if observed == expected {
        Ok(())
    } else {
        Err(PeerIdentityError::WrongImage)
    }
}

fn expected_daemon_image() -> io::Result<PathBuf> {
    let cli = fs::canonicalize(std::env::current_exe()?)?;
    let directory = cli
        .parent()
        .ok_or_else(|| io::Error::other("CLI executable has no parent directory"))?;
    fs::canonicalize(directory.join("nerd-daemon.exe"))
}

fn pipe_server_pid(pipe: HANDLE) -> io::Result<u32> {
    let mut process_id = 0;
    // SAFETY: The handle is borrowed from a live Tokio named-pipe client and output is writable.
    let result = unsafe { GetNamedPipeServerProcessId(pipe, &mut process_id) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    if process_id == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "named-pipe server returned an invalid PID",
        ));
    }
    Ok(process_id)
}

fn current_process_identity() -> io::Result<TokenIdentity> {
    // SAFETY: The current-process pseudo-handle is always valid and must not be closed.
    process_identity(unsafe { GetCurrentProcess() })
}

fn process_identity(process: HANDLE) -> io::Result<TokenIdentity> {
    let mut token = null_mut();
    // SAFETY: Process handle is queryable and `token` is a valid output pointer.
    let opened = unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) };
    if opened == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = OwnedHandle::new(token)?;
    query_token_identity(token.as_raw())
}

fn query_token_identity(token: HANDLE) -> io::Result<TokenIdentity> {
    let mut elevation = TOKEN_ELEVATION::default();
    let mut elevation_size = 0;
    // SAFETY: `elevation` and reported size are valid writable outputs.
    let loaded = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            (&raw mut elevation).cast::<c_void>(),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut elevation_size,
        )
    };
    if loaded == 0 {
        return Err(io::Error::last_os_error());
    }
    if elevation_size != size_of::<TOKEN_ELEVATION>() as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "token elevation response had an invalid size",
        ));
    }

    let mut required = 0;
    // SAFETY: Null buffer and zero size perform the documented TokenUser size query.
    let queried = unsafe { GetTokenInformation(token, TokenUser, null_mut(), 0, &mut required) };
    // SAFETY: This immediately follows the size query.
    let query_error = unsafe { GetLastError() };
    if queried != 0
        || query_error != ERROR_INSUFFICIENT_BUFFER
        || required < size_of::<TOKEN_USER>() as u32
    {
        return if query_error == 0 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "token user size query returned invalid metadata",
            ))
        } else {
            Err(io::Error::from_raw_os_error(query_error as i32))
        };
    }

    let mut buffer = vec![0_usize; (required as usize).div_ceil(size_of::<usize>())];
    // SAFETY: The usize-backed buffer has sufficient size and alignment for TOKEN_USER.
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
    // SAFETY: SID is backed by the retained TokenUser buffer.
    if identity.sid().is_null() || unsafe { IsValidSid(identity.sid()) } == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "token user SID was invalid",
        ));
    }
    Ok(identity)
}

fn query_process_image(process: HANDLE) -> io::Result<PathBuf> {
    let mut buffer = vec![0_u16; MAX_IMAGE_PATH_UNITS];
    let mut length = u32::try_from(buffer.len()).expect("fixed image buffer fits u32");
    // SAFETY: Process handle has limited-query access and buffer/length are valid outputs.
    let result =
        unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    let length = usize::try_from(length)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "image path length overflow"))?;
    if length == 0 || length > buffer.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process image path had an invalid length",
        ));
    }
    Ok(PathBuf::from(OsString::from_wide(&buffer[..length])))
}

struct TokenIdentity {
    buffer: Vec<usize>,
    elevated: bool,
}

impl TokenIdentity {
    fn sid(&self) -> windows_sys::Win32::Security::PSID {
        // SAFETY: TokenUser was initialized at the aligned start of this retained buffer.
        unsafe { (*self.buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }

    fn is_local_system(&self) -> bool {
        // SAFETY: The SID was validated before the identity was returned.
        unsafe { IsWellKnownSid(self.sid(), WinLocalSystemSid) != 0 }
    }
}

fn classify_token(
    elevated: bool,
    local_system: bool,
    subject: &'static str,
) -> Result<(), PeerIdentityError> {
    if local_system {
        Err(PeerIdentityError::LocalSystem(subject))
    } else if elevated {
        Err(PeerIdentityError::Elevated(subject))
    } else {
        Ok(())
    }
}

struct OwnedHandle(HANDLE);

impl OwnedHandle {
    fn new(handle: HANDLE) -> io::Result<Self> {
        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(handle))
        }
    }

    fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: This guard owns one valid handle and closes it exactly once.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[derive(Debug)]
pub(crate) enum PeerIdentityError {
    Elevated(&'static str),
    LocalSystem(&'static str),
    WrongUser,
    WrongImage,
    ProcessChanged,
    Query(io::Error),
}

impl fmt::Display for PeerIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Elevated(subject) => write!(formatter, "{subject} process is elevated"),
            Self::LocalSystem(subject) => write!(formatter, "{subject} process uses LocalSystem"),
            Self::WrongUser => formatter.write_str("daemon belongs to a different Windows user"),
            Self::WrongImage => formatter.write_str("daemon executable path is not trusted"),
            Self::ProcessChanged => formatter.write_str("daemon process identity changed"),
            Self::Query(_) => formatter.write_str("daemon process identity could not be verified"),
        }
    }
}

impl std::error::Error for PeerIdentityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Query(error) => Some(error),
            Self::Elevated(_)
            | Self::LocalSystem(_)
            | Self::WrongUser
            | Self::WrongImage
            | Self::ProcessChanged => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};
    use uuid::Uuid;

    use super::{PeerIdentityError, classify_token, ensure_expected_image, verify_server_image};

    #[test]
    fn elevated_and_system_tokens_fail_closed() {
        assert!(classify_token(false, false, "test").is_ok());
        assert!(matches!(
            classify_token(true, false, "test"),
            Err(PeerIdentityError::Elevated("test"))
        ));
        assert!(matches!(
            classify_token(false, true, "test"),
            Err(PeerIdentityError::LocalSystem("test"))
        ));
    }

    #[test]
    fn mismatched_image_path_is_rejected() {
        assert!(matches!(
            ensure_expected_image(
                std::path::Path::new(r"C:\Nerd\nerd-daemon.exe"),
                std::path::Path::new(r"C:\Other\nerd-daemon.exe"),
            ),
            Err(PeerIdentityError::WrongImage)
        ));
    }

    #[test]
    fn pipe_created_by_wrong_executable_is_rejected() {
        let fixture = TempFixture::new();
        let expected_file = fixture.path.join("expected-daemon.exe");
        fs::write(&expected_file, b"not an executable").expect("write expected image fixture");
        let expected_file = fs::canonicalize(expected_file).expect("canonical expected image");
        let pipe_name = format!(r"\\.\pipe\Nerd.Cli.Identity.Test.{}", Uuid::new_v4());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        runtime.block_on(async {
            let server = ServerOptions::new()
                .first_pipe_instance(true)
                .create(&pipe_name)
                .expect("create fake server");
            let client = ClientOptions::new()
                .open(&pipe_name)
                .expect("connect fake client");
            server.connect().await.expect("accept fake client");
            let result = verify_server_image(&client, expected_file);
            assert!(matches!(
                result,
                Err(PeerIdentityError::WrongImage | PeerIdentityError::Elevated("CLI"))
            ));
        });
    }

    struct TempFixture {
        path: PathBuf,
    }

    impl TempFixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("nerd-cli-peer-{}", Uuid::new_v4()));
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
