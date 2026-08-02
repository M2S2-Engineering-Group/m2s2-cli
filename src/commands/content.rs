use crate::content::{self, config::DEFAULT_ARTICLES_DIR, config::DEFAULT_ASSETS_DIR};
use crate::report::OutputFormat;
use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use console::style;
use serde::Serialize;
use std::path::PathBuf;

const AFTER_HELP: &str = "\
Works with the canonical content-delivery article schema (see
docs/m2s2-cli-content-delivery-integration.md \u{a7}7) — offline only, no network access, and
independent of the direct-publish `m2s2 publish` command and its schema.

Frontmatter fields: title, slug (optional, derived from the filename), summary, tags,
canonical_url (required, must be an absolute https:// URL), cover_image (optional — a URL or a
local path resolved relative to the article), schema_version (optional, defaults to 1).

`validate`/`inspect` require a `.m2s2/config.toml` in the current directory (see `content init`)
so duplicate-slug and local-path-escape checks know which directories to scan.

Examples:
  m2s2 content init --canonical-base-url https://example.com/blog
      Create .m2s2/config.toml plus empty articles/ and assets/ directories.

  m2s2 content validate articles/my-post.md
      Run every offline validation rule and print a pass/fail report.

  m2s2 content inspect articles/my-post.md --format json
      Print the parsed article and its validation report as JSON.";

#[derive(Args)]
#[command(after_help = AFTER_HELP)]
pub struct ContentArgs {
    #[command(subcommand)]
    pub command: ContentCommands,
}

#[derive(Subcommand)]
pub enum ContentCommands {
    /// Create .m2s2/config.toml and the articles/assets directories
    Init(InitArgs),
    /// Run every offline validation rule against a canonical article
    Validate(ValidateArgs),
    /// Print a parsed article plus its validation status
    Inspect(InspectArgs),
}

#[derive(Args)]
pub struct InitArgs {
    /// Directory to initialize (defaults to the current directory)
    pub path: Option<PathBuf>,

    /// Directory (relative to `path`) that holds article Markdown files
    #[arg(long, default_value = DEFAULT_ARTICLES_DIR)]
    pub articles_dir: String,

    /// Directory (relative to `path`) that holds local assets (e.g. cover images)
    #[arg(long, default_value = DEFAULT_ASSETS_DIR)]
    pub assets_dir: String,

    /// Base URL articles are canonically published under, e.g. https://example.com/blog
    #[arg(long)]
    pub canonical_base_url: String,

    /// Overwrite an existing .m2s2/config.toml
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the Markdown article
    pub file: PathBuf,

    #[arg(long, value_enum, default_value = "human")]
    pub format: OutputFormat,
}

#[derive(Args)]
pub struct InspectArgs {
    /// Path to the Markdown article
    pub file: PathBuf,

    #[arg(long, value_enum, default_value = "human")]
    pub format: OutputFormat,
}

pub fn run(args: ContentArgs) -> Result<()> {
    match args.command {
        ContentCommands::Init(args) => run_init(args),
        ContentCommands::Validate(args) => run_validate(args),
        ContentCommands::Inspect(args) => run_inspect(args),
    }
}

fn run_init(args: InitArgs) -> Result<()> {
    let root = args.path.unwrap_or_else(|| PathBuf::from("."));
    let path = content::config::init(
        &root,
        &args.articles_dir,
        &args.assets_dir,
        &args.canonical_base_url,
        args.force,
    )?;
    println!(
        "{} wrote {}",
        style("✓").green().bold(),
        style(path.display().to_string()).cyan()
    );
    Ok(())
}

fn run_validate(args: ValidateArgs) -> Result<()> {
    let root = PathBuf::from(".");
    let config = content::ContentConfig::load(&root)?;
    let article = content::parse_article(&args.file)?;
    let report = content::validate(
        &article,
        &args.file,
        &config.articles_dir(&root),
        &config.assets_dir(&root),
        &config.content.canonical_base_url,
    );

    report.print(args.format)?;

    if !report.passed() {
        bail!("content validation failed for {}", args.file.display());
    }
    Ok(())
}

#[derive(Serialize)]
struct Inspect<'a> {
    article: &'a content::Article,
    report: &'a crate::report::OutputReport,
}

fn run_inspect(args: InspectArgs) -> Result<()> {
    let root = PathBuf::from(".");
    let config = content::ContentConfig::load(&root)?;
    let article = content::parse_article(&args.file)?;
    let report = content::validate(
        &article,
        &args.file,
        &config.articles_dir(&root),
        &config.assets_dir(&root),
        &config.content.canonical_base_url,
    );

    match args.format {
        OutputFormat::Json => {
            let inspect = Inspect {
                article: &article,
                report: &report,
            };
            println!("{}", serde_json::to_string_pretty(&inspect)?);
        }
        OutputFormat::Human => {
            println!(
                "{}",
                style(article.title.as_deref().unwrap_or("(untitled)")).bold()
            );
            println!("  slug: {}", article.slug);
            println!(
                "  summary: {}",
                article.summary.as_deref().unwrap_or("(none)")
            );
            println!("  tags: {}", article.tags.join(", "));
            println!(
                "  canonical_url: {}",
                article.canonical_url.as_deref().unwrap_or("(none)")
            );
            println!(
                "  cover_image: {}",
                article.cover_image.as_deref().unwrap_or("(none)")
            );
            println!("  schema_version: {}", article.schema_version);
            println!();
            report.print(OutputFormat::Human)?;
        }
    }

    Ok(())
}
