mod commands;
mod config;
mod npm;
mod scaffold;
mod utils;

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use console::style;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(
    name = "m2s2",
    version,
    about = "Scaffold M²S² design system projects",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project
    New(commands::new::NewArgs),
    /// Scaffold components and other project artifacts
    Generate(commands::generate::GenerateArgs),
    /// Check for and install updates
    Upgrade(commands::upgrade::UpgradeArgs),
    /// Install shell completions (auto-detects shell from $SHELL)
    Completions {
        /// Shell to generate completions for (overrides auto-detection)
        #[arg(value_enum)]
        shell: Option<Shell>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New(args) => commands::new::run(args).await,
        Commands::Generate(args) => match args.command {
            commands::generate::GenerateCommands::Component(a) => {
                commands::generate::component::run(a).await
            }
            commands::generate::GenerateCommands::Page(a) => {
                commands::generate::page::run(a).await
            }
            commands::generate::GenerateCommands::Service(a) => {
                commands::generate::service::run(a).await
            }
        },
        Commands::Upgrade(args) => commands::upgrade::run(args).await,
        Commands::Completions { shell } => {
            let shell = match shell {
                Some(s) => s,
                None => detect_shell()?,
            };
            install_completions(shell)
        }
    }
}

fn detect_shell() -> Result<Shell> {
    let path = env::var("SHELL").context(
        "$SHELL is not set — pass the shell name explicitly (e.g. m2s2 completions zsh)",
    )?;
    let name = Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    match name {
        "zsh" => Ok(Shell::Zsh),
        "bash" => Ok(Shell::Bash),
        "fish" => Ok(Shell::Fish),
        "elvish" => Ok(Shell::Elvish),
        other => bail!(
            "shell '{other}' is not supported — pass one of: zsh, bash, fish, elvish, powershell"
        ),
    }
}

fn install_completions(shell: Shell) -> Result<()> {
    let home = env::var("HOME").context("$HOME is not set")?;
    let home = PathBuf::from(home);

    let mut buf = Vec::new();
    generate(shell, &mut Cli::command(), "m2s2", &mut buf);
    let script = String::from_utf8(buf)?;

    match shell {
        Shell::Fish => {
            let dir = home.join(".config/fish/completions");
            fs::create_dir_all(&dir)?;
            let file = dir.join("m2s2.fish");
            fs::write(&file, &script)?;
            println!(
                "{} wrote {}",
                style("✓").green().bold(),
                style(file.display().to_string()).cyan()
            );
            println!(
                "  {} fish sources completions in that directory automatically",
                style("→").dim()
            );
        }
        _ => {
            let (file_name, rc_file) = rc_paths(shell, &home)?;
            fs::write(&file_name, &script)?;
            println!(
                "{} wrote {}",
                style("✓").green().bold(),
                style(file_name.display().to_string()).cyan()
            );
            ensure_sourced(&rc_file, &file_name)?;
        }
    }

    Ok(())
}

fn rc_paths(shell: Shell, home: &Path) -> Result<(PathBuf, PathBuf)> {
    let (dot, rc) = match shell {
        Shell::Zsh => (".m2s2-completions.zsh", ".zshrc"),
        Shell::Bash => (".m2s2-completions.bash", ".bashrc"),
        Shell::Elvish => (".m2s2-completions.elv", ".elvish/rc.elv"),
        Shell::PowerShell => (
            ".m2s2-completions.ps1",
            ".config/powershell/Microsoft.PowerShell_profile.ps1",
        ),
        _ => bail!("unsupported shell"),
    };
    Ok((home.join(dot), home.join(rc)))
}

fn ensure_sourced(rc: &Path, completions_file: &Path) -> Result<()> {
    let source_line = format!("source {}", completions_file.display());

    let existing = fs::read_to_string(rc).unwrap_or_default();
    if existing.contains(&source_line) {
        println!(
            "  {} {} already sources completions",
            style("→").dim(),
            style(rc.display().to_string()).cyan()
        );
        return Ok(());
    }

    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(rc)
        .with_context(|| format!("could not open {}", rc.display()))?;

    writeln!(f, "\n# m2s2 shell completions\n{source_line}")?;

    println!(
        "{} added source line to {}",
        style("✓").green().bold(),
        style(rc.display().to_string()).cyan()
    );
    println!(
        "  {} reload your shell or run: {}",
        style("→").dim(),
        style(format!("source {}", rc.display())).cyan()
    );

    Ok(())
}
