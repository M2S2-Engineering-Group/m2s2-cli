//! Shared structured-check report shape. `content validate`/`inspect` build one of these today;
//! `docs/api-verification-preflight.md` specifies the same `code`/`status`/`message` shape for
//! the still-pending `publish --preflight-only --format json` work, so this exists once here
//! rather than being invented twice with incompatible JSON output.

use console::style;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckStatus {
    Passed,
    /// No `content` check produces this yet — reserved because `publish --preflight-only` needs
    /// it (a truncated tag list, a missing-but-not-required canonical URL) once that work reuses
    /// this same shape.
    #[allow(dead_code)]
    Warning,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct Check {
    /// Stable machine identifier, e.g. `article.canonical_url.https`. Human messages may evolve;
    /// automation should depend on this, not on `message`.
    pub code: &'static str,
    pub status: CheckStatus,
    pub message: String,
}

#[derive(Debug, Default, Serialize)]
pub struct OutputReport {
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Human,
    Json,
}

impl OutputReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, code: &'static str, status: CheckStatus, message: impl Into<String>) {
        self.checks.push(Check {
            code,
            status,
            message: message.into(),
        });
    }

    /// `false` if any check `Failed` — the caller's signal to treat this as a nonzero-exit
    /// failure, regardless of how many `Warning`s are mixed in.
    pub fn passed(&self) -> bool {
        !self.checks.iter().any(|c| c.status == CheckStatus::Failed)
    }

    pub fn print(&self, format: OutputFormat) -> anyhow::Result<()> {
        match format {
            OutputFormat::Human => {
                self.print_human();
                Ok(())
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(self)?);
                Ok(())
            }
        }
    }

    fn print_human(&self) {
        for check in &self.checks {
            let (glyph, message) = match check.status {
                CheckStatus::Passed => (style("✓").green().bold(), check.message.as_str()),
                CheckStatus::Warning => (style("⚠").yellow().bold(), check.message.as_str()),
                CheckStatus::Failed => (style("✗").red().bold(), check.message.as_str()),
            };
            println!("{glyph} {message}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passed_is_true_with_no_checks() {
        assert!(OutputReport::new().passed());
    }

    #[test]
    fn passed_is_true_with_only_warnings() {
        let mut report = OutputReport::new();
        report.push("a.check", CheckStatus::Warning, "hm");
        assert!(report.passed());
    }

    #[test]
    fn passed_is_false_with_any_failure() {
        let mut report = OutputReport::new();
        report.push("a.check", CheckStatus::Passed, "ok");
        report.push("b.check", CheckStatus::Failed, "bad");
        assert!(!report.passed());
    }

    #[test]
    fn json_output_includes_stable_codes() {
        let mut report = OutputReport::new();
        report.push("article.title.present", CheckStatus::Passed, "has a title");
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"code\":\"article.title.present\""));
        assert!(json.contains("\"status\":\"passed\""));
    }
}
