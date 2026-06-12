use anyhow::{Context, Result, bail};
use clap::Args;
use console::style;
use std::process::Command;

use crate::config::resolve_framework;

#[derive(Args)]
pub struct DevArgs {
    /// Framework to use (overrides auto-detection)
    #[arg(long, value_parser = ["angular", "react", "vue"])]
    pub framework: Option<String>,

    /// Extra arguments passed through to the underlying dev server
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Args)]
pub struct BuildArgs {
    /// Framework to use (overrides auto-detection)
    #[arg(long, value_parser = ["angular", "react", "vue"])]
    pub framework: Option<String>,

    /// Extra arguments passed through to the underlying build tool
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Args)]
pub struct TestArgs {
    /// Framework to use (overrides auto-detection)
    #[arg(long, value_parser = ["angular", "react", "vue"])]
    pub framework: Option<String>,

    /// Extra arguments passed through to the test runner
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Args)]
pub struct LintArgs {
    /// Framework to use (overrides auto-detection)
    #[arg(long, value_parser = ["angular", "react", "vue"])]
    pub framework: Option<String>,

    /// Extra arguments passed through to the linter
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

pub async fn dev(args: DevArgs) -> Result<()> {
    let framework = resolve_framework(args.framework)?;
    let script = dev_script(&framework);
    run_npm_script(script, &args.extra)
}

pub async fn build(args: BuildArgs) -> Result<()> {
    let framework = resolve_framework(args.framework)?;
    let script = build_script(&framework);
    run_npm_script(script, &args.extra)
}

pub async fn test(args: TestArgs) -> Result<()> {
    let framework = resolve_framework(args.framework)?;
    let script = test_script(&framework);
    run_npm_script(script, &args.extra)
}

pub async fn lint(args: LintArgs) -> Result<()> {
    let framework = resolve_framework(args.framework)?;
    let script = lint_script(&framework);
    run_npm_script(script, &args.extra)
}

fn dev_script(framework: &str) -> &'static str {
    match framework {
        "angular" => "start",
        _ => "dev",
    }
}

fn build_script(_framework: &str) -> &'static str {
    "build"
}

fn test_script(_framework: &str) -> &'static str {
    "test"
}

fn lint_script(_framework: &str) -> &'static str {
    "lint"
}

fn run_npm_script(script: &str, extra: &[String]) -> Result<()> {
    let npm = npm_bin();

    println!(
        "{} {} run {}{}",
        style("→").dim(),
        npm,
        style(script).cyan().bold(),
        if extra.is_empty() {
            String::new()
        } else {
            format!(" {}", extra.join(" "))
        }
    );

    let status = Command::new(npm)
        .args(["run", script])
        .args(extra)
        .status()
        .with_context(|| format!("failed to spawn `{npm} run {script}`"))?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        bail!("`{npm} run {script}` exited with code {code}");
    }

    Ok(())
}

fn npm_bin() -> &'static str {
    if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    }
}
