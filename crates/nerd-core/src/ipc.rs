use serde::{Deserialize, Deserializer, Serialize, de};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestEnvelope {
    #[serde(deserialize_with = "positive_u32")]
    pub protocol_version: u32,
    pub request_id: Uuid,
    pub request: Request,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Request {
    Handshake(HandshakeRequest),
    Status(StatusRequest),
    NetworkSetup(NetworkSetupRequest),
    NetworkUninstall(NetworkUninstallRequest),
    NetworkRepair(NetworkRepairRequest),
    NetworkStatus(NetworkStatusRequest),
    RuntimeInstall(RuntimeInstallRequest),
    RuntimeList(RuntimeListRequest),
    RuntimeRemove(RuntimeRemoveRequest),
    RuntimeSetDefault(RuntimeSetDefaultRequest),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HandshakeRequest {
    pub client_kind: ClientKind,
    #[serde(deserialize_with = "short_string")]
    pub client_version: String,
    #[serde(deserialize_with = "positive_u32")]
    pub minimum_protocol_version: u32,
    #[serde(deserialize_with = "positive_u32")]
    pub maximum_protocol_version: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Cli,
    Desktop,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatusRequest {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResponseEnvelope {
    #[serde(deserialize_with = "positive_u32")]
    pub protocol_version: u32,
    pub request_id: Uuid,
    pub response: Response,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Response {
    Handshake(HandshakeResponse),
    Status(StatusResponse),
    NetworkSetup(NetworkSetupResponse),
    NetworkUninstall(NetworkUninstallResponse),
    NetworkRepair(NetworkRepairResponse),
    NetworkStatus(NetworkStatusResponse),
    RuntimeInstall(RuntimeInstallResponse),
    RuntimeList(RuntimeListResponse),
    RuntimeRemove(RuntimeRemoveResponse),
    RuntimeSetDefault(RuntimeSetDefaultResponse),
    Error(ErrorResponse),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HandshakeResponse {
    pub daemon_instance_id: Uuid,
    #[serde(deserialize_with = "short_string")]
    pub application_version: String,
    #[serde(deserialize_with = "positive_u32")]
    pub selected_protocol_version: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StatusResponse {
    pub daemon: DaemonIdentity,
    pub health: DaemonHealth,
    pub paths: DataPaths,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ProcessResources>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DaemonIdentity {
    pub instance_id: Uuid,
    #[serde(deserialize_with = "positive_u32")]
    pub process_id: u32,
    #[serde(deserialize_with = "short_string")]
    pub application_version: String,
    #[serde(deserialize_with = "positive_u32")]
    pub protocol_version: u32,
    pub uptime_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DaemonHealth {
    pub status: HealthStatus,
    #[serde(deserialize_with = "health_components")]
    pub components: Vec<HealthComponent>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HealthComponent {
    pub component: HealthComponentName,
    pub status: HealthStatus,
    #[serde(
        default,
        deserialize_with = "optional_message",
        skip_serializing_if = "Option::is_none"
    )]
    pub message: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthComponentName {
    State,
    Logging,
    Ipc,
    Resources,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataPaths {
    #[serde(deserialize_with = "nonempty_string")]
    pub data_directory: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub database_path: String,
    #[serde(deserialize_with = "nonempty_string")]
    pub log_directory: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessResources {
    pub working_set_bytes: u64,
    pub peak_working_set_bytes: u64,
    pub private_usage_bytes: u64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkSetupRequest {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkSetupResponse {
    pub success: bool,
    pub rolled_back: bool,
    #[serde(
        default,
        deserialize_with = "optional_message",
        skip_serializing_if = "Option::is_none"
    )]
    pub nrpt_rule_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "optional_message",
        skip_serializing_if = "Option::is_none"
    )]
    pub ca_fingerprint: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkUninstallRequest {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkUninstallResponse {
    pub success: bool,
    pub removed_nrpt_rule: bool,
    pub removed_ca: bool,
    #[serde(deserialize_with = "nonnegative_u32")]
    pub preserved_unrelated_rules: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkRepairRequest {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkRepairResponse {
    pub success: bool,
    #[serde(deserialize_with = "nonempty_string")]
    pub action: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkStatusRequest {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkStatusResponse {
    pub dns_listener_active: bool,
    pub nrpt_rule_present: bool,
    pub ca_present: bool,
    #[serde(
        default,
        deserialize_with = "optional_port_conflict",
        skip_serializing_if = "Option::is_none"
    )]
    pub port_53_conflict: Option<PortConflict>,
    #[serde(
        default,
        deserialize_with = "optional_port_conflict",
        skip_serializing_if = "Option::is_none"
    )]
    pub port_80_conflict: Option<PortConflict>,
    #[serde(
        default,
        deserialize_with = "optional_port_conflict",
        skip_serializing_if = "Option::is_none"
    )]
    pub port_443_conflict: Option<PortConflict>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortConflict {
    #[serde(deserialize_with = "positive_u16")]
    pub port: u16,
    #[serde(deserialize_with = "nonempty_string")]
    pub protocol: String,
    #[serde(deserialize_with = "positive_u32")]
    pub owning_process_id: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeInstallRequest {
    #[serde(deserialize_with = "nonempty_string")]
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeInstallResponse {
    pub installed: bool,
    pub version: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeListRequest {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeListResponse {
    pub runtimes: Vec<crate::runtime::RuntimeInfo>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeRemoveRequest {
    pub runtime_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeRemoveResponse {
    pub removed: bool,
    pub was_managed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeSetDefaultRequest {
    #[serde(deserialize_with = "nonempty_string")]
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeSetDefaultResponse {
    pub version: String,
}

fn positive_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value = u16::deserialize(deserializer)?;
    if value == 0 {
        Err(de::Error::custom("value must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn nonnegative_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: de::Deserializer<'de>,
{
    u32::deserialize(deserializer)
}

fn optional_port_conflict<'de, D>(deserializer: D) -> Result<Option<PortConflict>, D::Error>
where
    D: de::Deserializer<'de>,
{
    Option::<PortConflict>::deserialize(deserializer)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "ErrorResponseWire")]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_protocol_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_protocol_version: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ErrorResponseWire {
    code: ErrorCode,
    #[serde(deserialize_with = "message_string")]
    message: String,
    retryable: bool,
    #[serde(default, deserialize_with = "optional_positive_u32")]
    minimum_protocol_version: Option<u32>,
    #[serde(default, deserialize_with = "optional_positive_u32")]
    maximum_protocol_version: Option<u32>,
}

impl TryFrom<ErrorResponseWire> for ErrorResponse {
    type Error = String;

    fn try_from(value: ErrorResponseWire) -> Result<Self, Self::Error> {
        let has_protocol_range =
            value.minimum_protocol_version.is_some() && value.maximum_protocol_version.is_some();
        if value.code == ErrorCode::ProtocolMismatch && !has_protocol_range {
            return Err("protocol mismatch errors require a complete protocol range".to_owned());
        }
        if value.code != ErrorCode::ProtocolMismatch
            && (value.minimum_protocol_version.is_some()
                || value.maximum_protocol_version.is_some())
        {
            return Err("only protocol mismatch errors may include a protocol range".to_owned());
        }
        Ok(Self {
            code: value.code,
            message: value.message,
            retryable: value.retryable,
            minimum_protocol_version: value.minimum_protocol_version,
            maximum_protocol_version: value.maximum_protocol_version,
        })
    }
}

impl ErrorResponse {
    pub fn protocol_mismatch() -> Self {
        Self {
            code: ErrorCode::ProtocolMismatch,
            message: "client and daemon IPC protocol versions are incompatible".to_owned(),
            retryable: false,
            minimum_protocol_version: Some(crate::IPC_PROTOCOL_VERSION),
            maximum_protocol_version: Some(crate::IPC_PROTOCOL_VERSION),
        }
    }

    pub fn new(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            retryable,
            minimum_protocol_version: None,
            maximum_protocol_version: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ProtocolMismatch,
    HandshakeRequired,
    InvalidRequest,
    DaemonUnhealthy,
    ShuttingDown,
    Internal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventEnvelope {
    #[serde(deserialize_with = "positive_u32")]
    pub protocol_version: u32,
    pub operation_id: Uuid,
    #[serde(deserialize_with = "positive_u64")]
    pub sequence: u64,
    pub event: Event,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "type",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Event {
    Progress(ProgressEvent),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProgressEvent {
    #[serde(deserialize_with = "stage_string")]
    pub stage: String,
    #[serde(deserialize_with = "message_string")]
    pub message: String,
    pub cancellable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_units: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_units: Option<u64>,
}

fn positive_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u32::deserialize(deserializer)?;
    if value == 0 {
        Err(de::Error::custom("value must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn optional_positive_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<u32>::deserialize(deserializer)?;
    if value == Some(0) {
        Err(de::Error::custom("value must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn positive_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;
    if value == 0 {
        Err(de::Error::custom("value must be greater than zero"))
    } else {
        Ok(value)
    }
}

fn short_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    bounded_string(deserializer, 64)
}

fn stage_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    bounded_string(deserializer, 128)
}

fn message_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    bounded_string(deserializer, 256)
}

fn nonempty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() {
        Err(de::Error::custom("string must not be empty"))
    } else {
        Ok(value)
    }
}

fn bounded_string<'de, D>(deserializer: D, maximum: usize) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.is_empty() || value.chars().count() > maximum || value.contains('\0') {
        Err(de::Error::custom(format!(
            "string length must be between 1 and {maximum} characters"
        )))
    } else {
        Ok(value)
    }
}

fn optional_message<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if let Some(message) = &value
        && (message.is_empty() || message.chars().count() > 256)
    {
        return Err(de::Error::custom(
            "message length must be between 1 and 256 characters",
        ));
    }
    Ok(value)
}

fn health_components<'de, D>(deserializer: D) -> Result<Vec<HealthComponent>, D::Error>
where
    D: Deserializer<'de>,
{
    let components = Vec::<HealthComponent>::deserialize(deserializer)?;
    if components.len() != 4 {
        Err(de::Error::custom(
            "health must contain exactly four components",
        ))
    } else {
        Ok(components)
    }
}
