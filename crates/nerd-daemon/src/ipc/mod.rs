mod framing;

use std::{
    collections::BTreeSet,
    fmt, io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use nerd_core::{
    APPLICATION_VERSION, IPC_PROTOCOL_VERSION, PIPE_NAME,
    ipc::{
        DaemonHealth, DaemonIdentity, DataPaths, ErrorCode, ErrorResponse, HandshakeResponse,
        HealthComponent, HealthComponentName, HealthStatus, ProcessResources, Request,
        RequestEnvelope, Response, ResponseEnvelope, RuntimeInstallResponse, RuntimeListResponse,
        RuntimeRemoveResponse, RuntimeSetDefaultResponse, StatusResponse,
    },
};
use serde::Deserialize;
use tokio::{
    net::windows::named_pipe::{NamedPipeServer, ServerOptions},
    sync::{Semaphore, watch},
    task::{JoinError, JoinSet},
    time::{Instant as TokioInstant, timeout, timeout_at},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    logging::LogHealthHandle,
    node::NodeManager,
    paths::AppPaths,
    setup::{NetworkRuntime, NetworkSetup, SetupError},
    state::{RuntimeKind, RuntimeRecord, SUPPORTED_SCHEMA_VERSION, StateClient},
    version::parse_spec,
    windows::{self, SecurityDescriptor},
};

const MAX_PIPE_INSTANCES: usize = 32;
const MAX_ACTIVE_CONNECTIONS: usize = MAX_PIPE_INSTANCES - 1;
const PIPE_BUFFER_BYTES: u32 = 16 * 1024;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const FRAME_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const RESPONSE_WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const STATE_RESPONSE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct DaemonContext {
    instance_id: Uuid,
    started_at: Instant,
    paths: AppPaths,
    state: StateClient,
    logging: LogHealthHandle,
    network: NetworkSetup,
    node: NodeManager,
}

impl DaemonContext {
    pub fn new(
        instance_id: Uuid,
        paths: AppPaths,
        state: StateClient,
        logging: LogHealthHandle,
        runtime: std::sync::Arc<NetworkRuntime>,
    ) -> Self {
        Self {
            instance_id,
            started_at: Instant::now(),
            paths: paths.clone(),
            state: state.clone(),
            logging,
            network: NetworkSetup::new(paths.clone(), runtime),
            node: NodeManager::new(paths, state),
        }
    }

    async fn status(&self) -> StatusResponse {
        let mut components = Vec::with_capacity(4);

        let state_component = match timeout(STATE_RESPONSE_TIMEOUT, self.state.health()).await {
            Ok(Ok(health))
                if health.schema_version == SUPPORTED_SCHEMA_VERSION
                    && health.foreign_keys_enabled =>
            {
                HealthComponent {
                    component: HealthComponentName::State,
                    status: HealthStatus::Healthy,
                    message: None,
                }
            }
            Ok(Ok(_)) => HealthComponent {
                component: HealthComponentName::State,
                status: HealthStatus::Unhealthy,
                message: Some("state invariants are not satisfied".to_owned()),
            },
            Ok(Err(_)) => HealthComponent {
                component: HealthComponentName::State,
                status: HealthStatus::Unhealthy,
                message: Some("state health check failed".to_owned()),
            },
            Err(_) => HealthComponent {
                component: HealthComponentName::State,
                status: HealthStatus::Unhealthy,
                message: Some("state health check timed out".to_owned()),
            },
        };
        components.push(state_component);

        let log_health = self.logging.snapshot();
        components.push(HealthComponent {
            component: HealthComponentName::Logging,
            status: if log_health.degraded {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            },
            message: log_health.degraded.then(|| {
                format!(
                    "logging degraded: {} I/O errors, {} dropped events",
                    log_health.io_errors, log_health.dropped_events
                )
            }),
        });

        components.push(HealthComponent {
            component: HealthComponentName::Ipc,
            status: HealthStatus::Healthy,
            message: None,
        });

        let resources = match windows::process_memory() {
            Ok(memory) => {
                components.push(HealthComponent {
                    component: HealthComponentName::Resources,
                    status: HealthStatus::Healthy,
                    message: None,
                });
                Some(ProcessResources {
                    working_set_bytes: memory.working_set_bytes,
                    peak_working_set_bytes: memory.peak_working_set_bytes,
                    private_usage_bytes: memory.private_usage_bytes,
                })
            }
            Err(_) => {
                components.push(HealthComponent {
                    component: HealthComponentName::Resources,
                    status: HealthStatus::Degraded,
                    message: Some("process resource metrics are unavailable".to_owned()),
                });
                None
            }
        };

        let health_status = components
            .iter()
            .fold(HealthStatus::Healthy, |overall, component| {
                worse_health(overall, component.status)
            });
        let uptime_ms = u64::try_from(self.started_at.elapsed().as_millis()).unwrap_or(u64::MAX);

        StatusResponse {
            daemon: DaemonIdentity {
                instance_id: self.instance_id,
                process_id: std::process::id(),
                application_version: APPLICATION_VERSION.to_owned(),
                protocol_version: IPC_PROTOCOL_VERSION,
                uptime_ms,
            },
            health: DaemonHealth {
                status: health_status,
                components,
            },
            paths: DataPaths {
                data_directory: self.paths.data_dir.to_string_lossy().into_owned(),
                database_path: self.paths.database_path.to_string_lossy().into_owned(),
                log_directory: self.paths.log_dir.to_string_lossy().into_owned(),
            },
            resources,
        }
    }
}

pub(crate) async fn serve(
    context: DaemonContext,
    security: &SecurityDescriptor,
    shutdown: watch::Receiver<Option<Instant>>,
) -> Result<(), IpcServerError> {
    serve_named(context, security, shutdown, PIPE_NAME).await
}

async fn serve_named(
    context: DaemonContext,
    security: &SecurityDescriptor,
    mut shutdown: watch::Receiver<Option<Instant>>,
    pipe_name: &str,
) -> Result<(), IpcServerError> {
    let mut listener =
        create_listener(security, pipe_name, true).map_err(IpcServerError::Listen)?;
    let permits = Arc::new(Semaphore::new(MAX_ACTIVE_CONNECTIONS));
    let active_tasks = Arc::new(Mutex::new(BTreeSet::new()));
    let mut tasks = JoinSet::new();

    info!(pipe = pipe_name, "IPC server listening");
    loop {
        drain_completed(&mut tasks);
        if shutdown.borrow().is_some() {
            break;
        }

        let permit = tokio::select! {
            changed = shutdown.changed() => {
                let _ = changed;
                break;
            }
            permit = Arc::clone(&permits).acquire_owned() => {
                permit.map_err(|_| IpcServerError::ConnectionLimitClosed)?
            }
        };

        let connected = tokio::select! {
            changed = shutdown.changed() => {
                let _ = changed;
                drop(permit);
                break;
            }
            result = listener.connect() => result,
        };
        if let Err(source) = connected {
            warn!(error = %source, "named-pipe accept failed");
            listener =
                create_listener(security, pipe_name, false).map_err(IpcServerError::Listen)?;
            continue;
        }

        let next_listener =
            create_listener(security, pipe_name, false).map_err(IpcServerError::Listen)?;
        let connection_id = Uuid::new_v4();
        let task_context = context.clone();
        let task_shutdown = shutdown.clone();
        let task_set = Arc::clone(&active_tasks);
        register_task(&task_set, connection_id);
        tasks.spawn(async move {
            let registration = ActiveTask::new(connection_id, task_set);
            let result = handle_connection(listener, task_context, task_shutdown).await;
            drop(permit);
            drop(registration);
            (connection_id, result)
        });
        listener = next_listener;
    }

    drop(listener);
    let shutdown_deadline = shutdown.borrow().unwrap_or_else(Instant::now);
    drain_tasks(&mut tasks, &active_tasks, shutdown_deadline).await;
    info!("IPC server stopped");
    Ok(())
}

async fn handle_connection(
    mut pipe: NamedPipeServer,
    context: DaemonContext,
    mut shutdown: watch::Receiver<Option<Instant>>,
) -> io::Result<()> {
    let first = tokio::select! {
        changed = shutdown.changed() => {
            let _ = changed;
            return Ok(());
        }
        result = read_request(
            &mut pipe,
            HANDSHAKE_TIMEOUT,
            FRAME_COMPLETION_TIMEOUT,
        ) => result?,
    };
    let Some(first) = first else {
        return Ok(());
    };
    let first = match first {
        ParsedRequest::Valid(request) => request,
        ParsedRequest::Invalid(request_id) => {
            write_invalid_request(&mut pipe, request_id).await?;
            return Ok(());
        }
    };

    if first.protocol_version != IPC_PROTOCOL_VERSION {
        write_error(
            &mut pipe,
            first.request_id,
            ErrorResponse::protocol_mismatch(),
        )
        .await?;
        return Ok(());
    }

    let Request::Handshake(handshake) = first.request else {
        write_error(
            &mut pipe,
            first.request_id,
            ErrorResponse::new(
                ErrorCode::HandshakeRequired,
                "handshake must be the first request",
                false,
            ),
        )
        .await?;
        return Ok(());
    };

    let valid_protocol_range = handshake.minimum_protocol_version >= 1
        && handshake.minimum_protocol_version <= handshake.maximum_protocol_version;
    let compatible = valid_protocol_range
        && handshake.minimum_protocol_version <= IPC_PROTOCOL_VERSION
        && handshake.maximum_protocol_version >= IPC_PROTOCOL_VERSION
        && handshake.maximum_protocol_version >= 1;
    if !valid_protocol_range {
        write_error(
            &mut pipe,
            first.request_id,
            ErrorResponse::new(
                ErrorCode::InvalidRequest,
                "protocol range is invalid",
                false,
            ),
        )
        .await?;
        return Ok(());
    }
    if !compatible {
        write_error(
            &mut pipe,
            first.request_id,
            ErrorResponse::protocol_mismatch(),
        )
        .await?;
        return Ok(());
    }
    write_response(
        &mut pipe,
        &ResponseEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id: first.request_id,
            response: Response::Handshake(HandshakeResponse {
                daemon_instance_id: context.instance_id,
                application_version: APPLICATION_VERSION.to_owned(),
                selected_protocol_version: IPC_PROTOCOL_VERSION,
            }),
        },
    )
    .await?;

    loop {
        let request = tokio::select! {
            changed = shutdown.changed() => {
                let _ = changed;
                return Ok(());
            }
            result = read_request(
                &mut pipe,
                REQUEST_IDLE_TIMEOUT,
                FRAME_COMPLETION_TIMEOUT,
            ) => result?,
        };
        let Some(request) = request else {
            return Ok(());
        };
        let request = match request {
            ParsedRequest::Valid(request) => request,
            ParsedRequest::Invalid(request_id) => {
                write_invalid_request(&mut pipe, request_id).await?;
                continue;
            }
        };

        if request.protocol_version != IPC_PROTOCOL_VERSION {
            write_error(
                &mut pipe,
                request.request_id,
                ErrorResponse::protocol_mismatch(),
            )
            .await?;
            return Ok(());
        }

        let response = match request.request {
            Request::Status(_) => Response::Status(context.status().await),
            Request::NetworkSetup(_) => match context.network.setup().await {
                Ok(response) => Response::NetworkSetup(response),
                Err(error) => Response::Error(network_error(error)),
            },
            Request::NetworkUninstall(_) => match context.network.uninstall().await {
                Ok(response) => Response::NetworkUninstall(response),
                Err(error) => Response::Error(network_error(error)),
            },
            Request::NetworkRepair(_) => match context.network.repair().await {
                Ok(response) => Response::NetworkRepair(response),
                Err(error) => Response::Error(network_error(error)),
            },
            Request::NetworkStatus(_) => match context.network.status() {
                Ok(response) => Response::NetworkStatus(response),
                Err(error) => Response::Error(network_error(error)),
            },
            Request::RuntimeInstall(request) => {
                match context.node.install(&request.version).await {
                    Ok(version) => Response::RuntimeInstall(RuntimeInstallResponse {
                        installed: true,
                        version,
                    }),
                    Err(error) => Response::Error(node_error(error)),
                }
            }
            Request::RuntimeList(_) => match context.node.list().await {
                Ok(runtimes) => Response::RuntimeList(RuntimeListResponse {
                    runtimes: runtimes.into_iter().map(into_runtime_info).collect(),
                }),
                Err(error) => Response::Error(node_error(error)),
            },
            Request::RuntimeRemove(request) => {
                match context.node.uninstall(request.runtime_id).await {
                    Ok(removed) => Response::RuntimeRemove(RuntimeRemoveResponse {
                        removed,
                        was_managed: removed,
                    }),
                    Err(error) => Response::Error(node_error(error)),
                }
            }
            Request::RuntimeSetDefault(request) => {
                let spec = parse_spec(&request.version).ok_or_else(|| {
                    ErrorResponse::new(
                        ErrorCode::InvalidRequest,
                        "invalid node version declaration",
                        false,
                    )
                });
                match spec {
                    Ok(spec) => match context.node.resolve(&spec).await {
                        Ok(version) => {
                            let _ = context
                                .state
                                .set_setting(
                                    "runtime.default".to_owned(),
                                    serde_json::json!({ "version": version }).to_string(),
                                )
                                .await;
                            Response::RuntimeSetDefault(RuntimeSetDefaultResponse { version })
                        }
                        Err(error) => Response::Error(node_error(error)),
                    },
                    Err(error) => Response::Error(error),
                }
            }
            Request::Handshake(_) => Response::Error(ErrorResponse::new(
                ErrorCode::InvalidRequest,
                "handshake is already complete",
                false,
            )),
        };
        write_response(
            &mut pipe,
            &ResponseEnvelope {
                protocol_version: IPC_PROTOCOL_VERSION,
                request_id: request.request_id,
                response,
            },
        )
        .await?;
    }
}

enum ParsedRequest {
    Valid(RequestEnvelope),
    Invalid(Uuid),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestIdentity {
    request_id: Uuid,
}

async fn read_request(
    pipe: &mut NamedPipeServer,
    idle_timeout: Duration,
    completion_timeout: Duration,
) -> io::Result<Option<ParsedRequest>> {
    let Some(payload) =
        framing::read_payload_bounded(pipe, idle_timeout, completion_timeout).await?
    else {
        return Ok(None);
    };
    match serde_json::from_slice(&payload) {
        Ok(request) => Ok(Some(ParsedRequest::Valid(request))),
        Err(_) => match serde_json::from_slice::<RequestIdentity>(&payload) {
            Ok(identity) => Ok(Some(ParsedRequest::Invalid(identity.request_id))),
            Err(source) => Err(io::Error::new(io::ErrorKind::InvalidData, source)),
        },
    }
}

async fn write_invalid_request(pipe: &mut NamedPipeServer, request_id: Uuid) -> io::Result<()> {
    write_error(
        pipe,
        request_id,
        ErrorResponse::new(
            ErrorCode::InvalidRequest,
            "request does not match the IPC contract",
            false,
        ),
    )
    .await
}

async fn write_error(
    pipe: &mut NamedPipeServer,
    request_id: Uuid,
    error: ErrorResponse,
) -> io::Result<()> {
    write_response(
        pipe,
        &ResponseEnvelope {
            protocol_version: IPC_PROTOCOL_VERSION,
            request_id,
            response: Response::Error(error),
        },
    )
    .await
}

async fn write_response(pipe: &mut NamedPipeServer, response: &ResponseEnvelope) -> io::Result<()> {
    timeout(
        RESPONSE_WRITE_TIMEOUT,
        framing::write_message(pipe, response),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IPC response write timed out"))?
}

fn create_listener(
    security: &SecurityDescriptor,
    pipe_name: &str,
    first_instance: bool,
) -> io::Result<NamedPipeServer> {
    let mut options = ServerOptions::new();
    options
        .first_pipe_instance(first_instance)
        .reject_remote_clients(true)
        .max_instances(MAX_PIPE_INSTANCES)
        .in_buffer_size(PIPE_BUFFER_BYTES)
        .out_buffer_size(PIPE_BUFFER_BYTES);
    let mut attributes = security.attributes();

    // SAFETY: The descriptor and attributes remain valid through this synchronous creation call.
    unsafe {
        options.create_with_security_attributes_raw(
            pipe_name,
            (&raw mut attributes).cast::<std::ffi::c_void>(),
        )
    }
}

fn network_error(error: SetupError) -> ErrorResponse {
    let message = error.to_string();
    ErrorResponse::new(ErrorCode::Internal, message, false)
}

fn node_error(error: crate::node::NodeError) -> ErrorResponse {
    let message = error.to_string();
    ErrorResponse::new(ErrorCode::Internal, message, false)
}

fn into_runtime_info(record: RuntimeRecord) -> nerd_core::runtime::RuntimeInfo {
    nerd_core::runtime::RuntimeInfo {
        runtime_id: record.runtime_id,
        kind: match record.kind {
            RuntimeKind::Managed => nerd_core::runtime::RuntimeKind::Managed,
            RuntimeKind::External => nerd_core::runtime::RuntimeKind::External,
        },
        tool: record.tool,
        version: record.version,
        executable_path: record.executable_path,
        architecture: record.architecture,
        status: match record.status {
            crate::state::RuntimeStatus::Ready => nerd_core::runtime::RuntimeStatus::Ready,
            crate::state::RuntimeStatus::Degraded => nerd_core::runtime::RuntimeStatus::Degraded,
        },
    }
}

fn worse_health(left: HealthStatus, right: HealthStatus) -> HealthStatus {
    match (left, right) {
        (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
        (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
        (HealthStatus::Healthy, HealthStatus::Healthy) => HealthStatus::Healthy,
    }
}

fn register_task(tasks: &Mutex<BTreeSet<Uuid>>, id: Uuid) {
    tasks
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(id);
}

fn drain_completed(tasks: &mut JoinSet<(Uuid, io::Result<()>)>) {
    while let Some(result) = tasks.try_join_next() {
        report_task(result);
    }
}

async fn drain_tasks(
    tasks: &mut JoinSet<(Uuid, io::Result<()>)>,
    active: &Mutex<BTreeSet<Uuid>>,
    deadline: Instant,
) {
    let deadline = TokioInstant::from_std(deadline);
    while !tasks.is_empty() {
        match timeout_at(deadline, tasks.join_next()).await {
            Ok(Some(result)) => report_task(result),
            Ok(None) => break,
            Err(_) => {
                let unfinished: Vec<_> = active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .iter()
                    .copied()
                    .collect();
                error!(?unfinished, "IPC shutdown deadline expired");
                tasks.abort_all();
                while let Some(result) = tasks.join_next().await {
                    report_task(result);
                }
                break;
            }
        }
    }
}

fn report_task(result: Result<(Uuid, io::Result<()>), JoinError>) {
    match result {
        Ok((id, Ok(()))) => debug!(connection_id = %id, "IPC connection closed"),
        Ok((id, Err(source)))
            if matches!(
                source.kind(),
                io::ErrorKind::BrokenPipe
                    | io::ErrorKind::ConnectionAborted
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::UnexpectedEof
            ) =>
        {
            debug!(connection_id = %id, "IPC client disconnected");
        }
        Ok((id, Err(source))) => {
            warn!(connection_id = %id, error = %source, "IPC connection failed");
        }
        Err(source) if source.is_cancelled() => {}
        Err(source) => error!(error = %source, "IPC connection task failed"),
    }
}

struct ActiveTask {
    id: Uuid,
    tasks: Arc<Mutex<BTreeSet<Uuid>>>,
}

impl ActiveTask {
    fn new(id: Uuid, tasks: Arc<Mutex<BTreeSet<Uuid>>>) -> Self {
        Self { id, tasks }
    }
}

impl Drop for ActiveTask {
    fn drop(&mut self) {
        self.tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&self.id);
    }
}

#[derive(Debug)]
pub enum IpcServerError {
    Listen(io::Error),
    ConnectionLimitClosed,
}

impl fmt::Display for IpcServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Listen(_) => formatter.write_str("failed to create the daemon IPC endpoint"),
            Self::ConnectionLimitClosed => formatter.write_str("IPC connection limit closed"),
        }
    }
}

impl std::error::Error for IpcServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Listen(error) => Some(error),
            Self::ConnectionLimitClosed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        fs, io,
        path::PathBuf,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use nerd_core::{
        APPLICATION_VERSION, IPC_PROTOCOL_VERSION,
        ipc::{
            ClientKind, ErrorCode, HandshakeRequest, Request, RequestEnvelope, Response,
            ResponseEnvelope, StatusRequest,
        },
    };
    use tokio::{
        net::windows::named_pipe::{ClientOptions, NamedPipeClient},
        sync::watch,
        task::JoinSet,
        time::{Instant, sleep},
    };
    use uuid::Uuid;

    use super::{ActiveTask, DaemonContext, drain_tasks, framing, register_task, serve_named};
    use crate::{
        logging::LogHealthHandle, paths::AppPaths, state::StateStore, windows::SecurityDescriptor,
    };

    const ERROR_PIPE_BUSY: i32 = 231;

    #[test]
    fn concurrent_clients_keep_request_ids_correlated() {
        let fixture = TempFixture::new("ipc-concurrency");
        let paths = AppPaths::from_root(fixture.path.clone());
        paths
            .create_state_directory()
            .expect("create state directory");
        let state = StateStore::open(&paths.database_path).expect("open state");
        let security = SecurityDescriptor::current_user_and_system().expect("security descriptor");
        let pipe_name = format!(r"\\.\pipe\Nerd.Test.{}", Uuid::new_v4());
        let context = DaemonContext::new(
            Uuid::new_v4(),
            paths,
            state.client(),
            LogHealthHandle::default(),
            std::sync::Arc::new(crate::setup::NetworkRuntime::default()),
        );
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        runtime.block_on(async {
            let (shutdown_sender, shutdown_receiver) = watch::channel(None);
            let server = serve_named(context, &security, shutdown_receiver, &pipe_name);
            let clients = async {
                let mut tasks = JoinSet::new();
                for _ in 0..16 {
                    let pipe_name = pipe_name.clone();
                    tasks.spawn(async move { request_status(&pipe_name).await });
                }
                while let Some(result) = tasks.join_next().await {
                    result.expect("client task").expect("status exchange");
                }
                shutdown_sender
                    .send_replace(Some(std::time::Instant::now() + Duration::from_secs(2)));
            };

            let (server_result, ()) = tokio::join!(server, clients);
            server_result.expect("IPC server");
        });
        drop(runtime);
        state.shutdown().expect("shutdown state");
    }

    #[test]
    fn handshake_is_required_and_protocol_mismatch_is_typed() {
        let fixture = TempFixture::new("ipc-errors");
        let paths = AppPaths::from_root(fixture.path.clone());
        paths
            .create_state_directory()
            .expect("create state directory");
        let state = StateStore::open(&paths.database_path).expect("open state");
        let security = SecurityDescriptor::current_user_and_system().expect("security descriptor");
        let pipe_name = format!(r"\\.\pipe\Nerd.Test.{}", Uuid::new_v4());
        let context = DaemonContext::new(
            Uuid::new_v4(),
            paths,
            state.client(),
            LogHealthHandle::default(),
            std::sync::Arc::new(crate::setup::NetworkRuntime::default()),
        );
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        runtime.block_on(async {
            let (shutdown_sender, shutdown_receiver) = watch::channel(None);
            let server = serve_named(context, &security, shutdown_receiver, &pipe_name);
            let clients = async {
                let mut client = connect(&pipe_name).await.expect("connect first client");
                let request_id = Uuid::new_v4();
                framing::write_message(
                    &mut client,
                    &RequestEnvelope {
                        protocol_version: IPC_PROTOCOL_VERSION,
                        request_id,
                        request: Request::Status(StatusRequest {}),
                    },
                )
                .await
                .expect("write status without handshake");
                let response: ResponseEnvelope = framing::read_message(&mut client)
                    .await
                    .expect("read handshake-required response")
                    .expect("response frame");
                assert_eq!(response.request_id, request_id);
                assert!(matches!(
                    response.response,
                    Response::Error(error) if error.code == ErrorCode::HandshakeRequired
                ));

                let mut client = connect(&pipe_name).await.expect("connect second client");
                let request_id = Uuid::new_v4();
                framing::write_message(
                    &mut client,
                    &RequestEnvelope {
                        protocol_version: IPC_PROTOCOL_VERSION + 1,
                        request_id,
                        request: Request::Handshake(HandshakeRequest {
                            client_kind: ClientKind::Cli,
                            client_version: APPLICATION_VERSION.to_owned(),
                            minimum_protocol_version: IPC_PROTOCOL_VERSION + 1,
                            maximum_protocol_version: IPC_PROTOCOL_VERSION + 1,
                        }),
                    },
                )
                .await
                .expect("write incompatible handshake");
                let response: ResponseEnvelope = framing::read_message(&mut client)
                    .await
                    .expect("read mismatch response")
                    .expect("response frame");
                assert_eq!(response.request_id, request_id);
                assert!(matches!(
                    response.response,
                    Response::Error(error) if error.code == ErrorCode::ProtocolMismatch
                ));

                let mut client = connect(&pipe_name).await.expect("connect third client");
                let request_id = Uuid::new_v4();
                framing::write_message(
                    &mut client,
                    &serde_json::json!({
                        "protocolVersion": IPC_PROTOCOL_VERSION,
                        "requestId": request_id,
                        "request": {
                            "type": "handshake",
                            "payload": {
                                "clientKind": "cli",
                                "clientVersion": APPLICATION_VERSION,
                                "minimumProtocolVersion": 0,
                                "maximumProtocolVersion": IPC_PROTOCOL_VERSION
                            }
                        }
                    }),
                )
                .await
                .expect("write zero-based protocol range");
                let response: ResponseEnvelope = framing::read_message(&mut client)
                    .await
                    .expect("read invalid-range response")
                    .expect("response frame");
                assert_eq!(response.request_id, request_id);
                assert!(matches!(
                    response.response,
                    Response::Error(error) if error.code == ErrorCode::InvalidRequest
                ));

                let mut client = connect(&pipe_name).await.expect("connect fourth client");
                let request_id = Uuid::new_v4();
                framing::write_message(
                    &mut client,
                    &serde_json::json!({
                        "protocolVersion": IPC_PROTOCOL_VERSION,
                        "requestId": request_id,
                        "request": { "type": "unknown", "payload": {} }
                    }),
                )
                .await
                .expect("write unknown request");
                let response: ResponseEnvelope = framing::read_message(&mut client)
                    .await
                    .expect("read invalid-request response")
                    .expect("response frame");
                assert_eq!(response.request_id, request_id);
                assert!(matches!(
                    response.response,
                    Response::Error(error) if error.code == ErrorCode::InvalidRequest
                ));
                shutdown_sender
                    .send_replace(Some(std::time::Instant::now() + Duration::from_secs(2)));
            };

            let (server_result, ()) = tokio::join!(server, clients);
            server_result.expect("IPC server");
        });
        drop(runtime);
        state.shutdown().expect("shutdown state");
    }

    #[test]
    fn shutdown_deadline_aborts_and_unregisters_active_tasks() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");
        runtime.block_on(async {
            let active = Arc::new(Mutex::new(BTreeSet::new()));
            let mut tasks = JoinSet::new();
            let connection_id = Uuid::new_v4();
            register_task(&active, connection_id);
            let task_active = Arc::clone(&active);
            tasks.spawn(async move {
                let registration = ActiveTask::new(connection_id, task_active);
                std::future::pending::<()>().await;
                drop(registration);
                (connection_id, Ok(()))
            });
            tokio::task::yield_now().await;

            drain_tasks(
                &mut tasks,
                &active,
                std::time::Instant::now() + Duration::from_millis(10),
            )
            .await;

            assert!(tasks.is_empty());
            assert!(
                active
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .is_empty()
            );
        });
    }

    async fn request_status(pipe_name: &str) -> io::Result<()> {
        let mut client = connect(pipe_name).await?;
        let handshake_id = Uuid::new_v4();
        framing::write_message(
            &mut client,
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
        let handshake: ResponseEnvelope = framing::read_message(&mut client)
            .await?
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing handshake"))?;
        if handshake.request_id != handshake_id
            || !matches!(handshake.response, Response::Handshake(_))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "handshake correlation failed",
            ));
        }

        let status_id = Uuid::new_v4();
        framing::write_message(
            &mut client,
            &RequestEnvelope {
                protocol_version: IPC_PROTOCOL_VERSION,
                request_id: status_id,
                request: Request::Status(StatusRequest {}),
            },
        )
        .await?;
        let status: ResponseEnvelope = framing::read_message(&mut client)
            .await?
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing status"))?;
        if status.request_id != status_id || !matches!(status.response, Response::Status(_)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "status correlation failed",
            ));
        }
        Ok(())
    }

    async fn connect(pipe_name: &str) -> io::Result<NamedPipeClient> {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match ClientOptions::new().open(pipe_name) {
                Ok(client) => return Ok(client),
                Err(error)
                    if (error.kind() == io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(ERROR_PIPE_BUSY))
                        && Instant::now() < deadline =>
                {
                    sleep(Duration::from_millis(10)).await;
                }
                Err(error) => return Err(error),
            }
        }
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
