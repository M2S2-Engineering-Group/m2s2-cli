use crate::publish::article::Article;
use crate::publish::config::DevToConfig;
use crate::publish::cover_image;
use crate::publish::target::{HttpTarget, PublishOutcome};
use anyhow::{Context, Result, bail};
use serde::Serialize;

pub struct DevTo {
    api_key: String,
    http: HttpTarget,
}

impl DevTo {
    pub fn new(client: reqwest::Client, cfg: &DevToConfig) -> Self {
        Self::with_base_url(client, cfg, "https://dev.to")
    }

    fn with_base_url(
        client: reqwest::Client,
        cfg: &DevToConfig,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            api_key: cfg.api_key.clone(),
            http: HttpTarget::new(client, base_url),
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
    tags: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    main_image: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_url: Option<&'a str>,
}

/// Everything `execute` needs, computed once by `prepare` — no lifetime parameter (owned data)
/// so it can be built for every selected target before any of them sends a request, without
/// tying its lifetime to how long the command loop holds onto `Article`.
#[derive(Debug)]
pub struct PreparedRequest {
    title: String,
    body_markdown: String,
    description: String,
    tags: String,
    main_image: Option<String>,
    canonical_url: Option<String>,
}

impl DevTo {
    /// Shared by `prepare` and (indirectly, via `prepare`) `execute`'s callers, so the two can't
    /// drift apart on what's checked before any network call happens.
    pub(crate) fn check_update_supported(update: bool) -> Result<()> {
        if update {
            bail!("the devto target doesn't support --update yet");
        }
        Ok(())
    }

    /// Local-only: validates `--update` support and `cover_image` compatibility, and builds the
    /// exact request `execute` will send. No network access.
    pub fn prepare(&self, article: &Article, update: bool) -> Result<PreparedRequest> {
        Self::check_update_supported(update)?;

        let main_image = cover_image::resolve_url_only(
            article.cover_image.as_deref(),
            &article.base_dir,
            "devto",
        )?;

        Ok(PreparedRequest {
            title: article.title.clone(),
            body_markdown: article.content.clone(),
            description: article.summary.clone(),
            tags: article
                .tags
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>()
                .join(","),
            main_image,
            canonical_url: article.canonical_url.clone(),
        })
    }

    pub async fn execute(&self, prepared: PreparedRequest) -> Result<PublishOutcome> {
        let body = ArticleBody {
            article: ArticleFields {
                title: &prepared.title,
                body_markdown: &prepared.body_markdown,
                published: true,
                description: &prepared.description,
                tags: &prepared.tags,
                main_image: prepared.main_image.as_deref(),
                canonical_url: prepared.canonical_url.as_deref(),
            },
        };

        let resp = self
            .http
            .client
            .post(format!("{}/api/articles", self.http.base_url))
            .header("api-key", &self.api_key)
            .header("Accept", "application/vnd.forem.api-v1+json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            bail!("dev.to returned {status}: {text}");
        }

        let parsed: serde_json::Value = serde_json::from_str(&text).with_context(|| {
            format!("dev.to returned {status} but the response body wasn't valid JSON: {text}")
        })?;
        let url = parsed
            .get("url")
            .and_then(|v| v.as_str())
            .with_context(|| {
                format!("dev.to returned {status} but no \"url\" field in the response: {text}")
            })?
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
                .json_body_partial(
                    r#"{"article":{"title":"Hello","tags":"rust,cli,testing,extra"}}"#,
                );
            then.status(201)
                .json_body(serde_json::json!({"url": "https://dev.to/x/hello"}));
        });

        let target = DevTo::with_base_url(
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "secret".into(),
            },
            server.base_url(),
        );

        let prepared = target.prepare(&article, false).unwrap();
        let outcome = target.execute(prepared).await.unwrap();
        mock.assert();
        assert_eq!(outcome.message, "https://dev.to/x/hello");
    }

    #[tokio::test]
    async fn malformed_json_on_success_status_is_an_error() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        server.mock(|when, then| {
            when.method(POST).path("/api/articles");
            then.status(201).body("<html>not json</html>");
        });

        let target = DevTo::with_base_url(
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "secret".into(),
            },
            server.base_url(),
        );

        let prepared = target.prepare(&article, false).unwrap();
        let err = target.execute(prepared).await.unwrap_err();
        assert!(err.to_string().contains("wasn't valid JSON"));
    }

    #[tokio::test]
    async fn missing_url_field_on_success_status_is_an_error() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        server.mock(|when, then| {
            when.method(POST).path("/api/articles");
            then.status(201).json_body(serde_json::json!({"id": 123}));
        });

        let target = DevTo::with_base_url(
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "secret".into(),
            },
            server.base_url(),
        );

        let prepared = target.prepare(&article, false).unwrap();
        let err = target.execute(prepared).await.unwrap_err();
        assert!(err.to_string().contains("no \"url\" field"));
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
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "secret".into(),
            },
            server.base_url(),
        );

        let prepared = target.prepare(&article, false).unwrap();
        let err = target.execute(prepared).await.unwrap_err();
        assert!(err.to_string().contains("422"));
    }

    #[tokio::test]
    async fn update_is_not_supported() {
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);
        let target = DevTo::with_base_url(
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "x".into(),
            },
            "http://unused",
        );
        let err = target.prepare(&article, true).unwrap_err();
        assert!(err.to_string().contains("doesn't support --update"));
    }

    #[tokio::test]
    async fn cover_image_url_is_sent_as_main_image() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\ntitle: \"Hello\"\ndate: 2026-07-30\nsummary: \"s\"\n\
             cover_image: https://example.com/hero.jpg\npublish: [devto]\n---\nBody.\n",
        )
        .unwrap();
        let article = parse_article(file.path(), None).unwrap();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/articles")
                .json_body_partial(r#"{"article":{"main_image":"https://example.com/hero.jpg"}}"#);
            then.status(201)
                .json_body(serde_json::json!({"url": "https://dev.to/x/hello"}));
        });

        let target = DevTo::with_base_url(
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "x".into(),
            },
            server.base_url(),
        );
        let prepared = target.prepare(&article, false).unwrap();
        target.execute(prepared).await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn cover_image_local_path_is_a_clear_error() {
        let dir = TempDir::new().unwrap();
        dir.child("hero.jpg").write_binary(&[0xff, 0xd8]).unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\ntitle: \"Hello\"\ndate: 2026-07-30\nsummary: \"s\"\n\
             cover_image: hero.jpg\npublish: [devto]\n---\nBody.\n",
        )
        .unwrap();
        let article = parse_article(file.path(), None).unwrap();

        let target = DevTo::with_base_url(
            reqwest::Client::new(),
            &DevToConfig {
                api_key: "x".into(),
            },
            "http://unused",
        );
        let err = target.prepare(&article, false).unwrap_err();
        assert!(
            err.to_string()
                .contains("only accepts an already-hosted URL")
        );
    }
}
