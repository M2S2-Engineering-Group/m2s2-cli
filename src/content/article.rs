//! The canonical content-delivery article schema (`docs/m2s2-cli-content-delivery-integration.md`
//! §7) — deliberately a separate schema from `publish::article::Article`, not a variant or
//! superset of it: this one requires `canonical_url` and drops `date`/`targets` entirely (target
//! selection here is a CLI-flag/platform-policy concern, not a frontmatter field). Unifying the
//! two structs would silently break one side's requiredness rules.

use crate::markdown::{slugify, split_frontmatter};
use crate::publish::cover_image::{self, CoverImage};
use crate::report::{CheckStatus, OutputReport};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// A parsed article. Structural parsing only validates that the file is readable and has
/// well-formed YAML frontmatter — every field below is left as-authored (including absent
/// required fields, as `None`/empty) so `validate` can report a full checklist instead of
/// stopping at the first missing field.
#[derive(Debug, Serialize)]
pub struct Article {
    pub schema_version: u32,
    pub title: Option<String>,
    pub slug: String,
    pub summary: Option<String>,
    pub tags: Vec<String>,
    pub canonical_url: Option<String>,
    /// Raw frontmatter value — a URL, or a path to a local file resolved relative to `base_dir`.
    pub cover_image: Option<String>,
    pub content: String,
    /// The directory containing the source Markdown file — relative paths in frontmatter or body
    /// links are resolved against this.
    pub base_dir: PathBuf,
}

#[derive(Deserialize)]
struct Frontmatter {
    schema_version: Option<u32>,
    title: Option<String>,
    slug: Option<String>,
    summary: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    canonical_url: Option<String>,
    cover_image: Option<String>,
}

/// Parse a Markdown file with a leading `---`-delimited YAML frontmatter block. Fails only on
/// structural problems (unreadable file, no frontmatter, invalid YAML) — a missing `title` or
/// `canonical_url` parses fine and is reported by `validate` instead.
pub fn parse_article(path: &Path) -> Result<Article> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let (fm_src, body) = split_frontmatter(&raw).with_context(|| {
        format!(
            "{} has no YAML frontmatter (expected a leading `---` block)",
            path.display()
        )
    })?;

    let fm: Frontmatter = serde_yaml::from_str(&fm_src)
        .with_context(|| format!("invalid frontmatter in {}", path.display()))?;

    let slug = fm
        .slug
        .unwrap_or_else(|| slugify(path.file_stem().and_then(|s| s.to_str()).unwrap_or("post")));

    let base_dir = path.parent().map(Path::to_path_buf).unwrap_or_default();

    Ok(Article {
        schema_version: fm.schema_version.unwrap_or(CURRENT_SCHEMA_VERSION),
        title: fm.title,
        slug,
        summary: fm.summary,
        tags: fm.tags,
        canonical_url: fm.canonical_url,
        cover_image: fm.cover_image,
        content: body.trim().to_string(),
        base_dir,
    })
}

/// Runs every offline validation rule from `docs/m2s2-cli-content-delivery-integration.md` §7 and
/// returns a full report — never short-circuits on the first failure, so authors see every
/// problem in one pass. `source_path` is excluded from the duplicate-slug scan of `articles_dir`.
pub fn validate(
    article: &Article,
    source_path: &Path,
    articles_dir: &Path,
    assets_dir: &Path,
    canonical_base_url: &str,
) -> OutputReport {
    let mut report = OutputReport::new();
    let roots = [articles_dir, assets_dir];

    check_required(article, &mut report);
    check_canonical_url_https(article, &mut report);
    check_canonical_url_matches_base(article, canonical_base_url, &mut report);
    check_slug(article, source_path, articles_dir, &mut report);
    check_cover_image(article, &roots, &mut report);
    check_body_links(article, &roots, &mut report);
    check_placeholders(article, &mut report);
    check_schema_version(article, &mut report);

    report
}

fn check_required(article: &Article, report: &mut OutputReport) {
    check_present(
        report,
        "content.title.present",
        "title",
        article.title.as_deref(),
    );
    check_present(
        report,
        "content.summary.present",
        "summary",
        article.summary.as_deref(),
    );
    check_present(
        report,
        "content.canonical_url.present",
        "canonical_url",
        article.canonical_url.as_deref(),
    );

    if article.tags.is_empty() {
        report.push(
            "content.tags.present",
            CheckStatus::Failed,
            "tags is empty — at least one tag is required",
        );
    } else {
        report.push(
            "content.tags.present",
            CheckStatus::Passed,
            format!("{} tag(s) present", article.tags.len()),
        );
    }

    if article.content.trim().is_empty() {
        report.push(
            "content.body.present",
            CheckStatus::Failed,
            "article body is empty",
        );
    } else {
        report.push(
            "content.body.present",
            CheckStatus::Passed,
            "article body is non-empty",
        );
    }
}

fn check_present(report: &mut OutputReport, code: &'static str, field: &str, value: Option<&str>) {
    match value {
        Some(v) if !v.trim().is_empty() => {
            report.push(code, CheckStatus::Passed, format!("{field} is present"));
        }
        _ => report.push(
            code,
            CheckStatus::Failed,
            format!("{field} is missing or empty"),
        ),
    }
}

fn check_canonical_url_https(article: &Article, report: &mut OutputReport) {
    let Some(url) = article
        .canonical_url
        .as_deref()
        .filter(|u| !u.trim().is_empty())
    else {
        return; // already reported by check_required
    };

    if url.starts_with("https://") && url.len() > "https://".len() {
        report.push(
            "content.canonical_url.https",
            CheckStatus::Passed,
            "canonical_url is an absolute https URL",
        );
    } else {
        report.push(
            "content.canonical_url.https",
            CheckStatus::Failed,
            format!("canonical_url '{url}' must be an absolute https:// URL"),
        );
    }
}

fn check_canonical_url_matches_base(
    article: &Article,
    canonical_base_url: &str,
    report: &mut OutputReport,
) {
    let Some(url) = article
        .canonical_url
        .as_deref()
        .filter(|u| !u.trim().is_empty())
    else {
        return; // already reported by check_required
    };

    if url.starts_with(canonical_base_url) {
        report.push(
            "content.canonical_url.matches_base",
            CheckStatus::Passed,
            "canonical_url is under the configured canonical_base_url",
        );
    } else {
        report.push(
            "content.canonical_url.matches_base",
            CheckStatus::Failed,
            format!(
                "canonical_url '{url}' is not under the configured canonical_base_url \
                 '{canonical_base_url}'"
            ),
        );
    }
}

fn check_slug(
    article: &Article,
    source_path: &Path,
    articles_dir: &Path,
    report: &mut OutputReport,
) {
    let well_formed = !article.slug.is_empty() && slugify(&article.slug) == article.slug;
    if well_formed {
        report.push(
            "content.slug.wellformed",
            CheckStatus::Passed,
            format!("slug '{}' is well-formed", article.slug),
        );
    } else {
        report.push(
            "content.slug.wellformed",
            CheckStatus::Failed,
            format!(
                "slug '{}' is not well-formed (expected lowercase, alphanumeric, dash-separated)",
                article.slug
            ),
        );
    }

    let canonical_source = source_path.canonicalize().ok();
    let duplicate = walk_markdown_files(articles_dir).into_iter().find(|path| {
        if path.canonicalize().ok() == canonical_source {
            return false;
        }
        matches!(parse_article(path), Ok(other) if other.slug == article.slug)
    });

    match duplicate {
        Some(path) => report.push(
            "content.slug.duplicate",
            CheckStatus::Failed,
            format!(
                "slug '{}' is already used by {}",
                article.slug,
                path.display()
            ),
        ),
        None => report.push(
            "content.slug.duplicate",
            CheckStatus::Passed,
            format!(
                "slug '{}' is unique under {}",
                article.slug,
                articles_dir.display()
            ),
        ),
    }
}

fn walk_markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_markdown_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    out
}

/// `Err`/`true` both mean "don't trust this path": unresolvable paths (missing file, broken
/// symlink) are treated the same as a confirmed escape by the caller.
fn path_escapes_roots(path: &Path, roots: &[&Path]) -> Result<bool> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("couldn't resolve path {}", path.display()))?;
    for root in roots {
        if let Ok(canonical_root) = root.canonicalize()
            && canonical.starts_with(&canonical_root)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn check_cover_image(article: &Article, roots: &[&Path], report: &mut OutputReport) {
    match cover_image::resolve(article.cover_image.as_deref(), &article.base_dir) {
        Ok(None) => {}
        Ok(Some(CoverImage::Url(_))) => {
            report.push(
                "content.cover_image.resolves",
                CheckStatus::Passed,
                "cover_image is a hosted URL",
            );
        }
        Ok(Some(CoverImage::Local(img))) => {
            report.push(
                "content.cover_image.resolves",
                CheckStatus::Passed,
                format!("cover_image resolves to local file {}", img.filename),
            );
            let raw = article.cover_image.as_deref().unwrap_or_default();
            let full_path = article.base_dir.join(raw);
            match path_escapes_roots(&full_path, roots) {
                Ok(false) => report.push(
                    "content.cover_image.path_escape",
                    CheckStatus::Passed,
                    "cover_image path stays within the configured content roots",
                ),
                Ok(true) => report.push(
                    "content.cover_image.path_escape",
                    CheckStatus::Failed,
                    format!("cover_image '{raw}' escapes the configured content roots"),
                ),
                Err(e) => report.push(
                    "content.cover_image.path_escape",
                    CheckStatus::Failed,
                    format!("couldn't verify cover_image path: {e:#}"),
                ),
            }
        }
        Err(e) => report.push(
            "content.cover_image.resolves",
            CheckStatus::Failed,
            format!("{e:#}"),
        ),
    }
}

fn check_body_links(article: &Article, roots: &[&Path], report: &mut OutputReport) {
    let targets = extract_local_link_targets(&article.content);
    if targets.is_empty() {
        return;
    }

    let escaped: Vec<&String> = targets
        .iter()
        .filter(|target| {
            let full_path = article.base_dir.join(target);
            !matches!(path_escapes_roots(&full_path, roots), Ok(false))
        })
        .collect();

    if escaped.is_empty() {
        report.push(
            "content.links.path_escape",
            CheckStatus::Passed,
            format!(
                "{} local link(s)/image(s) stay within the configured content roots",
                targets.len()
            ),
        );
    } else {
        let list = escaped
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        report.push(
            "content.links.path_escape",
            CheckStatus::Failed,
            format!("local reference(s) escape the configured content roots: {list}"),
        );
    }
}

/// Extracts Markdown `[text](target)`/`![alt](target)` targets that look like local paths (not
/// `http(s)://` or an in-page `#anchor`). A plain substring scan, not a full Markdown-AST walk —
/// deliberately minimal, not a promise of complete link-checking.
fn extract_local_link_targets(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(rel_start) = body[cursor..].find("](") {
        let open = cursor + rel_start + 2;
        let Some(rel_end) = body[open..].find(')') else {
            break;
        };
        let raw = &body[open..open + rel_end];
        let target = raw.split_whitespace().next().unwrap_or(raw);
        if !target.is_empty()
            && !target.starts_with("http://")
            && !target.starts_with("https://")
            && !target.starts_with('#')
        {
            out.push(target.to_string());
        }
        cursor = open + rel_end + 1;
    }
    out
}

fn check_placeholders(article: &Article, report: &mut OutputReport) {
    let has_placeholder = [article.title.as_deref(), article.summary.as_deref()]
        .into_iter()
        .flatten()
        .chain(std::iter::once(article.content.as_str()))
        .any(|s| s.contains("{{") || s.contains("}}"));

    if has_placeholder {
        report.push(
            "content.placeholder.unresolved",
            CheckStatus::Failed,
            "title/summary/body contains an unresolved {{ }} placeholder",
        );
    } else {
        report.push(
            "content.placeholder.unresolved",
            CheckStatus::Passed,
            "no unresolved placeholders found",
        );
    }
}

fn check_schema_version(article: &Article, report: &mut OutputReport) {
    if article.schema_version == CURRENT_SCHEMA_VERSION {
        report.push(
            "content.schema_version.supported",
            CheckStatus::Passed,
            format!("schema_version {} is supported", article.schema_version),
        );
    } else {
        report.push(
            "content.schema_version.supported",
            CheckStatus::Failed,
            format!(
                "schema_version {} is not supported by this CLI (expected {})",
                article.schema_version, CURRENT_SCHEMA_VERSION
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::{TempDir, prelude::*};

    const VALID_FRONTMATTER: &str = "---\n\
        title: \"Hello World\"\n\
        slug: hello-world\n\
        summary: \"A test post\"\n\
        tags: [rust, cli]\n\
        canonical_url: https://m2s2.io/blog/hello-world\n\
        ---\n\
        Body text.\n";

    #[test]
    fn parses_full_frontmatter() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("hello-world.md");
        file.write_str(VALID_FRONTMATTER).unwrap();

        let article = parse_article(file.path()).unwrap();
        assert_eq!(article.title.as_deref(), Some("Hello World"));
        assert_eq!(article.slug, "hello-world");
        assert_eq!(article.tags, vec!["rust", "cli"]);
        assert_eq!(
            article.canonical_url.as_deref(),
            Some("https://m2s2.io/blog/hello-world")
        );
        assert_eq!(article.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(article.content, "Body text.");
    }

    #[test]
    fn slug_defaults_to_filename_when_absent() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("my-post.md");
        file.write_str(
            "---\ntitle: \"T\"\nsummary: \"s\"\ncanonical_url: https://x.io/y\n---\nbody\n",
        )
        .unwrap();
        let article = parse_article(file.path()).unwrap();
        assert_eq!(article.slug, "my-post");
    }

    #[test]
    fn missing_frontmatter_is_an_error() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("post.md");
        file.write_str("# Just markdown, no frontmatter\n").unwrap();
        let err = parse_article(file.path()).unwrap_err();
        assert!(err.to_string().contains("no YAML frontmatter"));
    }

    #[test]
    fn missing_required_fields_are_reported_not_a_parse_error() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("post.md");
        file.write_str("---\ntitle: \"T\"\n---\nbody\n").unwrap();
        let article = parse_article(file.path()).unwrap();

        let report = validate(&article, file.path(), dir.path(), dir.path(), "");
        let summary_check = report
            .checks
            .iter()
            .find(|c| c.code == "content.summary.present")
            .unwrap();
        assert_eq!(summary_check.status, CheckStatus::Failed);
        let canonical_check = report
            .checks
            .iter()
            .find(|c| c.code == "content.canonical_url.present")
            .unwrap();
        assert_eq!(canonical_check.status, CheckStatus::Failed);
    }

    #[test]
    fn canonical_url_must_be_https() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\ntitle: \"T\"\nsummary: \"s\"\ntags: [a]\ncanonical_url: http://x.io/y\n---\nbody\n",
        )
        .unwrap();
        let article = parse_article(file.path()).unwrap();

        let report = validate(&article, file.path(), dir.path(), dir.path(), "");
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.canonical_url.https")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Failed);
    }

    #[test]
    fn valid_article_passes_every_check() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("hello-world.md");
        file.write_str(VALID_FRONTMATTER).unwrap();
        let article = parse_article(file.path()).unwrap();

        let report = validate(&article, file.path(), dir.path(), dir.path(), "");
        assert!(report.passed(), "{:#?}", report.checks);
    }

    #[test]
    fn canonical_url_must_be_under_the_configured_base() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("hello-world.md");
        file.write_str(VALID_FRONTMATTER).unwrap();
        let article = parse_article(file.path()).unwrap();

        let report = validate(
            &article,
            file.path(),
            dir.path(),
            dir.path(),
            "https://other-domain.example",
        );
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.canonical_url.matches_base")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Failed);
    }

    #[test]
    fn canonical_url_under_the_configured_base_passes() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("hello-world.md");
        file.write_str(VALID_FRONTMATTER).unwrap();
        let article = parse_article(file.path()).unwrap();

        let report = validate(
            &article,
            file.path(),
            dir.path(),
            dir.path(),
            "https://m2s2.io/blog",
        );
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.canonical_url.matches_base")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Passed);
    }

    #[test]
    fn duplicate_slug_across_articles_dir_is_a_failure() {
        let dir = TempDir::new().unwrap();
        dir.child("hello-world.md")
            .write_str(VALID_FRONTMATTER)
            .unwrap();
        let other = dir.child("other.md");
        other
            .write_str(
                "---\ntitle: \"Other\"\nslug: hello-world\nsummary: \"s\"\ntags: [a]\n\
                 canonical_url: https://m2s2.io/blog/other\n---\nbody\n",
            )
            .unwrap();

        let article = parse_article(other.path()).unwrap();
        let report = validate(&article, other.path(), dir.path(), dir.path(), "");
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.slug.duplicate")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Failed);
    }

    #[test]
    fn cover_image_escaping_articles_dir_is_a_failure() {
        let root = TempDir::new().unwrap();
        let articles_dir = root.child("articles");
        articles_dir.create_dir_all().unwrap();
        let assets_dir = root.child("assets");
        assets_dir.create_dir_all().unwrap();
        let outside = root.child("secret.png");
        outside.write_binary(&[0xff]).unwrap();

        let file = articles_dir.child("post.md");
        file.write_str(
            "---\ntitle: \"T\"\nsummary: \"s\"\ntags: [a]\ncanonical_url: https://x.io/y\n\
             cover_image: ../secret.png\n---\nbody\n",
        )
        .unwrap();

        let article = parse_article(file.path()).unwrap();
        let report = validate(
            &article,
            file.path(),
            articles_dir.path(),
            assets_dir.path(),
            "",
        );
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.cover_image.path_escape")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Failed);
    }

    #[test]
    fn cover_image_within_assets_dir_passes() {
        let root = TempDir::new().unwrap();
        let articles_dir = root.child("articles");
        articles_dir.create_dir_all().unwrap();
        let assets_dir = root.child("assets");
        assets_dir.create_dir_all().unwrap();
        assets_dir.child("hero.png").write_binary(&[0xff]).unwrap();

        let file = articles_dir.child("post.md");
        file.write_str(
            "---\ntitle: \"T\"\nsummary: \"s\"\ntags: [a]\ncanonical_url: https://x.io/y\n\
             cover_image: ../assets/hero.png\n---\nbody\n",
        )
        .unwrap();

        let article = parse_article(file.path()).unwrap();
        let report = validate(
            &article,
            file.path(),
            articles_dir.path(),
            assets_dir.path(),
            "",
        );
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.cover_image.path_escape")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Passed);
    }

    #[test]
    fn body_link_escaping_roots_is_a_failure() {
        let root = TempDir::new().unwrap();
        let articles_dir = root.child("articles");
        articles_dir.create_dir_all().unwrap();
        let assets_dir = root.child("assets");
        assets_dir.create_dir_all().unwrap();
        root.child("secret.png").write_binary(&[0xff]).unwrap();

        let file = articles_dir.child("post.md");
        file.write_str(
            "---\ntitle: \"T\"\nsummary: \"s\"\ntags: [a]\ncanonical_url: https://x.io/y\n---\n\
             Body with an image: ![alt](../secret.png)\n",
        )
        .unwrap();

        let article = parse_article(file.path()).unwrap();
        let report = validate(
            &article,
            file.path(),
            articles_dir.path(),
            assets_dir.path(),
            "",
        );
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.links.path_escape")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Failed);
    }

    #[test]
    fn unresolved_placeholder_in_body_is_a_failure() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\ntitle: \"T\"\nsummary: \"s\"\ntags: [a]\ncanonical_url: https://x.io/y\n---\n\
             Hello {{name}}, welcome.\n",
        )
        .unwrap();

        let article = parse_article(file.path()).unwrap();
        let report = validate(&article, file.path(), dir.path(), dir.path(), "");
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.placeholder.unresolved")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Failed);
    }

    #[test]
    fn unsupported_schema_version_is_a_failure() {
        let dir = TempDir::new().unwrap();
        let file = dir.child("post.md");
        file.write_str(
            "---\nschema_version: 99\ntitle: \"T\"\nsummary: \"s\"\ntags: [a]\n\
             canonical_url: https://x.io/y\n---\nbody\n",
        )
        .unwrap();

        let article = parse_article(file.path()).unwrap();
        let report = validate(&article, file.path(), dir.path(), dir.path(), "");
        let check = report
            .checks
            .iter()
            .find(|c| c.code == "content.schema_version.supported")
            .unwrap();
        assert_eq!(check.status, CheckStatus::Failed);
    }
}
