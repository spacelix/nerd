//! Isolated child-process execution with a project-scoped PATH.

use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// Build a child `PATH` value and return the resulting environment map.
pub fn child_environment(
    path_entries: &[PathBuf],
    extra: &[(String, String)],
) -> Vec<(String, String)> {
    let path_value = std::env::join_paths(path_entries)
        .ok()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut env = Vec::new();
    env.push(("PATH".to_owned(), path_value));
    for (key, value) in extra {
        env.push((key.clone(), value.clone()));
    }
    env
}

/// Spawn a child with an isolated environment. The program and current
/// directory are explicit; no stdin is attached.
pub fn spawn_isolated(
    program: &Path,
    args: &[String],
    current_dir: &Path,
    environment: &[(String, String)],
) -> io::Result<std::process::Child> {
    let mut command = Command::new(program);
    command.args(args);
    command.current_dir(current_dir);
    command.stdin(Stdio::null());
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::inherit());
    for (key, value) in environment {
        command.env(key, value);
    }
    command.spawn()
}

#[cfg(test)]
mod tests {
    use super::child_environment;
    use std::path::PathBuf;

    #[test]
    fn child_path_prepends_entries_and_keeps_parent_path() {
        let entries = vec![
            PathBuf::from(r"C:\Nerd\node-v20-win-x64"),
            PathBuf::from(r"C:\proj\node_modules\.bin"),
        ];
        let env = child_environment(&entries, &[]);
        let path = env.iter().find(|(k, _)| k == "PATH").expect("PATH set");
        let parts: Vec<&str> = path.1.split(';').collect();
        assert!(parts[0].contains("node-v20-win-x64"));
        assert!(parts[1].contains("node_modules\\.bin"));
    }
}
