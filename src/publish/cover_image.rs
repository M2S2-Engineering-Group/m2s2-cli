use crate::publish::article::Article;
use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

/// A frontmatter `cover_image` value, resolved into what it actually is: either an
/// already-hosted URL, or a local file — read into memory since every "local" target that uses
/// it needs the encoded bytes, not the path.
#[derive(Debug)]
pub enum CoverImage {
    Url(String),
    Local(LocalImage),
}

#[derive(Debug)]
pub struct LocalImage {
    pub filename: String,
    pub content_type: &'static str,
    pub base64_data: String,
}

/// `None` if the article has no `cover_image`. Reads local files eagerly (`Local` variant),
/// so callers that can't use one (e.g. Dev.to/Hashnode — see their targets) can fail before
/// spending a network round trip.
pub fn resolve(article: &Article) -> Result<Option<CoverImage>> {
    let Some(raw) = article.cover_image.as_deref() else {
        return Ok(None);
    };

    if raw.starts_with("http://") || raw.starts_with("https://") {
        return Ok(Some(CoverImage::Url(raw.to_string())));
    }

    let path = article.base_dir.join(raw);
    let bytes = std::fs::read(&path)
        .with_context(|| format!("cover_image '{raw}' looks like a local path but couldn't be read at {}", path.display()))?;
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("cover")
        .to_string();

    Ok(Some(CoverImage::Local(LocalImage {
        content_type: guess_content_type(&path),
        base64_data: BASE64.encode(bytes),
        filename,
    })))
}

fn guess_content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()) {
        Some(ext) if ext == "png" => "image/png",
        Some(ext) if ext == "gif" => "image/gif",
        Some(ext) if ext == "webp" => "image/webp",
        Some(ext) if ext == "svg" => "image/svg+xml",
        _ => "image/jpeg",
    }
}

/// A local `cover_image` an error message can point at, for targets whose API only accepts an
/// already-hosted URL (Dev.to, Hashnode — neither has an image-upload endpoint at all).
pub fn local_path_not_supported_error(target: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "cover_image is a local file, but {target}'s API only accepts an already-hosted URL \
         (it has no image upload endpoint) — host the image yourself and put its URL in \
         frontmatter instead"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::publish::article::parse_article;
    use assert_fs::{TempDir, prelude::*};

    fn article_with_cover(dir: &TempDir, cover_image_line: &str) -> Article {
        let file = dir.child("post.md");
        file.write_str(&format!(
            "---\ntitle: \"T\"\ndate: 2026-07-30\nsummary: \"s\"\npublish: [devto]\n{cover_image_line}\n---\nbody\n"
        ))
        .unwrap();
        parse_article(file.path(), None).unwrap()
    }

    #[test]
    fn url_passes_through_unread() {
        let dir = TempDir::new().unwrap();
        let article = article_with_cover(&dir, "cover_image: https://example.com/hero.jpg");
        match resolve(&article).unwrap().unwrap() {
            CoverImage::Url(url) => assert_eq!(url, "https://example.com/hero.jpg"),
            CoverImage::Local(_) => panic!("expected Url variant"),
        }
    }

    #[test]
    fn local_path_is_read_and_base64_encoded() {
        let dir = TempDir::new().unwrap();
        dir.child("hero.png").write_binary(&[0x89, 0x50, 0x4e, 0x47]).unwrap();
        let article = article_with_cover(&dir, "cover_image: hero.png");

        match resolve(&article).unwrap().unwrap() {
            CoverImage::Local(img) => {
                assert_eq!(img.filename, "hero.png");
                assert_eq!(img.content_type, "image/png");
                assert_eq!(img.base64_data, BASE64.encode([0x89, 0x50, 0x4e, 0x47]));
            }
            CoverImage::Url(_) => panic!("expected Local variant"),
        }
    }

    #[test]
    fn missing_local_file_is_a_clear_error() {
        let dir = TempDir::new().unwrap();
        let article = article_with_cover(&dir, "cover_image: does-not-exist.jpg");
        let err = resolve(&article).unwrap_err();
        assert!(err.to_string().contains("couldn't be read"));
    }

    #[test]
    fn no_cover_image_resolves_to_none() {
        let dir = TempDir::new().unwrap();
        let article = article_with_cover(&dir, "");
        assert!(resolve(&article).unwrap().is_none());
    }
}
