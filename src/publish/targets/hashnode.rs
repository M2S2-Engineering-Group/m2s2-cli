use crate::publish::article::{Article, slugify};
use crate::publish::config::HashnodeConfig;
use crate::publish::target::{HttpTarget, PublishOutcome};
use anyhow::{Result, bail};
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

    fn with_endpoint(client: reqwest::Client, cfg: &HashnodeConfig, endpoint: impl Into<String>) -> Self {
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
struct PublishPostInput<'a> {
    title: &'a str,
    #[serde(rename = "contentMarkdown")]
    content_markdown: &'a str,
    #[serde(rename = "publicationId")]
    publication_id: &'a str,
    tags: Vec<Tag>,
    slug: &'a str,
}

impl Hashnode {
    pub async fn publish(&self, article: &Article, update: bool) -> Result<PublishOutcome> {
        if update {
            bail!("the hashnode target doesn't support --update yet");
        }

        let input = PublishPostInput {
            title: &article.title,
            content_markdown: &article.content,
            publication_id: &self.publication_id,
            tags: article
                .tags
                .iter()
                .map(|t| Tag { name: t.clone(), slug: slugify(t) })
                .collect(),
            slug: &article.slug,
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

        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
        if let Some(errors) = parsed.get("errors").filter(|e| !e.is_null()) {
            bail!("hashnode returned GraphQL errors: {errors}");
        }

        let url = parsed
            .pointer("/data/publishPost/post/url")
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
            &HashnodeConfig { token: "secret-pat".into(), publication_id: "pub1".into() },
            server.base_url(),
        );

        let outcome = target.publish(&article, false).await.unwrap();
        mock.assert();
        assert_eq!(outcome.message, "https://blog.example.com/hello");
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
            &HashnodeConfig { token: "t".into(), publication_id: "p".into() },
            server.base_url(),
        );

        let err = target.publish(&article, false).await.unwrap_err();
        assert!(err.to_string().contains("Hashnode Pro"));
    }
}
