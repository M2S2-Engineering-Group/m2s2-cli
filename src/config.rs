use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct M2S2Config {
    pub framework: Option<String>,
}

impl M2S2Config {
    pub fn load() -> Option<Self> {
        let raw = fs::read_to_string(".m2s2.json").ok()?;
        serde_json::from_str(&raw).ok()
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(".m2s2.json", json + "\n")?;
        Ok(())
    }
}

/// Resolve framework from an explicit CLI flag, falling back to `.m2s2.json`
/// then `package.json`. Returns an error if none of those sources yield a result.
pub fn resolve_framework(explicit: Option<String>) -> Result<String> {
    if let Some(f) = explicit {
        return Ok(f);
    }
    detect_framework()
        .context("could not detect framework — run from your project root or pass --framework")
}

/// Detect the framework by reading `.m2s2.json` first, then `package.json`.
pub fn detect_framework() -> Option<String> {
    if let Some(cfg) = M2S2Config::load().filter(|c| c.framework.is_some()) {
        return cfg.framework;
    }
    let raw = fs::read_to_string("package.json").ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let deps = pkg.get("dependencies")?;
    if deps.get("@m2s2/react-lib").is_some() {
        Some("react".into())
    } else if deps.get("@m2s2/ng-lib").is_some() {
        Some("angular".into())
    } else if deps.get("@m2s2/vue-lib").is_some() {
        Some("vue".into())
    } else {
        None
    }
}
