mod commands;
mod npm;
mod scaffold;

use anyhow::Result;
use clap::{Parser, Subcommand};

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
        },
        Commands::Upgrade(args) => commands::upgrade::run(args).await,
    }
}
