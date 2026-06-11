use crate::config::M2S2Config;
use crate::npm;
use crate::scaffold::{self, ScaffoldContext};
use anyhow::Result;
use clap::Args;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Select;
use std::time::Duration;
use tokio::process::Command;

#[derive(Args)]
pub struct NewArgs {
    /// Project name
    pub name: String,

    /// Framework to scaffold (react, angular, or vue)
    #[arg(long, value_parser = ["react", "angular", "vue"])]
    pub framework: Option<String>,

    /// Skip running npm install after scaffolding
    #[arg(long)]
    pub skip_install: bool,
}

pub async fn run(args: NewArgs) -> Result<()> {
    let framework = match args.framework {
        Some(f) => f,
        None => Select::new("Which framework?", vec!["react", "angular", "vue"])
            .prompt()
            .map(|s| s.to_string())?,
    };

    println!(
        "\n{} {} with {}\n",
        style("Scaffolding").green().bold(),
        style(&args.name).cyan().bold(),
        style(&framework).cyan().bold(),
    );

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner} {msg}")?,
    );
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message("Resolving package versions…");

    let versions = match framework.as_str() {
        "angular" => {
            npm::resolve_for_framework("@m2s2/ng-lib", &["rxjs", "zone.js", "typescript", "tslib"])
                .await?
        }
        "react" => {
            npm::resolve_for_framework(
                "@m2s2/react-lib",
                &[
                    "typescript",
                    "vite",
                    "@types/react",
                    "@types/react-dom",
                    "@vitejs/plugin-react",
                    "sass-embedded",
                ],
            )
            .await?
        }
        "vue" => {
            npm::resolve_for_framework(
                "@m2s2/vue-lib",
                &[
                    "typescript",
                    "vite",
                    "@vitejs/plugin-vue",
                    "vue-tsc",
                    "sass-embedded",
                ],
            )
            .await?
        }
        _ => Default::default(),
    };

    spinner.set_message("Writing project files…");

    scaffold::run(&ScaffoldContext {
        name: args.name.clone(),
        framework: framework.clone(),
        versions,
    })?;

    let prev = std::env::current_dir()?;
    std::env::set_current_dir(&args.name)?;
    let _ = M2S2Config {
        framework: Some(framework.clone()),
    }
    .save();
    std::env::set_current_dir(prev)?;

    if !args.skip_install {
        spinner.set_message("Running npm install…");
        let status = Command::new("npm")
            .arg("install")
            .current_dir(&args.name)
            .status()
            .await?;

        if !status.success() {
            spinner.finish_and_clear();
            anyhow::bail!("npm install failed");
        }
    }

    spinner.finish_and_clear();

    println!("{}\n", style("Done!").green().bold());
    println!("  {} {}", style("cd").dim(), style(&args.name).cyan());
    if args.skip_install {
        println!("  {}", style("npm install").dim());
    }
    println!(
        "  {}",
        style(if framework == "angular" {
            "npm start"
        } else {
            "npm run dev"
        })
        .dim()
    );
    println!();

    Ok(())
}
