use crate::publish::article::Article;
use crate::publish::config::DevToConfig;
use crate::publish::target::{PublishOutcome, PublishTarget};
use anyhow::{Result, bail};
use async_trait::async_trait;
use serde::Serialize;

pub struct DevTo {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl DevTo {
    pub fn new(cfg: &DevToConfig) -> Self {
        Self::with_base_url(cfg, "https://dev.to".to_string())
    }

    fn with_base_url(cfg: &DevToConfig, base_url: String) -> Self {
        Self {
            api_key: cfg.api_key.clone(),
            client: reqwest::Client::new(),
            base_url,
        }
    }
}

#[derive(Serialize)]
struct ArticleBody<'a> {
    article: ArticleFields<'a>,
}

#[derive(Serialize)]
struct ArticleFields<'a> {
    title: &'a str,
    body_markdown: &'a str,
    published: bool,
    description: &'a str,
    /// Dev.to takes a comma-separated string, capped at 4 tags.
    tags: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    main_image: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_url: Option<&'a str>,
}

#[async_trait]
impl PublishTarget for DevTo {
    fn name(&self) -> &'static str {
        "devto"
    }

    async fn publish(&self, article: &Article, update: bool) -> Result<PublishOutcome> {
        if update {
            bail!("the devto target doesn't support --update yet");
        }

        let body = ArticleBody {
            article: ArticleFields {
                title: &article.title,
                body_markdown: &article.content,
                published: true,
                description: &article.summary,
                tags: article.tags.iter().take(4).cloned().collect::<Vec<_>>().join(","),
                main_image: article.cover_image.as_deref(),
                canonical_url: article.canonical_url.as_deref(),
            },
        };

        let resp = self
            .client
            .post(format!("{}/api/articles", self.base_url))
            .header("api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            bail!("dev.to returned {status}: {text}");
        }

        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
        let url = parsed
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("(published, no URL in response)")
            .to_string();
        Ok(PublishOutcome { message: url })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::publish::article::parse_article;
    use assert_fs::{TempDir, prelude::*};
    use httpmock::prelude::*;

    fn sample_article(dir: &TempDir) -> Article {
        let file = dir.child("post.md");
        file.write_str(
            "---\n\
             title: \"Hello\"\n\
             date: 2026-07-30\n\
             summary: \"A test\"\n\
             tags: [rust, cli, testing, extra, dropped]\n\
             publish: [devto]\n\
             ---\n\
             Body.\n",
        )
        .unwrap();
        parse_article(file.path(), None).unwrap()
    }

    #[tokio::test]
    async fn publish_posts_expected_body_and_parses_url() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/articles")
                .header("api-key", "secret")
                .json_body_partial(r#"{"article":{"title":"Hello","tags":"rust,cli,testing,extra"}}"#);
            then.status(201)
                .json_body(serde_json::json!({"url": "https://dev.to/x/hello"}));
        });

        let target = DevTo::with_base_url(
            &DevToConfig { api_key: "secret".into() },
            server.base_url(),
        );

        let outcome = target.publish(&article, false).await.unwrap();
        mock.assert();
        assert_eq!(outcome.message, "https://dev.to/x/hello");
    }

    #[tokio::test]
    async fn non_success_status_is_an_error() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        server.mock(|when, then| {
            when.method(POST).path("/api/articles");
            then.status(422).body(r#"{"error":"invalid"}"#);
        });

        let target = DevTo::with_base_url(
            &DevToConfig { api_key: "secret".into() },
            server.base_url(),
        );

        let err = target.publish(&article, false).await.unwrap_err();
        assert!(err.to_string().contains("422"));
    }

    #[tokio::test]
    async fn update_is_not_supported() {
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);
        let target = DevTo::with_base_url(&DevToConfig { api_key: "x".into() }, "http://unused".into());
        let err = target.publish(&article, true).await.unwrap_err();
        assert!(err.to_string().contains("doesn't support --update"));
    }
}
