use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::utils::json;

/// Data container for the users tool configuration such as aws sso
/// start urls.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserConfig {
    pub sso_config: Vec<SsoConfig>,
}

impl UserConfig {
    pub fn has_config(&self) -> bool {
        !self.sso_config.is_empty()
    }

    pub fn get_default(&self) -> Option<&SsoConfig> {
        self.sso_config.first()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SsoConfig {
    pub start_url: String,
    pub region: String,
}

impl SsoConfig {
    pub fn new(start_url: &str, sso_region: &str) -> Self {
        Self {
            start_url: String::from(start_url),
            region: String::from(sso_region),
        }
    }
}

impl std::fmt::Display for SsoConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.start_url, self.region)
    }
}

/// Service to handle reading and writing `config.json` file in `config_dir`.
pub struct ConfigProvider {
    config_dir: PathBuf
}

impl ConfigProvider {
    pub fn new(config_dir: &Path) -> Self {
        Self { config_dir: config_dir.to_path_buf() }
    }

    pub fn get_user_config(&self) -> Result<UserConfig> {
        let config_file_path = self.config_dir.join("config.json");
        if config_file_path.exists() {
            json::read_from_file(config_file_path.as_path())
        } else {
            Ok(UserConfig {
                sso_config: Vec::new()
            })
        }
    }

    pub fn update_user_config(&self, user_config: &UserConfig) -> Result<()> {
        let config_file_path = self.config_dir.join("config.json");
        json::write_to_file(config_file_path.as_path(), user_config)
    }
}
