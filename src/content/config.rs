//! The minimal `.m2s2/config.toml` from `docs/m2s2-cli-content-delivery-integration.md` §5 —
//! only the `[content]` section, which has no platform dependency. `[delivery]` and friends stay
//! out until the platform API client (Phase 6b) actually exists; shipping commented-out
//! speculative sections now would just invite drift once that schema is defined server-side.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const DEFAULT_ARTICLES_DIR: &str = "articles";
pub const DEFAULT_ASSETS_DIR: &str = "assets";

/// The only `.m2s2/config.toml` `schema_version` this CLI understands — bumped if the config
/// shape ever needs a breaking change.
pub const SUPPORTED_CONFIG_SCHEMA_VERSION: u32 = 1;

#[derive(Deserialize, Debug)]
pub struct ContentConfig {
    pub schema_version: u32,
    pub content: ContentSection,
}

#[derive(Deserialize, Debug)]
pub struct ContentSection {
    pub articles_dir: String,
    pub assets_dir: String,
    pub canonical_base_url: String,
}

fn config_path(root: &Path) -> PathBuf {
    root.join(".m2s2").join("config.toml")
}

impl ContentConfig {
    pub fn load(root: &Path) -> Result<Self> {
        let path = config_path(root);
        let raw = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "no {} found — run `m2s2 content init` first",
                path.display()
            )
        })?;
        let config: ContentConfig =
            toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;
        if config.schema_version != SUPPORTED_CONFIG_SCHEMA_VERSION {
            bail!(
                "{} has schema_version {} but this CLI only supports {} — upgrade m2s2-cli or \
                 update the config",
                path.display(),
                config.schema_version,
                SUPPORTED_CONFIG_SCHEMA_VERSION
            );
        }
        Ok(config)
    }

    pub fn articles_dir(&self, root: &Path) -> PathBuf {
        root.join(&self.content.articles_dir)
    }

    pub fn assets_dir(&self, root: &Path) -> PathBuf {
        root.join(&self.content.assets_dir)
    }
}

/// Writes `.m2s2/config.toml` under `root` and creates `articles_dir`/`assets_dir` alongside it
/// so `content validate`/`inspect` have somewhere to scan immediately.
pub fn init(
    root: &Path,
    articles_dir: &str,
    assets_dir: &str,
    canonical_base_url: &str,
    force: bool,
) -> Result<PathBuf> {
    if !canonical_base_url.starts_with("https://") {
        bail!("canonical_base_url must be an absolute https:// URL, got '{canonical_base_url}'");
    }

    let path = config_path(root);
    if path.exists() && !force {
        bail!(
            "{} already exists — pass --force to overwrite it",
            path.display()
        );
    }

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create {}", dir.display()))?;
    }
    std::fs::create_dir_all(root.join(articles_dir))
        .with_context(|| format!("failed to create {articles_dir}"))?;
    std::fs::create_dir_all(root.join(assets_dir))
        .with_context(|| format!("failed to create {assets_dir}"))?;

    let contents = format!(
        "schema_version = {}\n\n[content]\narticles_dir = \"{articles_dir}\"\nassets_dir = \"{assets_dir}\"\ncanonical_base_url = \"{canonical_base_url}\"\n",
        SUPPORTED_CONFIG_SCHEMA_VERSION
    );
    std::fs::write(&path, contents)
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;

    #[test]
    fn init_writes_expected_toml_shape() {
        let dir = TempDir::new().unwrap();
        let path = init(
            dir.path(),
            DEFAULT_ARTICLES_DIR,
            DEFAULT_ASSETS_DIR,
            "https://m2s2.io/blog",
            false,
        )
        .unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("schema_version = 1"));
        assert!(raw.contains("[content]"));
        assert!(raw.contains("articles_dir = \"articles\""));
        assert!(raw.contains("assets_dir = \"assets\""));
        assert!(raw.contains("canonical_base_url = \"https://m2s2.io/blog\""));
        assert!(dir.path().join("articles").is_dir());
        assert!(dir.path().join("assets").is_dir());
    }

    #[test]
    fn init_then_load_round_trips() {
        let dir = TempDir::new().unwrap();
        init(
            dir.path(),
            DEFAULT_ARTICLES_DIR,
            DEFAULT_ASSETS_DIR,
            "https://m2s2.io/blog",
            false,
        )
        .unwrap();

        let config = ContentConfig::load(dir.path()).unwrap();
        assert_eq!(config.schema_version, 1);
        assert_eq!(config.content.articles_dir, "articles");
        assert_eq!(config.articles_dir(dir.path()), dir.path().join("articles"));
    }

    #[test]
    fn init_refuses_to_overwrite_without_force() {
        let dir = TempDir::new().unwrap();
        init(
            dir.path(),
            DEFAULT_ARTICLES_DIR,
            DEFAULT_ASSETS_DIR,
            "https://m2s2.io/blog",
            false,
        )
        .unwrap();

        let err = init(
            dir.path(),
            DEFAULT_ARTICLES_DIR,
            DEFAULT_ASSETS_DIR,
            "https://m2s2.io/blog",
            false,
        )
        .unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn init_rejects_non_https_canonical_base_url() {
        let dir = TempDir::new().unwrap();
        let err = init(
            dir.path(),
            DEFAULT_ARTICLES_DIR,
            DEFAULT_ASSETS_DIR,
            "http://m2s2.io/blog",
            false,
        )
        .unwrap_err();
        assert!(err.to_string().contains("https://"));
    }

    #[test]
    fn load_without_init_is_a_clear_error() {
        let dir = TempDir::new().unwrap();
        let err = ContentConfig::load(dir.path()).unwrap_err();
        assert!(err.to_string().contains("content init"));
    }

    #[test]
    fn load_rejects_an_unsupported_schema_version() {
        let dir = TempDir::new().unwrap();
        let path = config_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "schema_version = 99\n\n[content]\narticles_dir = \"articles\"\nassets_dir = \"assets\"\ncanonical_base_url = \"https://x.io\"\n",
        )
        .unwrap();

        let err = ContentConfig::load(dir.path()).unwrap_err();
        assert!(err.to_string().contains("schema_version 99"));
    }
}
