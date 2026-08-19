use rusqlite::{Connection, TransactionBehavior, params};

use super::{StateError, StateIntegrityViolation, unix_timestamp_ms};

pub const SUPPORTED_SCHEMA_VERSION: u32 = 2;
pub const APPLICATION_ID: u32 = 0x4E45_5244;

const SCHEMA_MIGRATIONS_SQL: &str = r#"
    CREATE TABLE schema_migrations (
        migration_id INTEGER PRIMARY KEY CHECK (migration_id > 0),
        name TEXT NOT NULL UNIQUE,
        fingerprint TEXT NOT NULL CHECK (length(fingerprint) = 16),
        applied_at_unix_ms INTEGER NOT NULL CHECK (applied_at_unix_ms >= 0)
    );
"#;

const GLOBAL_SETTINGS_SQL: &str = r#"
    CREATE TABLE global_settings (
        setting_key TEXT PRIMARY KEY CHECK (
            length(setting_key) BETWEEN 1 AND 128 AND instr(setting_key, char(0)) = 0
        ),
        value_json TEXT NOT NULL CHECK (json_valid(value_json)),
        updated_at_unix_ms INTEGER NOT NULL CHECK (updated_at_unix_ms >= 0)
    ) WITHOUT ROWID;
"#;

const OPERATION_HISTORY_SQL: &str = r#"
    CREATE TABLE operation_history (
        operation_id TEXT PRIMARY KEY CHECK (length(operation_id) = 36),
        operation_type TEXT NOT NULL CHECK (length(operation_type) BETWEEN 1 AND 128),
        status TEXT NOT NULL CHECK (status IN ('running', 'succeeded', 'failed', 'cancelled')),
        started_at_unix_ms INTEGER NOT NULL CHECK (started_at_unix_ms >= 0),
        finished_at_unix_ms INTEGER,
        recovery_state_json TEXT CHECK (
            recovery_state_json IS NULL OR json_valid(recovery_state_json)
        ),
        CHECK (
            (status = 'running' AND finished_at_unix_ms IS NULL) OR
            (status != 'running' AND finished_at_unix_ms IS NOT NULL)
        )
    ) WITHOUT ROWID;
"#;

const ARTIFACT_INVENTORY_SQL: &str = r#"
    CREATE TABLE artifact_inventory (
        artifact_id TEXT PRIMARY KEY CHECK (length(artifact_id) = 36),
        artifact_kind TEXT NOT NULL CHECK (length(artifact_kind) BETWEEN 1 AND 64),
        metadata_json TEXT NOT NULL CHECK (json_valid(metadata_json)),
        recorded_at_unix_ms INTEGER NOT NULL CHECK (recorded_at_unix_ms >= 0)
    ) WITHOUT ROWID;
"#;

const PROJECT_REGISTRY_SQL: &str = r#"
    CREATE TABLE project_registry (
        project_id TEXT PRIMARY KEY CHECK (length(project_id) = 36),
        metadata_json TEXT NOT NULL CHECK (json_valid(metadata_json)),
        registered_at_unix_ms INTEGER NOT NULL CHECK (registered_at_unix_ms >= 0)
    ) WITHOUT ROWID;
"#;

const RUNTIMES_SQL: &str = r#"
    CREATE TABLE runtimes (
        runtime_id TEXT PRIMARY KEY CHECK (length(runtime_id) = 36),
        kind TEXT NOT NULL CHECK (kind IN ('managed', 'external')),
        tool TEXT NOT NULL CHECK (tool = 'node'),
        version TEXT NOT NULL CHECK (length(version) BETWEEN 1 AND 32),
        executable_path TEXT NOT NULL CHECK (length(executable_path) BETWEEN 1 AND 1024),
        architecture TEXT NOT NULL CHECK (architecture IN ('x64', 'arm64')),
        binary_identity TEXT NOT NULL CHECK (length(binary_identity) BETWEEN 1 AND 256),
        status TEXT NOT NULL CHECK (status IN ('ready', 'degraded')),
        recorded_at_unix_ms INTEGER NOT NULL CHECK (recorded_at_unix_ms >= 0)
    ) WITHOUT ROWID;
"#;

const FOUNDATION_STATEMENTS: &[&str] = &[
    SCHEMA_MIGRATIONS_SQL,
    GLOBAL_SETTINGS_SQL,
    OPERATION_HISTORY_SQL,
    ARTIFACT_INVENTORY_SQL,
    PROJECT_REGISTRY_SQL,
];

const RUNTIMES_STATEMENTS: &[&str] = &[RUNTIMES_SQL];

const EXPECTED_TABLES: &[(&str, &str)] = &[
    ("artifact_inventory", ARTIFACT_INVENTORY_SQL),
    ("global_settings", GLOBAL_SETTINGS_SQL),
    ("operation_history", OPERATION_HISTORY_SQL),
    ("project_registry", PROJECT_REGISTRY_SQL),
    ("runtimes", RUNTIMES_SQL),
    ("schema_migrations", SCHEMA_MIGRATIONS_SQL),
];

struct Migration {
    version: u32,
    name: &'static str,
    statements: &'static [&'static str],
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "foundation_state",
        statements: FOUNDATION_STATEMENTS,
    },
    Migration {
        version: 2,
        name: "node_runtimes",
        statements: RUNTIMES_STATEMENTS,
    },
];

pub(super) fn migrate(connection: &mut Connection) -> Result<(), StateError> {
    migrate_with(connection, MIGRATIONS, SUPPORTED_SCHEMA_VERSION)
}

pub(super) fn validate_integrity(connection: &Connection) -> Result<(), StateError> {
    let check: String = connection
        .query_row("PRAGMA quick_check(1)", [], |row| row.get(0))
        .map_err(StateError::IntegrityQuery)?;
    if check != "ok" {
        return Err(StateError::IntegrityViolation(
            StateIntegrityViolation::QuickCheck,
        ));
    }

    let application_id: u32 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(StateError::IntegrityQuery)?;
    if application_id != APPLICATION_ID {
        return Err(StateError::IntegrityViolation(
            StateIntegrityViolation::ApplicationId,
        ));
    }

    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(StateError::IntegrityQuery)?;
    if version != SUPPORTED_SCHEMA_VERSION {
        return Err(StateError::IntegrityViolation(
            StateIntegrityViolation::SchemaVersion,
        ));
    }

    validate_migration_ledger(connection)?;
    validate_schema_objects(connection)?;
    Ok(())
}

fn migrate_with(
    connection: &mut Connection,
    migrations: &[Migration],
    supported_version: u32,
) -> Result<(), StateError> {
    let current_version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(StateError::Configure)?;
    if current_version > supported_version {
        return Err(StateError::UnsupportedVersion {
            found: current_version,
            supported: supported_version,
        });
    }
    validate_pre_migration_identity(connection, current_version)?;

    for migration in migrations
        .iter()
        .filter(|migration| migration.version > current_version)
    {
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StateError::Migration {
                version: migration.version,
                source,
            })?;
        transaction
            .pragma_update(None, "application_id", APPLICATION_ID)
            .map_err(|source| StateError::Migration {
                version: migration.version,
                source,
            })?;
        for statement in migration.statements {
            transaction
                .execute_batch(statement)
                .map_err(|source| StateError::Migration {
                    version: migration.version,
                    source,
                })?;
        }
        transaction
            .execute(
                "INSERT INTO schema_migrations \
                 (migration_id, name, fingerprint, applied_at_unix_ms) \
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    migration.version,
                    migration.name,
                    migration_fingerprint(migration),
                    unix_timestamp_ms()?
                ],
            )
            .map_err(|source| StateError::Migration {
                version: migration.version,
                source,
            })?;
        transaction
            .pragma_update(None, "user_version", migration.version)
            .map_err(|source| StateError::Migration {
                version: migration.version,
                source,
            })?;
        transaction
            .commit()
            .map_err(|source| StateError::Migration {
                version: migration.version,
                source,
            })?;
    }

    if std::ptr::eq(migrations, MIGRATIONS) {
        validate_integrity(connection)?;
    }
    Ok(())
}

fn validate_pre_migration_identity(
    connection: &Connection,
    current_version: u32,
) -> Result<(), StateError> {
    let application_id: u32 = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(StateError::Configure)?;
    let user_object_count: u32 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(StateError::Configure)?;

    if current_version == 0 && application_id == 0 && user_object_count == 0 {
        return Ok(());
    }
    if application_id != APPLICATION_ID {
        return Err(StateError::IntegrityViolation(
            StateIntegrityViolation::ApplicationId,
        ));
    }
    if current_version == 0 && user_object_count != 0 {
        return Err(StateError::IntegrityViolation(
            StateIntegrityViolation::SchemaObjects,
        ));
    }
    Ok(())
}

fn validate_migration_ledger(connection: &Connection) -> Result<(), StateError> {
    let mut statement = connection
        .prepare(
            "SELECT migration_id, name, fingerprint \
             FROM schema_migrations ORDER BY migration_id",
        )
        .map_err(StateError::IntegrityQuery)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(StateError::IntegrityQuery)?;
    let actual: Vec<_> = rows
        .collect::<Result<_, _>>()
        .map_err(StateError::IntegrityQuery)?;
    let expected: Vec<_> = MIGRATIONS
        .iter()
        .map(|migration| {
            (
                migration.version,
                migration.name.to_owned(),
                migration_fingerprint(migration),
            )
        })
        .collect();
    if actual != expected {
        return Err(StateError::IntegrityViolation(
            StateIntegrityViolation::MigrationLedger,
        ));
    }
    Ok(())
}

fn validate_schema_objects(connection: &Connection) -> Result<(), StateError> {
    let mut statement = connection
        .prepare(
            "SELECT type, name, sql FROM sqlite_schema \
             WHERE name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .map_err(StateError::IntegrityQuery)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(StateError::IntegrityQuery)?;
    let actual: Vec<_> = rows
        .collect::<Result<_, _>>()
        .map_err(StateError::IntegrityQuery)?;

    if actual.len() != EXPECTED_TABLES.len()
        || actual.iter().zip(EXPECTED_TABLES).any(
            |((actual_type, actual_name, actual_sql), (expected_name, _))| {
                actual_type != "table" || actual_name != expected_name || actual_sql.is_empty()
            },
        )
    {
        return Err(StateError::IntegrityViolation(
            StateIntegrityViolation::SchemaObjects,
        ));
    }

    for ((_, _, actual_sql), (_, expected_sql)) in actual.iter().zip(EXPECTED_TABLES) {
        if normalize_sql(actual_sql) != normalize_sql(expected_sql) {
            return Err(StateError::IntegrityViolation(
                StateIntegrityViolation::SchemaDefinition,
            ));
        }
    }
    Ok(())
}

fn migration_fingerprint(migration: &Migration) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    hash_bytes(&mut hash, &migration.version.to_le_bytes());
    hash_bytes(&mut hash, migration.name.as_bytes());
    for statement in migration.statements {
        hash_bytes(&mut hash, &[0]);
        hash_bytes(&mut hash, statement.as_bytes());
    }
    format!("{hash:016x}")
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
}

fn normalize_sql(sql: &str) -> String {
    sql.trim()
        .trim_end_matches(';')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{APPLICATION_ID, Migration, migrate, migrate_with, validate_integrity};
    use crate::state::{StateError, StateIntegrityViolation};

    #[test]
    fn failed_migration_rolls_back_schema_identity_and_version() {
        let mut connection = Connection::open_in_memory().expect("open SQLite");
        let statements = &[
            "CREATE TABLE rollback_probe (id INTEGER);",
            "THIS IS NOT SQL;",
        ];
        let migrations = [Migration {
            version: 1,
            name: "injected_failure",
            statements,
        }];

        let error = migrate_with(&mut connection, &migrations, 1)
            .expect_err("injected migration must fail");
        assert!(matches!(error, StateError::Migration { version: 1, .. }));

        let table_count: u32 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'rollback_probe'",
                [],
                |row| row.get(0),
            )
            .expect("query schema");
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read version");
        let application_id: u32 = connection
            .pragma_query_value(None, "application_id", |row| row.get(0))
            .expect("read application ID");
        assert_eq!(table_count, 0);
        assert_eq!(version, 0);
        assert_eq!(application_id, 0);
    }

    #[test]
    fn newer_schema_is_rejected_before_mutation() {
        let mut connection = Connection::open_in_memory().expect("open SQLite");
        connection
            .pragma_update(None, "user_version", 2)
            .expect("set newer version");

        let error =
            migrate_with(&mut connection, &[], 1).expect_err("newer schema must be rejected");
        assert!(matches!(
            error,
            StateError::UnsupportedVersion {
                found: 2,
                supported: 1
            }
        ));
    }

    #[test]
    fn foreign_database_is_rejected_before_mutation() {
        let mut connection = Connection::open_in_memory().expect("open SQLite");
        connection
            .execute_batch("CREATE TABLE foreign_data (id INTEGER);")
            .expect("create foreign table");

        let error = migrate(&mut connection).expect_err("foreign database must fail");
        assert!(matches!(
            error,
            StateError::IntegrityViolation(StateIntegrityViolation::ApplicationId)
        ));
        let application_id: u32 = connection
            .pragma_query_value(None, "application_id", |row| row.get(0))
            .expect("read application ID");
        assert_eq!(application_id, 0);
    }

    #[test]
    fn altered_ledger_and_schema_are_detected() {
        let mut connection = Connection::open_in_memory().expect("open SQLite");
        migrate(&mut connection).expect("migrate state");
        assert_eq!(
            connection
                .pragma_query_value(None, "application_id", |row| row.get::<_, u32>(0))
                .expect("application ID"),
            APPLICATION_ID
        );

        connection
            .execute(
                "UPDATE schema_migrations SET fingerprint = '0000000000000000'",
                [],
            )
            .expect("alter ledger");
        assert!(matches!(
            validate_integrity(&connection),
            Err(StateError::IntegrityViolation(
                StateIntegrityViolation::MigrationLedger
            ))
        ));

        for migration in super::MIGRATIONS {
            connection
                .execute(
                    "UPDATE schema_migrations SET fingerprint = ?1 WHERE migration_id = ?2",
                    rusqlite::params![super::migration_fingerprint(migration), migration.version],
                )
                .expect("restore ledger");
        }
        connection
            .execute_batch("ALTER TABLE global_settings ADD COLUMN injected TEXT;")
            .expect("alter schema");
        assert!(matches!(
            validate_integrity(&connection),
            Err(StateError::IntegrityViolation(
                StateIntegrityViolation::SchemaDefinition
            ))
        ));

        connection
            .execute_batch(
                "CREATE TRIGGER injected_trigger AFTER INSERT ON global_settings BEGIN SELECT 1; END;",
            )
            .expect("add unexpected trigger");
        assert!(matches!(
            validate_integrity(&connection),
            Err(StateError::IntegrityViolation(
                StateIntegrityViolation::SchemaObjects
            ))
        ));
    }
}
