use serde::{Serialize, Deserialize};
use std::path::Path;
use eyre::Result;
use crate::authenticate::Creds;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub creds: Creds,
}

impl Config {
    pub fn save_to(&self, config_path: Option<&Path>) -> Result<()> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))?;

        if !xdg_dirs.get_config_home().exists() {
            std::fs::create_dir_all(xdg_dirs.get_config_home())?;
        }

        let default_config_path = xdg_dirs.get_config_home().join("config.json");
        let config_path = config_path.unwrap_or(&default_config_path);

        std::fs::write(config_path, serde_json::to_string_pretty(self)?)?;

        Ok(())
    }
}
