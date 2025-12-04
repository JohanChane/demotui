//! under the data folder:
//! * [`BasicInfo`] basic_clash_config.yaml
//! * [`ProfileManager`] clashtui.db
//! * [`log`] clashtui.log
//! * [`ConfigFile`] config.yaml
//! * `Folder` profiles/
//! * `Folder` templates/

use anyhow::{Context, Result, ensure};
pub use core::*;
pub use database::*;
use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};
use util::*;

mod core;
#[macro_use]
mod util;
mod database;
// #[cfg(feature = "migration_v0_2_3")]
// pub mod v0_2_3;

pub const CONFIG: Wrapper = Wrapper;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
static _CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Wrapper;

impl AsRef<Config> for Wrapper {
    fn as_ref(&self) -> &Config {
        _CONFIG.get().expect("uninited")
    }
}

pub struct Config {
    pub cfg_file: ConfigFile,
    pub data: Mutex<ProfileManager>,
    pub external_controller: String,
    pub proxy_addr: String,
    pub secret: Option<String>,
    pub global_ua: Option<String>,
}

impl Config {
    fn load() -> Result<Self> {
        let cfg_file = ConfigFile::from_file()?;
        let basic_info = BasicInfo::from_file()?;
        let data = ProfileManager::from_file()?.into();
        Ok(Self {
            cfg_file,
            data,
            external_controller: basic_info.get_external_controller(),
            proxy_addr: basic_info
                .get_proxy_addr()
                .context("Failed to determine proxy port")?,
            secret: basic_info.secret,
            global_ua: basic_info.global_ua,
        })
    }
    pub fn save(&self) -> Result<()> {
        todo!()
    }
}

pub fn init(base_path: Option<PathBuf>) -> Result<()> {
    let path = if let Some(path) = base_path {
        ensure!(path.exists(), "{} does NOT exists", path.display());
        ensure!(path.is_dir(), "{} is not a dir", path.display());

        let path = path
            .canonicalize()
            .context(format!("Failed to canonicalize path: {}", path.display()))?;
        std::path::absolute(&path).context(format!("{} is not an absolute path", path.display()))?
    } else {
        load_home_dir()?
    };

    if DATA_DIR.set(path).is_err() || _CONFIG.set(Config::load()?).is_err() {
        unreachable!("init twice")
    }

    Ok(())
}

pub fn init_config() -> Result<()> {
    use std::fs;

    let path = match DATA_DIR.get() {
        Some(path) => path,
        None => unreachable!(),
    };

    fs::create_dir_all(path)?;

    fs::write(path.join(defs::BASIC_FILE), BasicInfo::DEFAULT)?;
    ConfigFile::default().to_file()?;
    ProfileManager::default().to_file()?;

    fs::create_dir(path.join(defs::TEMPLATE_DIR))?;
    fs::create_dir(path.join(defs::PROFILE_DIR))?;

    Ok(())
}

load_save!(BasicInfo, defs::BASIC_FILE, no_save);
load_save!(ConfigFile, defs::CONFIG_FILE);
load_save!(ProfileManager, defs::DATA_FILE);
