use anyhow::{Context, Result};
use serde::Deserialize;

pub const CONFIG_FILE: &str = ".m2s2-publish.toml";

#[derive(Deserialize, Default)]
pub struct PublishConfig {
    pub devto: Option<DevToConfig>,
    pub hashnode: Option<HashnodeConfig>,
    pub m2s2: Option<M2s2Config>,
}

#[derive(Deserialize)]
pub struct DevToConfig {
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct HashnodeConfig {
    pub token: String,
    pub publication_id: String,
}

#[derive(Deserialize)]
pub struct M2s2Config {
    /// Base URL of the m2s2-platform API, e.g. `https://api.m2s2.io` — `/admin/blog` is
    /// appended by the target itself.
    pub endpoint: String,
    /// Cognito JWT (admin group) bearer token. Short-lived; refresh manually when it expires.
    pub token: String,
}

impl PublishConfig {
    pub fn load() -> Result<Self> {
        let raw = std::fs::read_to_string(CONFIG_FILE).with_context(|| {
            format!(
                "no {CONFIG_FILE} found in the current directory — create one with the \
                 credentials for whichever targets you're publishing to (see `m2s2 publish --help`)"
            )
        })?;
        toml::from_str(&raw).with_context(|| format!("failed to parse {CONFIG_FILE}"))
    }
}
