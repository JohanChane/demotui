use anyhow::{Context, Result, bail, ensure};
use std::{path::PathBuf, sync::OnceLock};

pub const CONFIG: Wrapper = Wrapper;

// static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
static _CONFIG: OnceLock<Config> = OnceLock::new();

struct Config;

pub struct Wrapper;

impl Wrapper {
    pub fn init(base_path: Option<PathBuf>) -> Result<()> {
        let path = if let Some(path) = base_path {
            ensure!(path.exists(), "{} does NOT exists", path.display());
            ensure!(path.is_dir(), "{} is not a dir", path.display());

            let path = path
                .canonicalize()
                .context(format!("Failed to canonicalize path: {}", path.display()))?;
            std::path::absolute(&path)
                .context(format!("{} is not an absolute path", path.display()))?
        } else {
            load_home_dir()?
        };

        // do something to load config

        if _CONFIG.set(Config).is_err() {
            unreachable!("init twice")
        }

        Ok(())
    }
}

impl AsRef<Config> for Wrapper {
    fn as_ref(&self) -> &Config {
        _CONFIG.get().expect("uninited")
    }
}

fn load_home_dir() -> Result<std::path::PathBuf> {
    use std::{env, path};
    let data_dir = env::current_exe()
        .context("Err loading exe_file_path")?
        .parent()
        .context("Err finding exe_dir")?
        .join("data");
    if data_dir.exists() && data_dir.is_dir() {
        // portable mode
        Ok(data_dir)
    } else {
        if cfg!(target_os = "linux") {
            env::var_os("XDG_CONFIG_HOME")
                .map(path::PathBuf::from)
                .or(env::var_os("HOME").map(|h| path::PathBuf::from(h).join(".config")))
        } else if cfg!(target_os = "windows") {
            env::var_os("APPDATA").map(path::PathBuf::from)
        } else if cfg!(target_os = "macos") {
            env::var_os("HOME").map(|h| path::PathBuf::from(h).join(".config"))
        } else {
            bail!("Not supported platform")
        }
        .map(|c| c.join("clashtui"))
        .context("failed to load home dir")
    }
}
