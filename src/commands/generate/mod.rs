pub mod component;
pub mod page;
pub mod service;

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
    /// Scaffold a new page (route-level component)
    Page(page::PageArgs),
    /// Scaffold a new service (Angular only)
    Service(service::ServiceArgs),
}
