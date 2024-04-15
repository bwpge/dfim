use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{bail, Result};

use crate::path;

static OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Default)]
pub struct Config {
    // TODO: add config fields
}

impl Config {
    pub fn set_override(path: &Path) -> Result<()> {
        if OVERRIDE.set(path.to_owned()).is_err() {
            bail!("failed to set config override path");
        }
        Ok(())
    }

    /// Searches for the configuration entry module.
    pub fn get_module_file() -> Option<PathBuf> {
        if let Some(p) = OVERRIDE.get() {
            return Some(p.to_owned());
        }

        if let Some(v) = option_env!("DFIM_CONFIG") {
            let path = PathBuf::from(v);
            if path.is_file() {
                return Some(path);
            }
            return None;
        }

        let p = config_dir().join("dfim.lua");
        if p.is_file() {
            return Some(p);
        }
        None
    }
}

pub fn home_dir() -> &'static Path {
    static HOME_DIR: OnceLock<PathBuf> = OnceLock::new();
    HOME_DIR.get_or_init(|| home::home_dir().unwrap())
}

pub fn config_dir() -> &'static Path {
    static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
    CONFIG_DIR.get_or_init(|| {
        if let Some(d) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(d);
        }
        home_dir().join(path!(".config", env!("CARGO_PKG_NAME")))
    })
}

pub fn data_dir() -> &'static Path {
    static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    DATA_DIR.get_or_init(|| {
        if let Some(d) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(d);
        }
        if cfg!(windows) {
            config_dir().join("data")
        } else {
            home_dir().join(path!(".local", "share", env!("CARGO_PKG_NAME")))
        }
    })
}

pub fn plugin_dir() -> &'static Path {
    static PLUGIN_DIR: OnceLock<PathBuf> = OnceLock::new();
    PLUGIN_DIR.get_or_init(|| data_dir().join("plugins"))
}
