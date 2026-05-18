pub mod component;

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub command: GenerateCommands,
}

#[derive(Subcommand)]
pub enum GenerateCommands {
    /// Scaffold a new component
    Component(component::ComponentArgs),
}
