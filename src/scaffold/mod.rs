use anyhow::{Context, Result};
use console::style;
use handlebars::Handlebars;
use rust_embed::Embed;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

pub fn get_template(path: &str) -> Option<String> {
    Templates::get(path).and_then(|f| {
        std::str::from_utf8(f.data.as_ref())
            .ok()
            .map(|s| s.to_string())
    })
}

pub fn render(raw: &str, data: &Value) -> Result<String> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);
    hbs.render_template(raw, data).map_err(Into::into)
}

/// Render and write a set of template files into `out_dir`, printing each created path.
///
/// `files` is a slice of `(template_path, output_filename)` pairs.
pub fn write_files(out_dir: &Path, files: &[(&str, &str)], data: &Value) -> Result<()> {
    for (template_path, file_name) in files {
        let raw = get_template(template_path)
            .with_context(|| format!("missing embedded template '{template_path}'"))?;
        let content = render(&raw, data)?;
        let out = out_dir.join(file_name);
        fs::write(&out, content).with_context(|| format!("failed to write '{}'", out.display()))?;
        println!("  {} {}", style("create").green(), out.display());
    }
    Ok(())
}

pub struct ScaffoldContext {
    pub name: String,
    pub framework: String,
    pub versions: serde_json::Map<String, Value>,
}

pub fn run(ctx: &ScaffoldContext) -> Result<()> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);

    let mut data = json!({ "name": ctx.name });
    for (k, v) in &ctx.versions {
        data[k] = v.clone();
    }
    let prefix = format!("{}/", ctx.framework);
    let target = Path::new(&ctx.name);

    fs::create_dir_all(target)
        .with_context(|| format!("failed to create directory '{}'", ctx.name))?;

    for path in Templates::iter() {
        let path_str = path.as_ref();

        if !path_str.starts_with(&prefix) {
            continue;
        }

        let relative = &path_str[prefix.len()..];
        let (out_relative, render) = if let Some(stem) = relative.strip_suffix(".hbs") {
            (stem.to_string(), true)
        } else if let Some(stem) = relative.strip_prefix('_') {
            // _gitignore → .gitignore
            let dir = Path::new(relative)
                .parent()
                .map(|p| p.to_str().unwrap_or(""))
                .unwrap_or("");
            let out = if dir.is_empty() {
                format!(".{stem}")
            } else {
                format!("{dir}/.{stem}")
            };
            (out, false)
        } else {
            (relative.to_string(), false)
        };

        let out_path = target.join(&out_relative);

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = Templates::get(path_str).expect("embedded file missing");
        let raw = std::str::from_utf8(file.data.as_ref())
            .with_context(|| format!("template '{path_str}' is not valid UTF-8"))?;

        let content = if render {
            hbs.render_template(raw, &data)
                .with_context(|| format!("failed to render '{path_str}'"))?
        } else {
            raw.to_string()
        };

        fs::write(&out_path, content)
            .with_context(|| format!("failed to write '{}'", out_path.display()))?;
    }

    Ok(())
}
