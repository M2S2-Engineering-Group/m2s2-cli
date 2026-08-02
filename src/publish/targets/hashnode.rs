use crate::markdown::slugify;
use crate::publish::article::Article;
use crate::publish::config::HashnodeConfig;
use crate::publish::cover_image;
use crate::publish::target::{HttpTarget, PublishOutcome};
use anyhow::{Context, Result, bail};
use serde::Serialize;

const PUBLISH_POST_MUTATION: &str = r#"
mutation PublishPost($input: PublishPostInput!) {
  publishPost(input: $input) {
    post { id url }
  }
}
"#;

pub struct Hashnode {
    token: String,
    publication_id: String,
    http: HttpTarget,
}

impl Hashnode {
    pub fn new(client: reqwest::Client, cfg: &HashnodeConfig) -> Self {
        Self::with_endpoint(client, cfg, "https://gql.hashnode.com")
    }

    fn with_endpoint(
        client: reqwest::Client,
        cfg: &HashnodeConfig,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            token: cfg.token.clone(),
            publication_id: cfg.publication_id.clone(),
            http: HttpTarget::new(client, endpoint),
        }
    }
}

#[derive(Serialize)]
struct Tag {
    name: String,
    slug: String,
}

#[derive(Serialize)]
struct CoverImageOptions<'a> {
    #[serde(rename = "coverImageURL")]
    cover_image_url: &'a str,
}

#[derive(Serialize)]
struct PublishPostInput<'a> {
    title: &'a str,
    #[serde(rename = "contentMarkdown")]
    content_markdown: &'a str,
    #[serde(rename = "publicationId")]
    publication_id: &'a str,
    tags: Vec<Tag>,
    slug: &'a str,
    #[serde(rename = "coverImageOptions", skip_serializing_if = "Option::is_none")]
    cover_image_options: Option<CoverImageOptions<'a>>,
}

/// Everything `execute` needs, computed once by `prepare` — no lifetime parameter, same
/// rationale as `devto::PreparedRequest`.
#[derive(Debug)]
pub struct PreparedRequest {
    title: String,
    content_markdown: String,
    tags: Vec<String>,
    slug: String,
    cover_image_url: Option<String>,
}

impl Hashnode {
    /// Shared by `prepare` and (indirectly, via `prepare`) `execute`'s callers, so the two can't
    /// drift apart on what's checked before any network call happens.
    pub(crate) fn check_update_supported(update: bool) -> Result<()> {
        if update {
            bail!("the hashnode target doesn't support --update yet");
        }
        Ok(())
    }

    /// Local-only: validates `--update` support and `cover_image` compatibility, and builds the
    /// exact request `execute` will send. No network access.
    pub fn prepare(&self, article: &Article, update: bool) -> Result<PreparedRequest> {
        Self::check_update_supported(update)?;

        let cover_image_url = cover_image::resolve_url_only(
            article.cover_image.as_deref(),
            &article.base_dir,
            "hashnode",
        )?;

        Ok(PreparedRequest {
            title: article.title.clone(),
            content_markdown: article.content.clone(),
            tags: article.tags.clone(),
            slug: article.slug.clone(),
            cover_image_url,
        })
    }

    pub async fn execute(&self, prepared: PreparedRequest) -> Result<PublishOutcome> {
        let input = PublishPostInput {
            title: &prepared.title,
            content_markdown: &prepared.content_markdown,
            publication_id: &self.publication_id,
            tags: prepared
                .tags
                .iter()
                .map(|t| Tag {
                    name: t.clone(),
                    slug: slugify(t),
                })
                .collect(),
            slug: &prepared.slug,
            cover_image_options: prepared
                .cover_image_url
                .as_deref()
                .map(|url| CoverImageOptions {
                    cover_image_url: url,
                }),
        };

        let resp = self
            .http
            .client
            .post(&self.http.base_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&serde_json::json!({
                "query": PUBLISH_POST_MUTATION,
                "variables": { "input": input },
            }))
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            bail!("hashnode returned {status}: {text}");
        }

        let parsed: serde_json::Value = serde_json::from_str(&text).with_context(|| {
            format!("hashnode returned {status} but the response body wasn't valid JSON: {text}")
        })?;
        if let Some(errors) = parsed.get("errors").filter(|e| !e.is_null()) {
            bail!("hashnode returned GraphQL errors: {errors}");
        }

        let url = parsed
            .pointer("/data/publishPost/post/url")
            .and_then(|v| v.as_str())
            .with_context(|| {
                format!("hashnode returned {status} but no post url in the response: {text}")
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
             tags: [rust, cli]\n\
             publish: [hashnode]\n\
             ---\n\
             Body.\n",
        )
        .unwrap();
        parse_article(file.path(), None).unwrap()
    }

    #[tokio::test]
    async fn publish_sends_bearer_token_and_parses_url() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .header("Authorization", "Bearer secret-pat");
            then.status(200).json_body(serde_json::json!({
                "data": { "publishPost": { "post": { "id": "1", "url": "https://blog.example.com/hello" } } }
            }));
        });

        let target = Hashnode::with_endpoint(
            reqwest::Client::new(),
            &HashnodeConfig {
                token: "secret-pat".into(),
                publication_id: "pub1".into(),
            },
            server.base_url(),
        );

        let prepared = target.prepare(&article, false).unwrap();
        let outcome = target.execute(prepared).await.unwrap();
        mock.assert();
        assert_eq!(outcome.message, "https://blog.example.com/hello");
    }

    #[tokio::test]
    async fn malformed_json_on_success_status_is_an_error() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).body("<html>not json</html>");
        });

        let target = Hashnode::with_endpoint(
            reqwest::Client::new(),
            &HashnodeConfig {
                token: "t".into(),
                publication_id: "p".into(),
            },
            server.base_url(),
        );

        let prepared = target.prepare(&article, false).unwrap();
        let err = target.execute(prepared).await.unwrap_err();
        assert!(err.to_string().contains("wasn't valid JSON"));
    }

    #[tokio::test]
    async fn missing_post_url_on_success_status_is_an_error() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(serde_json::json!({"data": {}}));
        });

        let target = Hashnode::with_endpoint(
            reqwest::Client::new(),
            &HashnodeConfig {
                token: "t".into(),
                publication_id: "p".into(),
            },
            server.base_url(),
        );

        let prepared = target.prepare(&article, false).unwrap();
        let err = target.execute(prepared).await.unwrap_err();
        assert!(err.to_string().contains("no post url"));
    }

    #[tokio::test]
    async fn graphql_errors_field_is_surfaced_even_on_http_200() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(serde_json::json!({
                "errors": [{"message": "publication requires Hashnode Pro"}]
            }));
        });

        let target = Hashnode::with_endpoint(
            reqwest::Client::new(),
            &HashnodeConfig {
                token: "t".into(),
                publication_id: "p".into(),
            },
            server.base_url(),
        );

        let prepared = target.prepare(&article, false).unwrap();
        let err = target.execute(prepared).await.unwrap_err();
        assert!(err.to_string().contains("Hashnode Pro"));
    }

    #[tokio::test]
    async fn cover_image_url_is_sent_via_cover_image_options() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\ntitle: \"Hello\"\ndate: 2026-07-30\nsummary: \"s\"\n\
             cover_image: https://example.com/hero.jpg\npublish: [hashnode]\n---\nBody.\n",
        )
        .unwrap();
        let article = parse_article(file.path(), None).unwrap();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/").json_body_partial(
                r#"{"variables":{"input":{"coverImageOptions":{"coverImageURL":"https://example.com/hero.jpg"}}}}"#,
            );
            then.status(200).json_body(serde_json::json!({
                "data": { "publishPost": { "post": { "id": "1", "url": "https://blog.example.com/hello" } } }
            }));
        });

        let target = Hashnode::with_endpoint(
            reqwest::Client::new(),
            &HashnodeConfig {
                token: "t".into(),
                publication_id: "p".into(),
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
             cover_image: hero.jpg\npublish: [hashnode]\n---\nBody.\n",
        )
        .unwrap();
        let article = parse_article(file.path(), None).unwrap();

        let target = Hashnode::with_endpoint(
            reqwest::Client::new(),
            &HashnodeConfig {
                token: "t".into(),
                publication_id: "p".into(),
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
