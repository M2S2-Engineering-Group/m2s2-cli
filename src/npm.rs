use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize, Default)]
struct NpmPackageMeta {
    #[serde(rename = "peerDependencies", default)]
    peer_dependencies: Map<String, Value>,
    #[serde(default)]
    dependencies: Map<String, Value>,
    #[serde(rename = "devDependencies", default)]
    dev_dependencies: Map<String, Value>,
    version: String,
}

pub fn sanitize_key(package: &str) -> String {
    package.replace(['@', '/', '.'], "_").replace('-', "_")
}

// Strip range operators (^, ~, >=, etc.) to get a plain semver string.
fn extract_version(range: &str) -> Option<String> {
    let v: String = range
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    if v.contains('.') {
        Some(v)
    } else {
        None
    }
}

async fn fetch_meta(client: &Client, package: &str) -> Result<NpmPackageMeta> {
    let url = format!("https://registry.npmjs.org/{}/latest", package);
    client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("failed to fetch npm metadata for '{package}'"))?
        .error_for_status()
        .with_context(|| format!("'{package}' not found on npm registry"))?
        .json()
        .await
        .with_context(|| format!("failed to parse npm metadata for '{package}'"))
}

/// Resolve all package versions needed for scaffolding.
///
/// Uses the m2s2 library as the source of truth: its peer, regular, and dev
/// dependencies determine compatible versions. Any packages still missing after
/// that are fetched individually from npm.
pub async fn resolve_for_framework(
    m2s2_lib: &str,
    supplemental: &[&str],
) -> Result<Map<String, Value>> {
    let client = Client::new();
    let meta = fetch_meta(&client, m2s2_lib).await?;

    let mut versions: Map<String, Value> = Map::new();

    for dep_map in [
        &meta.peer_dependencies,
        &meta.dependencies,
        &meta.dev_dependencies,
    ] {
        for (pkg, range) in dep_map {
            if let Some(v) = range.as_str().and_then(extract_version) {
                versions
                    .entry(sanitize_key(pkg))
                    .or_insert(Value::String(v));
            }
        }
    }

    // All @angular/* packages share the same version — fill in any not explicitly listed.
    let angular_version = ["_angular_core", "_angular_build", "_angular_cli"]
        .iter()
        .find_map(|k| {
            versions
                .get(*k)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

    if let Some(ref av) = angular_version {
        for pkg in &[
            "@angular/animations",
            "@angular/cdk",
            "@angular/common",
            "@angular/compiler",
            "@angular/compiler-cli",
            "@angular/core",
            "@angular/forms",
            "@angular/material",
            "@angular/platform-browser",
            "@angular/platform-browser-dynamic",
            "@angular/router",
            "@angular/build",
            "@angular/cli",
        ] {
            versions
                .entry(sanitize_key(pkg))
                .or_insert_with(|| Value::String(av.clone()));
        }
    }

    // Fetch any supplemental packages not already resolved.
    let missing: Vec<&str> = supplemental
        .iter()
        .copied()
        .filter(|&pkg| !versions.contains_key(&sanitize_key(pkg)))
        .collect();

    if !missing.is_empty() {
        let futures: Vec<_> = missing
            .iter()
            .map(|&pkg| {
                let client = client.clone();
                async move {
                    let m = fetch_meta(&client, pkg).await?;
                    Ok::<_, anyhow::Error>((pkg.to_string(), m.version))
                }
            })
            .collect();

        for result in futures::future::join_all(futures).await {
            let (pkg, version) = result?;
            versions.insert(sanitize_key(&pkg), Value::String(version));
        }
    }

    Ok(versions)
}
