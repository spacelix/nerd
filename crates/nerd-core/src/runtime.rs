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

/// Wire shape of a registered project.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectInfo {
    pub project_id: Uuid,
    pub kind: ProjectKind,
    pub path: String,
    pub name: String,
    pub status: ProjectStatus,
    pub manifest_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_name: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Parked,
    Linked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Untrusted,
    Trusted,
    Conflict,
    Missing,
    Replaced,
    Unsupported,
}

impl ProjectKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Parked => "parked",
            Self::Linked => "linked",
        }
    }
}

impl ProjectStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Untrusted => "untrusted",
            Self::Trusted => "trusted",
            Self::Conflict => "conflict",
            Self::Missing => "missing",
            Self::Replaced => "replaced",
            Self::Unsupported => "unsupported",
        }
    }
}
