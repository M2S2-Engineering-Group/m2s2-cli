//! Schema-agnostic Markdown/frontmatter helpers shared by every article schema this CLI parses
//! (`publish::article`, `content::article`, ...) — nothing here knows about a specific field
//! set, so a new schema never needs to reimplement `---`-splitting or slugification.

/// Split `---\n<yaml>\n---\n<body>` into `(yaml, body)`.
pub fn split_frontmatter(raw: &str) -> Option<(String, String)> {
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

    #[test]
    fn slugify_strips_punctuation_and_lowercases() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("Rust & CLI Tools"), "rust-cli-tools");
        assert_eq!(slugify("already-kebab"), "already-kebab");
    }

    #[test]
    fn split_frontmatter_separates_yaml_and_body() {
        let raw = "---\ntitle: X\ntags: [a, b]\n---\nBody line 1\nBody line 2\n";
        let (yaml, body) = split_frontmatter(raw).unwrap();
        assert_eq!(yaml, "title: X\ntags: [a, b]");
        assert_eq!(body, "Body line 1\nBody line 2");
    }

    #[test]
    fn split_frontmatter_returns_none_without_leading_dashes() {
        assert!(split_frontmatter("# Just markdown, no frontmatter\n").is_none());
    }

    #[test]
    fn split_frontmatter_returns_none_when_closing_delimiter_is_missing() {
        assert!(split_frontmatter("---\ntitle: X\nno closing delimiter\n").is_none());
    }
}
