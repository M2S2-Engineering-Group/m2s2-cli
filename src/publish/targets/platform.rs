use crate::publish::article::Article;
use crate::publish::config::PlatformConfig;
use crate::publish::cover_image::{self, CoverImage};
use crate::publish::target::{HttpTarget, PublishOutcome};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::io::Write;
use std::process::Stdio;

const DEFAULT_PATH: &str = "/admin/blog";

pub struct Platform {
    token: String,
    path: String,
    body_command: Option<String>,
    http: HttpTarget,
}

impl Platform {
    pub fn new(client: reqwest::Client, cfg: &PlatformConfig) -> Self {
        let path = cfg.path.clone().unwrap_or_else(|| DEFAULT_PATH.to_string());
        Self {
            token: cfg.token.clone(),
            path: if path.starts_with('/') {
                path
            } else {
                format!("/{path}")
            },
            body_command: cfg.body_command.clone(),
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
    /// Set when `cover_image` resolved to an already-hosted URL.
    #[serde(rename = "coverImage", skip_serializing_if = "Option::is_none")]
    cover_image: Option<&'a str>,
    /// These three are set together, only when `cover_image` resolved to a local file — the
    /// server (`maybeUploadCover` in m2s2-platform's blog.go) decodes and uploads it, then
    /// fills in `coverImage` itself.
    #[serde(rename = "coverImageFilename", skip_serializing_if = "Option::is_none")]
    cover_image_filename: Option<&'a str>,
    #[serde(rename = "coverImageData", skip_serializing_if = "Option::is_none")]
    cover_image_data: Option<&'a str>,
    #[serde(
        rename = "coverImageContentType",
        skip_serializing_if = "Option::is_none"
    )]
    cover_image_content_type: Option<&'a str>,
    content: &'a str,
}

/// What `body_command` receives on stdin, as JSON: the article's fields plus whether this is a
/// create or an update, in case the hook needs to build a different body for each.
#[derive(Serialize)]
struct BodyCommandInput<'a> {
    #[serde(flatten)]
    article: &'a Article,
    update: bool,
}

impl Platform {
    pub async fn publish(&self, article: &Article, update: bool) -> Result<PublishOutcome> {
        let body = self.build_body(article, update)?;

        let url = format!("{}{}", self.http.base_url, self.path);
        let req = if update {
            self.http.client.put(&url).query(&[("slug", &article.slug)])
        } else {
            self.http.client.post(&url)
        };

        let resp = req.bearer_auth(&self.token).json(&body).send().await?;

        let status = resp.status();
        if status.as_u16() == 409 {
            bail!(
                "slug '{}' already exists on the platform blog — rerun with --update to overwrite it",
                article.slug
            );
        }
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            bail!("platform returned {status}: {text}");
        }

        Ok(PublishOutcome {
            message: if update {
                "updated".to_string()
            } else {
                "created".to_string()
            },
        })
    }

    /// The default field mapping, unless `body_command` is configured — in which case that
    /// command's stdout, verbatim, *is* the request body.
    fn build_body(&self, article: &Article, update: bool) -> Result<serde_json::Value> {
        match &self.body_command {
            Some(command) => run_body_command(command, article, update),
            None => {
                let cover = cover_image::resolve(article)?;
                let (cover_image, cover_image_filename, cover_image_data, cover_image_content_type) =
                    match &cover {
                        Some(CoverImage::Url(url)) => (Some(url.as_str()), None, None, None),
                        Some(CoverImage::Local(img)) => (
                            None,
                            Some(img.filename.as_str()),
                            Some(img.base64_data.as_str()),
                            Some(img.content_type),
                        ),
                        None => (None, None, None, None),
                    };

                Ok(serde_json::to_value(BlogPostRequest {
                    slug: &article.slug,
                    title: &article.title,
                    date: &article.date,
                    summary: &article.summary,
                    excerpt: article.excerpt.as_deref(),
                    tags: &article.tags,
                    cover_image,
                    cover_image_filename,
                    cover_image_data,
                    cover_image_content_type,
                    content: &article.content,
                })?)
            }
        }
    }
}

/// Run `command` through a shell, piping the article (+ `update`) to it as JSON on stdin, and
/// parse its stdout as the JSON request body.
fn run_body_command(command: &str, article: &Article, update: bool) -> Result<serde_json::Value> {
    let payload = serde_json::to_vec(&BodyCommandInput { article, update })
        .context("failed to serialize article for body_command")?;

    let mut child = shell_command(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to run body_command `{command}`"))?;

    // The child may exit (closing its stdin) before or while we write — e.g. plain `echo`
    // never reads stdin at all. That alone isn't a failure: the child's exit status and
    // stdout, checked below, are the real source of truth. Only a write error that ISN'T the
    // child simply hanging up is treated as fatal here.
    if let Err(err) = child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(&payload)
        && err.kind() != std::io::ErrorKind::BrokenPipe
    {
        return Err(err).with_context(|| {
            format!("failed to write the article to body_command `{command}`'s stdin")
        });
    }

    let output = child
        .wait_with_output()
        .with_context(|| format!("failed to run body_command `{command}`"))?;

    if !output.status.success() {
        bail!(
            "body_command `{command}` exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("body_command `{command}` didn't print a JSON object on stdout"))
}

#[cfg(target_os = "windows")]
fn shell_command(command: &str) -> std::process::Command {
    let mut c = std::process::Command::new("cmd");
    c.args(["/C", command]);
    c
}

#[cfg(not(target_os = "windows"))]
fn shell_command(command: &str) -> std::process::Command {
    let mut c = std::process::Command::new("sh");
    c.args(["-c", command]);
    c
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
             publish: [platform]\n\
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

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: None,
                token: "tok".into(),
                body_command: None,
            },
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
            when.method(PUT)
                .path("/admin/blog")
                .query_param("slug", "hello");
            then.status(200).body(r#"{"message":"post updated"}"#);
        });

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: None,
                token: "tok".into(),
                body_command: None,
            },
        );
        let outcome = target.publish(&article, true).await.unwrap();
        mock.assert();
        assert_eq!(outcome.message, "updated");
    }

    #[tokio::test]
    async fn cover_image_url_is_sent_as_cover_image_field() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\ntitle: \"Hello\"\nslug: hello\ndate: 2026-07-30\nsummary: \"s\"\n\
             cover_image: https://example.com/hero.jpg\npublish: [platform]\n---\nBody.\n",
        )
        .unwrap();
        let article = parse_article(file.path(), None).unwrap();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/admin/blog")
                .json_body_partial(r#"{"coverImage":"https://example.com/hero.jpg"}"#);
            then.status(201).body(r#"{"message":"post created"}"#);
        });

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: None,
                token: "tok".into(),
                body_command: None,
            },
        );
        target.publish(&article, false).await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn cover_image_local_path_is_uploaded_as_base64() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        dir.child("hero.png")
            .write_binary(&[0x89, 0x50, 0x4e, 0x47])
            .unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\ntitle: \"Hello\"\nslug: hello\ndate: 2026-07-30\nsummary: \"s\"\n\
             cover_image: hero.png\npublish: [platform]\n---\nBody.\n",
        )
        .unwrap();
        let article = parse_article(file.path(), None).unwrap();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/admin/blog").json_body_partial(format!(
                r#"{{"coverImageFilename":"hero.png","coverImageContentType":"image/png","coverImageData":"{}"}}"#,
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [0x89, 0x50, 0x4e, 0x47]),
            ));
            then.status(201).body(r#"{"message":"post created"}"#);
        });

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: None,
                token: "tok".into(),
                body_command: None,
            },
        );
        target.publish(&article, false).await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn custom_path_overrides_the_default() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let mock = server.mock(|when, then| {
            when.method(POST).path("/wp-json/blog/v1/posts");
            then.status(201).body(r#"{"message":"post created"}"#);
        });

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: Some("wp-json/blog/v1/posts".to_string()),
                token: "tok".into(),
                body_command: None,
            },
        );
        target.publish(&article, false).await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn body_command_output_is_sent_verbatim() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/admin/blog")
                .json_body(serde_json::json!({"custom": "shape", "heading": "Hello"}));
            then.status(201).body(r#"{"message":"post created"}"#);
        });

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: None,
                token: "tok".into(),
                body_command: Some(
                    r#"cat >/dev/null; echo '{"custom":"shape","heading":"Hello"}'"#.to_string(),
                ),
            },
        );
        target.publish(&article, false).await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn body_command_that_ignores_stdin_still_succeeds() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let mock = server.mock(|when, then| {
            when.method(POST).path("/admin/blog");
            then.status(201).body(r#"{"message":"post created"}"#);
        });

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: None,
                token: "tok".into(),
                body_command: Some(r#"echo '{"ignored":"stdin"}'"#.to_string()),
            },
        );
        target.publish(&article, false).await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn body_command_receives_the_article_and_update_flag_on_stdin() {
        let server = MockServer::start();
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        // `cat` echoes stdin back, so the response body doubles as proof of what was sent.
        let mock = server.mock(|when, then| {
            when.method(PUT)
                .path("/admin/blog")
                .json_body_partial(r#"{"slug":"hello","update":true}"#);
            then.status(200).body(r#"{"message":"post updated"}"#);
        });

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: None,
                token: "tok".into(),
                body_command: Some("cat".to_string()),
            },
        );
        target.publish(&article, true).await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn body_command_failure_is_a_clear_error() {
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: "http://unused".to_string(),
                path: None,
                token: "tok".into(),
                body_command: Some("cat >/dev/null; echo 'not json' >&2; exit 1".to_string()),
            },
        );
        let err = target.publish(&article, false).await.unwrap_err();
        assert!(err.to_string().contains("body_command"));
    }

    #[tokio::test]
    async fn body_command_non_json_output_is_a_clear_error() {
        let dir = TempDir::new().unwrap();
        let article = sample_article(&dir);

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: "http://unused".to_string(),
                path: None,
                token: "tok".into(),
                body_command: Some("cat >/dev/null; echo 'not json'".to_string()),
            },
        );
        let err = target.publish(&article, false).await.unwrap_err();
        assert!(err.to_string().contains("didn't print a JSON object"));
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

        let target = Platform::new(
            reqwest::Client::new(),
            &PlatformConfig {
                endpoint: server.base_url(),
                path: None,
                token: "tok".into(),
                body_command: None,
            },
        );
        let err = target.publish(&article, false).await.unwrap_err();
        assert!(err.to_string().contains("--update"));
    }
}
