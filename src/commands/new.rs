use crate::config::M2S2Config;
use crate::scaffold::{ScaffoldPlan, Scaffolder};
use crate::types::{ApiFramework, Frontend, ProjectType, Runtime};
use anyhow::Result;
use clap::{Args, ValueEnum};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Select};
use std::time::Duration;

const AFTER_HELP: &str = "\
Examples:
  m2s2 new my-app
      Interactive prompts guide you through every choice.

  m2s2 new my-app --project-type frontend --framework react
      Scaffold a React frontend, no prompts.

  m2s2 new my-api --project-type backend --runtime go --api-framework gin
      Scaffold a Go/Gin API, no prompts.

  m2s2 new my-api --project-type backend --runtime node --api-framework express
      Scaffold a Node/Express API, no prompts.

  m2s2 new my-api --project-type backend --runtime python --api-framework fastapi
      Scaffold a Python/FastAPI service, no prompts.

  m2s2 new my-app --project-type fullstack --framework react --runtime go --api-framework gin
      Scaffold a React + Gin fullstack project, no prompts.

  m2s2 new my-app --project-type fullstack --framework vue --runtime python --api-framework fastapi
      Scaffold a Vue + FastAPI fullstack project, no prompts.

  m2s2 new my-app --project-type fullstack --framework angular --api-framework gin --auth yes --billing yes
      Scaffold a fullstack project with Cognito auth and Stripe billing.";

#[derive(Args)]
#[command(after_help = AFTER_HELP)]
pub struct NewArgs {
    /// Project name
    pub name: String,

    /// Project type
    #[arg(long)]
    pub project_type: Option<ProjectType>,

    /// Frontend framework
    #[arg(long)]
    pub framework: Option<Frontend>,

    /// Backend runtime
    #[arg(long)]
    pub runtime: Option<Runtime>,

    /// API framework — gin/echo/fiber (Go), express/fastify (Node), fastapi/flask (Python)
    #[arg(long)]
    pub api_framework: Option<ApiFramework>,

    /// Include AWS Cognito authentication scaffolding
    #[arg(long, value_parser = ["yes", "no"])]
    pub auth: Option<String>,

    /// Include Stripe billing scaffolding
    #[arg(long, value_parser = ["yes", "no"])]
    pub billing: Option<String>,

    /// Skip running npm install / go mod tidy / pip install after scaffolding
    #[arg(long)]
    pub skip_install: bool,

    /// Skip network calls to resolve package versions (for offline use / tests)
    #[arg(long)]
    pub offline: bool,
}

pub async fn run(args: NewArgs) -> Result<()> {
    // ── Prompt for choices ────────────────────────────────────────────────────

    let project_type = match args.project_type {
        Some(t) => t,
        None => Select::new("Project type?", ProjectType::value_variants().to_vec()).prompt()?,
    };

    let framework = if project_type != ProjectType::Backend {
        Some(match args.framework {
            Some(f) => f,
            None => {
                Select::new("Frontend framework?", Frontend::value_variants().to_vec()).prompt()?
            }
        })
    } else {
        None
    };

    let (runtime, api_framework) = if project_type != ProjectType::Frontend {
        if let Some(fw) = args.api_framework {
            let rt = args.runtime.unwrap_or_else(|| fw.runtime());
            (Some(rt), Some(fw))
        } else {
            let rt = match args.runtime {
                Some(r) => r,
                None => {
                    Select::new("Backend runtime?", Runtime::value_variants().to_vec()).prompt()?
                }
            };
            let fw = Select::new("API framework?", rt.api_frameworks()).prompt()?;
            (Some(rt), Some(fw))
        }
    } else {
        (None, None)
    };

    let auth = match args.auth.as_deref() {
        Some("yes") => true,
        Some("no") => false,
        _ => Confirm::new("Include authentication (AWS Cognito)?")
            .with_default(false)
            .prompt()?,
    };

    let billing = match args.billing.as_deref() {
        Some("yes") => true,
        Some("no") => false,
        _ => Confirm::new("Include billing (Stripe)?")
            .with_default(false)
            .prompt()?,
    };

    // ── Capture config strings before values are consumed into the plan ───────

    let config_framework = framework.as_ref().map(|f| f.to_string());
    let config_api_framework = api_framework.as_ref().map(|f| f.to_string());
    let config_project_type = project_type.to_string();
    let config_runtime = runtime.as_ref().map(|r| r.to_string());

    // ── Build plan ────────────────────────────────────────────────────────────

    let plan = match project_type {
        ProjectType::Frontend => {
            ScaffoldPlan::Frontend(Scaffolder::for_frontend(framework.unwrap(), auth, billing))
        }
        ProjectType::Backend => ScaffoldPlan::Backend(Scaffolder::for_backend(
            api_framework.unwrap(),
            auth,
            billing,
        )),
        ProjectType::Fullstack => ScaffoldPlan::Fullstack(
            Scaffolder::for_frontend(framework.unwrap(), auth, billing),
            Scaffolder::for_backend(api_framework.unwrap(), auth, billing),
        ),
    };

    let label = match &plan {
        ScaffoldPlan::Frontend(fe) => fe.template_prefix.to_string(),
        ScaffoldPlan::Backend(be) => be.template_prefix.to_string(),
        ScaffoldPlan::Fullstack(fe, be) => {
            format!("{} + {}", fe.template_prefix, be.template_prefix)
        }
    };

    println!(
        "\n{} {} ({})\n",
        style("Scaffolding").green().bold(),
        style(&args.name).cyan().bold(),
        style(&label).cyan(),
    );

    // ── Execute ───────────────────────────────────────────────────────────────

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner} {msg}")?,
    );
    spinner.enable_steady_tick(Duration::from_millis(80));

    if !args.offline {
        spinner.set_message("Resolving package versions…");
    }

    plan.execute(&args.name, args.offline, args.skip_install, &spinner)
        .await?;

    // ── Save config ───────────────────────────────────────────────────────────

    let prev = std::env::current_dir()?;
    std::env::set_current_dir(&args.name)?;
    let _ = M2S2Config {
        framework: config_framework,
        api_framework: config_api_framework,
        project_type: Some(config_project_type),
        runtime: config_runtime,
        auth,
        billing,
    }
    .save();
    std::env::set_current_dir(&prev)?;

    spinner.finish_and_clear();

    println!("{}\n", style("Done!").green().bold());
    println!("  {} {}", style("cd").dim(), style(&args.name).cyan());
    println!("  {}", style("m2s2 dev").cyan().bold());
    println!();

    Ok(())
}
