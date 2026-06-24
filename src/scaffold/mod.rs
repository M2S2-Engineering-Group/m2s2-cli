pub mod scaffolder;

pub use scaffolder::Scaffolder;

use crate::types::Runtime;
use crate::utils::to_pascal_case;
use anyhow::{Context, Result};
use console::style;
use handlebars::Handlebars;
use indicatif::ProgressBar;
use rust_embed::Embed;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

// ── Public template helpers (used by generate commands) ──────────────────────

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

// ── ScaffoldPlan ─────────────────────────────────────────────────────────────

pub enum ScaffoldPlan {
    Frontend(Scaffolder),
    Backend(Scaffolder),
    /// Fullstack composes two scaffolders: frontend into `apps/web`, backend into `apps/api`.
    Fullstack(Scaffolder, Scaffolder),
}

impl ScaffoldPlan {
    pub async fn execute(
        &self,
        name: &str,
        offline: bool,
        skip_install: bool,
        spinner: &ProgressBar,
    ) -> Result<()> {
        let root = Path::new(name);

        match self {
            ScaffoldPlan::Frontend(fe) => {
                let versions = if offline {
                    offline_versions()
                } else {
                    fe.resolve_versions().await?
                };
                let data = build_data(name, versions, Some(fe), None);
                spinner.set_message("Writing project files…");
                scaffold_into(root, fe.template_prefix, &data)?;
                if fe.auth {
                    scaffold_into(root, &format!("auth/{}", fe.template_prefix), &data)?;
                }
                if fe.billing {
                    scaffold_into(root, &format!("billing/{}", fe.template_prefix), &data)?;
                }
                scaffold_into(&root.join("cdk"), "cdk", &data)?;
                if fe.auth {
                    scaffold_into(&root.join("cdk"), "cdk-auth", &data)?;
                }
                if fe.billing {
                    scaffold_into(&root.join("cdk"), "cdk-billing", &data)?;
                }
                scaffold_into(&root.join(".github/workflows"), "github-actions", &data)?;
                scaffold_into(root, "root", &data)?;
                if !skip_install {
                    fe.install(root, spinner).await?;
                }
            }

            ScaffoldPlan::Backend(be) => {
                let versions = if offline {
                    offline_versions()
                } else {
                    be.resolve_versions().await?
                };
                let data = build_data(name, versions, None, Some(be));
                spinner.set_message("Writing project files…");
                scaffold_into(root, be.template_prefix, &data)?;
                if be.auth {
                    scaffold_into(root, &format!("auth/{}", be.template_prefix), &data)?;
                }
                if be.billing {
                    scaffold_into(root, &format!("billing/{}", be.template_prefix), &data)?;
                }
                scaffold_into(&root.join("cdk"), "cdk", &data)?;
                if be.auth {
                    scaffold_into(&root.join("cdk"), "cdk-auth", &data)?;
                }
                if be.billing {
                    scaffold_into(&root.join("cdk"), "cdk-billing", &data)?;
                }
                scaffold_into(&root.join(".github/workflows"), "github-actions", &data)?;
                scaffold_into(root, "root", &data)?;
                if !skip_install {
                    be.install(root, spinner).await?;
                }
            }

            ScaffoldPlan::Fullstack(fe, be) => {
                let versions = if offline {
                    offline_versions()
                } else {
                    let mut v = fe.resolve_versions().await?;
                    v.extend(be.resolve_versions().await?);
                    v
                };
                let data = build_data(name, versions, Some(fe), Some(be));
                spinner.set_message("Writing project files…");
                scaffold_into(&root.join("apps/web"), fe.template_prefix, &data)?;
                if fe.auth {
                    scaffold_into(
                        &root.join("apps/web"),
                        &format!("auth/{}", fe.template_prefix),
                        &data,
                    )?;
                }
                if fe.billing {
                    scaffold_into(
                        &root.join("apps/web"),
                        &format!("billing/{}", fe.template_prefix),
                        &data,
                    )?;
                }
                scaffold_into(&root.join("apps/api"), be.template_prefix, &data)?;
                if be.auth {
                    scaffold_into(
                        &root.join("apps/api"),
                        &format!("auth/{}", be.template_prefix),
                        &data,
                    )?;
                }
                if be.billing {
                    scaffold_into(
                        &root.join("apps/api"),
                        &format!("billing/{}", be.template_prefix),
                        &data,
                    )?;
                }
                scaffold_into(root, "fullstack", &data)?;
                scaffold_into(&root.join("cdk"), "cdk", &data)?;
                if fe.auth {
                    scaffold_into(&root.join("cdk"), "cdk-auth", &data)?;
                }
                if fe.billing {
                    scaffold_into(&root.join("cdk"), "cdk-billing", &data)?;
                }
                scaffold_into(&root.join(".github/workflows"), "github-actions", &data)?;
                scaffold_into(root, "root", &data)?;
                if !skip_install {
                    fe.install(&root.join("apps/web"), spinner).await?;
                    be.install(&root.join("apps/api"), spinner).await?;
                }
            }
        }

        Ok(())
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn build_data(
    name: &str,
    versions: serde_json::Map<String, Value>,
    frontend: Option<&Scaffolder>,
    backend: Option<&Scaffolder>,
) -> Value {
    let auth = frontend
        .map(|s| s.auth)
        .or_else(|| backend.map(|s| s.auth))
        .unwrap_or(false);
    let billing = frontend
        .map(|s| s.billing)
        .or_else(|| backend.map(|s| s.billing))
        .unwrap_or(false);
    let backend_runtime = backend.and_then(|b| b.backend_runtime);

    let mut data = json!({
        "name": name,
        "pascal_name": to_pascal_case(name),
        "auth": auth,
        "billing": billing,
        "has_frontend": frontend.is_some(),
        "has_backend": backend.is_some(),
        "is_fullstack": frontend.is_some() && backend.is_some(),
        "lambda_runtime": backend.map(|b| b.lambda_runtime.as_str()).unwrap_or(""),
        "lambda_handler": backend.map(|b| b.lambda_handler.as_str()).unwrap_or(""),
        "backend_runtime_go": backend_runtime == Some(Runtime::Go),
        "backend_runtime_node": backend_runtime == Some(Runtime::Node),
        "backend_runtime_python": backend_runtime == Some(Runtime::Python),
        "go_version": scaffolder::runtimes().versions.go.as_str(),
        "node_version": scaffolder::runtimes().versions.node.as_str(),
        "python_version": scaffolder::runtimes().versions.python.as_str(),
    });

    for (k, v) in versions {
        data[k] = v;
    }
    if let Some(fe) = frontend
        && let Some(script) = fe.dev_script
    {
        data["frontend_dev_script"] = Value::String(script.to_string());
    }
    data
}

fn scaffold_into(target: &Path, template_prefix: &str, data: &Value) -> Result<()> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(true);

    let prefix = format!("{template_prefix}/");

    fs::create_dir_all(target)
        .with_context(|| format!("failed to create directory '{}'", target.display()))?;

    for path in Templates::iter() {
        let path_str = path.as_ref();

        if !path_str.starts_with(&prefix) {
            continue;
        }
        if path_str.contains("/generate/") {
            continue;
        }

        let relative = &path_str[prefix.len()..];
        let (out_relative, do_render) = if let Some(stem) = relative.strip_suffix(".hbs") {
            (stem.to_string(), true)
        } else if let Some(stem) = relative.strip_prefix('_') {
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

        let content = if do_render {
            hbs.render_template(raw, data)
                .with_context(|| format!("failed to render '{path_str}'"))?
        } else {
            raw.to_string()
        };

        fs::write(&out_path, content)
            .with_context(|| format!("failed to write '{}'", out_path.display()))?;
    }

    Ok(())
}

pub fn offline_versions() -> serde_json::Map<String, serde_json::Value> {
    let keys = [
        "_angular_animations",
        "_angular_build",
        "_angular_cdk",
        "_angular_cli",
        "_angular_common",
        "_angular_compiler",
        "_angular_compiler_cli",
        "_angular_core",
        "_angular_forms",
        "_angular_material",
        "_angular_platform_browser",
        "_angular_platform_browser_dynamic",
        "_angular_router",
        "_eslint_js",
        "_testing_library_jest_dom",
        "_testing_library_react",
        "_testing_library_vue",
        "_types_express",
        "_types_jest",
        "_types_node",
        "_types_react",
        "_types_react_dom",
        "_vitejs_plugin_react",
        "_vitejs_plugin_vue",
        "angular_eslint",
        "eslint",
        "eslint_plugin_react_hooks",
        "eslint_plugin_react_refresh",
        "eslint_plugin_vue",
        "express",
        "fastify",
        "jest",
        "jest_preset_angular",
        "jsdom",
        "react",
        "react_dom",
        "react_router",
        "rxjs",
        "sass_embedded",
        "tslib",
        "tsx",
        "typescript",
        "typescript_eslint",
        "vite",
        "vitest",
        "vue",
        "vue_tsc",
        "zone_js",
        // auth packages
        "aws_amplify",
        "_aws_amplify_ui_react",
        "_aws_amplify_ui_angular",
        "_aws_amplify_ui_vue",
        "jsonwebtoken",
        "_types_jsonwebtoken",
        "jwks_rsa",
        "_fastify_jwt",
        // billing packages
        "_stripe_stripe_js",
        "_stripe_react_stripe_js",
        "stripe",
    ];
    let placeholder = serde_json::Value::String("0.0.0".into());
    keys.iter()
        .map(|k| (k.to_string(), placeholder.clone()))
        .collect()
}
