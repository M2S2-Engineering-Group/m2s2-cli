use crate::publish;
use anyhow::{Result, bail};
use clap::Args;
use console::style;
use std::io::Write;
use std::path::PathBuf;

const AFTER_HELP: &str = "\
Publishes a Markdown article (with YAML frontmatter) to one or more configured targets.

Frontmatter fields: title, date, summary, tags, slug (optional, derived from the filename),
excerpt (optional), cover_image (optional), canonical_url (optional), and publish (a list of
target names — overridden by --to when given).

Credentials live in .m2s2-publish.toml in the current directory, e.g.:

  [devto]
  api_key = \"...\"

  [hashnode]
  token = \"...\"
  publication_id = \"...\"

  [m2s2]
  endpoint = \"https://api.m2s2.io\"
  token = \"...\"

Examples:
  m2s2 publish posts/my-article.md
      Publish to whatever targets are listed in the article's frontmatter.

  m2s2 publish posts/my-article.md --to devto,m2s2
      Publish to specific targets regardless of frontmatter.

  m2s2 publish posts/my-article.md --to m2s2 --update
      Update an existing m2s2 blog post instead of creating a new one.";

#[derive(Args)]
#[command(after_help = AFTER_HELP)]
pub struct PublishArgs {
    /// Path to the Markdown article
    pub file: PathBuf,

    /// Comma-separated target list, overriding the frontmatter's `publish:` list
    #[arg(long, value_delimiter = ',')]
    pub to: Option<Vec<String>>,

    /// Update an existing post instead of creating a new one (target-dependent support)
    #[arg(long)]
    pub update: bool,
}

pub async fn run(args: PublishArgs) -> Result<()> {
    let article = publish::parse_article(&args.file, args.to.as_deref())?;
    let config = publish::PublishConfig::load()?;
    let targets = publish::build_targets(&article.targets, &config)?;

    let mut had_error = false;
    for target in targets {
        print!("{} publishing to {}... ", style("→").dim(), target.name());
        std::io::stdout().flush().ok();
        match target.publish(&article, args.update).await {
            Ok(outcome) => println!("{} {}", style("✓").green().bold(), outcome.message),
            Err(e) => {
                had_error = true;
                println!("{} {e:#}", style("✗").red().bold());
            }
        }
    }

    if had_error {
        bail!("one or more targets failed to publish");
    }
    Ok(())
}
