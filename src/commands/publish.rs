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

If a [section] is missing entirely, its credentials fall back to environment variables instead
(the file not existing at all is fine too, as long as env vars cover every target you select):
M2S2_PUBLISH_DEVTO_API_KEY; M2S2_PUBLISH_HASHNODE_TOKEN + M2S2_PUBLISH_HASHNODE_PUBLICATION_ID;
M2S2_PUBLISH_PLATFORM_ENDPOINT + M2S2_PUBLISH_PLATFORM_TOKEN (+ optional _PATH / _BODY_COMMAND).
This only fills in a section that's entirely absent — a section that's present in the file wins
over env vars even if some of its fields could otherwise come from the environment.

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
      Update an existing platform blog post instead of creating a new one.

  m2s2 publish posts/my-article.md --preflight-only
      Validate every target and build the exact request each would send, without publishing.";

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

    /// Validate and build every target's request, then stop without publishing anything
    #[arg(long)]
    pub preflight_only: bool,
}

pub async fn run(args: PublishArgs) -> Result<()> {
    let article = publish::parse_article(&args.file, args.to.as_deref())?;
    let config = publish::PublishConfig::load()?;
    let targets = publish::build_targets(&article.targets, &config)?;

    // Prepare every target — local validation plus building the exact request that will be
    // sent — before any of them makes a network request. An earlier target succeeding (a real,
    // side-effecting POST) before a later target's purely-local validation failure surfaces
    // would leave a partial, not-safely-retryable publish (see Target::prepare).
    let prepared: Vec<_> = targets
        .iter()
        .map(|target| target.prepare(&article, args.update))
        .collect::<Result<_>>()?;

    if args.preflight_only {
        for target in &targets {
            println!(
                "{} {} — ready to publish",
                style("✓").green().bold(),
                target.kind()
            );
        }
        return Ok(());
    }

    let mut had_error = false;
    for (target, prepared) in targets.iter().zip(prepared) {
        print!("{} publishing to {}... ", style("→").dim(), target.kind());
        std::io::stdout().flush().ok();
        match target.execute(prepared).await {
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
