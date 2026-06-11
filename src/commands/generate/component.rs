use crate::config::resolve_framework;
use crate::scaffold;
use crate::utils::{to_kebab_case, to_pascal_case};
use anyhow::{Result, bail};
use clap::Args;
use console::style;
use serde_json::json;
use std::{fs, path::Path};

#[derive(Args)]
pub struct ComponentArgs {
    /// Component name (e.g. MyCard, my-card)
    pub name: String,

    /// Framework to target; detected from .m2s2.json or package.json if omitted
    #[arg(long, value_parser = ["react", "angular", "vue"])]
    pub framework: Option<String>,

    /// Output directory (default: src/components/<Name> for React/Vue,
    /// src/app/components/<name> for Angular)
    #[arg(long)]
    pub path: Option<String>,
}

pub async fn run(args: ComponentArgs) -> Result<()> {
    let pascal = to_pascal_case(&args.name);
    let kebab = to_kebab_case(&pascal);
    let framework = resolve_framework(args.framework)?;

    let out_dir = match args.path {
        Some(ref p) => Path::new(p).join(&pascal),
        None => match framework.as_str() {
            "react" | "vue" => Path::new("src/components").join(&pascal),
            _ => Path::new("src/app/components").join(&kebab),
        },
    };

    if out_dir.exists() {
        bail!("'{}' already exists", out_dir.display());
    }

    fs::create_dir_all(&out_dir)?;

    let data = json!({ "name": pascal, "selector": kebab, "file_name": kebab });

    let files: &[(&str, &str)] = match framework.as_str() {
        "react" => &[
            ("generate/react/component.tsx.hbs", &format!("{pascal}.tsx")),
            (
                "generate/react/component.scss.hbs",
                &format!("{pascal}.scss"),
            ),
            ("generate/react/index.ts.hbs", "index.ts"),
        ],
        "vue" => &[
            ("generate/vue/component.vue.hbs", &format!("{pascal}.vue")),
            ("generate/vue/component.scss.hbs", &format!("{pascal}.scss")),
            ("generate/vue/index.ts.hbs", "index.ts"),
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

    scaffold::write_files(&out_dir, files, &data)?;

    println!(
        "\n{} {} component generated.\n",
        style("✓").green().bold(),
        style(&pascal).cyan().bold(),
    );

    Ok(())
}
