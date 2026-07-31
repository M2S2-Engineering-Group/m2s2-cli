use crate::publish::article::Article;
use crate::publish::target_kind::TargetKind;
use crate::publish::targets::devto::DevTo;
use crate::publish::targets::hashnode::Hashnode;
use crate::publish::targets::platform::Platform;
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

impl Target {
    pub fn kind(&self) -> TargetKind {
        match self {
            Self::Devto(_) => TargetKind::Devto,
            Self::Hashnode(_) => TargetKind::Hashnode,
            Self::Platform(_) => TargetKind::Platform,
        }
    }

    /// `update`: create (`false`) vs. update an existing post (`true`), where the target
    /// supports the distinction. Targets that don't support updates return an error.
    pub async fn publish(&self, article: &Article, update: bool) -> Result<PublishOutcome> {
        match self {
            Self::Devto(t) => t.publish(article, update).await,
            Self::Hashnode(t) => t.publish(article, update).await,
            Self::Platform(t) => t.publish(article, update).await,
        }
    }
}
