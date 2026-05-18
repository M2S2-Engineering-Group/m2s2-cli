use anyhow::{Context, Result};
use clap::Args;
use console::style;
use reqwest::Client;
use semver::Version;
use serde::Deserialize;

const REPO: &str = "M2S2-Engineering-Group/m2s2-cli";
const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Args)]
pub struct UpgradeArgs {
    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

pub async fn run(args: UpgradeArgs) -> Result<()> {
    let client = Client::builder()
        .user_agent(format!("m2s2-cli/{CURRENT}"))
        .build()?;

    let release: Release = client
        .get(format!(
            "https://api.github.com/repos/{REPO}/releases/latest"
        ))
        .send()
        .await
        .context("failed to reach GitHub API")?
        .error_for_status()
        .context("GitHub API returned an error")?
        .json()
        .await?;

    let latest_str = release.tag_name.trim_start_matches('v');
    let current = Version::parse(CURRENT).expect("CARGO_PKG_VERSION is always valid semver");
    let latest = Version::parse(latest_str)
        .with_context(|| format!("unexpected release tag: {}", release.tag_name))?;

    if latest <= current {
        println!(
            "{} {} is the latest version.",
            style("✓").green().bold(),
            style(format!("v{CURRENT}")).cyan()
        );
        return Ok(());
    }

    println!(
        "\n{} {} → {}\n",
        style("Update available:").yellow().bold(),
        style(format!("v{current}")).dim(),
        style(format!("v{latest}")).cyan().bold(),
    );
    println!("  Release notes: {}\n", style(&release.html_url).dim());

    if args.check {
        println!("Run {} to install.", style("m2s2 upgrade").cyan());
        return Ok(());
    }

    install_update().await
}

async fn install_update() -> Result<()> {
    use tokio::process::Command;

    println!("{}", style("Installing update…").dim());

    #[cfg(target_family = "windows")]
    let status = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &format!(
                "irm https://github.com/{REPO}/releases/latest/download/m2s2-cli-installer.ps1 | iex"
            ),
        ])
        .status()
        .await?;

    #[cfg(not(target_family = "windows"))]
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "curl --proto '=https' --tlsv1.2 -LsSf \
             https://github.com/{REPO}/releases/latest/download/m2s2-cli-installer.sh | sh"
        ))
        .status()
        .await?;

    if status.success() {
        println!(
            "\n{} Restart your terminal to use the new version.",
            style("Done!").green().bold()
        );
    } else {
        anyhow::bail!("installer exited with a non-zero status");
    }

    Ok(())
}
