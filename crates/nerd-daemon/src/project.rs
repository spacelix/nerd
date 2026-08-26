//! Project registration and reconciliation: park/unpark/link/unlink, name
//! derivation, conflict state, and replace-versus-rename identity handling.

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::{
    identity,
    location,
    paths::AppPaths,
    state::{
        ProjectKind, ProjectRecord, ProjectStatus, RouteSource, StateClient, TrustKind,
        TrustRecord,
    },
};

#[derive(Debug)]
pub enum ProjectError {
    State(crate::state::StateError),
    Location(crate::location::LocationError),
    Identity(crate::identity::IdentityError),
    Manifest(nerd_core::manifest::ManifestError),
    NotAProject(String),
    NotFound(String),
    Unsupported(String),
    Io(std::io::Error),
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::State(error) => error.fmt(formatter),
            Self::Location(error) => error.fmt(formatter),
            Self::Identity(error) => error.fmt(formatter),
            Self::Manifest(error) => error.fmt(formatter),
            Self::NotAProject(path) => write!(formatter, "'{path}' has no package.json"),
            Self::NotFound(name) => write!(formatter, "project '{name}' is not registered"),
            Self::Unsupported(message) => formatter.write_str(message),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ProjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::State(error) => Some(error),
            Self::Location(error) => Some(error),
            Self::Identity(error) => Some(error),
            Self::Manifest(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::NotAProject(_) | Self::NotFound(_) | Self::Unsupported(_) => None,
        }
    }
}

impl From<crate::state::StateError> for ProjectError {
    fn from(error: crate::state::StateError) -> Self {
        Self::State(error)
    }
}

impl From<crate::location::LocationError> for ProjectError {
    fn from(error: crate::location::LocationError) -> Self {
        Self::Location(error)
    }
}

impl From<crate::identity::IdentityError> for ProjectError {
    fn from(error: crate::identity::IdentityError) -> Self {
        Self::Identity(error)
    }
}

impl From<std::io::Error> for ProjectError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<nerd_core::manifest::ManifestError> for ProjectError {
    fn from(error: nerd_core::manifest::ManifestError) -> Self {
        Self::Manifest(error)
    }
}

pub struct ProjectService {
    paths: AppPaths,
    state: StateClient,
}

impl Clone for ProjectService {
    fn clone(&self) -> Self {
        Self {
            paths: self.paths.clone(),
            state: self.state.clone(),
        }
    }
}

fn load_manifest(project_dir: &Path) -> (bool, Option<String>) {
    match std::fs::read_to_string(project_dir.join("nerd.json")) {
        // Absent manifest is fine: defaults apply.
        Err(_) => (true, None),
        Ok(text) => match nerd_core::manifest::parse(&text) {
            Ok(_) => (true, None),
            Err(error) => (false, Some(error.to_string())),
        },
    }
}

impl ProjectService {
    pub fn new(paths: AppPaths, state: StateClient) -> Self {
        Self { paths, state }
    }

    /// Register a parked root. Empty parked roots remain registered.
    pub async fn park(&self, root: &Path) -> Result<PathBuf, ProjectError> {
        let canonical = root.canonicalize()?;
        if !canonical.is_dir() {
            return Err(ProjectError::Unsupported(format!(
                "'{}' is not a directory",
                root.display()
            )));
        }
        location::preflight(&canonical)?;
        let key = format!(
            "park.root.{}",
            canonical.to_string_lossy().to_ascii_lowercase()
        );
        self.state
            .set_setting(key, serde_json::json!({ "parked": true }).to_string())
            .await?;
        self.reconcile_parked_root(&canonical).await?;
        Ok(canonical)
    }

    /// Unpark removes the parked root and every parked child project under it.
    pub async fn unpark(&self, root: &Path) -> Result<usize, ProjectError> {
        let canonical = root.canonicalize()?;
        let prefix = format!("{}\\", canonical.to_string_lossy().to_ascii_lowercase());
        let mut removed = 0usize;
        for project in self.state.list_projects().await? {
            if project.kind == ProjectKind::Parked
                && project.path.to_ascii_lowercase().starts_with(&prefix)
            {
                self.state.clear_routes_for_project(project.project_id).await?;
                self.state.remove_project(project.project_id).await?;
                removed += 1;
            }
        }
        let key = format!(
            "park.root.{}",
            canonical.to_string_lossy().to_ascii_lowercase()
        );
        // Removing a missing setting is not an error.
        let _ = self.state.set_setting(key, "null".to_owned()).await;
        self.assign_routes().await?;
        Ok(removed)
    }

    /// Link one directory as a project after preflight and package.json check.
    pub async fn link(&self, path: &Path) -> Result<ProjectRecord, ProjectError> {
        let canonical = path.canonicalize()?;
        if !canonical.is_dir() {
            return Err(ProjectError::Unsupported(format!(
                "'{}' is not a directory",
                path.display()
            )));
        }
        location::preflight(&canonical)?;
        if !canonical.join("package.json").is_file() {
            return Err(ProjectError::NotAProject(
                canonical.to_string_lossy().into_owned(),
            ));
        }
        self.register_or_update(&canonical, ProjectKind::Linked)
            .await?
            .ok_or_else(|| ProjectError::NotAProject(canonical.to_string_lossy().into_owned()))
    }

    pub async fn unlink(&self, name_or_id: &str) -> Result<bool, ProjectError> {
        let Some(project) = self.find_project(name_or_id).await? else {
            return Ok(false);
        };
        if project.kind == ProjectKind::Parked {
            return Err(ProjectError::Unsupported(
                "unregister a discovered child by removing the directory or unparking its root"
                    .to_owned(),
            ));
        }
        self.state.clear_routes_for_project(project.project_id).await?;
        self.state.remove_project(project.project_id).await?;
        self.assign_routes().await?;
        Ok(true)
    }

    pub async fn list(&self) -> Result<Vec<ProjectRecord>, ProjectError> {
        Ok(self.state.list_projects().await?)
    }

    pub async fn detail(&self, name_or_id: &str) -> Result<ProjectRecord, ProjectError> {
        self.find_project(name_or_id)
            .await?
            .ok_or_else(|| ProjectError::NotFound(name_or_id.to_owned()))
    }

    /// Reconcile every immediate child of a parked root. Candidates become
    /// projects only when their root has package.json.
    pub async fn reconcile_parked_root(&self, root: &Path) -> Result<Vec<ProjectRecord>, ProjectError> {
        let mut reconciled = Vec::new();
        let entries = std::fs::read_dir(root)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if !path.join("package.json").is_file() {
                continue;
            }
            if let Some(record) = self.register_or_update(&path, ProjectKind::Parked).await? {
                reconciled.push(record);
            }
        }
        self.assign_routes().await?;
        // Re-park with 1 candidate minimum so empty roots stay registered.
        Ok(reconciled)
    }

    /// Register or update one project directory using identity semantics:
    /// same path + new identity = replaced; same identity + new path = rename.
    pub async fn register_or_update(
        &self,
        path: &Path,
        kind: ProjectKind,
    ) -> Result<Option<ProjectRecord>, ProjectError> {
        let canonical = path.canonicalize()?;
        if !canonical.is_dir() {
            return Ok(None);
        }
        let Ok(identity) = identity::identify(&canonical) else {
            return Ok(None);
        };

        let (manifest_valid, manifest_reason) = load_manifest(&canonical);
        let folder_name = canonical
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let derived_name = derive_name(&folder_name);
        let path_text = canonical.to_string_lossy().into_owned();

        let projects = self.state.list_projects().await?;
        let existing_same_path = projects
            .iter()
            .find(|p| p.path.eq_ignore_ascii_case(&path_text))
            .cloned();
        let existing_same_identity = projects
            .iter()
            .find(|p| {
                p.dir_volume_serial == identity.volume_serial
                    && p.dir_file_id == identity.file_id
                    && existing_same_path
                        .as_ref()
                        .is_none_or(|same| same.project_id != p.project_id)
            })
            .cloned();

        let project_id = match (&existing_same_path, &existing_same_identity) {
            (Some(old), _) if old.dir_volume_serial != identity.volume_serial || old.dir_file_id != identity.file_id => {
                // The old directory was replaced; this new directory gets a new row.
                Uuid::new_v4()
            }
            (_, Some(other)) => other.project_id,
            (Some(same), None) => same.project_id,
            _ => Uuid::new_v4(),
        };

        let name = match &existing_same_identity {
            Some(other) => other.name.clone(),
            None => derived_name,
        };
        // Renamed directories keep trust binding to their stable identity;
        // replaced directories start over as untrusted.
        let status = match &existing_same_path {
            Some(old)
                if old.dir_volume_serial != identity.volume_serial
                    || old.dir_file_id != identity.file_id =>
            {
                ProjectStatus::Replaced
            }
            _ => {
                let trusted = matches!(
                    self.state.get_trust(project_id).await?,
                    Some(trust) if trust.trust_kind == TrustKind::Trusted,
                );
                if trusted
                    && matches!(&existing_same_identity, Some(other) if other.project_id == project_id)
                {
                    ProjectStatus::Trusted
                } else {
                    ProjectStatus::Untrusted
                }
            }
        };

        let record = ProjectRecord {
            project_id,
            kind,
            path: path_text,
            dir_volume_serial: identity.volume_serial,
            dir_file_id: identity.file_id,
            name,
            status,
            manifest_valid,
            manifest_reason: manifest_reason.filter(|_| !manifest_valid),
        };
        self.state.upsert_project(&record).await?;
        Ok(Some(record))
    }

    /// Recompute routes deterministically: explicit wins, first-by-name keeps a
    /// unique derived route, conflicts mark both sides without any route.
    pub async fn assign_routes(&self) -> Result<(), ProjectError> {
        let projects = self.state.list_projects().await?;
        let routes = self.state.list_routes().await?;
        let explicit: Vec<&crate::state::RouteRow> =
            routes.iter().filter(|r| r.source == RouteSource::Explicit).collect();

        // Keep explicit rows only for still-registered projects.
        let mut desired: Vec<(String, Uuid, RouteSource)> = explicit
            .iter()
            .filter(|r| projects.iter().any(|p| p.project_id == r.project_id))
            .map(|r| (r.route_name.clone(), r.project_id, RouteSource::Explicit))
            .collect();

        let mut claimed: std::collections::BTreeMap<String, usize> = Default::default();
        for (route, _, _) in &desired {
            *claimed.entry(route.clone()).or_default() += 1;
        }
        for project in &projects {
            if !explicit.iter().any(|r| r.project_id == project.project_id) {
                *claimed.entry(project.name.clone()).or_default() += 1;
                desired.push((project.name.clone(), project.project_id, RouteSource::Derived));
            }
        }

        for project in &projects {
            self.state.clear_routes_for_project(project.project_id).await?;
        }

        for (route_name, owner, source) in desired {
            if claimed.get(&route_name).copied().unwrap_or(0) > 1 {
                for record in projects.iter().filter(|p| p.name.eq_ignore_ascii_case(&route_name)) {
                    let mut updated = record.clone();
                    if matches!(
                        updated.status,
                        ProjectStatus::Untrusted | ProjectStatus::Trusted
                    ) {
                        updated.status = ProjectStatus::Conflict;
                        self.state.upsert_project(&updated).await?;
                    }
                }
                continue;
            }
            self.state.set_route(route_name, owner, source).await?;
        }
        Ok(())
    }

    pub async fn set_explicit_route(
        &self,
        project_id: Uuid,
        route: &str,
    ) -> Result<(), ProjectError> {
        if !is_dns_label(route) {
            return Err(ProjectError::Unsupported(format!(
                "route '{route}' must be a lowercase DNS label"
            )));
        }
        self.state
            .set_route(route.to_owned(), project_id, RouteSource::Explicit)
            .await?;
        self.assign_routes().await?;
        Ok(())
    }

    /// Bind trust for a project against its current directory identity
    /// (ADR 002; OD-010 invalidation handled during reconciliation).
    pub async fn bind_trust(&self, project_id: Uuid) -> Result<(), ProjectError> {
        let Some(record) = self
            .state
            .list_projects()
            .await?
            .into_iter()
            .find(|p| p.project_id == project_id)
        else {
            return Err(ProjectError::NotFound(project_id.to_string()));
        };
        let repository_identity = read_git_origin(Path::new(&record.path));
        let trust = TrustRecord {
            project_id,
            trust_kind: TrustKind::Trusted,
            directory_volume_serial: record.dir_volume_serial,
            directory_file_id: record.dir_file_id,
            repository_identity,
            trusted_at_unix_ms: Some(unix_ms()),
        };
        self.state.bind_trust(&trust).await?;
        let mut updated = record;
        updated.status = ProjectStatus::Trusted;
        self.state.upsert_project(&updated).await?;
        Ok(())
    }

    async fn find_project(&self, name_or_id: &str) -> Result<Option<ProjectRecord>, ProjectError> {
        if let Ok(id) = Uuid::parse_str(name_or_id) {
            return Ok(self
                .state
                .list_projects()
                .await?
                .into_iter()
                .find(|p| p.project_id == id));
        }
        Ok(self
            .state
            .list_projects()
            .await?
            .into_iter()
            .find(|p| p.name == name_or_id.to_ascii_lowercase()))
    }
}

fn derive_name(folder_name: &str) -> String {
    let cleaned: String = folder_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('-');
    if trimmed.is_empty() {
        "project".to_owned()
    } else {
        trimmed.chars().take(63).collect()
    }
}

fn is_dns_label(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 63
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

/// Read-only git origin probe used as optional extra identity material.
fn read_git_origin(project_dir: &Path) -> Option<String> {
    let config = project_dir.join(".git").join("config");
    let text = std::fs::read_to_string(config).ok()?;
    let section = text.split("[remote \"origin\"]").nth(1)?;
    let line = section.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix("url").map(|rest| rest.trim_start_matches(['=', ' ']).trim().to_owned())
    })?;
    if line.is_empty() { None } else { Some(line) }
}

fn unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::derive_name;

    #[test]
    fn derives_dns_safe_names() {
        assert_eq!(derive_name("My App"), "my-app");
        assert_eq!(derive_name("app---v2!!"), "app-v2");
        assert_eq!(derive_name("!!"), "project");
    }
}
