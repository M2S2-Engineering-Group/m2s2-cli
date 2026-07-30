pub mod devto;
pub mod hashnode;
pub mod m2s2;

use crate::publish::config::PublishConfig;
use crate::publish::target::Target;
use crate::publish::target_kind::TargetKind;
use anyhow::{Context, Result};

/// Build a target from its config section, reusing one `reqwest::Client` (a connection
/// pool/TLS cache) across every target rather than each one building its own. This is the only
/// place that needs to know about a new connector: add a [`TargetKind`] variant, a config
/// section, and a match arm here.
fn build_one(kind: TargetKind, client: &reqwest::Client, config: &PublishConfig) -> Result<Target> {
    match kind {
        TargetKind::Devto => {
            let cfg = config
                .devto
                .as_ref()
                .context("no [devto] section in .m2s2-publish.toml")?;
            Ok(Target::Devto(devto::DevTo::new(client.clone(), cfg)))
        }
        TargetKind::Hashnode => {
            let cfg = config
                .hashnode
                .as_ref()
                .context("no [hashnode] section in .m2s2-publish.toml")?;
            Ok(Target::Hashnode(hashnode::Hashnode::new(client.clone(), cfg)))
        }
        TargetKind::M2s2 => {
            let cfg = config
                .m2s2
                .as_ref()
                .context("no [m2s2] section in .m2s2-publish.toml")?;
            Ok(Target::M2s2(m2s2::M2s2::new(client.clone(), cfg)))
        }
    }
}

pub fn build_targets(kinds: &[TargetKind], config: &PublishConfig) -> Result<Vec<Target>> {
    let client = reqwest::Client::new();
    kinds.iter().map(|kind| build_one(*kind, &client, config)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // No "unknown target" test anymore — TargetKind is a closed enum, so an invalid name is
    // now rejected at parse time (clap for --to, serde for frontmatter), before build_one ever
    // sees it.

    #[test]
    fn known_target_without_config_section_is_a_clear_error() {
        let Err(err) =
            build_one(TargetKind::Devto, &reqwest::Client::new(), &PublishConfig::default())
        else {
            panic!("expected an error");
        };
        assert!(err.to_string().contains("no [devto] section"));
    }
}
