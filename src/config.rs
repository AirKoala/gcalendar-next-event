use crate::authenticate::Creds;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub creds: Creds,
    pub nocache: bool,
    pub cache_duration_seconds: i64,
    pub selected_calendars: SelectedCalendars,
    pub max_time_until_event_seconds: Option<i64>,
}

impl Config {
    pub fn new_default() -> Self {
        Self {
            cache_duration_seconds: 30 * 60, // 30 minutes
            ..Default::default()
        }
    }

    pub fn save_to(&self, config_path: Option<&Path>) -> Result<()> {
        let default_config_path = Self::get_default_config_path()?;
        let config_path = config_path.unwrap_or(&default_config_path);

        std::fs::write(config_path, serde_json::to_string_pretty(self)?)?;

        Ok(())
    }

    pub fn load_from(config_path: Option<&Path>) -> Result<Self> {
        let default_config_path = Self::get_default_config_path()?;
        let config_path = config_path.unwrap_or(&default_config_path);
        let config = std::fs::read_to_string(config_path)?;

        Ok(serde_json::from_str(&config)?)
    }

    fn get_default_config_path() -> Result<PathBuf> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))?;

        if !xdg_dirs.get_config_home().exists() {
            std::fs::create_dir_all(xdg_dirs.get_config_home())?;
        }

        Ok(xdg_dirs.get_config_home().join("config.json"))
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum SelectedCalendars {
    #[default]
    All,
    Whitelist(Vec<String>),
    Blacklist(Vec<String>),
}
