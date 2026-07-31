use anyhow::{Context, Result};
use serde::Deserialize;

pub const CONFIG_FILE: &str = ".m2s2-publish.toml";

#[derive(Deserialize, Default)]
pub struct PublishConfig {
    pub devto: Option<DevToConfig>,
    pub hashnode: Option<HashnodeConfig>,
    pub platform: Option<PlatformConfig>,
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

/// Any blog with a dedicated create/update API following the same shape this target speaks
/// (JSON body, bearer auth) — not specific to any one site. Nothing about a particular
/// deployment (path, credentials) is hardcoded in the tool itself; it all comes from here.
#[derive(Deserialize)]
pub struct PlatformConfig {
    /// Base URL of the blog's API, e.g. `https://api.example.com`.
    pub endpoint: String,
    /// Path appended to `endpoint` for both create (`POST <path>`) and update
    /// (`PUT <path>?slug=...`). Defaults to `/admin/blog` — override if your deployment mounts
    /// its blog API elsewhere.
    pub path: Option<String>,
    /// Bearer token sent as `Authorization: Bearer <token>`. Short-lived tokens (e.g. a Cognito
    /// JWT) need to be refreshed manually when they expire.
    pub token: String,
    /// If set, this command builds the request body instead of the built-in field mapping: the
    /// article (plus `update: true/false`) is piped to it as JSON on stdin, and whatever JSON
    /// object it prints on stdout is sent verbatim as the request body. Run through a shell
    /// (`sh -c` / `cmd /C`), so it can be a script path or a full command line with arguments.
    pub body_command: Option<String>,
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
