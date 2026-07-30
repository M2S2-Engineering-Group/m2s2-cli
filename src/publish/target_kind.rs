use clap::ValueEnum;
use serde::Deserialize;
use std::fmt;

/// The closed set of things `m2s2 publish` knows how to publish to — used for both the
/// frontmatter `publish:` list and the `--to` flag, so an unrecognized name is a parse error
/// with the valid options listed, not a runtime string mismatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Devto,
    Hashnode,
    M2s2,
}

impl fmt::Display for TargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no skipped variants")
            .get_name()
            .fmt(f)
    }
}
