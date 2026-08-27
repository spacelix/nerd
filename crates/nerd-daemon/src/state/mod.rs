mod migrations;

use std::{
    fmt, io,
    path::Path,
    sync::mpsc::{self, SyncSender, TrySendError},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, OptionalExtension, params};
use tokio::sync::oneshot;
use uuid::Uuid;

pub use migrations::SUPPORTED_SCHEMA_VERSION;

const COMMAND_QUEUE_CAPACITY: usize = 64;
const DATABASE_BUSY_TIMEOUT: Duration = Duration::from_secs(2);
const STATE_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

pub struct StateStore {
    client: StateClient,
    worker: Option<JoinHandle<()>>,
}

impl StateStore {
    pub fn open(path: &Path) -> Result<Self, StateError> {
        let (command_sender, command_receiver) = mpsc::sync_channel(COMMAND_QUEUE_CAPACITY);
        let (init_sender, init_receiver) = mpsc::sync_channel(1);
        let database_path = path.to_owned();
        let worker = thread::Builder::new()
            .name("nerd-state".to_owned())
            .spawn(move || {
                let connection = initialize_connection(&database_path);
                match connection {
                    Ok(connection) => {
                        let _ = init_sender.send(Ok(()));
                        run_worker(&connection, command_receiver);
                    }
                    Err(error) => {
                        let _ = init_sender.send(Err(error));
                    }
                }
            })
            .map_err(StateError::WorkerSpawn)?;

        match init_receiver.recv() {
            Ok(Ok(())) => Ok(Self {
                client: StateClient {
                    sender: command_sender,
                },
                worker: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => {
                let _ = worker.join();
                Err(StateError::WorkerStopped)
            }
        }
    }

    pub fn client(&self) -> StateClient {
        self.client.clone()
    }

    pub fn shutdown(mut self) -> Result<(), StateError> {
        self.stop_worker_before(Instant::now() + STATE_SHUTDOWN_TIMEOUT)
    }

    pub fn shutdown_before(mut self, deadline: Instant) -> Result<(), StateError> {
        self.stop_worker_before(deadline)
    }

    fn stop_worker_before(&mut self, deadline: Instant) -> Result<(), StateError> {
        if self.worker.is_none() {
            return Ok(());
        }

        let control_result = {
            let (reply_sender, reply_receiver) = mpsc::sync_channel(1);
            let send_result = send_shutdown_before(
                &self.client.sender,
                StateCommand::Shutdown(reply_sender),
                deadline,
            );
            match send_result {
                Ok(()) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    match reply_receiver.recv_timeout(remaining) {
                        Ok(result) => result,
                        Err(mpsc::RecvTimeoutError::Timeout) => Err(StateError::ShutdownTimeout),
                        Err(mpsc::RecvTimeoutError::Disconnected) => Err(StateError::WorkerStopped),
                    }
                }
                Err(error) => Err(error),
            }
        };

        let worker = self.worker.take().expect("worker presence checked");
        while !worker.is_finished() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        let join_result = if worker.is_finished() {
            worker.join().map_err(|_| StateError::WorkerPanicked)
        } else {
            Err(StateError::ShutdownTimeout)
        };

        control_result?;
        join_result
    }
}

impl Drop for StateStore {
    fn drop(&mut self) {
        let _ = self.stop_worker_before(Instant::now() + STATE_SHUTDOWN_TIMEOUT);
    }
}

#[derive(Clone)]
pub struct StateClient {
    sender: SyncSender<StateCommand>,
}

impl StateClient {
    pub async fn health(&self) -> Result<StateHealth, StateError> {
        self.request(StateCommand::Health).await
    }

    pub async fn get_setting(&self, key: String) -> Result<Option<String>, StateError> {
        validate_key(&key)?;
        self.request(|reply| StateCommand::GetSetting { key, reply })
            .await
    }

    pub async fn set_setting(&self, key: String, value_json: String) -> Result<(), StateError> {
        validate_key(&key)?;
        serde_json::from_str::<serde_json::Value>(&value_json)
            .map_err(|_| StateError::InvalidJson)?;
        self.request(|reply| StateCommand::SetSetting {
            key,
            value_json,
            reply,
        })
        .await
    }

    pub async fn begin_operation(
        &self,
        operation_id: Uuid,
        operation_type: String,
        recovery_state_json: Option<String>,
    ) -> Result<(), StateError> {
        validate_key(&operation_type)?;
        if let Some(value) = &recovery_state_json {
            serde_json::from_str::<serde_json::Value>(value)
                .map_err(|_| StateError::InvalidJson)?;
        }
        self.request(|reply| StateCommand::BeginOperation {
            operation_id,
            operation_type,
            recovery_state_json,
            reply,
        })
        .await
    }

    pub async fn list_runtimes(&self) -> Result<Vec<RuntimeRecord>, StateError> {
        self.request(StateCommand::ListRuntimes).await
    }

    pub async fn register_runtime(&self, runtime: &RuntimeRecord) -> Result<(), StateError> {
        self.request(|reply| StateCommand::RegisterRuntime {
            runtime: runtime.clone(),
            reply,
        })
        .await
    }

    pub async fn set_runtime_status(
        &self,
        runtime_id: Uuid,
        status: RuntimeStatus,
    ) -> Result<(), StateError> {
        self.request(|reply| StateCommand::SetRuntimeStatus {
            runtime_id,
            status,
            reply,
        })
        .await
    }

    pub async fn remove_runtime(&self, runtime_id: Uuid) -> Result<(), StateError> {
        self.request(|reply| StateCommand::RemoveRuntime { runtime_id, reply })
            .await
    }

    pub async fn list_projects(&self) -> Result<Vec<ProjectRecord>, StateError> {
        self.request(StateCommand::ListProjects).await
    }

    pub async fn upsert_project(&self, project: &ProjectRecord) -> Result<(), StateError> {
        self.request(|reply| StateCommand::UpsertProject {
            project: project.clone(),
            reply,
        })
        .await
    }

    pub async fn remove_project(&self, project_id: Uuid) -> Result<(), StateError> {
        self.request(|reply| StateCommand::RemoveProject { project_id, reply })
            .await
    }

    pub async fn list_routes(&self) -> Result<Vec<RouteRow>, StateError> {
        self.request(StateCommand::ListRoutes).await
    }

    pub async fn set_route(
        &self,
        route_name: String,
        project_id: Uuid,
        source: RouteSource,
    ) -> Result<(), StateError> {
        self.request(|reply| StateCommand::SetRoute {
            route_name,
            project_id,
            source,
            reply,
        })
        .await
    }

    pub async fn clear_routes_for_project(&self, project_id: Uuid) -> Result<(), StateError> {
        self.request(|reply| StateCommand::ClearRoutesForProject { project_id, reply })
            .await
    }

    pub async fn get_trust(&self, project_id: Uuid) -> Result<Option<TrustRecord>, StateError> {
        self.request(|reply| StateCommand::GetTrust { project_id, reply })
            .await
    }

    pub async fn bind_trust(&self, trust: &TrustRecord) -> Result<(), StateError> {
        self.request(|reply| StateCommand::BindTrust {
            trust: trust.clone(),
            reply,
        })
        .await
    }

    pub async fn finish_operation(
        &self,
        operation_id: Uuid,
        succeeded: bool,
    ) -> Result<(), StateError> {
        self.request(|reply| StateCommand::FinishOperation {
            operation_id,
            succeeded,
            reply,
        })
        .await
    }

    async fn request<T>(
        &self,
        build: impl FnOnce(oneshot::Sender<Result<T, StateError>>) -> StateCommand,
    ) -> Result<T, StateError> {
        let (reply, receiver) = oneshot::channel();
        match self.sender.try_send(build(reply)) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(StateError::QueueFull),
            Err(TrySendError::Disconnected(_)) => return Err(StateError::WorkerStopped),
        }
        receiver.await.map_err(|_| StateError::WorkerStopped)?
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StateHealth {
    pub schema_version: u32,
    pub foreign_keys_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeRecord {
    pub runtime_id: Uuid,
    pub kind: RuntimeKind,
    pub tool: String,
    pub version: String,
    pub executable_path: String,
    pub architecture: String,
    pub binary_identity: String,
    pub status: RuntimeStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeKind {
    Managed,
    External,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeStatus {
    Ready,
    Degraded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectKind {
    Parked,
    Linked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectStatus {
    Untrusted,
    Trusted,
    Conflict,
    Missing,
    Replaced,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteSource {
    Derived,
    Explicit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustKind {
    Untrusted,
    Trusted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRecord {
    pub project_id: Uuid,
    pub kind: ProjectKind,
    pub path: String,
    pub dir_volume_serial: u64,
    pub dir_file_id: u64,
    pub name: String,
    pub status: ProjectStatus,
    pub manifest_valid: bool,
    pub manifest_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteRow {
    pub route_name: String,
    pub project_id: Uuid,
    pub source: RouteSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustRecord {
    pub project_id: Uuid,
    pub trust_kind: TrustKind,
    pub directory_volume_serial: u64,
    pub directory_file_id: u64,
    pub repository_identity: Option<String>,
    pub trusted_at_unix_ms: Option<u64>,
}

impl RuntimeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Managed => "managed",
            Self::External => "external",
        }
    }
}

impl RuntimeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
        }
    }
}

enum StateCommand {
    Health(oneshot::Sender<Result<StateHealth, StateError>>),
    GetSetting {
        key: String,
        reply: oneshot::Sender<Result<Option<String>, StateError>>,
    },
    SetSetting {
        key: String,
        value_json: String,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    BeginOperation {
        operation_id: Uuid,
        operation_type: String,
        recovery_state_json: Option<String>,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    FinishOperation {
        operation_id: Uuid,
        succeeded: bool,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    ListRuntimes(oneshot::Sender<Result<Vec<RuntimeRecord>, StateError>>),
    RegisterRuntime {
        runtime: RuntimeRecord,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    SetRuntimeStatus {
        runtime_id: Uuid,
        status: RuntimeStatus,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    RemoveRuntime {
        runtime_id: Uuid,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    ListProjects(oneshot::Sender<Result<Vec<ProjectRecord>, StateError>>),
    UpsertProject {
        project: ProjectRecord,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    RemoveProject {
        project_id: Uuid,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    ListRoutes(oneshot::Sender<Result<Vec<RouteRow>, StateError>>),
    SetRoute {
        route_name: String,
        project_id: Uuid,
        source: RouteSource,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    ClearRoutesForProject {
        project_id: Uuid,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    GetTrust {
        project_id: Uuid,
        reply: oneshot::Sender<Result<Option<TrustRecord>, StateError>>,
    },
    BindTrust {
        trust: TrustRecord,
        reply: oneshot::Sender<Result<(), StateError>>,
    },
    Shutdown(SyncSender<Result<(), StateError>>),
}

fn initialize_connection(path: &Path) -> Result<Connection, StateError> {
    let mut connection = Connection::open(path).map_err(StateError::Open)?;
    connection
        .busy_timeout(DATABASE_BUSY_TIMEOUT)
        .map_err(StateError::Configure)?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(StateError::Configure)?;

    let foreign_keys_enabled: bool = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .map_err(StateError::Configure)?;
    if !foreign_keys_enabled {
        return Err(StateError::ForeignKeysUnavailable);
    }

    migrations::migrate(&mut connection)?;
    Ok(connection)
}

fn run_worker(connection: &Connection, receiver: mpsc::Receiver<StateCommand>) {
    while let Ok(command) = receiver.recv() {
        match command {
            StateCommand::Health(reply) => {
                let _ = reply.send(read_health(connection));
            }
            StateCommand::GetSetting { key, reply } => {
                let _ = reply.send(get_setting(connection, &key));
            }
            StateCommand::SetSetting {
                key,
                value_json,
                reply,
            } => {
                let _ = reply.send(set_setting(connection, &key, &value_json));
            }
            StateCommand::BeginOperation {
                operation_id,
                operation_type,
                recovery_state_json,
                reply,
            } => {
                let _ = reply.send(begin_operation(
                    connection,
                    operation_id,
                    &operation_type,
                    recovery_state_json.as_deref(),
                ));
            }
            StateCommand::FinishOperation {
                operation_id,
                succeeded,
                reply,
            } => {
                let _ = reply.send(finish_operation(connection, operation_id, succeeded));
            }
            StateCommand::ListRuntimes(reply) => {
                let _ = reply.send(list_runtimes(connection));
            }
            StateCommand::RegisterRuntime { runtime, reply } => {
                let _ = reply.send(register_runtime(connection, &runtime));
            }
            StateCommand::SetRuntimeStatus {
                runtime_id,
                status,
                reply,
            } => {
                let _ = reply.send(set_runtime_status(connection, runtime_id, status));
            }
            StateCommand::RemoveRuntime { runtime_id, reply } => {
                let _ = reply.send(remove_runtime(connection, runtime_id));
            }
            StateCommand::ListProjects(reply) => {
                let _ = reply.send(list_projects(connection));
            }
            StateCommand::UpsertProject { project, reply } => {
                let _ = reply.send(upsert_project(connection, &project));
            }
            StateCommand::RemoveProject { project_id, reply } => {
                let _ = reply.send(remove_project(connection, project_id));
            }
            StateCommand::ListRoutes(reply) => {
                let _ = reply.send(list_routes(connection));
            }
            StateCommand::SetRoute {
                route_name,
                project_id,
                source,
                reply,
            } => {
                let _ = reply.send(set_route(connection, &route_name, project_id, source));
            }
            StateCommand::ClearRoutesForProject { project_id, reply } => {
                let _ = reply.send(clear_routes_for_project(connection, project_id));
            }
            StateCommand::GetTrust { project_id, reply } => {
                let _ = reply.send(get_trust(connection, project_id));
            }
            StateCommand::BindTrust { trust, reply } => {
                let _ = reply.send(bind_trust(connection, &trust));
            }
            StateCommand::Shutdown(reply) => {
                let result = connection
                    .execute_batch("PRAGMA optimize;")
                    .map_err(StateError::Shutdown);
                let _ = reply.send(result);
                break;
            }
        }
    }
}

fn send_shutdown_before(
    sender: &SyncSender<StateCommand>,
    mut command: StateCommand,
    deadline: Instant,
) -> Result<(), StateError> {
    loop {
        match sender.try_send(command) {
            Ok(()) => return Ok(()),
            Err(TrySendError::Full(returned)) if Instant::now() < deadline => {
                command = returned;
                thread::sleep(Duration::from_millis(5));
            }
            Err(TrySendError::Full(_)) => return Err(StateError::ShutdownTimeout),
            Err(TrySendError::Disconnected(_)) => return Err(StateError::WorkerStopped),
        }
    }
}

fn read_health(connection: &Connection) -> Result<StateHealth, StateError> {
    migrations::validate_integrity(connection)?;
    let schema_version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(StateError::IntegrityQuery)?;
    let foreign_keys_enabled = connection
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .map_err(StateError::IntegrityQuery)?;
    Ok(StateHealth {
        schema_version,
        foreign_keys_enabled,
    })
}

fn get_setting(connection: &Connection, key: &str) -> Result<Option<String>, StateError> {
    connection
        .query_row(
            "SELECT value_json FROM global_settings WHERE setting_key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()
        .map_err(StateError::Repository)
}

fn set_setting(connection: &Connection, key: &str, value_json: &str) -> Result<(), StateError> {
    connection
        .execute(
            "INSERT INTO global_settings (setting_key, value_json, updated_at_unix_ms) \
             VALUES (?1, ?2, ?3) \
             ON CONFLICT(setting_key) DO UPDATE SET \
             value_json = excluded.value_json, updated_at_unix_ms = excluded.updated_at_unix_ms",
            params![key, value_json, unix_timestamp_ms()?],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn begin_operation(
    connection: &Connection,
    operation_id: Uuid,
    operation_type: &str,
    recovery_state_json: Option<&str>,
) -> Result<(), StateError> {
    connection
        .execute(
            "INSERT INTO operation_history (operation_id, operation_type, status, \
             started_at_unix_ms, recovery_state_json) VALUES (?1, ?2, 'running', ?3, ?4)",
            params![
                operation_id.to_string(),
                operation_type,
                unix_timestamp_ms()?,
                recovery_state_json
            ],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn finish_operation(
    connection: &Connection,
    operation_id: Uuid,
    succeeded: bool,
) -> Result<(), StateError> {
    let status = if succeeded { "succeeded" } else { "failed" };
    let changed = connection
        .execute(
            "UPDATE operation_history SET status = ?2, finished_at_unix_ms = ?3 \
             WHERE operation_id = ?1 AND status = 'running'",
            params![operation_id.to_string(), status, unix_timestamp_ms()?],
        )
        .map_err(StateError::Repository)?;
    if changed == 0 {
        return Err(StateError::OperationNotRunning(operation_id));
    }
    Ok(())
}

fn list_runtimes(connection: &Connection) -> Result<Vec<RuntimeRecord>, StateError> {
    let mut statement = connection
        .prepare(
            "SELECT runtime_id, kind, tool, version, executable_path, architecture, \
                    binary_identity, status \
             FROM runtimes ORDER BY tool, version",
        )
        .map_err(StateError::Repository)?;
    let rows = statement
        .query_map([], |row| {
            let runtime_id: String = row.get(0)?;
            Ok(RuntimeRecord {
                runtime_id: Uuid::parse_str(&runtime_id)
                    .map_err(|_| rusqlite::Error::InvalidColumnName("runtime_id".to_owned()))?,
                kind: runtime_kind_from_str(&row.get::<_, String>(1)?)
                    .ok_or_else(|| rusqlite::Error::InvalidColumnName("kind".to_owned()))?,
                tool: row.get(2)?,
                version: row.get(3)?,
                executable_path: row.get(4)?,
                architecture: row.get(5)?,
                binary_identity: row.get(6)?,
                status: runtime_status_from_str(&row.get::<_, String>(7)?)
                    .ok_or_else(|| rusqlite::Error::InvalidColumnName("status".to_owned()))?,
            })
        })
        .map_err(StateError::Repository)?;
    rows.collect::<Result<_, _>>()
        .map_err(StateError::Repository)
}

fn register_runtime(connection: &Connection, runtime: &RuntimeRecord) -> Result<(), StateError> {
    connection
        .execute(
            "INSERT INTO runtimes (runtime_id, kind, tool, version, executable_path, \
                    architecture, binary_identity, status, recorded_at_unix_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
             ON CONFLICT(runtime_id) DO UPDATE SET \
             executable_path = excluded.executable_path, \
             binary_identity = excluded.binary_identity, \
             status = excluded.status",
            params![
                runtime.runtime_id.to_string(),
                runtime.kind.as_str(),
                runtime.tool,
                runtime.version,
                runtime.executable_path,
                runtime.architecture,
                runtime.binary_identity,
                runtime.status.as_str(),
                unix_timestamp_ms()?
            ],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn set_runtime_status(
    connection: &Connection,
    runtime_id: Uuid,
    status: RuntimeStatus,
) -> Result<(), StateError> {
    connection
        .execute(
            "UPDATE runtimes SET status = ?2 WHERE runtime_id = ?1",
            params![runtime_id.to_string(), status.as_str()],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn remove_runtime(connection: &Connection, runtime_id: Uuid) -> Result<(), StateError> {
    connection
        .execute(
            "DELETE FROM runtimes WHERE runtime_id = ?1",
            [runtime_id.to_string()],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn runtime_kind_from_str(value: &str) -> Option<RuntimeKind> {
    match value {
        "managed" => Some(RuntimeKind::Managed),
        "external" => Some(RuntimeKind::External),
        _ => None,
    }
}

fn list_projects(connection: &Connection) -> Result<Vec<ProjectRecord>, StateError> {
    let mut statement = connection
        .prepare(
            "SELECT project_id, kind, path, dir_volume_serial, dir_file_id, name, status, \
                    manifest_valid, manifest_reason \
             FROM projects ORDER BY name",
        )
        .map_err(StateError::Repository)?;
    let rows = statement
        .query_map([], map_project_row)
        .map_err(StateError::Repository)?;
    rows.collect::<Result<_, _>>()
        .map_err(StateError::Repository)
}

fn upsert_project(connection: &Connection, project: &ProjectRecord) -> Result<(), StateError> {
    connection
        .execute(
            "INSERT INTO projects (project_id, kind, path, dir_volume_serial, dir_file_id, \
                    name, status, manifest_valid, manifest_reason, registered_at_unix_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
             ON CONFLICT(project_id) DO UPDATE SET \
             kind = excluded.kind, \
             path = excluded.path, \
             dir_volume_serial = excluded.dir_volume_serial, \
             dir_file_id = excluded.dir_file_id, \
             name = excluded.name, \
             status = excluded.status, \
             manifest_valid = excluded.manifest_valid, \
             manifest_reason = excluded.manifest_reason",
            params![
                project.project_id.to_string(),
                project_kind_str(project.kind),
                project.path,
                project.dir_volume_serial as i64,
                project.dir_file_id as i64,
                project.name,
                project_status_str(project.status),
                i64::from(project.manifest_valid),
                project.manifest_reason,
                unix_timestamp_ms()?,
            ],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn remove_project(connection: &Connection, project_id: Uuid) -> Result<(), StateError> {
    connection
        .execute(
            "DELETE FROM projects WHERE project_id = ?1",
            [project_id.to_string()],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn list_routes(connection: &Connection) -> Result<Vec<RouteRow>, StateError> {
    let mut statement = connection
        .prepare("SELECT route_name, project_id, source FROM project_routes ORDER BY route_name")
        .map_err(StateError::Repository)?;
    let rows = statement
        .query_map([], |row| {
            Ok(RouteRow {
                route_name: row.get(0)?,
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?)
                    .map_err(|_| rusqlite::Error::InvalidColumnName("project_id".to_owned()))?,
                source: match row.get::<_, String>(2)?.as_str() {
                    "derived" => RouteSource::Derived,
                    _ => RouteSource::Explicit,
                },
            })
        })
        .map_err(StateError::Repository)?;
    rows.collect::<Result<_, _>>()
        .map_err(StateError::Repository)
}

fn set_route(
    connection: &Connection,
    route_name: &str,
    project_id: Uuid,
    source: RouteSource,
) -> Result<(), StateError> {
    connection
        .execute(
            "INSERT INTO project_routes (route_name, project_id, source) VALUES (?1, ?2, ?3) \
             ON CONFLICT(route_name) DO UPDATE SET \
             project_id = excluded.project_id, source = excluded.source",
            params![route_name, project_id.to_string(), route_source_str(source)],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn clear_routes_for_project(connection: &Connection, project_id: Uuid) -> Result<(), StateError> {
    connection
        .execute(
            "DELETE FROM project_routes WHERE project_id = ?1",
            [project_id.to_string()],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn get_trust(connection: &Connection, project_id: Uuid) -> Result<Option<TrustRecord>, StateError> {
    connection
        .query_row(
            "SELECT project_id, trust_kind, directory_volume_serial, directory_file_id, \
                    repository_identity, trusted_at_unix_ms \
             FROM project_trust WHERE project_id = ?1",
            [project_id.to_string()],
            |row| {
                let trusted_at: Option<i64> = row.get(5)?;
                Ok(TrustRecord {
                    project_id: Uuid::parse_str(&row.get::<_, String>(0)?)
                        .map_err(|_| rusqlite::Error::InvalidColumnName("project_id".to_owned()))?,
                    trust_kind: match row.get::<_, String>(1)?.as_str() {
                        "trusted" => TrustKind::Trusted,
                        _ => TrustKind::Untrusted,
                    },
                    directory_volume_serial: row.get::<_, i64>(2)?.unsigned_abs(),
                    directory_file_id: row.get::<_, i64>(3)?.unsigned_abs(),
                    repository_identity: row.get(4)?,
                    trusted_at_unix_ms: trusted_at.map(|value| value.unsigned_abs()),
                })
            },
        )
        .optional()
        .map_err(StateError::Repository)
}

fn bind_trust(connection: &Connection, trust: &TrustRecord) -> Result<(), StateError> {
    connection
        .execute(
            "INSERT INTO project_trust (project_id, trust_kind, directory_volume_serial, \
                    directory_file_id, repository_identity, trusted_at_unix_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(project_id) DO UPDATE SET \
             trust_kind = excluded.trust_kind, \
             directory_volume_serial = excluded.directory_volume_serial, \
             directory_file_id = excluded.directory_file_id, \
             repository_identity = excluded.repository_identity, \
             trusted_at_unix_ms = excluded.trusted_at_unix_ms",
            params![
                trust.project_id.to_string(),
                match trust.trust_kind {
                    TrustKind::Trusted => "trusted",
                    TrustKind::Untrusted => "untrusted",
                },
                trust.directory_volume_serial as i64,
                trust.directory_file_id as i64,
                trust.repository_identity,
                trust.trusted_at_unix_ms.map(|v| v as i64),
            ],
        )
        .map_err(StateError::Repository)?;
    Ok(())
}

fn map_project_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRecord> {
    Ok(ProjectRecord {
        project_id: Uuid::parse_str(&row.get::<_, String>(0)?)
            .map_err(|_| rusqlite::Error::InvalidColumnName("project_id".to_owned()))?,
        kind: match row.get::<_, String>(1)?.as_str() {
            "parked" => ProjectKind::Parked,
            _ => ProjectKind::Linked,
        },
        path: row.get(2)?,
        dir_volume_serial: row.get::<_, i64>(3)?.unsigned_abs(),
        dir_file_id: row.get::<_, i64>(4)?.unsigned_abs(),
        name: row.get(5)?,
        status: match row.get::<_, String>(6)?.as_str() {
            "untrusted" => ProjectStatus::Untrusted,
            "trusted" => ProjectStatus::Trusted,
            "conflict" => ProjectStatus::Conflict,
            "missing" => ProjectStatus::Missing,
            "replaced" => ProjectStatus::Replaced,
            _ => ProjectStatus::Unsupported,
        },
        manifest_valid: row.get::<_, i64>(7)? != 0,
        manifest_reason: row.get(8)?,
    })
}

const fn project_kind_str(kind: ProjectKind) -> &'static str {
    match kind {
        ProjectKind::Parked => "parked",
        ProjectKind::Linked => "linked",
    }
}

const fn project_status_str(status: ProjectStatus) -> &'static str {
    match status {
        ProjectStatus::Untrusted => "untrusted",
        ProjectStatus::Trusted => "trusted",
        ProjectStatus::Conflict => "conflict",
        ProjectStatus::Missing => "missing",
        ProjectStatus::Replaced => "replaced",
        ProjectStatus::Unsupported => "unsupported",
    }
}

const fn route_source_str(source: RouteSource) -> &'static str {
    match source {
        RouteSource::Derived => "derived",
        RouteSource::Explicit => "explicit",
    }
}

fn runtime_status_from_str(value: &str) -> Option<RuntimeStatus> {
    match value {
        "ready" => Some(RuntimeStatus::Ready),
        "degraded" => Some(RuntimeStatus::Degraded),
        _ => None,
    }
}

fn validate_key(value: &str) -> Result<(), StateError> {
    if value.is_empty() || value.len() > 128 || value.contains('\0') {
        Err(StateError::InvalidKey)
    } else {
        Ok(())
    }
}

fn unix_timestamp_ms() -> Result<i64, StateError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StateError::Clock)?
        .as_millis();
    i64::try_from(millis).map_err(|_| StateError::Clock)
}

#[derive(Debug)]
pub enum StateError {
    Open(rusqlite::Error),
    Configure(rusqlite::Error),
    UnsupportedVersion {
        found: u32,
        supported: u32,
    },
    Migration {
        version: u32,
        source: rusqlite::Error,
    },
    IntegrityQuery(rusqlite::Error),
    IntegrityViolation(StateIntegrityViolation),
    ForeignKeysUnavailable,
    Repository(rusqlite::Error),
    Shutdown(rusqlite::Error),
    ShutdownTimeout,
    WorkerSpawn(io::Error),
    WorkerStopped,
    WorkerPanicked,
    QueueFull,
    InvalidKey,
    InvalidJson,
    OperationNotRunning(Uuid),
    Clock,
}

impl StateError {
    pub fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Open(_) => "state_open_failed",
            Self::Configure(_) | Self::ForeignKeysUnavailable => "state_configuration_failed",
            Self::UnsupportedVersion { .. } => "state_schema_too_new",
            Self::Migration { .. } => "state_migration_failed",
            Self::IntegrityQuery(_) => "state_integrity_query_failed",
            Self::IntegrityViolation(_) => "state_integrity_failed",
            Self::Repository(_) => "state_repository_failed",
            Self::Shutdown(_) | Self::ShutdownTimeout => "state_shutdown_failed",
            Self::WorkerSpawn(_) | Self::WorkerStopped | Self::WorkerPanicked => {
                "state_worker_failed"
            }
            Self::QueueFull => "state_queue_full",
            Self::InvalidKey | Self::InvalidJson => "state_input_invalid",
            Self::OperationNotRunning(_) => "state_operation_not_running",
            Self::Clock => "state_clock_invalid",
        }
    }

    pub fn recovery_guidance(&self) -> &'static str {
        match self {
            Self::UnsupportedVersion { .. } => {
                "install a compatible newer Nerd version or restore a verified backup; the database was not modified"
            }
            Self::Migration { .. } => {
                "the migration was rolled back; preserve nerd.db and inspect daemon logs before retrying"
            }
            Self::IntegrityQuery(_) | Self::IntegrityViolation(_) => {
                "preserve nerd.db and restore only from a verified backup; do not delete the original"
            }
            Self::Open(_) | Self::Configure(_) | Self::ForeignKeysUnavailable => {
                "preserve nerd.db, verify disk access and free space, then inspect daemon logs"
            }
            _ => "inspect daemon logs and retry after the reported condition is resolved",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateIntegrityViolation {
    ApplicationId,
    SchemaVersion,
    MigrationLedger,
    SchemaObjects,
    SchemaDefinition,
    QuickCheck,
}

impl fmt::Display for StateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(_) => formatter.write_str("failed to open daemon state"),
            Self::Configure(_) => formatter.write_str("failed to configure daemon state"),
            Self::UnsupportedVersion { found, supported } => write!(
                formatter,
                "state schema {found} is newer than supported schema {supported}"
            ),
            Self::Migration { version, .. } => {
                write!(formatter, "state migration {version} failed")
            }
            Self::IntegrityQuery(_) => {
                formatter.write_str("state integrity check could not complete")
            }
            Self::IntegrityViolation(violation) => match violation {
                StateIntegrityViolation::ApplicationId => {
                    formatter.write_str("state database does not belong to Nerd")
                }
                StateIntegrityViolation::SchemaVersion => {
                    formatter.write_str("state schema version is inconsistent")
                }
                StateIntegrityViolation::MigrationLedger => {
                    formatter.write_str("state migration ledger is inconsistent")
                }
                StateIntegrityViolation::SchemaObjects => {
                    formatter.write_str("state database contains unexpected schema objects")
                }
                StateIntegrityViolation::SchemaDefinition => {
                    formatter.write_str("state schema definition is inconsistent")
                }
                StateIntegrityViolation::QuickCheck => {
                    formatter.write_str("state database failed SQLite quick check")
                }
            },
            Self::ForeignKeysUnavailable => {
                formatter.write_str("SQLite foreign-key enforcement is unavailable")
            }
            Self::Repository(_) => formatter.write_str("daemon state operation failed"),
            Self::Shutdown(_) => formatter.write_str("daemon state shutdown failed"),
            Self::ShutdownTimeout => formatter.write_str("nerd-state worker shutdown timed out"),
            Self::WorkerSpawn(_) => formatter.write_str("failed to start daemon state worker"),
            Self::WorkerStopped => formatter.write_str("daemon state worker stopped"),
            Self::WorkerPanicked => formatter.write_str("daemon state worker panicked"),
            Self::QueueFull => formatter.write_str("daemon state queue is full"),
            Self::InvalidKey => formatter.write_str("state key is invalid"),
            Self::InvalidJson => formatter.write_str("state value is not valid JSON"),
            Self::OperationNotRunning(id) => {
                write!(formatter, "operation {id} is not running")
            }
            Self::Clock => formatter.write_str("system clock cannot represent a UTC timestamp"),
        }
    }
}

impl std::error::Error for StateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Open(error)
            | Self::Configure(error)
            | Self::IntegrityQuery(error)
            | Self::Repository(error)
            | Self::Shutdown(error) => Some(error),
            Self::Migration { source, .. } => Some(source),
            Self::WorkerSpawn(error) => Some(error),
            Self::UnsupportedVersion { .. }
            | Self::IntegrityViolation(_)
            | Self::ForeignKeysUnavailable
            | Self::ShutdownTimeout
            | Self::WorkerStopped
            | Self::WorkerPanicked
            | Self::QueueFull
            | Self::InvalidKey
            | Self::InvalidJson
            | Self::OperationNotRunning(_)
            | Self::Clock => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use rusqlite::Connection;
    use uuid::Uuid;

    use super::{
        RuntimeKind, RuntimeRecord, RuntimeStatus, SUPPORTED_SCHEMA_VERSION, StateError, StateStore,
    };

    #[test]
    fn worker_migrates_serves_repositories_and_releases_database() {
        let fixture = TempFixture::new("state-worker");
        let database_path = fixture.path.join("state.db");
        let store = StateStore::open(&database_path).expect("open state store");
        let client = store.client();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        runtime.block_on(async {
            let health = client.health().await.expect("state health");
            assert_eq!(health.schema_version, SUPPORTED_SCHEMA_VERSION);
            assert!(health.foreign_keys_enabled);

            client
                .set_setting("theme".to_owned(), r#"{"mode":"dark"}"#.to_owned())
                .await
                .expect("set setting");
            assert_eq!(
                client
                    .get_setting("theme".to_owned())
                    .await
                    .expect("get setting")
                    .as_deref(),
                Some(r#"{"mode":"dark"}"#)
            );

            let operation_id = Uuid::new_v4();
            client
                .begin_operation(
                    operation_id,
                    "test_operation".to_owned(),
                    Some(r#"{"step":1}"#.to_owned()),
                )
                .await
                .expect("begin operation");
            client
                .finish_operation(operation_id, true)
                .await
                .expect("finish operation");
        });
        drop(runtime);
        store.shutdown().expect("shutdown state store");

        let moved_path = fixture.path.join("state-moved.db");
        fs::rename(&database_path, &moved_path).expect("database handle must be closed");
        fs::rename(&moved_path, &database_path).expect("restore database path");

        let connection = Connection::open(&database_path).expect("reopen state database");
        let operation_status: String = connection
            .query_row("SELECT status FROM operation_history", [], |row| row.get(0))
            .expect("read operation status");
        assert_eq!(operation_status, "succeeded");
    }

    #[test]
    fn repository_rejects_invalid_inputs_before_queueing() {
        let fixture = TempFixture::new("state-validation");
        let store = StateStore::open(&fixture.path.join("state.db")).expect("open state store");
        let client = store.client();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        runtime.block_on(async {
            assert!(matches!(
                client.set_setting(String::new(), "null".to_owned()).await,
                Err(StateError::InvalidKey)
            ));
            assert!(matches!(
                client
                    .set_setting("valid".to_owned(), "not-json".to_owned())
                    .await,
                Err(StateError::InvalidJson)
            ));
        });
        drop(runtime);
        store.shutdown().expect("shutdown state store");
    }

    #[test]
    fn runtime_repository_registers_lists_degrades_and_removes() {
        let fixture = TempFixture::new("state-runtimes");
        let store = StateStore::open(&fixture.path.join("state.db")).expect("open state store");
        let client = store.client();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        runtime.block_on(async {
            let runtime_id = Uuid::new_v4();
            let record = RuntimeRecord {
                runtime_id,
                kind: RuntimeKind::External,
                tool: "node".to_owned(),
                version: "20.11.1".to_owned(),
                executable_path: r"C:\node\node.exe".to_owned(),
                architecture: "x64".to_owned(),
                binary_identity: "abc123".to_owned(),
                status: RuntimeStatus::Ready,
            };
            client
                .register_runtime(&record)
                .await
                .expect("register runtime");

            let listed = client.list_runtimes().await.expect("list runtimes");
            assert_eq!(listed.len(), 1);
            assert_eq!(listed[0].runtime_id, runtime_id);
            assert_eq!(listed[0].version, "20.11.1");

            client
                .set_runtime_status(runtime_id, RuntimeStatus::Degraded)
                .await
                .expect("degrade runtime");
            let listed = client.list_runtimes().await.expect("list runtimes");
            assert_eq!(listed[0].status, RuntimeStatus::Degraded);

            client
                .remove_runtime(runtime_id)
                .await
                .expect("remove runtime");
            let listed = client.list_runtimes().await.expect("list runtimes");
            assert!(listed.is_empty());
        });
        drop(runtime);
        store.shutdown().expect("shutdown state store");
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
