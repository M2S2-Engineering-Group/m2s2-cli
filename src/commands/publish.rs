use crate::publish;
use crate::publish::TargetKind;
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

cover_image can be a URL (used as-is) or a path to a local file, resolved relative to the
article. A local path is uploaded automatically for the platform target; Dev.to and Hashnode
have no image upload endpoint in their APIs, so a local path there is an error — host the image
yourself first and use its URL instead.

Credentials live in .m2s2-publish.toml in the current directory, e.g.:

  [devto]
  api_key = \"...\"

  [hashnode]
  token = \"...\"
  publication_id = \"...\"

  [platform]
  endpoint = \"https://api.example.com\"
  # path = \"/admin/blog\"   (optional, this is the default)
  token = \"...\"
  # body_command = \"./hooks/build-body.sh\"   (optional — see below)

The platform target's request body is a fixed field mapping by default. Set body_command to
build it yourself instead: the article (plus `update: true/false`) is piped to the command as
JSON on stdin, and whatever JSON object it prints on stdout is sent verbatim as the request
body — no merging with the default mapping. Runs through a shell, so it can be a script path or
a full command line with arguments.

Examples:
  m2s2 publish posts/my-article.md
      Publish to whatever targets are listed in the article's frontmatter.

  m2s2 publish posts/my-article.md --to devto,platform
      Publish to specific targets regardless of frontmatter.

  m2s2 publish posts/my-article.md --to platform --update
      Update an existing platform blog post instead of creating a new one.";

#[derive(Args)]
#[command(after_help = AFTER_HELP)]
pub struct PublishArgs {
    /// Path to the Markdown article
    pub file: PathBuf,

    /// Comma-separated target list, overriding the frontmatter's `publish:` list
    #[arg(long, value_delimiter = ',')]
    pub to: Option<Vec<TargetKind>>,

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
        print!("{} publishing to {}... ", style("→").dim(), target.kind());
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
