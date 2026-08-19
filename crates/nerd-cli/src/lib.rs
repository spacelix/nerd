use std::{ffi::OsString, fmt, io, time::Duration};

use nerd_core::{
    APPLICATION_VERSION, IPC_PROTOCOL_VERSION, PIPE_NAME,
    codec::{FRAME_PREFIX_BYTES, MAX_FRAME_BYTES, decode_payload, encode_frame},
    ipc::{
        ClientKind, ErrorCode, ErrorResponse, HandshakeRequest, HealthStatus, NetworkRepairRequest,
        NetworkSetupRequest, NetworkStatusRequest, NetworkUninstallRequest, Request,
        RequestEnvelope, Response, ResponseEnvelope, RuntimeInstallRequest, RuntimeListRequest,
        RuntimeRemoveRequest, RuntimeSetDefaultRequest, StatusRequest, StatusResponse,
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient},
    time::{Instant, sleep, timeout},
};
use uuid::Uuid;

mod windows;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
const RETRY_INTERVAL: Duration = Duration::from_millis(50);
const ERROR_PIPE_BUSY: i32 = 231;

pub fn run_from_env() -> i32 {
    match parse_command(std::env::args_os().skip(1)) {
        Ok(Command::Version) => {
            println!("nerd {APPLICATION_VERSION}");
            0
        }
        Ok(Command::Status) => match run_status() {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("nerd: {error}");
                error.exit_code()
            }
        },
        Ok(Command::Network { action }) => match run_network(action) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("nerd: {error}");
                error.exit_code()
            }
        },
        Ok(Command::Runtime { action, arg }) => match run_runtime(action, arg) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("nerd: {error}");
                error.exit_code()
            }
        },
        Err(error) => {
            eprintln!("nerd: {error}");
            eprintln!(
                "usage: nerd <status|network <setup|uninstall|repair|status>|runtime <install <ver>|list|remove <id>|set-default <ver>>|--version>"
            );
            error.exit_code()
        }
    }
}

fn run_runtime(action: RuntimeAction, arg: Option<String>) -> Result<(), CliError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(CliError::Runtime)?;
    runtime.block_on(async move {
        let mut connection = connect().await?;
        let request = match action {
            RuntimeAction::Install => {
                let version = arg.ok_or(CliError::Usage)?;
                Request::RuntimeInstall(RuntimeInstallRequest { version })
            }
            RuntimeAction::List => Request::RuntimeList(RuntimeListRequest {}),
            RuntimeAction::Remove => {
                let id = arg.ok_or(CliError::Usage)?;
                let runtime_id = uuid::Uuid::parse_str(&id).map_err(|_| CliError::Usage)?;
                Request::RuntimeRemove(RuntimeRemoveRequest { runtime_id })
            }
            RuntimeAction::SetDefault => {
                let version = arg.ok_or(CliError::Usage)?;
                Request::RuntimeSetDefault(RuntimeSetDefaultRequest { version })
            }
        };
        let response = timeout(
            REQUEST_TIMEOUT,
            exchange_network(&mut connection.client, request),
        )
        .await
        .map_err(|_| CliError::Timeout)??;
        print_runtime(&response);
        Ok(())
    })
}

fn print_runtime(response: &Response) {
    match response {
        Response::RuntimeInstall(result) => {
            if result.installed {
                println!("Installed Node {}", result.version);
            } else {
                println!("Node {} already installed", result.version);
            }
        }
        Response::RuntimeList(result) => {
            if result.runtimes.is_empty() {
                println!("No runtimes installed.");
            }
            for runtime in &result.runtimes {
                println!(
                    "{:<12} {:<9} {:<8} {}",
                    runtime.kind_str(),
                    runtime.version,
                    runtime.status_str(),
                    runtime.executable_path
                );
            }
        }
        Response::RuntimeRemove(result) => {
            if result.removed {
                println!("Removed runtime.");
            } else {
                println!("Runtime not found or not managed.");
            }
        }
        Response::RuntimeSetDefault(result) => {
            println!("Default Node set to {}", result.version);
        }
        Response::Error(error) => {
            println!("Runtime request rejected: {}", error.message);
        }
        _ => println!("Unexpected runtime response."),
    }
}

fn run_network(action: NetworkAction) -> Result<(), CliError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(CliError::Runtime)?;
    runtime.block_on(async move {
        let mut connection = connect().await?;
        let (_, request) = match action {
            NetworkAction::Setup => (
                Uuid::new_v4(),
                Request::NetworkSetup(NetworkSetupRequest {}),
            ),
            NetworkAction::Uninstall => (
                Uuid::new_v4(),
                Request::NetworkUninstall(NetworkUninstallRequest {}),
            ),
            NetworkAction::Repair => (
                Uuid::new_v4(),
                Request::NetworkRepair(NetworkRepairRequest {}),
            ),
            NetworkAction::Status => (
                Uuid::new_v4(),
                Request::NetworkStatus(NetworkStatusRequest {}),
            ),
        };
        let response = timeout(
            REQUEST_TIMEOUT,
            exchange_network(&mut connection.client, request),
        )
        .await
        .map_err(|_| CliError::Timeout)??;
        print_network(&response);
        Ok(())
    })
}

async fn exchange_network(
    client: &mut NamedPipeClient,
    request: Request,
) -> Result<Response, CliError> {
    let handshake_id = Uuid::new_v4();
    write_request(
        client,
        &RequestEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id: handshake_id,
            request: Request::Handshake(HandshakeRequest {
                client_kind: ClientKind::Cli,
                client_version: APPLICATION_VERSION.to_owned(),
                minimum_protocol_version: IPC_PROTOCOL_VERSION,
                maximum_protocol_version: IPC_PROTOCOL_VERSION,
            }),
        },
    )
    .await?;
    let handshake = read_response(client).await?;
    ensure_response_id(&handshake, handshake_id)?;
    match handshake.response {
        Response::Handshake(response)
            if response.selected_protocol_version == IPC_PROTOCOL_VERSION => {}
        Response::Error(error) => return Err(map_server_error(error)),
        _ => return Err(CliError::InvalidResponse),
    }

    let request_id = Uuid::new_v4();
    write_request(
        client,
        &RequestEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id,
            request,
        },
    )
    .await?;
    let response = read_response(client).await?;
    ensure_response_id(&response, request_id)?;
    Ok(response.response)
}

fn print_network(response: &Response) {
    match response {
        Response::NetworkSetup(setup) => {
            if setup.success {
                println!("Network setup complete.");
                if let Some(rule) = &setup.nrpt_rule_name {
                    println!("NRPT rule: {rule}");
                }
                if let Some(fingerprint) = &setup.ca_fingerprint {
                    println!("CA fingerprint: {fingerprint}");
                }
            } else {
                println!("Network setup failed (rolled back).");
            }
        }
        Response::NetworkUninstall(uninstall) => {
            println!("Network uninstall complete.");
            println!("Removed NRPT rule: {}", uninstall.removed_nrpt_rule);
            println!("Removed CA: {}", uninstall.removed_ca);
            println!(
                "Preserved unrelated NRPT rules: {}",
                uninstall.preserved_unrelated_rules
            );
        }
        Response::NetworkRepair(repair) => {
            println!("Network repair: {}", repair.action);
        }
        Response::NetworkStatus(status) => {
            println!("DNS listener active: {}", status.dns_listener_active);
            println!("NRPT rule present: {}", status.nrpt_rule_present);
            println!("CA present: {}", status.ca_present);
            if let Some(conflict) = &status.port_53_conflict {
                println!(
                    "Port 53 conflict: PID {} owns {}:{}",
                    conflict.owning_process_id, conflict.protocol, conflict.port
                );
            }
            if let Some(conflict) = &status.port_80_conflict {
                println!(
                    "Port 80 conflict: PID {} owns {}:{}",
                    conflict.owning_process_id, conflict.protocol, conflict.port
                );
            }
            if let Some(conflict) = &status.port_443_conflict {
                println!(
                    "Port 443 conflict: PID {} owns {}:{}",
                    conflict.owning_process_id, conflict.protocol, conflict.port
                );
            }
        }
        Response::Error(error) => {
            println!("Network request rejected: {}", error.message);
        }
        _ => println!("Unexpected network response."),
    }
}

fn run_status() -> Result<(), CliError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(CliError::Runtime)?;
    let status = runtime.block_on(fetch_status())?;
    print_status(&status);
    if status.health.status == HealthStatus::Unhealthy {
        return Err(CliError::DaemonUnhealthy);
    }
    Ok(())
}

async fn fetch_status() -> Result<StatusResponse, CliError> {
    let mut connection = connect().await?;
    timeout(REQUEST_TIMEOUT, exchange_status(&mut connection.client))
        .await
        .map_err(|_| CliError::Timeout)?
}

async fn connect() -> Result<ConnectedDaemon, CliError> {
    let deadline = Instant::now() + CONNECT_TIMEOUT;
    loop {
        match ClientOptions::new().open(PIPE_NAME) {
            Ok(client) => {
                let identity =
                    windows::verify_server(&client).map_err(CliError::UntrustedServer)?;
                return Ok(ConnectedDaemon {
                    client,
                    _identity: identity,
                });
            }
            Err(error)
                if error.raw_os_error() == Some(ERROR_PIPE_BUSY) && Instant::now() < deadline =>
            {
                sleep(RETRY_INTERVAL).await;
            }
            Err(error) if error.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                return Err(CliError::Timeout);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(CliError::DaemonAbsent);
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                return Err(CliError::AccessDenied);
            }
            Err(error) => return Err(CliError::Io(error)),
        }
    }
}

struct ConnectedDaemon {
    client: NamedPipeClient,
    _identity: windows::VerifiedServer,
}

async fn exchange_status(client: &mut NamedPipeClient) -> Result<StatusResponse, CliError> {
    let handshake_id = Uuid::new_v4();
    write_request(
        client,
        &RequestEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id: handshake_id,
            request: Request::Handshake(HandshakeRequest {
                client_kind: ClientKind::Cli,
                client_version: APPLICATION_VERSION.to_owned(),
                minimum_protocol_version: IPC_PROTOCOL_VERSION,
                maximum_protocol_version: IPC_PROTOCOL_VERSION,
            }),
        },
    )
    .await?;
    let handshake = read_response(client).await?;
    ensure_response_id(&handshake, handshake_id)?;
    match handshake.response {
        Response::Handshake(response)
            if response.selected_protocol_version == IPC_PROTOCOL_VERSION => {}
        Response::Error(error) => return Err(map_server_error(error)),
        _ => return Err(CliError::InvalidResponse),
    }

    let status_id = Uuid::new_v4();
    write_request(
        client,
        &RequestEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id: status_id,
            request: Request::Status(StatusRequest {}),
        },
    )
    .await?;
    let status = read_response(client).await?;
    ensure_response_id(&status, status_id)?;
    match status.response {
        Response::Status(status) => Ok(status),
        Response::Error(error) => Err(map_server_error(error)),
        _ => Err(CliError::InvalidResponse),
    }
}

async fn write_request(
    client: &mut NamedPipeClient,
    request: &RequestEnvelope,
) -> Result<(), CliError> {
    let frame = encode_frame(request).map_err(|_| CliError::InvalidResponse)?;
    client.write_all(&frame).await.map_err(CliError::Io)?;
    client.flush().await.map_err(CliError::Io)
}

async fn read_response(client: &mut NamedPipeClient) -> Result<ResponseEnvelope, CliError> {
    let mut prefix = [0_u8; FRAME_PREFIX_BYTES];
    client.read_exact(&mut prefix).await.map_err(CliError::Io)?;
    let length = u32::from_le_bytes(prefix) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(CliError::InvalidResponse);
    }
    let mut payload = vec![0_u8; length];
    client
        .read_exact(&mut payload)
        .await
        .map_err(CliError::Io)?;
    let response: ResponseEnvelope =
        decode_payload(&payload).map_err(|_| CliError::InvalidResponse)?;
    if response.protocol_version != IPC_PROTOCOL_VERSION {
        return Err(CliError::ProtocolMismatch {
            minimum: response.protocol_version,
            maximum: response.protocol_version,
        });
    }
    Ok(response)
}

fn ensure_response_id(response: &ResponseEnvelope, expected: Uuid) -> Result<(), CliError> {
    if response.request_id == expected {
        Ok(())
    } else {
        Err(CliError::RequestIdMismatch)
    }
}

fn map_server_error(error: ErrorResponse) -> CliError {
    if error.code == ErrorCode::ProtocolMismatch {
        CliError::ProtocolMismatch {
            minimum: error.minimum_protocol_version.unwrap_or(0),
            maximum: error.maximum_protocol_version.unwrap_or(0),
        }
    } else if error.code == ErrorCode::DaemonUnhealthy {
        CliError::DaemonUnhealthy
    } else {
        CliError::Server(error)
    }
}

fn print_status(status: &StatusResponse) {
    println!("Daemon: {}", health_label(status.health.status));
    println!("Version: {}", status.daemon.application_version);
    println!("Protocol: {}", status.daemon.protocol_version);
    println!("Instance: {}", status.daemon.instance_id);
    println!("PID: {}", status.daemon.process_id);
    println!("Uptime: {}", format_duration(status.daemon.uptime_ms));
    println!("Data: {}", status.paths.data_directory);
    println!("Database: {}", status.paths.database_path);
    println!("Logs: {}", status.paths.log_directory);
    if let Some(resources) = status.resources {
        println!(
            "Working set: {:.1} MiB",
            resources.working_set_bytes as f64 / (1024.0 * 1024.0)
        );
        println!(
            "Private usage: {:.1} MiB",
            resources.private_usage_bytes as f64 / (1024.0 * 1024.0)
        );
    }
    println!("Components:");
    for component in &status.health.components {
        let name = match component.component {
            nerd_core::ipc::HealthComponentName::State => "state",
            nerd_core::ipc::HealthComponentName::Logging => "logging",
            nerd_core::ipc::HealthComponentName::Ipc => "ipc",
            nerd_core::ipc::HealthComponentName::Resources => "resources",
        };
        match &component.message {
            Some(message) => println!("  {name}: {} ({message})", health_label(component.status)),
            None => println!("  {name}: {}", health_label(component.status)),
        }
    }
}

fn health_label(status: HealthStatus) -> &'static str {
    match status {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded => "degraded",
        HealthStatus::Unhealthy => "unhealthy",
    }
}

fn format_duration(milliseconds: u64) -> String {
    let seconds = milliseconds / 1_000;
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn parse_command(mut arguments: impl Iterator<Item = OsString>) -> Result<Command, CliError> {
    let Some(command) = arguments.next() else {
        return Err(CliError::Usage);
    };
    let command = command.to_string_lossy();
    match command.as_ref() {
        "status" => {
            if arguments.next().is_some() {
                return Err(CliError::Usage);
            }
            Ok(Command::Status)
        }
        "--version" => {
            if arguments.next().is_some() {
                return Err(CliError::Usage);
            }
            Ok(Command::Version)
        }
        "network" => {
            let action = arguments.next().ok_or(CliError::Usage)?;
            if arguments.next().is_some() {
                return Err(CliError::Usage);
            }
            let action = NetworkAction::parse(&action.to_string_lossy())?;
            Ok(Command::Network { action })
        }
        "runtime" => {
            let action = arguments.next().ok_or(CliError::Usage)?;
            let action = RuntimeAction::parse(&action.to_string_lossy())?;
            let arg = arguments.next();
            if arguments.next().is_some() {
                return Err(CliError::Usage);
            }
            let arg = arg.map(|value| value.to_string_lossy().into_owned());
            Ok(Command::Runtime { action, arg })
        }
        _ => Err(CliError::Usage),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Status,
    Version,
    Network {
        action: NetworkAction,
    },
    Runtime {
        action: RuntimeAction,
        arg: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeAction {
    Install,
    List,
    Remove,
    SetDefault,
}

impl RuntimeAction {
    fn parse(value: &str) -> Result<Self, CliError> {
        match value {
            "install" => Ok(Self::Install),
            "list" => Ok(Self::List),
            "remove" => Ok(Self::Remove),
            "set-default" => Ok(Self::SetDefault),
            _ => Err(CliError::Usage),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NetworkAction {
    Setup,
    Uninstall,
    Repair,
    Status,
}

impl NetworkAction {
    fn parse(value: &str) -> Result<Self, CliError> {
        match value {
            "setup" => Ok(Self::Setup),
            "uninstall" => Ok(Self::Uninstall),
            "repair" => Ok(Self::Repair),
            "status" => Ok(Self::Status),
            _ => Err(CliError::Usage),
        }
    }
}

#[derive(Debug)]
enum CliError {
    Usage,
    DaemonAbsent,
    AccessDenied,
    ProtocolMismatch { minimum: u32, maximum: u32 },
    DaemonUnhealthy,
    Timeout,
    RequestIdMismatch,
    InvalidResponse,
    UntrustedServer(windows::PeerIdentityError),
    Server(ErrorResponse),
    Runtime(io::Error),
    Io(io::Error),
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Usage => 2,
            Self::DaemonAbsent => 3,
            Self::ProtocolMismatch { .. } => 4,
            Self::DaemonUnhealthy => 5,
            Self::AccessDenied
            | Self::Timeout
            | Self::RequestIdMismatch
            | Self::InvalidResponse
            | Self::UntrustedServer(_)
            | Self::Server(_)
            | Self::Runtime(_)
            | Self::Io(_) => 6,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str("invalid command"),
            Self::DaemonAbsent => formatter.write_str("daemon is not running"),
            Self::AccessDenied => formatter.write_str(
                "daemon belongs to another Windows session or its IPC access policy failed",
            ),
            Self::ProtocolMismatch { minimum, maximum } => write!(
                formatter,
                "daemon protocol mismatch; supported daemon range is {minimum}..={maximum}"
            ),
            Self::DaemonUnhealthy => formatter.write_str("daemon is unhealthy"),
            Self::Timeout => formatter.write_str("daemon did not respond before the timeout"),
            Self::RequestIdMismatch => {
                formatter.write_str("daemon returned a mismatched request ID")
            }
            Self::InvalidResponse => formatter.write_str("daemon returned an invalid response"),
            Self::UntrustedServer(error) => {
                write!(formatter, "refusing untrusted daemon endpoint: {error}")
            }
            Self::Server(error) => write!(formatter, "daemon rejected request: {}", error.message),
            Self::Runtime(_) => formatter.write_str("failed to initialize CLI runtime"),
            Self::Io(_) => formatter.write_str("daemon IPC failed"),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) | Self::Io(error) => Some(error),
            Self::UntrustedServer(error) => Some(error),
            Self::Usage
            | Self::DaemonAbsent
            | Self::AccessDenied
            | Self::ProtocolMismatch { .. }
            | Self::DaemonUnhealthy
            | Self::Timeout
            | Self::RequestIdMismatch
            | Self::InvalidResponse
            | Self::Server(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use nerd_core::ipc::ErrorResponse;

    use super::{CliError, Command, format_duration, map_server_error, parse_command};

    #[test]
    fn parses_only_supported_commands() {
        assert_eq!(
            parse_command([OsString::from("status")].into_iter()).expect("status command"),
            Command::Status
        );
        assert_eq!(
            parse_command([OsString::from("--version")].into_iter()).expect("version command"),
            Command::Version
        );
        assert!(parse_command([].into_iter()).is_err());
        assert!(
            parse_command([OsString::from("status"), OsString::from("extra")].into_iter()).is_err()
        );
    }

    #[test]
    fn protocol_mismatch_has_distinct_exit_code() {
        let error = map_server_error(ErrorResponse::protocol_mismatch());
        assert!(matches!(
            error,
            CliError::ProtocolMismatch {
                minimum: 1,
                maximum: 1
            }
        ));
        assert_eq!(error.exit_code(), 4);
        assert_eq!(CliError::DaemonAbsent.exit_code(), 3);
        assert_eq!(CliError::DaemonUnhealthy.exit_code(), 5);
        assert_eq!(
            map_server_error(ErrorResponse::new(
                nerd_core::ipc::ErrorCode::DaemonUnhealthy,
                "unhealthy",
                false,
            ))
            .exit_code(),
            5
        );
    }

    #[test]
    fn formats_uptime_without_wall_clock_dependencies() {
        assert_eq!(format_duration(0), "00:00:00");
        assert_eq!(format_duration(3_661_999), "01:01:01");
    }
}
