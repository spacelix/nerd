//! Trust and Start preflight: assemble the final command, runtime, package
//! manager, port, working directory, and environment provenance for explicit
//! user approval before any process is launched.

use std::path::PathBuf;

use uuid::Uuid;

use crate::{
    framework::{Framework, PortKind, dev_command},
    node::NodeManager,
    state::{ProjectRecord, ProjectStatus, StateClient},
    version::{VersionSpec, parse_spec},
};

#[derive(Clone, Debug)]
pub struct Preflight {
    pub project_id: Uuid,
    pub project_name: String,
    pub status: ProjectStatus,
    pub framework: Option<Framework>,
    pub node_version: Option<String>,
    pub node_source: Option<String>,
    pub node_exe: Option<PathBuf>,
    pub package_manager: Option<String>,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub port: u16,
    pub port_kind: PortKind,
    pub env_provenance: Vec<(String, String)>,
    pub untrusted: bool,
    pub material_change: bool,
}

#[derive(Debug)]
pub enum PreflightError {
    NotFound(String),
    MissingManifest(String),
    RuntimeUnavailable(String),
    Degraded(String),
    NoCommand(String),
    State(crate::state::StateError),
    Io(std::io::Error),
}

impl std::fmt::Display for PreflightError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(formatter, "project '{name}' is not registered"),
            Self::MissingManifest(name) => {
                write!(formatter, "project '{name}' has an invalid nerd.json")
            }
            Self::RuntimeUnavailable(message) => {
                write!(formatter, "runtime unavailable: {message}")
            }
            Self::Degraded(message) => write!(formatter, "runtime degraded: {message}"),
            Self::NoCommand(name) => write!(formatter, "project '{name}' has no dev script"),
            Self::State(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for PreflightError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::State(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<crate::state::StateError> for PreflightError {
    fn from(error: crate::state::StateError) -> Self {
        Self::State(error)
    }
}

impl From<std::io::Error> for PreflightError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct PreflightService {
    state: StateClient,
    node: NodeManager,
}

impl Clone for PreflightService {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            node: self.node.clone(),
        }
    }
}

impl PreflightService {
    pub fn new(state: StateClient, node: NodeManager) -> Self {
        Self { state, node }
    }

    /// Build a preflight for a project by name. Resolves runtime (re-probing
    /// external runtimes), package manager, framework, command, and port.
    pub async fn build(&self, name: &str, port: u16) -> Result<Preflight, PreflightError> {
        let Some(record) = self
            .state
            .list_projects()
            .await?
            .into_iter()
            .find(|p| p.name == name.to_ascii_lowercase())
        else {
            return Err(PreflightError::NotFound(name.to_owned()));
        };
        if !record.manifest_valid {
            return Err(PreflightError::MissingManifest(name.to_owned()));
        }
        let untrusted = !matches!(record.status, ProjectStatus::Trusted);

        let project_dir = PathBuf::from(&record.path);
        let package_json = std::fs::read_to_string(project_dir.join("package.json"))
            .map_err(|_| PreflightError::NoCommand(name.to_owned()))?;
        let scripts = parse_scripts(&package_json);
        let framework = crate::framework::detect(&package_json, scripts.as_str());
        let dev_script = scripts
            .lines()
            .find_map(|line| line.split_once(':').map(|(_, v)| v.trim().to_owned()))
            .unwrap_or_default();

        // Resolve runtime from nerd.json node field or default; re-probe.
        let (node_version, node_source, node_exe) = self.resolve_runtime(&record).await?;

        // Package manager from packageManager field when present.
        let package_manager = crate::package_manager::parse_package_manager(&package_json)
            .map(|pm| format!("{}@{}", pm.name, pm.version));

        let (command, args, port_kind) = match framework {
            Some(framework) => {
                let script = dev_script.clone();
                dev_command(
                    framework,
                    if script.is_empty() { "dev" } else { &script },
                    port,
                )
            }
            None => ("node".to_owned(), Vec::new(), PortKind::Env),
        };

        Ok(Preflight {
            project_id: record.project_id,
            project_name: record.name,
            status: record.status,
            framework,
            node_version: Some(node_version),
            node_source: Some(node_source),
            node_exe: Some(node_exe),
            package_manager,
            command,
            args,
            working_dir: project_dir,
            port,
            port_kind,
            env_provenance: vec![],
            untrusted,
            material_change: false,
        })
    }

    /// Resolve the concrete runtime (managed or external) for a project and
    /// re-probe it before launch.
    async fn resolve_runtime(
        &self,
        record: &ProjectRecord,
    ) -> Result<(String, String, std::path::PathBuf), PreflightError> {
        let runtimes = self.state.list_runtimes().await?;
        let spec = read_node_spec(record);
        let has_spec = spec.is_some();
        let chosen = if let Some(spec) = spec {
            let resolved = self
                .node
                .resolve(&spec)
                .await
                .map_err(|error| PreflightError::RuntimeUnavailable(error.to_string()))?;
            runtimes
                .iter()
                .find(|r| r.tool == "node" && r.version == resolved)
                .cloned()
                .ok_or_else(|| {
                    PreflightError::RuntimeUnavailable(format!(
                        "version {resolved} is not installed"
                    ))
                })?
        } else {
            runtimes
                .iter()
                .filter(|r| r.tool == "node")
                .max_by_key(|r| {
                    crate::version::compare_versions(&r.version, &r.version)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .cloned()
                .ok_or_else(|| {
                    PreflightError::RuntimeUnavailable("no Node runtime installed".to_owned())
                })?
        };

        // Re-probe external runtimes per Feature 03 rules.
        if chosen.kind == crate::state::RuntimeKind::External {
            let status = self
                .node
                .re_probe(&chosen)
                .await
                .map_err(|error| PreflightError::Degraded(error.to_string()))?;
            if status != crate::state::RuntimeStatus::Ready {
                return Err(PreflightError::Degraded(format!(
                    "external Node {} changed or is missing",
                    chosen.version
                )));
            }
        }

        let source = if has_spec { "nerd.json" } else { "default" };
        Ok((
            chosen.version,
            source.to_owned(),
            std::path::PathBuf::from(&chosen.executable_path),
        ))
    }

    /// Whether this preflight is safe to launch without re-approval.
    pub fn needs_approval(&self, preflight: &Preflight) -> bool {
        preflight.untrusted || preflight.material_change
    }
}

fn read_node_spec(record: &ProjectRecord) -> Option<VersionSpec> {
    let manifest_text =
        std::fs::read_to_string(std::path::PathBuf::from(&record.path).join("nerd.json")).ok()?;
    let manifest = nerd_core::manifest::parse(&manifest_text).ok()?;
    manifest.node.as_deref().and_then(parse_spec)
}

fn parse_scripts(package_json: &str) -> String {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(package_json) else {
        return String::new();
    };
    value
        .get("scripts")
        .and_then(|v| v.as_object())
        .map(|scripts| {
            scripts
                .iter()
                .map(|(k, v)| format!("{k}: {}", v.as_str().unwrap_or_default()))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}
