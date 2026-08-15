use std::{fs, io, path::PathBuf};

use crate::windows;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub log_dir: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> io::Result<Self> {
        Ok(Self::from_root(windows::local_app_data()?.join("Nerd")))
    }

    pub fn from_root(data_dir: PathBuf) -> Self {
        Self {
            database_path: data_dir.join("nerd.db"),
            log_dir: data_dir.join("logs"),
            data_dir,
        }
    }

    pub fn create_state_directory(&self) -> io::Result<()> {
        fs::create_dir_all(&self.data_dir)
    }
}
