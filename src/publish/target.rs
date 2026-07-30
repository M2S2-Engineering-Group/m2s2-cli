use crate::publish::article::Article;
use anyhow::Result;
use async_trait::async_trait;

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
        Self { client, base_url: base_url.into() }
    }
}

/// One place to publish an [`Article`] to. Implement this for a new connector and register it
/// in `publish::targets::build_one` — that's the only other place that needs to know about it.
#[async_trait]
pub trait PublishTarget {
    fn name(&self) -> &'static str;

    /// `update`: create (`false`) vs. update an existing post (`true`), where the target
    /// supports the distinction. Targets that don't support updates should return an error.
    async fn publish(&self, article: &Article, update: bool) -> Result<PublishOutcome>;
}
