use anyhow::{Context, Result, bail};
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
    /// Missing `.m2s2-publish.toml` is only an error if env vars don't cover it either — the
    /// file not existing at all is exactly what a fully env-based setup looks like (see
    /// `apply_env_fallbacks`), so it can't be a hard failure up front.
    pub fn load() -> Result<Self> {
        let mut config = match std::fs::read_to_string(CONFIG_FILE) {
            Ok(raw) => {
                toml::from_str(&raw).with_context(|| format!("failed to parse {CONFIG_FILE}"))?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => PublishConfig::default(),
            Err(e) => return Err(e).with_context(|| format!("failed to read {CONFIG_FILE}")),
        };
        config.apply_env_fallbacks(|k| std::env::var(k).ok())?;
        Ok(config)
    }

    /// For any target whose `[section]` is entirely absent from the TOML (not present-but-
    /// missing-a-field — this only fills in a *whole* section that isn't there), check whether
    /// the equivalent `M2S2_PUBLISH_<TARGET>_<FIELD>` env vars are set and use those instead.
    /// Lets credentials be kept out of `.m2s2-publish.toml` entirely, so there's nothing to
    /// accidentally commit if the user's content repo doesn't gitignore it.
    fn apply_env_fallbacks(&mut self, get: impl Fn(&str) -> Option<String>) -> Result<()> {
        if self.devto.is_none() {
            self.devto = get("M2S2_PUBLISH_DEVTO_API_KEY").map(|api_key| DevToConfig { api_key });
        }
        if self.hashnode.is_none() {
            self.hashnode = match (
                get("M2S2_PUBLISH_HASHNODE_TOKEN"),
                get("M2S2_PUBLISH_HASHNODE_PUBLICATION_ID"),
            ) {
                (Some(token), Some(publication_id)) => Some(HashnodeConfig {
                    token,
                    publication_id,
                }),
                (None, None) => None,
                _ => bail!(
                    "partial hashnode credentials in the environment — both \
                     $M2S2_PUBLISH_HASHNODE_TOKEN and $M2S2_PUBLISH_HASHNODE_PUBLICATION_ID \
                     must be set together"
                ),
            };
        }
        if self.platform.is_none() {
            self.platform = match (
                get("M2S2_PUBLISH_PLATFORM_ENDPOINT"),
                get("M2S2_PUBLISH_PLATFORM_TOKEN"),
            ) {
                (Some(endpoint), Some(token)) => Some(PlatformConfig {
                    endpoint,
                    token,
                    path: get("M2S2_PUBLISH_PLATFORM_PATH"),
                    body_command: get("M2S2_PUBLISH_PLATFORM_BODY_COMMAND"),
                }),
                (None, None) => None,
                _ => bail!(
                    "partial platform credentials in the environment — both \
                     $M2S2_PUBLISH_PLATFORM_ENDPOINT and $M2S2_PUBLISH_PLATFORM_TOKEN must be \
                     set together"
                ),
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn fills_in_a_wholly_absent_section_from_env() {
        let mut config = PublishConfig::default();
        let env = env_map(&[("M2S2_PUBLISH_DEVTO_API_KEY", "from-env")]);
        config.apply_env_fallbacks(|k| env.get(k).cloned()).unwrap();
        assert_eq!(config.devto.unwrap().api_key, "from-env");
    }

    #[test]
    fn does_not_override_a_section_already_present_from_toml() {
        let mut config = PublishConfig {
            devto: Some(DevToConfig {
                api_key: "from-toml".into(),
            }),
            ..Default::default()
        };
        let env = env_map(&[("M2S2_PUBLISH_DEVTO_API_KEY", "from-env")]);
        config.apply_env_fallbacks(|k| env.get(k).cloned()).unwrap();
        assert_eq!(config.devto.unwrap().api_key, "from-toml");
    }

    #[test]
    fn hashnode_needs_both_env_vars_together() {
        let mut config = PublishConfig::default();
        let env = env_map(&[("M2S2_PUBLISH_HASHNODE_TOKEN", "t")]);
        let err = config
            .apply_env_fallbacks(|k| env.get(k).cloned())
            .unwrap_err();
        assert!(err.to_string().contains("must be set together"));
    }

    #[test]
    fn platform_picks_up_optional_path_and_body_command_from_env_too() {
        let mut config = PublishConfig::default();
        let env = env_map(&[
            ("M2S2_PUBLISH_PLATFORM_ENDPOINT", "https://api.example.com"),
            ("M2S2_PUBLISH_PLATFORM_TOKEN", "t"),
            ("M2S2_PUBLISH_PLATFORM_PATH", "/custom"),
        ]);
        config.apply_env_fallbacks(|k| env.get(k).cloned()).unwrap();
        let platform = config.platform.unwrap();
        assert_eq!(platform.path.as_deref(), Some("/custom"));
        assert_eq!(platform.body_command, None);
    }

    #[test]
    fn no_env_vars_and_no_toml_section_leaves_it_absent() {
        let mut config = PublishConfig::default();
        config.apply_env_fallbacks(|_| None).unwrap();
        assert!(config.devto.is_none());
        assert!(config.hashnode.is_none());
        assert!(config.platform.is_none());
    }
}
