use crate::npm;
use crate::types::{ApiFramework, Frontend, Runtime};
use anyhow::Result;
use indicatif::ProgressBar;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::path::Path;
use std::sync::OnceLock;
use tokio::process::Command;

// ── Runtime config (embedded at compile time) ─────────────────────────────────

#[derive(Deserialize)]
struct RuntimeEntry {
    runtime: String,
    handler: String,
}

#[derive(Deserialize)]
struct LambdaConfig {
    go: RuntimeEntry,
    node: RuntimeEntry,
    python: RuntimeEntry,
}

#[derive(Deserialize)]
pub(crate) struct VersionConfig {
    pub(crate) go: String,
    pub(crate) node: String,
    pub(crate) python: String,
}

#[derive(Deserialize)]
pub(crate) struct Runtimes {
    lambda: LambdaConfig,
    pub(crate) versions: VersionConfig,
}

pub(crate) fn runtimes() -> &'static Runtimes {
    static RUNTIMES: OnceLock<Runtimes> = OnceLock::new();
    RUNTIMES.get_or_init(|| {
        toml::from_str(include_str!("../../config/runtimes.toml"))
            .expect("invalid config/runtimes.toml")
    })
}

// ── Strategy enums ────────────────────────────────────────────────────────────

enum VersionResolution {
    /// Frontend: resolve versions anchored to an m2s2 library package.
    Framework {
        anchor: &'static str,
        supplemental: &'static [&'static str],
    },
    /// Node backend: resolve a flat list of packages directly.
    Direct {
        base: &'static [&'static str],
        framework_extra: &'static [&'static str],
    },
    /// Go / Python: no npm resolution needed.
    None,
}

enum InstallKind {
    Npm,
    GoModTidy,
    PythonVenv,
}

// ── Scaffolder ────────────────────────────────────────────────────────────────

pub struct Scaffolder {
    pub template_prefix: &'static str,
    pub auth: bool,
    pub billing: bool,
    /// npm script name for `m2s2 dev`; None for backend scaffolders.
    pub dev_script: Option<&'static str>,
    /// CDK Lambda runtime string (from runtimes.toml); empty for frontend.
    pub lambda_runtime: String,
    /// CDK Lambda handler string (from runtimes.toml); empty for frontend.
    pub lambda_handler: String,
    /// Backend runtime variant; None for frontend scaffolders.
    pub backend_runtime: Option<Runtime>,
    resolution: VersionResolution,
    install_kind: InstallKind,
    auth_packages: &'static [&'static str],
    billing_packages: &'static [&'static str],
}

impl Scaffolder {
    pub fn for_frontend(framework: Frontend, auth: bool, billing: bool) -> Self {
        match framework {
            Frontend::React => Self {
                template_prefix: "react",
                dev_script: Some("dev"),
                lambda_runtime: String::new(),
                lambda_handler: String::new(),
                backend_runtime: None,
                resolution: VersionResolution::Framework {
                    anchor: "@m2s2/react-lib",
                    supplemental: &[
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
                },
                install_kind: InstallKind::Npm,
                auth_packages: &["aws-amplify", "@aws-amplify/ui-react"],
                billing_packages: &["@stripe/stripe-js", "@stripe/react-stripe-js"],
                auth,
                billing,
            },
            Frontend::Angular => Self {
                template_prefix: "angular",
                dev_script: Some("start"),
                lambda_runtime: String::new(),
                lambda_handler: String::new(),
                backend_runtime: None,
                resolution: VersionResolution::Framework {
                    anchor: "@m2s2/ng-lib",
                    supplemental: &[
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
                },
                install_kind: InstallKind::Npm,
                auth_packages: &["aws-amplify", "@aws-amplify/ui-angular"],
                billing_packages: &["@stripe/stripe-js"],
                auth,
                billing,
            },
            Frontend::Vue => Self {
                template_prefix: "vue",
                dev_script: Some("dev"),
                lambda_runtime: String::new(),
                lambda_handler: String::new(),
                backend_runtime: None,
                resolution: VersionResolution::Framework {
                    anchor: "@m2s2/vue-lib",
                    supplemental: &[
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
                },
                install_kind: InstallKind::Npm,
                auth_packages: &["aws-amplify", "@aws-amplify/ui-vue"],
                billing_packages: &["@stripe/stripe-js"],
                auth,
                billing,
            },
        }
    }

    pub fn for_backend(api_framework: ApiFramework, auth: bool, billing: bool) -> Self {
        let rt = api_framework.runtime();
        let cfg = runtimes();
        let (lambda_runtime, lambda_handler) = match rt {
            Runtime::Go => (cfg.lambda.go.runtime.clone(), cfg.lambda.go.handler.clone()),
            Runtime::Node => (
                cfg.lambda.node.runtime.clone(),
                cfg.lambda.node.handler.clone(),
            ),
            Runtime::Python => (
                cfg.lambda.python.runtime.clone(),
                cfg.lambda.python.handler.clone(),
            ),
        };

        const NODE_BASE: &[&str] = &[
            "@types/node",
            "tsx",
            "typescript",
            "vitest",
            "eslint",
            "@eslint/js",
            "typescript-eslint",
        ];

        match api_framework {
            ApiFramework::Gin => Self {
                template_prefix: "gin",
                dev_script: None,
                lambda_runtime,
                lambda_handler,
                backend_runtime: Some(rt),
                resolution: VersionResolution::None,
                install_kind: InstallKind::GoModTidy,
                auth_packages: &[],
                billing_packages: &[],
                auth,
                billing,
            },
            ApiFramework::Echo => Self {
                template_prefix: "echo",
                dev_script: None,
                lambda_runtime,
                lambda_handler,
                backend_runtime: Some(rt),
                resolution: VersionResolution::None,
                install_kind: InstallKind::GoModTidy,
                auth_packages: &[],
                billing_packages: &[],
                auth,
                billing,
            },
            ApiFramework::Fiber => Self {
                template_prefix: "fiber",
                dev_script: None,
                lambda_runtime,
                lambda_handler,
                backend_runtime: Some(rt),
                resolution: VersionResolution::None,
                install_kind: InstallKind::GoModTidy,
                auth_packages: &[],
                billing_packages: &[],
                auth,
                billing,
            },
            ApiFramework::Express => Self {
                template_prefix: "express",
                dev_script: None,
                lambda_runtime,
                lambda_handler,
                backend_runtime: Some(rt),
                resolution: VersionResolution::Direct {
                    base: NODE_BASE,
                    framework_extra: &["express", "@types/express"],
                },
                install_kind: InstallKind::Npm,
                auth_packages: &["jsonwebtoken", "@types/jsonwebtoken", "jwks-rsa"],
                billing_packages: &["stripe"],
                auth,
                billing,
            },
            ApiFramework::Fastify => Self {
                template_prefix: "fastify",
                dev_script: None,
                lambda_runtime,
                lambda_handler,
                backend_runtime: Some(rt),
                resolution: VersionResolution::Direct {
                    base: NODE_BASE,
                    framework_extra: &["fastify"],
                },
                install_kind: InstallKind::Npm,
                auth_packages: &["@fastify/jwt", "jwks-rsa"],
                billing_packages: &["stripe"],
                auth,
                billing,
            },
            ApiFramework::Fastapi => Self {
                template_prefix: "fastapi",
                dev_script: None,
                lambda_runtime,
                lambda_handler,
                backend_runtime: Some(rt),
                resolution: VersionResolution::None,
                install_kind: InstallKind::PythonVenv,
                auth_packages: &[],
                billing_packages: &[],
                auth,
                billing,
            },
            ApiFramework::Flask => Self {
                template_prefix: "flask",
                dev_script: None,
                lambda_runtime,
                lambda_handler,
                backend_runtime: Some(rt),
                resolution: VersionResolution::None,
                install_kind: InstallKind::PythonVenv,
                auth_packages: &[],
                billing_packages: &[],
                auth,
                billing,
            },
        }
    }

    pub async fn resolve_versions(&self) -> Result<Map<String, Value>> {
        let mut versions = match &self.resolution {
            VersionResolution::Framework {
                anchor,
                supplemental,
            } => npm::resolve_for_framework(anchor, supplemental).await?,
            VersionResolution::Direct {
                base,
                framework_extra,
            } => {
                let all: Vec<&str> = base.iter().chain(framework_extra.iter()).copied().collect();
                npm::resolve_packages(&all).await?
            }
            VersionResolution::None => Map::new(),
        };
        if self.auth && !self.auth_packages.is_empty() {
            versions.extend(npm::resolve_packages(self.auth_packages).await?);
        }
        if self.billing && !self.billing_packages.is_empty() {
            versions.extend(npm::resolve_packages(self.billing_packages).await?);
        }
        Ok(versions)
    }

    pub async fn install(&self, dir: &Path, spinner: &ProgressBar) -> Result<()> {
        match self.install_kind {
            InstallKind::Npm => {
                spinner.set_message("Running npm install…");
                let status = Command::new("npm")
                    .arg("install")
                    .current_dir(dir)
                    .status()
                    .await?;
                if !status.success() {
                    anyhow::bail!("npm install failed");
                }
            }
            InstallKind::GoModTidy => {
                spinner.set_message("Running go mod tidy…");
                let status = Command::new("go")
                    .args(["mod", "tidy"])
                    .current_dir(dir)
                    .status()
                    .await?;
                if !status.success() {
                    anyhow::bail!("go mod tidy failed");
                }
            }
            InstallKind::PythonVenv => {
                spinner.set_message("Creating virtual environment…");
                let status = Command::new("python3")
                    .args(["-m", "venv", ".venv"])
                    .current_dir(dir)
                    .status()
                    .await?;
                if !status.success() {
                    anyhow::bail!("python3 -m venv failed");
                }
                spinner.set_message("Running pip install…");
                let status = Command::new(".venv/bin/pip")
                    .args(["install", "-r", "requirements.txt"])
                    .current_dir(dir)
                    .status()
                    .await?;
                if !status.success() {
                    anyhow::bail!("pip install failed");
                }
            }
        }
        Ok(())
    }
}
