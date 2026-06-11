use crate::config::{detect_framework, resolve_framework};
use crate::scaffold;
use crate::utils::{to_kebab_case, to_pascal_case};
use anyhow::{Result, bail};
use clap::Args;
use console::style;
use serde_json::json;
use std::{fs, path::Path};

#[derive(Args)]
pub struct ServiceArgs {
    /// Service name (e.g. Auth, user-data)
    pub name: String,

    /// Output directory (overrides default: src/app/services)
    #[arg(long)]
    pub path: Option<String>,
}

pub async fn run(args: ServiceArgs) -> Result<()> {
    let pascal    = to_pascal_case(&args.name);
    let kebab     = to_kebab_case(&pascal);
    let framework = resolve_framework(None)?;

    if framework != "angular" {
        let detected = detect_framework().unwrap_or_default();
        bail!(
            "generate service is for Angular projects (detected: {detected}). \
             For React, create a custom hook (use{pascal}); \
             for Vue, create a composable (use{pascal})."
        );
    }

    let out_dir = match args.path {
        Some(ref p) => Path::new(p).to_path_buf(),
        None        => Path::new("src/app/services").to_path_buf(),
    };

    fs::create_dir_all(&out_dir)?;

    let out_file = out_dir.join(format!("{kebab}.service.ts"));
    if out_file.exists() {
        bail!("'{}' already exists", out_file.display());
    }

    let data = json!({ "name": pascal, "file_name": kebab });

    scaffold::write_files(&out_dir, &[("generate/angular/service.ts.hbs", &format!("{kebab}.service.ts"))], &data)?;

    println!(
        "\n{} {} service generated.\n",
        style("✓").green().bold(),
        style(&pascal).cyan().bold(),
    );

    Ok(())
}
