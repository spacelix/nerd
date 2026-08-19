//! Runtime (Node) wire types shared between daemon, IPC, and CLI.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeInfo {
    pub runtime_id: Uuid,
    pub kind: RuntimeKind,
    pub tool: String,
    pub version: String,
    pub executable_path: String,
    pub architecture: String,
    pub status: RuntimeStatus,
}

impl RuntimeInfo {
    pub fn kind_str(&self) -> &'static str {
        match self.kind {
            RuntimeKind::Managed => "managed",
            RuntimeKind::External => "external",
        }
    }

    pub fn status_str(&self) -> &'static str {
        match self.status {
            RuntimeStatus::Ready => "ready",
            RuntimeStatus::Degraded => "degraded",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Managed,
    External,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    Ready,
    Degraded,
}
