use crate::publish::target_kind::TargetKind;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A parsed article: YAML frontmatter + Markdown body.
#[derive(Debug, Serialize)]
pub struct Article {
    pub title: String,
    pub slug: String,
    pub date: String,
    pub summary: String,
    pub excerpt: Option<String>,
    pub tags: Vec<String>,
    /// Raw frontmatter value — a URL, or a path to a local file resolved relative to
    /// `base_dir`. See `publish::cover_image` for how targets are expected to tell these apart.
    pub cover_image: Option<String>,
    pub canonical_url: Option<String>,
    pub targets: Vec<TargetKind>,
    pub content: String,
    /// The directory containing the source Markdown file — relative paths in frontmatter (e.g.
    /// a local `cover_image`) are resolved against this.
    pub base_dir: PathBuf,
}

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
    slug: Option<String>,
    date: String,
    summary: String,
    excerpt: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    cover_image: Option<String>,
    canonical_url: Option<String>,
    #[serde(default)]
    publish: Vec<TargetKind>,
}

/// Parse a Markdown file with a leading `---`-delimited YAML frontmatter block.
///
/// `cli_targets`, if given (from `--to`), overrides the frontmatter's `publish:` list.
pub fn parse_article(path: &Path, cli_targets: Option<&[TargetKind]>) -> Result<Article> {
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

    let targets = match cli_targets {
        Some(t) => t.to_vec(),
        None => fm.publish,
    };
    if targets.is_empty() {
        bail!(
            "no publish targets — add `publish: [devto, ...]` to {}'s frontmatter or pass --to",
            path.display()
        );
    }

    let base_dir = path.parent().map(Path::to_path_buf).unwrap_or_default();

    Ok(Article {
        title: fm.title,
        slug,
        date: fm.date,
        summary: fm.summary,
        excerpt: fm.excerpt,
        tags: fm.tags,
        cover_image: fm.cover_image,
        canonical_url: fm.canonical_url,
        targets,
        content: body.trim().to_string(),
        base_dir,
    })
}

/// Split `---\n<yaml>\n---\n<body>` into `(yaml, body)`.
fn split_frontmatter(raw: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.first() != Some(&"---") {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| *l == "---")? + 1;
    Some((lines[1..end].join("\n"), lines[end + 1..].join("\n")))
}

/// Lowercase, alphanumeric-and-dashes slug (e.g. for tag slugs or a filename-derived post slug).
pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for c in input.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::{TempDir, prelude::*};

    fn write_temp(content: &str) -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let file = dir.child("my-post.md");
        file.write_str(content).unwrap();
        let path = file.path().to_path_buf();
        (dir, path)
    }

    #[test]
    fn parses_full_frontmatter() {
        let (_dir, path) = write_temp(
            "---\n\
             title: \"Hello World\"\n\
             date: 2026-07-30\n\
             summary: \"A test post\"\n\
             tags: [rust, cli]\n\
             publish: [devto, platform]\n\
             ---\n\
             \n\
             # Hello\n\
             \n\
             Body text.\n",
        );

        let article = parse_article(&path, None).unwrap();
        assert_eq!(article.title, "Hello World");
        assert_eq!(article.slug, "my-post");
        assert_eq!(article.date, "2026-07-30");
        assert_eq!(article.tags, vec!["rust", "cli"]);
        assert_eq!(
            article.targets,
            vec![TargetKind::Devto, TargetKind::Platform]
        );
        assert_eq!(article.content, "# Hello\n\nBody text.");
    }

    #[test]
    fn explicit_slug_overrides_filename() {
        let (_dir, path) = write_temp(
            "---\ntitle: \"T\"\nslug: custom-slug\ndate: 2026-07-30\nsummary: \"s\"\npublish: [devto]\n---\nbody\n",
        );
        let article = parse_article(&path, None).unwrap();
        assert_eq!(article.slug, "custom-slug");
    }

    #[test]
    fn cli_to_flag_overrides_frontmatter_publish_list() {
        let (_dir, path) = write_temp(
            "---\ntitle: \"T\"\ndate: 2026-07-30\nsummary: \"s\"\npublish: [devto]\n---\nbody\n",
        );
        let article = parse_article(&path, Some(&[TargetKind::Hashnode])).unwrap();
        assert_eq!(article.targets, vec![TargetKind::Hashnode]);
    }

    #[test]
    fn missing_frontmatter_is_an_error() {
        let (_dir, path) = write_temp("# Just markdown, no frontmatter\n");
        let err = parse_article(&path, None).unwrap_err();
        assert!(err.to_string().contains("no YAML frontmatter"));
    }

    #[test]
    fn no_targets_is_an_error() {
        let (_dir, path) =
            write_temp("---\ntitle: \"T\"\ndate: 2026-07-30\nsummary: \"s\"\n---\nbody\n");
        let err = parse_article(&path, None).unwrap_err();
        assert!(err.to_string().contains("no publish targets"));
    }

    #[test]
    fn slugify_strips_punctuation_and_lowercases() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("Rust & CLI Tools"), "rust-cli-tools");
        assert_eq!(slugify("already-kebab"), "already-kebab");
    }
}
