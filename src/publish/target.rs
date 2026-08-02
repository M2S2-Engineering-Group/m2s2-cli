use crate::publish::article::Article;
use crate::publish::target_kind::TargetKind;
use crate::publish::targets::devto::{self, DevTo};
use crate::publish::targets::hashnode::{self, Hashnode};
use crate::publish::targets::platform::{self, Platform};
use anyhow::Result;

#[derive(Debug)]
pub struct PublishOutcome {
    /// Human-readable result, e.g. the published article's URL.
    pub message: String,
}

/// The `client` + `base_url` pair every target needs. Pulled out because every target had it as
/// two separate fields; `client` is shared (built once in `targets::build_targets`) rather than
/// each target constructing — and thus pooling/TLS-caching — its own.
pub struct HttpTarget {
    pub client: reqwest::Client,
    pub base_url: String,
}

impl HttpTarget {
    pub fn new(client: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }
}

/// A constructed, ready-to-use publish target — one variant per [`TargetKind`], built in
/// `publish::targets::build_one`. A plain enum rather than `Box<dyn Trait>`: the set of targets
/// is closed and known at compile time (adding one means writing code here regardless), so
/// there's no need to pay for a heap allocation and vtable indirection per target just to get a
/// `Vec` of them — a match arm does the same job for free.
pub enum Target {
    Devto(DevTo),
    Hashnode(Hashnode),
    Platform(Platform),
}

/// The output of `Target::prepare` — one variant per [`TargetKind`], mirroring `Target` itself
/// for the same reason (closed set, no dynamic dispatch needed).
#[derive(Debug)]
pub enum PreparedTarget {
    Devto(devto::PreparedRequest),
    Hashnode(hashnode::PreparedRequest),
    Platform(platform::PreparedRequest),
}

impl Target {
    pub fn kind(&self) -> TargetKind {
        match self {
            Self::Devto(_) => TargetKind::Devto,
            Self::Hashnode(_) => TargetKind::Hashnode,
            Self::Platform(_) => TargetKind::Platform,
        }
    }

    /// Validates `article`/`update` against this target — `--update` support (`Devto`/
    /// `Hashnode` don't have it) and `cover_image` compatibility (a local path is fine for
    /// `Platform`, an error for `Devto`/`Hashnode`, since neither has an image-upload endpoint)
    /// — and builds the exact request `execute` will send. No network access; for `Platform`
    /// specifically, this is also where `body_command` runs, exactly once, since it may be an
    /// arbitrary side-effecting script that `execute` must not repeat.
    ///
    /// Call this for *every* selected target before any of them actually publish: without it,
    /// an earlier target in the list can succeed — a real, side-effecting POST — before a later
    /// target's purely-local validation failure is discovered, and a rerun isn't safe
    /// (Dev.to/Hashnode have no update support, so retrying creates a duplicate post there).
    pub fn prepare(&self, article: &Article, update: bool) -> Result<PreparedTarget> {
        match self {
            Self::Devto(t) => Ok(PreparedTarget::Devto(t.prepare(article, update)?)),
            Self::Hashnode(t) => Ok(PreparedTarget::Hashnode(t.prepare(article, update)?)),
            Self::Platform(t) => Ok(PreparedTarget::Platform(t.prepare(article, update)?)),
        }
    }

    /// Sends the request `prepare` already built. `prepared` must have come from calling
    /// `prepare` on this exact `Target` — always true in practice, since nothing else produces
    /// a `PreparedTarget`.
    pub async fn execute(&self, prepared: PreparedTarget) -> Result<PublishOutcome> {
        match (self, prepared) {
            (Self::Devto(t), PreparedTarget::Devto(p)) => t.execute(p).await,
            (Self::Hashnode(t), PreparedTarget::Hashnode(p)) => t.execute(p).await,
            (Self::Platform(t), PreparedTarget::Platform(p)) => t.execute(p).await,
            _ => unreachable!("a PreparedTarget always matches the Target it was built from"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::publish::article::parse_article;
    use crate::publish::config::{DevToConfig, PlatformConfig};
    use assert_fs::{TempDir, prelude::*};

    fn article_with_local_cover(dir: &TempDir) -> Article {
        dir.child("hero.jpg").write_binary(&[0xff, 0xd8]).unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\ntitle: \"T\"\ndate: 2026-07-30\nsummary: \"s\"\n\
             cover_image: hero.jpg\npublish: [devto]\n---\nbody\n",
        )
        .unwrap();
        parse_article(file.path(), None).unwrap()
    }

    #[test]
    fn devto_prepare_rejects_a_local_cover_image_before_any_publish_call() {
        let dir = TempDir::new().unwrap();
        let article = article_with_local_cover(&dir);
        let target = Target::Devto(DevTo::new(
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "x".into(),
            },
        ));

        let err = target.prepare(&article, false).unwrap_err();
        assert!(
            err.to_string()
                .contains("only accepts an already-hosted URL")
        );
    }

    #[test]
    fn devto_prepare_rejects_update_before_any_publish_call() {
        let dir = TempDir::new().unwrap();
        let article = article_with_local_cover(&dir);
        let target = Target::Devto(DevTo::new(
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "x".into(),
            },
        ));

        // update=true should fail on the --update check, not the (also-failing) cover_image
        // check — prepare must catch the *first* thing execute() would have rejected.
        let err = target.prepare(&article, true).unwrap_err();
        assert!(err.to_string().contains("doesn't support --update"));
    }

    #[test]
    fn platform_prepare_accepts_a_local_cover_image() {
        let dir = TempDir::new().unwrap();
        let article = article_with_local_cover(&dir);
        let target = Target::Platform(Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: "http://unused".into(),
                path: None,
                token: "x".into(),
                body_command: None,
            },
        ));

        target.prepare(&article, false).unwrap();
    }

    #[test]
    fn platform_prepare_accepts_update() {
        let dir = TempDir::new().unwrap();
        let article = article_with_local_cover(&dir);
        let target = Target::Platform(Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: "http://unused".into(),
                path: None,
                token: "x".into(),
                body_command: None,
            },
        ));

        target.prepare(&article, true).unwrap();
    }
}
