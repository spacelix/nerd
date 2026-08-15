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

    use super::{SUPPORTED_SCHEMA_VERSION, StateError, StateStore};

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
