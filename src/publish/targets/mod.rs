mod devto;
mod hashnode;
mod m2s2;

use crate::publish::config::PublishConfig;
use crate::publish::target::PublishTarget;
use anyhow::{Context, Result, bail};

/// Build a target by name from its config section. This is the only place that needs to know
/// about a new connector — implement [`PublishTarget`] for it and add a match arm here.
pub fn build_one(name: &str, config: &PublishConfig) -> Result<Box<dyn PublishTarget>> {
    match name {
        "devto" => {
            let cfg = config
                .devto
                .as_ref()
                .context("no [devto] section in .m2s2-publish.toml")?;
            Ok(Box::new(devto::DevTo::new(cfg)))
        }
        "hashnode" => {
            let cfg = config
                .hashnode
                .as_ref()
                .context("no [hashnode] section in .m2s2-publish.toml")?;
            Ok(Box::new(hashnode::Hashnode::new(cfg)))
        }
        "m2s2" => {
            let cfg = config
                .m2s2
                .as_ref()
                .context("no [m2s2] section in .m2s2-publish.toml")?;
            Ok(Box::new(m2s2::M2s2::new(cfg)))
        }
        other => bail!("unknown publish target '{other}' — expected one of: devto, hashnode, m2s2"),
    }
}

pub fn build_targets(names: &[String], config: &PublishConfig) -> Result<Vec<Box<dyn PublishTarget>>> {
    names.iter().map(|name| build_one(name, config)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_target_is_a_clear_error() {
        let Err(err) = build_one("medium", &PublishConfig::default()) else {
            panic!("expected an error");
        };
        assert!(err.to_string().contains("unknown publish target 'medium'"));
    }

    #[test]
    fn known_target_without_config_section_is_a_clear_error() {
        let Err(err) = build_one("devto", &PublishConfig::default()) else {
            panic!("expected an error");
        };
        assert!(err.to_string().contains("no [devto] section"));
    }
}
