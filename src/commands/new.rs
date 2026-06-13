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
      Scaffold a Vue + FastAPI fullstack project, no prompts.";

#[derive(Args)]
#[command(after_help = AFTER_HELP)]
pub struct NewArgs {
    /// Project name
    pub name: String,

    /// Project type
    #[arg(long, value_parser = ["frontend", "backend", "fullstack"])]
    pub project_type: Option<String>,

    /// Frontend framework
    #[arg(long, value_parser = ["react", "angular", "vue"])]
    pub framework: Option<String>,

    /// Backend runtime
    #[arg(long, value_parser = ["go", "node", "python"])]
    pub runtime: Option<String>,

    /// API framework — gin/echo/fiber (Go), express/fastify (Node), fastapi/flask (Python)
    #[arg(long, value_parser = ["gin", "echo", "fiber", "express", "fastify", "fastapi", "flask"])]
    pub api_framework: Option<String>,

    /// Skip running npm install / go mod tidy / pip install after scaffolding
    #[arg(long)]
    pub skip_install: bool,
}

pub async fn run(args: NewArgs) -> Result<()> {
    let project_type = match args.project_type {
        Some(t) => t,
        None => Select::new("Project type?", vec!["frontend", "backend", "fullstack"])
            .prompt()
            .map(|s| s.to_string())?,
    };

    let framework = if project_type != "backend" {
        Some(match args.framework {
            Some(f) => f,
            None => Select::new("Frontend framework?", vec!["react", "angular", "vue"])
                .prompt()
                .map(|s| s.to_string())?,
        })
    } else {
        None
    };

    let (runtime, api_framework) = if project_type != "frontend" {
        // If --api-framework was given, derive runtime from it
        if let Some(fw) = args.api_framework {
            let rt = args.runtime.unwrap_or_else(|| runtime_of(&fw).to_string());
            (Some(rt), Some(fw))
        } else {
            let rt = match args.runtime {
                Some(r) => r,
                None => Select::new("Backend runtime?", vec!["Go", "Node", "Python"])
                    .prompt()
                    .map(|s| s.to_lowercase())?,
            };
            let fw = Select::new("API framework?", frameworks_for_runtime(&rt))
                .prompt()
                .map(|s| s.to_string())?;
            (Some(rt), Some(fw))
        }
    } else {
        (None, None)
    };

    let label = match project_type.as_str() {
        "fullstack" => format!(
            "{} + {}",
            framework.as_deref().unwrap(),
            api_framework.as_deref().unwrap()
        ),
        "frontend" => framework.as_deref().unwrap().to_string(),
        _ => api_framework.as_deref().unwrap().to_string(),
    };

    println!(
        "\n{} {} ({})\n",
        style("Scaffolding").green().bold(),
        style(&args.name).cyan().bold(),
        style(&label).cyan(),
    );

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner} {msg}")?,
    );
    spinner.enable_steady_tick(Duration::from_millis(80));

    let versions = {
        let mut v = if project_type != "backend" {
            spinner.set_message("Resolving frontend package versions…");
            resolve_frontend_versions(framework.as_deref().unwrap()).await?
        } else {
            Default::default()
        };

        if project_type != "frontend"
            && let Some(rt) = runtime.as_deref()
            && let Some(api_fw) = api_framework.as_deref()
        {
            spinner.set_message("Resolving backend package versions…");
            let bv = resolve_backend_versions(rt, api_fw).await?;
            v.extend(bv);
        }
        v
    };

    spinner.set_message("Writing project files…");

    scaffold::run(&ScaffoldContext {
        name: args.name.clone(),
        project_type: project_type.clone(),
        framework: framework.clone(),
        api_framework: api_framework.clone(),
        versions,
    })?;

    let prev = std::env::current_dir()?;
    std::env::set_current_dir(&args.name)?;
    let _ = M2S2Config {
        framework: framework.clone(),
        api_framework: api_framework.clone(),
        project_type: Some(project_type.clone()),
        runtime: runtime.clone(),
    }
    .save();
    std::env::set_current_dir(&prev)?;

    if !args.skip_install {
        let web_dir = match project_type.as_str() {
            "fullstack" => Some(format!("{}/apps/web", args.name)),
            "frontend" => Some(args.name.clone()),
            _ => None,
        };

        if let Some(dir) = web_dir {
            spinner.set_message("Running npm install…");
            let status = Command::new("npm")
                .arg("install")
                .current_dir(&dir)
                .status()
                .await?;
            if !status.success() {
                spinner.finish_and_clear();
                anyhow::bail!("npm install failed");
            }
        }

        let api_dir = match project_type.as_str() {
            "fullstack" => Some(format!("{}/apps/api", args.name)),
            "backend" => Some(args.name.clone()),
            _ => None,
        };

        if let Some(dir) = api_dir {
            match runtime.as_deref() {
                Some("go") => {
                    spinner.set_message("Running go mod tidy…");
                    let status = Command::new("go")
                        .args(["mod", "tidy"])
                        .current_dir(&dir)
                        .status()
                        .await?;
                    if !status.success() {
                        spinner.finish_and_clear();
                        anyhow::bail!("go mod tidy failed");
                    }
                }
                Some("node") => {
                    spinner.set_message("Running npm install (API)…");
                    let status = Command::new("npm")
                        .arg("install")
                        .current_dir(&dir)
                        .status()
                        .await?;
                    if !status.success() {
                        spinner.finish_and_clear();
                        anyhow::bail!("npm install (API) failed");
                    }
                }
                Some("python") => {
                    spinner.set_message("Running pip install…");
                    let status = Command::new("pip")
                        .args(["install", "-r", "requirements.txt"])
                        .current_dir(&dir)
                        .status()
                        .await?;
                    if !status.success() {
                        spinner.finish_and_clear();
                        anyhow::bail!("pip install failed");
                    }
                }
                _ => {}
            }
        }
    }

    spinner.finish_and_clear();

    println!("{}\n", style("Done!").green().bold());
    println!("  {} {}", style("cd").dim(), style(&args.name).cyan());
    println!("  {}", style("m2s2 dev").cyan().bold());
    println!();

    Ok(())
}

fn runtime_of(api_framework: &str) -> &'static str {
    match api_framework {
        "gin" | "echo" | "fiber" => "go",
        "express" | "fastify" => "node",
        "fastapi" | "flask" => "python",
        _ => "go",
    }
}

fn frameworks_for_runtime(runtime: &str) -> Vec<&'static str> {
    match runtime {
        "go" => vec!["gin", "echo", "fiber"],
        "node" => vec!["express", "fastify"],
        "python" => vec!["fastapi", "flask"],
        _ => vec![],
    }
}

async fn resolve_frontend_versions(
    framework: &str,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    match framework {
        "angular" => {
            npm::resolve_for_framework(
                "@m2s2/ng-lib",
                &[
                    "rxjs",
                    "zone.js",
                    "typescript",
                    "tslib",
                    "jest",
                    "jest-preset-angular",
                    "@types/jest",
                    "eslint",
                    "@eslint/js",
                    "typescript-eslint",
                    "angular-eslint",
                ],
            )
            .await
        }
        "react" => {
            npm::resolve_for_framework(
                "@m2s2/react-lib",
                &[
                    "typescript",
                    "vite",
                    "vitest",
                    "@types/react",
                    "@types/react-dom",
                    "@vitejs/plugin-react",
                    "@testing-library/react",
                    "@testing-library/jest-dom",
                    "jsdom",
                    "sass-embedded",
                    "eslint",
                    "@eslint/js",
                    "typescript-eslint",
                    "eslint-plugin-react-hooks",
                    "eslint-plugin-react-refresh",
                ],
            )
            .await
        }
        "vue" => {
            npm::resolve_for_framework(
                "@m2s2/vue-lib",
                &[
                    "typescript",
                    "vite",
                    "vitest",
                    "@vitejs/plugin-vue",
                    "@testing-library/vue",
                    "@testing-library/jest-dom",
                    "jsdom",
                    "vue-tsc",
                    "sass-embedded",
                    "eslint",
                    "@eslint/js",
                    "typescript-eslint",
                    "eslint-plugin-vue",
                ],
            )
            .await
        }
        _ => Ok(Default::default()),
    }
}

async fn resolve_backend_versions(
    runtime: &str,
    api_framework: &str,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    match runtime {
        "node" => {
            let base = &[
                "@types/node",
                "tsx",
                "typescript",
                "vitest",
                "eslint",
                "@eslint/js",
                "typescript-eslint",
            ];
            let extra: &[&str] = match api_framework {
                "express" => &["express", "@types/express"],
                "fastify" => &["fastify"],
                _ => &[],
            };
            let all: Vec<&str> = base.iter().chain(extra.iter()).copied().collect();
            npm::resolve_packages(&all).await
        }
        _ => Ok(Default::default()),
    }
}
