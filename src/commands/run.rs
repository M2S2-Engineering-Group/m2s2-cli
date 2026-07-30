use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};
use console::style;
use std::path::Path;
use std::process::Command;

use crate::config::{
    resolve_api_framework, resolve_framework, resolve_project_type, resolve_runtime,
};
use crate::types::{ApiFramework, Frontend};

#[derive(Args)]
pub struct DevArgs {
    /// Frontend framework override (frontend/fullstack only)
    #[arg(long, value_parser = ["angular", "react", "vue"])]
    pub framework: Option<String>,

    /// Extra arguments passed through to the underlying dev server
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Args)]
pub struct BuildArgs {
    /// Frontend framework override (frontend/fullstack only)
    #[arg(long, value_parser = ["angular", "react", "vue"])]
    pub framework: Option<String>,

    /// Extra arguments passed through to the underlying build tool
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Args)]
pub struct TestArgs {
    /// Frontend framework override (frontend/fullstack only)
    #[arg(long, value_parser = ["angular", "react", "vue"])]
    pub framework: Option<String>,

    /// Extra arguments passed through to the test runner
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

#[derive(Args)]
pub struct LintArgs {
    /// Frontend framework override (frontend/fullstack only)
    #[arg(long, value_parser = ["angular", "react", "vue"])]
    pub framework: Option<String>,

    /// Extra arguments passed through to the linter
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

pub async fn dev(args: DevArgs) -> Result<()> {
    match resolve_project_type().as_str() {
        "backend" => run_backend_dev(Path::new(".")),
        "fullstack" => {
            let fw = resolve_framework(args.framework)?;
            println!(
                "{} Starting frontend and API concurrently…",
                style("→").dim()
            );
            let mut web = Command::new(npm_bin())
                .args(["run", dev_script(&fw)])
                .current_dir("apps/web")
                .spawn()
                .context("failed to spawn frontend dev server")?;
            let mut api = spawn_backend_dev(Path::new("apps/api"))?;
            web.wait()?;
            api.wait()?;
            Ok(())
        }
        _ => {
            let fw = resolve_framework(args.framework)?;
            run_npm_script(dev_script(&fw), &args.extra, Path::new("."))
        }
    }
}

pub async fn build(args: BuildArgs) -> Result<()> {
    match resolve_project_type().as_str() {
        "backend" => run_backend_build(&args.extra, Path::new(".")),
        "fullstack" => {
            run_npm_script("build", &[], Path::new("apps/web"))?;
            run_backend_build(&args.extra, Path::new("apps/api"))
        }
        _ => {
            let fw = resolve_framework(args.framework)?;
            run_npm_script(build_script(&fw), &args.extra, Path::new("."))
        }
    }
}

pub async fn test(args: TestArgs) -> Result<()> {
    match resolve_project_type().as_str() {
        "backend" => run_backend_test(&args.extra, Path::new(".")),
        "fullstack" => {
            run_npm_script("test", &[], Path::new("apps/web"))?;
            run_backend_test(&args.extra, Path::new("apps/api"))
        }
        _ => {
            let fw = resolve_framework(args.framework)?;
            run_npm_script(test_script(&fw), &args.extra, Path::new("."))
        }
    }
}

pub async fn lint(args: LintArgs) -> Result<()> {
    match resolve_project_type().as_str() {
        "backend" => run_backend_lint(&args.extra, Path::new(".")),
        "fullstack" => {
            run_npm_script("lint", &[], Path::new("apps/web"))?;
            run_backend_lint(&args.extra, Path::new("apps/api"))
        }
        _ => {
            let fw = resolve_framework(args.framework)?;
            run_npm_script(lint_script(&fw), &args.extra, Path::new("."))
        }
    }
}

// ── Backend dispatch ─────────────────────────────────────────────────────────

fn current_runtime() -> String {
    resolve_runtime()
        .or_else(|| {
            resolve_api_framework().and_then(|fw| {
                ApiFramework::from_str(&fw, true)
                    .ok()
                    .map(|f| f.runtime().to_string())
            })
        })
        .unwrap_or_else(|| "go".into())
}

fn run_backend_dev(dir: &Path) -> Result<()> {
    let mut child = spawn_backend_dev(dir)?;
    child.wait()?;
    Ok(())
}

fn spawn_backend_dev(dir: &Path) -> Result<std::process::Child> {
    match current_runtime().as_str() {
        "node" => {
            print_cmd("npm run dev");
            Command::new(npm_bin())
                .args(["run", "dev"])
                .current_dir(dir)
                .spawn()
                .context("failed to spawn Node dev server")
        }
        "python" => {
            let api_fw = resolve_api_framework().unwrap_or_default();
            let py_args = python_dev_args(&api_fw);
            print_cmd(&format!(".venv/bin/python3 -m {}", py_args.join(" ")));
            Command::new(".venv/bin/python3")
                .arg("-m")
                .args(&py_args)
                .current_dir(dir)
                .spawn()
                .context("failed to spawn Python dev server")
        }
        _ => {
            print_cmd("go run ./...");
            Command::new("go")
                .args(["run", "./..."])
                .current_dir(dir)
                .spawn()
                .context("failed to spawn Go server")
        }
    }
}

fn run_backend_build(extra: &[String], dir: &Path) -> Result<()> {
    match current_runtime().as_str() {
        "node" => run_npm_script("build", extra, dir),
        "python" => Ok(()), // Python has no build step
        _ => run_go(&["build", "-o", "bin/api", "."], extra, dir),
    }
}

fn run_backend_test(extra: &[String], dir: &Path) -> Result<()> {
    match current_runtime().as_str() {
        "node" => run_npm_script("test", extra, dir),
        "python" => run_python_module("pytest", extra, dir),
        _ => run_go(&["test", "./..."], extra, dir),
    }
}

fn run_backend_lint(extra: &[String], dir: &Path) -> Result<()> {
    match current_runtime().as_str() {
        "node" => run_npm_script("lint", extra, dir),
        "python" => run_python_module("ruff", &prepend2("check", ".", extra), dir),
        _ => run_go(&["vet", "./..."], extra, dir),
    }
}

// ── Command runners ───────────────────────────────────────────────────────────

fn run_npm_script(script: &str, extra: &[String], dir: &Path) -> Result<()> {
    let npm = npm_bin();
    print_cmd(&format!(
        "{npm} run {script}{}",
        if extra.is_empty() {
            String::new()
        } else {
            format!(" {}", extra.join(" "))
        }
    ));
    let status = Command::new(npm)
        .args(["run", script])
        .args(extra)
        .current_dir(dir)
        .status()
        .with_context(|| format!("failed to spawn `{npm} run {script}`"))?;
    if !status.success() {
        bail!(
            "`{npm} run {script}` exited with code {}",
            status.code().unwrap_or(1)
        );
    }
    Ok(())
}

fn run_go(go_args: &[&str], extra: &[String], dir: &Path) -> Result<()> {
    let cmd_str = go_args.join(" ");
    print_cmd(&format!(
        "go {cmd_str}{}",
        if extra.is_empty() {
            String::new()
        } else {
            format!(" {}", extra.join(" "))
        }
    ));
    let status = Command::new("go")
        .args(go_args)
        .args(extra)
        .current_dir(dir)
        .status()
        .with_context(|| format!("failed to spawn `go {cmd_str}`"))?;
    if !status.success() {
        bail!(
            "`go {cmd_str}` exited with code {}",
            status.code().unwrap_or(1)
        );
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn python_dev_args(api_framework: &str) -> Vec<&'static str> {
    ApiFramework::from_str(api_framework, true)
        .map(|f| f.python_dev_args())
        .unwrap_or_else(|_| vec!["main"])
}

fn run_python_module(module: &str, extra: &[String], dir: &Path) -> Result<()> {
    let extra_str = if extra.is_empty() {
        String::new()
    } else {
        format!(" {}", extra.join(" "))
    };
    print_cmd(&format!(".venv/bin/python3 -m {module}{extra_str}"));
    let status = Command::new(".venv/bin/python3")
        .args(["-m", module])
        .args(extra)
        .current_dir(dir)
        .status()
        .with_context(|| format!("failed to spawn `python3 -m {module}`"))?;
    if !status.success() {
        bail!(
            "`python3 -m {module}` exited with code {}",
            status.code().unwrap_or(1)
        );
    }
    Ok(())
}

fn dev_script(framework: &str) -> &'static str {
    Frontend::from_str(framework, true)
        .map(|f| f.dev_script())
        .unwrap_or("dev")
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

fn print_cmd(cmd: &str) {
    println!("{} {}", style("→").dim(), style(cmd).cyan().bold());
}

fn prepend2(first: &str, second: &str, rest: &[String]) -> Vec<String> {
    let mut v = vec![first.to_string(), second.to_string()];
    v.extend_from_slice(rest);
    v
}

fn npm_bin() -> &'static str {
    if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    }
}
