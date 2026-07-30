use crate::publish::article::Article;
use crate::publish::config::M2s2Config;
use crate::publish::target::{HttpTarget, PublishOutcome, PublishTarget};
use anyhow::{Result, bail};
use async_trait::async_trait;
use serde::Serialize;

pub struct M2s2 {
    token: String,
    http: HttpTarget,
}

impl M2s2 {
    pub fn new(client: reqwest::Client, cfg: &M2s2Config) -> Self {
        Self {
            token: cfg.token.clone(),
            http: HttpTarget::new(client, cfg.endpoint.trim_end_matches('/')),
        }
    }
}

#[derive(Serialize)]
struct BlogPostRequest<'a> {
    slug: &'a str,
    title: &'a str,
    date: &'a str,
    summary: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    excerpt: Option<&'a str>,
    tags: &'a [String],
    #[serde(rename = "coverImage", skip_serializing_if = "Option::is_none")]
    cover_image: Option<&'a str>,
    content: &'a str,
}

#[async_trait]
impl PublishTarget for M2s2 {
    fn name(&self) -> &'static str {
        "m2s2"
    }

    async fn publish(&self, article: &Article, update: bool) -> Result<PublishOutcome> {
        let body = BlogPostRequest {
            slug: &article.slug,
            title: &article.title,
            date: &article.date,
            summary: &article.summary,
            excerpt: article.excerpt.as_deref(),
            tags: &article.tags,
            cover_image: article.cover_image.as_deref(),
            content: &article.content,
        };

        let url = format!("{}/admin/blog", self.http.base_url);
        let req = if update {
            self.http.client.put(&url).query(&[("slug", &article.slug)])
        } else {
            self.http.client.post(&url)
        };

        let resp = req.bearer_auth(&self.token).json(&body).send().await?;

        let status = resp.status();
        if status.as_u16() == 409 {
            bail!(
                "slug '{}' already exists on the m2s2 blog — rerun with --update to overwrite it",
                article.slug
            );
        }
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            bail!("m2s2 platform returned {status}: {text}");
        }

        Ok(PublishOutcome {
            message: if update { "updated".to_string() } else { "created".to_string() },
        })
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
             slug: hello\n\
             date: 2026-07-30\n\
             summary: \"A test\"\n\
             tags: [rust]\n\
             publish: [m2s2]\n\
             ---\n\
             Body.\n",
        )
        .unwrap();
        parse_article(file.path(), None).unwrap()
    }

    #[tokio::test]
    async fn create_posts_to_admin_blog() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/admin/blog")
                .header("authorization", "Bearer tok")
                .json_body_partial(r#"{"slug":"hello","title":"Hello"}"#);
            then.status(201).body(r#"{"message":"post created"}"#);
        });

        let target = M2s2::new(
            reqwest::Client::new(),
            &M2s2Config { endpoint: server.base_url(), token: "tok".into() },
        );
        let outcome = target.publish(&article, false).await.unwrap();
        mock.assert();
        assert_eq!(outcome.message, "created");
    }

    #[tokio::test]
    async fn update_uses_put_with_slug_query_param() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let mock = server.mock(|when, then| {
            when.method(PUT).path("/admin/blog").query_param("slug", "hello");
            then.status(200).body(r#"{"message":"post updated"}"#);
        });

        let target = M2s2::new(
            reqwest::Client::new(),
            &M2s2Config { endpoint: server.base_url(), token: "tok".into() },
        );
        let outcome = target.publish(&article, true).await.unwrap();
        mock.assert();
        assert_eq!(outcome.message, "updated");
    }

    #[tokio::test]
    async fn conflict_suggests_update_flag() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        server.mock(|when, then| {
            when.method(POST).path("/admin/blog");
            then.status(409);
        });

        let target = M2s2::new(
            reqwest::Client::new(),
            &M2s2Config { endpoint: server.base_url(), token: "tok".into() },
        );
        let err = target.publish(&article, false).await.unwrap_err();
        assert!(err.to_string().contains("--update"));
    }
}
