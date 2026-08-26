pub mod cert;
pub mod dns;
pub mod exec;
pub mod identity;
pub mod instance;
pub mod ipc;
pub mod location;
pub mod logging;
pub mod node;
pub mod package_manager;
pub mod paths;
pub mod project;
pub mod setup;
pub mod shutdown;
pub mod state;
pub mod version;
pub mod watcher;

pub(crate) mod windows;

use std::{
    fmt, io,
    time::{Duration, Instant},
};

use tokio::sync::watch;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    instance::{InstanceGuard, InstanceGuardError},
    ipc::{DaemonContext, IpcServerError},
    logging::LoggingGuard,
    paths::AppPaths,
    state::{StateError, StateStore},
    windows::SecurityDescriptor,
};

const GLOBAL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(4);
const IPC_SHUTDOWN_BUDGET: Duration = Duration::from_secs(2);
const LOGGING_SHUTDOWN_RESERVE: Duration = Duration::from_millis(500);

pub fn application_version() -> &'static str {
    nerd_core::APPLICATION_VERSION
}

pub fn check_process_security() -> Result<(), ProcessSecurityError> {
    windows::ensure_non_elevated_current_process()
}

#[derive(Debug)]
pub enum ProcessSecurityError {
    Elevated,
    LocalSystem,
    Query(io::Error),
}

impl fmt::Display for ProcessSecurityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Elevated => formatter.write_str(
                "daemon refuses an elevated token; start it from a non-elevated terminal",
            ),
            Self::LocalSystem => {
                formatter.write_str("daemon refuses the LocalSystem account; run it as a user")
            }
            Self::Query(_) => formatter.write_str("failed to verify daemon process token"),
        }
    }
}

impl std::error::Error for ProcessSecurityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Query(error) => Some(error),
            Self::Elevated | Self::LocalSystem => None,
        }
    }
}

pub fn run() -> Result<(), DaemonRunError> {
    check_process_security().map_err(DaemonRunError::ProcessSecurity)?;
    let paths = AppPaths::resolve().map_err(DaemonRunError::Paths)?;
    let security =
        SecurityDescriptor::current_user_and_system().map_err(DaemonRunError::Security)?;
    let instance_guard = InstanceGuard::acquire(&security).map_err(DaemonRunError::Instance)?;
    paths
        .create_state_directory()
        .map_err(DaemonRunError::Paths)?;
    let logging = LoggingGuard::initialize(&paths.log_dir).map_err(DaemonRunError::Logging)?;
    if instance_guard.abandoned_predecessor() {
        warn!("previous daemon instance ended without releasing its guard");
    }

    let state = match StateStore::open(&paths.database_path) {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(
                code = error.diagnostic_code(),
                guidance = error.recovery_guidance(),
                "state startup failed"
            );
            return Err(DaemonRunError::State(error));
        }
    };
    let context = DaemonContext::new(
        Uuid::new_v4(),
        paths,
        state.client(),
        logging.health_handle(),
        std::sync::Arc::new(crate::setup::NetworkRuntime::default()),
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(DaemonRunError::Runtime)?;

    info!(
        application_version = nerd_core::APPLICATION_VERSION,
        protocol_version = nerd_core::IPC_PROTOCOL_VERSION,
        "daemon started"
    );
    let service = runtime.block_on(run_service(context, &security));
    drop(runtime);

    let state_deadline = service
        .shutdown_deadline
        .checked_sub(LOGGING_SHUTDOWN_RESERVE)
        .unwrap_or(service.shutdown_deadline);
    let state_result = state
        .shutdown_before(state_deadline)
        .map_err(DaemonRunError::State);
    info!("daemon stopped");
    logging.shutdown_before(service.shutdown_deadline);
    drop(instance_guard);

    service.result?;
    state_result?;
    Ok(())
}

async fn run_service(context: DaemonContext, security: &SecurityDescriptor) -> ServiceOutcome {
    let signal = match shutdown::wait_for_shutdown_signal() {
        Ok(signal) => signal,
        Err(source) => {
            return ServiceOutcome {
                result: Err(DaemonRunError::Signal(source)),
                shutdown_deadline: Instant::now() + GLOBAL_SHUTDOWN_TIMEOUT,
            };
        }
    };
    tokio::pin!(signal);
    let (shutdown_sender, shutdown_receiver) = watch::channel(None);
    let server = ipc::serve(context, security, shutdown_receiver);
    tokio::pin!(server);

    tokio::select! {
        result = &mut server => ServiceOutcome {
            result: result.map_err(DaemonRunError::Ipc),
            shutdown_deadline: Instant::now() + GLOBAL_SHUTDOWN_TIMEOUT,
        },
        signal = &mut signal => {
            let started = Instant::now();
            let shutdown_deadline = started + GLOBAL_SHUTDOWN_TIMEOUT;
            shutdown_sender.send_replace(Some(started + IPC_SHUTDOWN_BUDGET));
            info!(reason = %signal, "daemon shutdown requested");
            ServiceOutcome {
                result: server.await.map_err(DaemonRunError::Ipc),
                shutdown_deadline,
            }
        }
    }
}

struct ServiceOutcome {
    result: Result<(), DaemonRunError>,
    shutdown_deadline: Instant,
}

#[derive(Debug)]
pub enum DaemonRunError {
    ProcessSecurity(ProcessSecurityError),
    Paths(io::Error),
    Logging(io::Error),
    Security(io::Error),
    Instance(InstanceGuardError),
    State(StateError),
    Runtime(io::Error),
    Ipc(IpcServerError),
    Signal(io::Error),
}

impl DaemonRunError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ProcessSecurity(_) => 14,
            Self::Instance(InstanceGuardError::AlreadyRunning) => 10,
            Self::Instance(InstanceGuardError::MachineConflict) => 11,
            Self::State(_) => 12,
            Self::Ipc(_) | Self::Security(_) => 13,
            Self::Paths(_)
            | Self::Logging(_)
            | Self::Runtime(_)
            | Self::Signal(_)
            | Self::Instance(InstanceGuardError::Os(_)) => 1,
        }
    }
}

impl fmt::Display for DaemonRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessSecurity(error) => error.fmt(formatter),
            Self::Paths(_) => formatter.write_str("failed to prepare Nerd data paths"),
            Self::Logging(_) => formatter.write_str("failed to initialize daemon diagnostics"),
            Self::Security(_) => formatter.write_str("failed to create daemon security policy"),
            Self::Instance(error) => error.fmt(formatter),
            Self::State(error) => write!(formatter, "{error}; {}", error.recovery_guidance()),
            Self::Runtime(_) => formatter.write_str("failed to initialize daemon runtime"),
            Self::Ipc(error) => error.fmt(formatter),
            Self::Signal(_) => formatter.write_str("failed to register daemon shutdown signals"),
        }
    }
}

impl std::error::Error for DaemonRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ProcessSecurity(error) => Some(error),
            Self::Paths(error)
            | Self::Logging(error)
            | Self::Security(error)
            | Self::Runtime(error)
            | Self::Signal(error) => Some(error),
            Self::Instance(error) => Some(error),
            Self::State(error) => Some(error),
            Self::Ipc(error) => Some(error),
        }
    }
}
