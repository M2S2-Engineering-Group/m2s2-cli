use crate::config::resolve_framework;
use crate::scaffold;
use crate::utils::{to_kebab_case, to_pascal_case};
use anyhow::{Result, bail};
use clap::Args;
use console::style;
use serde_json::json;
use std::{fs, path::Path};

#[derive(Args)]
pub struct PageArgs {
    /// Page name (e.g. Dashboard, user-profile)
    pub name: String,

    /// Framework to target; detected from .m2s2.json or package.json if omitted
    #[arg(long, value_parser = ["react", "angular", "vue"])]
    pub framework: Option<String>,

    /// Output directory (overrides default)
    #[arg(long)]
    pub path: Option<String>,
}

pub async fn run(args: PageArgs) -> Result<()> {
    let pascal = to_pascal_case(&args.name);
    let kebab = to_kebab_case(&pascal);
    let framework = resolve_framework(args.framework)?;

    let out_dir = match args.path {
        Some(ref p) => Path::new(p).join(&pascal),
        None => match framework.as_str() {
            "angular" => Path::new("src/app/pages").join(&kebab),
            _ => Path::new("src/pages").join(&pascal),
        },
    };

    if out_dir.exists() {
        bail!("'{}' already exists", out_dir.display());
    }

    fs::create_dir_all(&out_dir)?;

    let data = json!({ "name": pascal, "selector": kebab, "file_name": kebab });

    let files: &[(&str, &str)] = match framework.as_str() {
        "react" => &[
            ("generate/react/page.tsx.hbs", &format!("{pascal}Page.tsx")),
            (
                "generate/react/page.scss.hbs",
                &format!("{pascal}Page.scss"),
            ),
            ("generate/react/page-index.ts.hbs", "index.ts"),
        ],
        "vue" => &[
            ("generate/vue/page.vue.hbs", &format!("{pascal}Page.vue")),
            ("generate/vue/page.scss.hbs", &format!("{pascal}Page.scss")),
            ("generate/vue/page-index.ts.hbs", "index.ts"),
        ],
        _ => &[
            (
                "generate/angular/page.ts.hbs",
                &format!("{kebab}.component.ts"),
            ),
            (
                "generate/angular/page.html.hbs",
                &format!("{kebab}.component.html"),
            ),
            (
                "generate/angular/page.scss.hbs",
                &format!("{kebab}.component.scss"),
            ),
        ],
    };

    scaffold::write_files(&out_dir, files, &data)?;

    println!(
        "\n{} {} page generated.\n",
        style("✓").green().bold(),
        style(&pascal).cyan().bold(),
    );

    if framework == "angular" {
        println!("  {} add to app.routes.ts:", style("next").dim());
        println!(
            "    {{ path: '{kebab}', loadComponent: () => import('./pages/{kebab}/{kebab}.component') }}",
        );
        println!();
    }

    Ok(())
}
