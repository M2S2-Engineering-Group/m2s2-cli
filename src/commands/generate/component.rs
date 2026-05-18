use crate::scaffold;
use anyhow::{bail, Context, Result};
use clap::Args;
use console::style;
use serde_json::json;
use std::{fs, path::Path};

#[derive(Args)]
pub struct ComponentArgs {
    /// Component name (e.g. MyCard, my-card)
    pub name: String,

    /// Framework to target; detected from package.json if omitted
    #[arg(long, value_parser = ["react", "angular"])]
    pub framework: Option<String>,

    /// Output directory (default: src/components/<Name> for React,
    /// src/app/components/<name> for Angular)
    #[arg(long)]
    pub path: Option<String>,
}

pub async fn run(args: ComponentArgs) -> Result<()> {
    let pascal = to_pascal_case(&args.name);
    let kebab = to_kebab_case(&pascal);

    let framework = match args.framework {
        Some(f) => f,
        None => detect_framework()
            .context("could not detect framework — run from your project root or pass --framework")?,
    };

    let out_dir = match args.path {
        Some(ref p) => Path::new(p).join(&pascal),
        None => match framework.as_str() {
            "react" => Path::new("src/components").join(&pascal),
            _ => Path::new("src/app/components").join(&kebab),
        },
    };

    if out_dir.exists() {
        bail!("'{}' already exists", out_dir.display());
    }

    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create '{}'", out_dir.display()))?;

    let data = json!({
        "name": pascal,
        "selector": kebab,
        "file_name": kebab,
    });

    let files: &[(&str, &str)] = match framework.as_str() {
        "react" => &[
            ("generate/react/component.tsx.hbs", &format!("{pascal}.tsx")),
            ("generate/react/component.scss.hbs", &format!("{pascal}.scss")),
            ("generate/react/index.ts.hbs", "index.ts"),
        ],
        _ => &[
            (
                "generate/angular/component.ts.hbs",
                &format!("{kebab}.component.ts"),
            ),
            (
                "generate/angular/component.html.hbs",
                &format!("{kebab}.component.html"),
            ),
            (
                "generate/angular/component.scss.hbs",
                &format!("{kebab}.component.scss"),
            ),
        ],
    };

    for (template_path, file_name) in files {
        let raw = scaffold::get_template(template_path)
            .with_context(|| format!("missing embedded template '{template_path}'"))?;
        let content = scaffold::render(&raw, &data)?;
        let out = out_dir.join(file_name);
        fs::write(&out, content)
            .with_context(|| format!("failed to write '{}'", out.display()))?;
        println!("  {} {}", style("create").green(), out.display());
    }

    println!(
        "\n{} {} component generated.\n",
        style("✓").green().bold(),
        style(&pascal).cyan().bold(),
    );

    Ok(())
}

fn detect_framework() -> Option<String> {
    let raw = fs::read_to_string("package.json").ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let deps = pkg.get("dependencies")?;
    if deps.get("@m2s2/react-lib").is_some() {
        Some("react".into())
    } else if deps.get("@m2s2/ng-lib").is_some() {
        Some("angular".into())
    } else {
        None
    }
}

fn to_pascal_case(input: &str) -> String {
    input
        .split(|c: char| c == '-' || c == '_' || c == ' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

fn to_kebab_case(pascal: &str) -> String {
    let mut out = String::new();
    for (i, c) in pascal.chars().enumerate() {
        if c.is_uppercase() && i != 0 {
            out.push('-');
        }
        out.extend(c.to_lowercase());
    }
    out
}
