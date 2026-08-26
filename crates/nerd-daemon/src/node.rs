//! Node runtime management: inventory, discovery, install/uninstall, and
//! version resolution against the official Node release index.

use std::{
    fmt,
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
};

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    paths::AppPaths,
    state::{RuntimeKind, RuntimeRecord, RuntimeStatus, StateClient},
    version::VersionSpec,
};

const NODE_INDEX_URL: &str = "https://nodejs.org/dist/index.json";
const NODE_BASE_URL: &str = "https://nodejs.org/dist";

#[derive(Debug)]
pub enum NodeError {
    State(crate::state::StateError),
    Io(io::Error),
    Http(String),
    Checksum,
    Traversal(String),
    Unsupported(String),
    NotFound(String),
    Degraded(String),
    Join(String),
}

impl fmt::Display for NodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
            Self::Http(message) => write!(formatter, "download failed: {message}"),
            Self::Checksum => {
                formatter.write_str("archive checksum did not match the official value")
            }
            Self::Traversal(path) => {
                write!(
                    formatter,
                    "archive entry escapes the staging directory: {path}"
                )
            }
            Self::Unsupported(message) => formatter.write_str(message),
            Self::NotFound(message) => write!(formatter, "no matching Node version: {message}"),
            Self::Degraded(message) => write!(formatter, "runtime is degraded: {message}"),
            Self::Join(message) => write!(formatter, "worker task failed: {message}"),
        }
    }
}

impl std::error::Error for NodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::State(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Http(_)
            | Self::Checksum
            | Self::Traversal(_)
            | Self::Unsupported(_)
            | Self::NotFound(_)
            | Self::Degraded(_)
            | Self::Join(_) => None,
        }
    }
}

impl From<crate::state::StateError> for NodeError {
    fn from(error: crate::state::StateError) -> Self {
        Self::State(error)
    }
}

impl From<io::Error> for NodeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct NodeManager {
    paths: AppPaths,
    state: StateClient,
    client: reqwest::Client,
}

impl Clone for NodeManager {
    fn clone(&self) -> Self {
        Self {
            paths: self.paths.clone(),
            state: self.state.clone(),
            client: self.client.clone(),
        }
    }
}

impl NodeManager {
    pub fn new(paths: AppPaths, state: StateClient) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("build HTTP client");
        Self {
            paths,
            state,
            client,
        }
    }

    pub fn managed_root(&self) -> PathBuf {
        self.paths.data_dir.join("node")
    }

    fn managed_dir_for(&self, version: &str) -> PathBuf {
        self.managed_root().join(format!("node-v{version}-win-x64"))
    }

    /// Fetch the official release index and return (version, lts) pairs.
    pub async fn fetch_index(&self) -> Result<Vec<(String, Option<String>)>, NodeError> {
        let response = self
            .client
            .get(NODE_INDEX_URL)
            .send()
            .await
            .map_err(|error| NodeError::Http(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(NodeError::Http(format!("HTTP {status}")));
        }
        let index: serde_json::Value = response
            .json()
            .await
            .map_err(|error| NodeError::Http(error.to_string()))?;
        let entries = index
            .as_array()
            .ok_or_else(|| NodeError::Http("index is not an array".to_owned()))?;
        let mut result = Vec::with_capacity(entries.len());
        for entry in entries {
            let version = entry
                .get("version")
                .and_then(|value| value.as_str())
                .map(|value| value.trim_start_matches('v').to_owned())
                .unwrap_or_default();
            let lts = entry
                .get("lts")
                .map(|value| match value {
                    serde_json::Value::String(text) if !text.is_empty() => Some(text.clone()),
                    serde_json::Value::Bool(true) => Some("LTS".to_owned()),
                    _ => None,
                })
                .unwrap_or_default();
            if !version.is_empty() {
                result.push((version, lts));
            }
        }
        Ok(result)
    }

    /// Resolve a version spec against the index to one concrete version.
    /// Resolve a version spec to one concrete version. Prefers an already
    /// installed compatible managed runtime before consulting the index.
    pub async fn resolve(&self, spec: &VersionSpec) -> Result<String, NodeError> {
        if let Some(installed) = self
            .state
            .list_runtimes()
            .await?
            .iter()
            .filter(|r| r.kind == RuntimeKind::Managed)
            .map(|r| r.version.clone())
            .find(|v| spec_matches(spec, v))
        {
            return Ok(installed);
        }

        let index = self.fetch_index().await?;
        match spec {
            VersionSpec::Exact(version) => {
                let normalized = crate::version::normalize_version(version);
                if index.iter().any(|(v, _)| *v == normalized) {
                    return Ok(normalized);
                }
                // Support major.minor prefix declarations such as "20.11".
                let prefix = format!("{normalized}.");
                index
                    .iter()
                    .map(|(v, _)| v)
                    .find(|v| v.starts_with(&prefix))
                    .cloned()
                    .ok_or_else(|| NodeError::NotFound(format!("exact version {version}")))
            }
            VersionSpec::Major(major) => {
                let prefix = format!("{major}.");
                index
                    .iter()
                    .map(|(v, _)| v)
                    .find(|v| v.starts_with(&prefix))
                    .cloned()
                    .ok_or_else(|| NodeError::NotFound(format!("major {major}")))
            }
            VersionSpec::Range(range) => {
                let major = range
                    .trim_start_matches(['^', '~', '>', '=', ' ', 'v'])
                    .split('.')
                    .next()
                    .and_then(|part| part.parse::<u32>().ok())
                    .ok_or_else(|| NodeError::Unsupported(format!("unsupported range {range}")))?;
                let prefix = format!("{major}.");
                index
                    .iter()
                    .map(|(v, _)| v)
                    .find(|v| v.starts_with(&prefix))
                    .cloned()
                    .ok_or_else(|| NodeError::NotFound(format!("range {range}")))
            }
            VersionSpec::Lts => index
                .iter()
                .find(|(_, lts)| lts.is_some())
                .map(|(v, _)| v.clone())
                .ok_or_else(|| NodeError::NotFound("active LTS".to_owned())),
        }
    }

    /// Install a managed Node version. Accepts exact, major, LTS, or range
    /// declarations. Prefers an already-installed compatible version.
    pub async fn install(&self, version: &str) -> Result<String, NodeError> {
        let spec = crate::version::parse_spec(version).ok_or_else(|| {
            NodeError::Unsupported(format!("invalid version declaration '{version}'"))
        })?;
        let resolved = self.resolve(&spec).await?;
        if self.is_installed(&resolved) {
            self.record_managed(&resolved).await?;
            return Ok(resolved);
        }

        let zip_url = format!("{NODE_BASE_URL}/v{resolved}/node-v{resolved}-win-x64.zip");
        let zip_name = format!("node-v{resolved}-win-x64.zip");
        let staging = self.staging_dir(&resolved);
        if staging.exists() {
            let _ = std::fs::remove_dir_all(&staging);
        }
        std::fs::create_dir_all(&staging)?;

        let archive_path = staging.join(&zip_name);
        self.download_to(&zip_url, &archive_path).await?;
        self.verify_checksum(&resolved, &archive_path).await?;
        self.extract_archive(&archive_path, &staging).await?;

        let extracted = staging.join(format!("node-v{resolved}-win-x64"));
        if !extracted.exists() {
            return Err(NodeError::Traversal(
                "expected top-level node directory missing".to_owned(),
            ));
        }
        let target = self.managed_dir_for(&resolved);
        if target.exists() {
            let _ = std::fs::remove_dir_all(&target);
        }
        std::fs::rename(&extracted, &target)?;
        let _ = std::fs::remove_file(&archive_path);

        self.record_managed(&resolved).await?;
        Ok(resolved)
    }

    pub fn is_installed(&self, version: &str) -> bool {
        let normalized = crate::version::normalize_version(version);
        self.managed_dir_for(&normalized).join("node.exe").exists()
    }

    pub async fn uninstall(&self, runtime_id: Uuid) -> Result<bool, NodeError> {
        let runtimes = self.state.list_runtimes().await?;
        let Some(runtime) = runtimes.iter().find(|r| r.runtime_id == runtime_id) else {
            return Ok(false);
        };
        // Managed: remove the on-disk directory and the inventory row.
        if runtime.kind == RuntimeKind::Managed {
            let normalized = crate::version::normalize_version(&runtime.version);
            let managed = self.managed_dir_for(&normalized);
            let canonical = managed.canonicalize().unwrap_or_else(|_| managed.clone());
            if canonical.starts_with(self.managed_root()) {
                std::fs::remove_dir_all(&canonical)?;
            }
        }
        // External: remove only the local reference (never the on-disk runtime).
        self.state.remove_runtime(runtime_id).await?;
        Ok(true)
    }

    pub async fn list(&self) -> Result<Vec<RuntimeRecord>, NodeError> {
        Ok(self.state.list_runtimes().await?)
    }

    /// Register an external runtime after explicit user selection.
    pub async fn register_external(
        &self,
        executable_path: &Path,
        architecture: &str,
    ) -> Result<RuntimeRecord, NodeError> {
        let version = probe_version(executable_path)
            .ok_or_else(|| NodeError::Degraded("node.exe did not report a version".to_owned()))?;
        let identity = binary_identity(executable_path)?;
        let record = RuntimeRecord {
            runtime_id: Uuid::new_v4(),
            kind: RuntimeKind::External,
            tool: "node".to_owned(),
            version,
            executable_path: executable_path.to_string_lossy().into_owned(),
            architecture: architecture.to_owned(),
            binary_identity: identity,
            status: RuntimeStatus::Ready,
        };
        self.state.register_runtime(&record).await?;
        Ok(record)
    }

    /// Re-probe an external runtime before launch; mark degraded on mismatch.
    pub async fn re_probe(&self, runtime: &RuntimeRecord) -> Result<RuntimeStatus, NodeError> {
        let path = Path::new(&runtime.executable_path);
        let (version, identity) = match probe_version(path) {
            Some(version) => match binary_identity(path) {
                Ok(identity) => (version, identity),
                Err(_) => {
                    self.state
                        .set_runtime_status(runtime.runtime_id, RuntimeStatus::Degraded)
                        .await?;
                    return Ok(RuntimeStatus::Degraded);
                }
            },
            None => {
                self.state
                    .set_runtime_status(runtime.runtime_id, RuntimeStatus::Degraded)
                    .await?;
                return Ok(RuntimeStatus::Degraded);
            }
        };
        if version != runtime.version || identity != runtime.binary_identity {
            self.state
                .set_runtime_status(runtime.runtime_id, RuntimeStatus::Degraded)
                .await?;
            return Ok(RuntimeStatus::Degraded);
        }
        Ok(RuntimeStatus::Ready)
    }

    async fn record_managed(&self, version: &str) -> Result<(), NodeError> {
        let node = self.managed_dir_for(version).join("node.exe");
        let reported = probe_version(&node).ok_or_else(|| {
            NodeError::Degraded("installed node.exe did not report a version".to_owned())
        })?;
        let normalized = crate::version::normalize_version(&reported);
        let identity = binary_identity(&node)?;
        // Reuse an existing managed record for the same tool+version so
        // reinstalls do not grow duplicate inventory rows.
        if let Some(existing) =
            self.state.list_runtimes().await?.into_iter().find(|r| {
                r.kind == RuntimeKind::Managed && r.tool == "node" && r.version == normalized
            })
        {
            let mut record = existing;
            record.executable_path = node.to_string_lossy().into_owned();
            record.binary_identity = identity;
            record.status = RuntimeStatus::Ready;
            self.state.register_runtime(&record).await?;
            return Ok(());
        }
        let record = RuntimeRecord {
            runtime_id: Uuid::new_v4(),
            kind: RuntimeKind::Managed,
            tool: "node".to_owned(),
            version: normalized,
            executable_path: node.to_string_lossy().into_owned(),
            architecture: "x64".to_owned(),
            binary_identity: identity,
            status: RuntimeStatus::Ready,
        };
        self.state.register_runtime(&record).await?;
        Ok(())
    }

    async fn download_to(&self, url: &str, destination: &Path) -> Result<(), NodeError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| NodeError::Http(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(NodeError::Http(format!("HTTP {status}")));
        }
        let mut file = std::fs::File::create(destination)?;
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| NodeError::Http(error.to_string()))?;
            std::io::Write::write_all(&mut file, &chunk)?;
        }
        Ok(())
    }

    async fn verify_checksum(&self, version: &str, archive_path: &Path) -> Result<(), NodeError> {
        let checksums_url = format!("{NODE_BASE_URL}/v{version}/SHASUMS256.txt");
        let text = self
            .client
            .get(&checksums_url)
            .send()
            .await
            .map_err(|error| NodeError::Http(error.to_string()))?
            .error_for_status()
            .map_err(|error| NodeError::Http(error.to_string()))?
            .text()
            .await
            .map_err(|error| NodeError::Http(error.to_string()))?;
        let file_name = archive_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| NodeError::Traversal("invalid archive filename".to_owned()))?;
        let expected = text
            .lines()
            .find_map(|line| {
                let mut parts = line.split_whitespace();
                let hash = parts.next()?;
                let name = parts.next()?;
                if name == file_name {
                    Some(hash.to_ascii_lowercase())
                } else {
                    None
                }
            })
            .ok_or(NodeError::Checksum)?;
        let actual = {
            let archive_path = archive_path.to_owned();
            tokio::task::spawn_blocking(move || hash_file(&archive_path))
                .await
                .map_err(|error| NodeError::Join(error.to_string()))?
        }?;
        if actual != expected {
            Err(NodeError::Checksum)
        } else {
            Ok(())
        }
    }

    async fn extract_archive(&self, archive_path: &Path, staging: &Path) -> Result<(), NodeError> {
        let archive_path = archive_path.to_owned();
        let staging = staging.to_owned();
        tokio::task::spawn_blocking(move || extract_archive_sync(&archive_path, &staging))
            .await
            .map_err(|error| NodeError::Join(error.to_string()))?
    }

    fn staging_dir(&self, version: &str) -> PathBuf {
        self.managed_root().join(format!(".staging-{version}"))
    }
}

pub async fn discover_external() -> Vec<(PathBuf, String)> {
    tokio::task::spawn_blocking(|| {
        let mut found = Vec::new();
        for candidate in known_node_candidates() {
            if candidate.exists()
                && let Some(version) = probe_version(&candidate)
            {
                found.push((candidate, version));
            }
        }
        found
    })
    .await
    .unwrap_or_default()
}

/// Whether a concrete version satisfies a version spec.
fn spec_matches(spec: &VersionSpec, version: &str) -> bool {
    let normalized = crate::version::normalize_version(version);
    match spec {
        VersionSpec::Exact(exact) => normalized == crate::version::normalize_version(exact),
        VersionSpec::Major(major) => {
            normalized
                .split('.')
                .next()
                .and_then(|part| part.parse::<u32>().ok())
                == Some(*major)
        }
        VersionSpec::Lts => true,
        VersionSpec::Range(range) => {
            let major = range.trim_start_matches(['^', '~', '>', '=', ' ', 'v']);
            let Some(major_num) = major.split('.').next().and_then(|p| p.parse::<u32>().ok())
            else {
                return false;
            };
            normalized
                .split('.')
                .next()
                .and_then(|part| part.parse::<u32>().ok())
                == Some(major_num)
        }
    }
}

fn known_node_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        candidates.push(PathBuf::from(program_files).join("nodejs\\node.exe"));
    }
    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        candidates.push(PathBuf::from(program_files_x86).join("nodejs\\node.exe"));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local).join("Programs\\nodejs\\node.exe"));
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join("node.exe");
            if candidate.exists() && !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        }
    }
    candidates
}

fn probe_version(node_exe: &Path) -> Option<String> {
    let output = Command::new(node_exe).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let version = text.trim().trim_start_matches('v').to_owned();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

fn binary_identity(node_exe: &Path) -> Result<String, io::Error> {
    hash_file(node_exe)
}

fn hash_file(path: &Path) -> Result<String, io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn extract_archive_sync(archive_path: &Path, staging: &Path) -> Result<(), NodeError> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| NodeError::Traversal(error.to_string()))?;
    let staging_canonical = staging
        .canonicalize()
        .unwrap_or_else(|_| staging.to_owned());
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| NodeError::Traversal(error.to_string()))?;
        let entry_path = entry
            .enclosed_name()
            .ok_or_else(|| NodeError::Traversal("archive entry is not enclosed".to_owned()))?
            .to_path_buf();
        let destination = staging_canonical.join(&entry_path);
        if !destination.starts_with(&staging_canonical) {
            return Err(NodeError::Traversal(
                entry_path.to_string_lossy().into_owned(),
            ));
        }
        if entry.is_dir() {
            std::fs::create_dir_all(&destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut output = std::fs::File::create(&destination)?;
            io::copy(&mut entry, &mut output)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{extract_archive_sync, hash_file};

    fn make_zip_with_traversal(path: &Path) {
        use std::io::Write;
        let file = std::fs::File::create(path).expect("create zip");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        // A traversal entry: "..\\escape.txt"
        zip.start_file("../escape.txt", options)
            .expect("start entry");
        zip.write_all(b"evil").expect("write entry");
        zip.finish().expect("finish zip");
    }

    #[test]
    fn traversal_entry_is_rejected() {
        let fixture =
            std::env::temp_dir().join(format!("nerd-node-traversal-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&fixture).expect("create fixture");
        let zip_path = fixture.join("t.zip");
        make_zip_with_traversal(&zip_path);
        let staging = fixture.join("staging");
        std::fs::create_dir_all(&staging).expect("create staging");

        let result = extract_archive_sync(&zip_path, &staging);
        assert!(matches!(result, Err(super::NodeError::Traversal(_))));
        assert!(!staging.join("escape.txt").exists());

        let _ = std::fs::remove_dir_all(&fixture);
    }

    #[test]
    fn hash_file_produces_sha256_hex() {
        let fixture = std::env::temp_dir().join(format!("nerd-node-hash-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&fixture).expect("create fixture");
        let path = fixture.join("f.bin");
        std::fs::write(&path, b"hello").expect("write file");
        let hash = hash_file(&path).expect("hash file");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        let _ = std::fs::remove_dir_all(&fixture);
    }

    #[test]
    fn path_helpers_exist() {
        let _ = PathBuf::new();
    }
}
