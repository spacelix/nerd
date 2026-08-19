//! Deterministic package-manager tooling: npm from the Node distribution,
//! pnpm/Yarn through Corepack, with a separate Corepack for Node 25+.

use std::path::{Path, PathBuf};

use crate::paths::AppPaths;

#[derive(Debug)]
pub enum PackageManagerError {
    Unsupported(String),
    Missing(String),
    Io(std::io::Error),
}

impl std::fmt::Display for PackageManagerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(message) => formatter.write_str(message),
            Self::Missing(message) => write!(formatter, "package manager not available: {message}"),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for PackageManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Unsupported(_) | Self::Missing(_) => None,
        }
    }
}

impl From<std::io::Error> for PackageManagerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// A parsed `packageManager` field, e.g. `"pnpm@9.1.0"` or `"yarn@1.22.19"`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageManager {
    pub name: String,
    pub version: String,
}

/// Parse the `packageManager` field from `package.json`.
pub fn parse_package_manager(package_json: &str) -> Option<PackageManager> {
    let value: serde_json::Value = serde_json::from_str(package_json).ok()?;
    let spec = value.get("packageManager")?.as_str()?;
    let (name, version) = spec.split_once('@')?;
    if name.is_empty() || version.is_empty() {
        return None;
    }
    Some(PackageManager {
        name: name.to_owned(),
        version: version.to_owned(),
    })
}

pub struct PackageManagerRunner {
    paths: AppPaths,
}

/// Resolved package-manager invocation: (program, args-prefix, extra-env).
pub type ResolvedTooling = (PathBuf, Vec<String>, Vec<(String, String)>);

impl PackageManagerRunner {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    /// Resolve the command + environment to run a package manager for a
    /// project.
    pub fn resolve(
        &self,
        manager: &PackageManager,
        node_dir: &Path,
        project_dir: &Path,
    ) -> Result<ResolvedTooling, PackageManagerError> {
        let npm = node_dir.join("npm.cmd");
        let corepack = node_dir.join("corepack.cmd");

        match manager.name.as_str() {
            "npm" => {
                if !npm.exists() {
                    return Err(PackageManagerError::Missing(
                        "npm.cmd in Node distribution".to_owned(),
                    ));
                }
                Ok((npm, Vec::new(), Vec::new()))
            }
            "pnpm" | "yarn" => {
                if corepack.exists() {
                    // Corepack bundled with Node <25.
                    Ok((
                        corepack,
                        vec![manager.name.clone(), manager.version.clone()],
                        Vec::new(),
                    ))
                } else {
                    // Node 25+: Corepack is not bundled; use the Nerd-managed
                    // standalone corepack under the data directory.
                    let standalone = self.standalone_corepack();
                    if !standalone.exists() {
                        return Err(PackageManagerError::Missing(
                            "Corepack not bundled with this Node; run nerd runtime install-corepack".to_owned(),
                        ));
                    }
                    let _ = project_dir;
                    Ok((
                        standalone,
                        vec![manager.name.clone(), manager.version.clone()],
                        Vec::new(),
                    ))
                }
            }
            other => Err(PackageManagerError::Unsupported(format!(
                "unsupported package manager '{other}'"
            ))),
        }
    }

    fn standalone_corepack(&self) -> PathBuf {
        self.paths.data_dir.join("corepack").join("corepack.cmd")
    }
}

/// Build an isolated child PATH for a project: node dir and node_modules/.bin
/// first, then the inherited PATH. The parent/global environment is unchanged.
pub fn isolated_path(node_dir: &Path, project_dir: &Path) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    entries.push(node_dir.to_owned());
    entries.push(project_dir.join("node_modules\\.bin"));
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            if !entries.contains(&dir) {
                entries.push(dir);
            }
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::{PackageManager, parse_package_manager};

    #[test]
    fn parses_package_manager_field() {
        assert_eq!(
            parse_package_manager(r#"{"packageManager":"pnpm@9.1.0"}"#),
            Some(PackageManager {
                name: "pnpm".to_owned(),
                version: "9.1.0".to_owned(),
            })
        );
        assert_eq!(
            parse_package_manager(r#"{"packageManager":"yarn@1.22.19"}"#),
            Some(PackageManager {
                name: "yarn".to_owned(),
                version: "1.22.19".to_owned(),
            })
        );
        assert_eq!(parse_package_manager(r#"{"name":"x"}"#), None);
    }
}
