use anyhow::{Context, Result};
use reqwest::Client;
use semver::{Version, VersionReq};
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

/// All `@angular/*` packages that must share the same resolved version.
const ANGULAR_PACKAGES: &[&str] = &[
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
];

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
    v.contains('.').then_some(v)
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

    if let Some(av) = angular_version {
        for pkg in ANGULAR_PACKAGES {
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

    // angular-eslint is resolved above as a plain "latest" fetch, independent of the
    // @angular/cli version anchored from ng-lib above — the two can drift out of
    // compatibility (angular-eslint's peer range on @angular/cli chases newer Angular
    // releases faster than ng-lib does). ng-lib's own pin is the authoritative one here
    // (see the module doc comment), so angular-eslint is what gets capped down to match it,
    // not the other way round.
    if supplemental.contains(&"angular-eslint") {
        reconcile_tooling_version(&client, &mut versions, "angular-eslint", "@angular/cli").await?;
    }

    Ok(versions)
}

/// If `tooling_pkg`'s declared peer requirement on `anchor_pkg` isn't satisfied by the version
/// of `anchor_pkg` already sitting in `versions`, re-resolve `anchor_pkg` down to the highest
/// published version that *does* satisfy it, and apply that version to every package in
/// `co_versioned` (which must include `anchor_pkg` itself).
///
/// Best-effort: any missing data or unparseable range leaves `versions` untouched rather than
/// failing the scaffold outright.
async fn reconcile_peer_dependency(
    client: &Client,
    versions: &mut Map<String, Value>,
    tooling_pkg: &str,
    anchor_pkg: &str,
    co_versioned: &[&str],
) -> Result<()> {
    let anchor_key = sanitize_key(anchor_pkg);
    let Some(current) = versions
        .get(&anchor_key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
    else {
        return Ok(());
    };

    let tooling_meta = fetch_meta(client, tooling_pkg).await?;
    let Some(range) = tooling_meta
        .peer_dependencies
        .get(anchor_pkg)
        .and_then(|v| v.as_str())
    else {
        return Ok(());
    };

    if version_satisfies(&current, range) {
        return Ok(());
    }

    if let Some(resolved) = max_satisfying(client, anchor_pkg, range).await? {
        for pkg in co_versioned {
            versions.insert(sanitize_key(pkg), Value::String(resolved.clone()));
        }
    }

    Ok(())
}

/// Convert an npm-style range (space-separated comparators, e.g. `">=5.9 <7.0"` or
/// `">= 22.0.0 < 23.0.0"`) into a `semver::VersionReq`, which instead expects comparators
/// joined with commas.
fn npm_range_to_req(range: &str) -> Option<VersionReq> {
    let tokens: Vec<&str> = range.split_whitespace().collect();
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let t = tokens[i];
        if matches!(t, "<" | "<=" | ">" | ">=" | "=" | "^" | "~") {
            let next = tokens.get(i + 1)?;
            parts.push(format!("{t}{next}"));
            i += 2;
        } else {
            parts.push(t.to_string());
            i += 1;
        }
    }
    VersionReq::parse(&parts.join(", ")).ok()
}

/// Whether `version` satisfies the npm-style range `range`. Fails open (returns `true`) if
/// either side can't be parsed, so an unusual range never blocks scaffolding outright.
fn version_satisfies(version: &str, range: &str) -> bool {
    match (Version::parse(version), npm_range_to_req(range)) {
        (Ok(v), Some(req)) => req.matches(&v),
        _ => true,
    }
}

/// Highest published version of `package` on npm that satisfies the npm-style range `range`.
async fn max_satisfying(client: &Client, package: &str, range: &str) -> Result<Option<String>> {
    let Some(req) = npm_range_to_req(range) else {
        return Ok(None);
    };

    #[derive(Deserialize)]
    struct RegistryDoc {
        versions: Map<String, Value>,
    }

    let url = format!("https://registry.npmjs.org/{package}");
    let doc: RegistryDoc = client
        .get(&url)
        .header("Accept", "application/vnd.npm.install-v1+json")
        .send()
        .await
        .with_context(|| format!("failed to fetch npm metadata for '{package}'"))?
        .error_for_status()
        .with_context(|| format!("'{package}' not found on npm registry"))?
        .json()
        .await
        .with_context(|| format!("failed to parse npm metadata for '{package}'"))?;

    let mut matching: Vec<Version> = doc
        .versions
        .keys()
        .filter_map(|v| Version::parse(v).ok())
        .filter(|v| req.matches(v))
        .collect();
    matching.sort();

    Ok(matching.pop().map(|v| v.to_string()))
}

/// If the currently-resolved version of `tooling_pkg` (already in `versions`, e.g. resolved as
/// a plain "latest" fetch) declares a peer requirement on `anchor_pkg` that the already-resolved
/// (and authoritative — anchored to the m2s2 library) version of `anchor_pkg` doesn't satisfy,
/// re-resolve `tooling_pkg` down to the highest version whose *own* declared peer requirement on
/// `anchor_pkg` the anchor version does satisfy.
///
/// This is the mirror image of [`reconcile_peer_dependency`]: that function moves the anchor to
/// match the tooling package; this one moves the tooling package to match a fixed anchor. Use
/// this when the anchor's version is the authoritative one and shouldn't be disturbed (e.g. it
/// was pinned via the m2s2 library's own peer dependencies).
async fn reconcile_tooling_version(
    client: &Client,
    versions: &mut Map<String, Value>,
    tooling_pkg: &str,
    anchor_pkg: &str,
) -> Result<()> {
    let tooling_key = sanitize_key(tooling_pkg);
    let anchor_key = sanitize_key(anchor_pkg);

    let Some(anchor_version) = versions
        .get(&anchor_key)
        .and_then(|v| v.as_str())
        .map(str::to_string)
    else {
        return Ok(());
    };

    let tooling_meta = fetch_meta(client, tooling_pkg).await?;
    let already_ok = match tooling_meta
        .peer_dependencies
        .get(anchor_pkg)
        .and_then(|v| v.as_str())
    {
        Some(range) => version_satisfies(&anchor_version, range),
        None => true,
    };
    if already_ok {
        return Ok(());
    }

    if let Some(resolved) =
        max_satisfying_own_peer(client, tooling_pkg, anchor_pkg, &anchor_version).await?
    {
        versions.insert(tooling_key, Value::String(resolved));
    }

    Ok(())
}

/// Highest published version of `package` whose own declared `peerDependencies[peer_pkg]` range
/// is satisfied by `peer_version` (unlike [`max_satisfying`], which matches a single fixed range
/// against every version of a package, this inspects each version's *own* peer dependency
/// declaration, since that declaration changes release to release).
async fn max_satisfying_own_peer(
    client: &Client,
    package: &str,
    peer_pkg: &str,
    peer_version: &str,
) -> Result<Option<String>> {
    #[derive(Deserialize)]
    struct VersionEntry {
        #[serde(rename = "peerDependencies", default)]
        peer_dependencies: Map<String, Value>,
    }
    #[derive(Deserialize)]
    struct RegistryDoc {
        versions: std::collections::HashMap<String, VersionEntry>,
    }

    let url = format!("https://registry.npmjs.org/{package}");
    let doc: RegistryDoc = client
        .get(&url)
        .header("Accept", "application/vnd.npm.install-v1+json")
        .send()
        .await
        .with_context(|| format!("failed to fetch npm metadata for '{package}'"))?
        .error_for_status()
        .with_context(|| format!("'{package}' not found on npm registry"))?
        .json()
        .await
        .with_context(|| format!("failed to parse npm metadata for '{package}'"))?;

    let mut matching: Vec<Version> = doc
        .versions
        .into_iter()
        .filter_map(|(v_str, entry)| {
            let v = Version::parse(&v_str).ok()?;
            let range = entry.peer_dependencies.get(peer_pkg)?.as_str()?;
            version_satisfies(peer_version, range).then_some(v)
        })
        .collect();
    matching.sort();

    Ok(matching.pop().map(|v| v.to_string()))
}

async fn fetch_meta(client: &Client, package: &str) -> Result<NpmPackageMeta> {
    let url = format!("https://registry.npmjs.org/{package}/latest");
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

/// Fetch the latest version of each package directly, with no anchor library.
/// Used for backend runtimes (Node) that have no m2s2 peer dependency to derive versions from.
pub async fn resolve_packages(packages: &[&str]) -> Result<Map<String, Value>> {
    let client = Client::new();
    let futures: Vec<_> = packages
        .iter()
        .map(|&pkg| {
            let client = client.clone();
            async move {
                let m = fetch_meta(&client, pkg).await?;
                Ok::<_, anyhow::Error>((pkg.to_string(), m.version))
            }
        })
        .collect();

    let mut versions = Map::new();
    for result in futures::future::join_all(futures).await {
        let (pkg, version) = result?;
        versions.insert(sanitize_key(&pkg), Value::String(version));
    }

    // typescript and typescript-eslint are both resolved above as independent "latest"
    // fetches — typescript-eslint's support for a brand-new TypeScript major routinely lags
    // behind, so the pair can end up mutually uninstallable. Re-pin typescript down to
    // whatever typescript-eslint actually supports.
    if packages.contains(&"typescript") && packages.contains(&"typescript-eslint") {
        reconcile_peer_dependency(
            &client,
            &mut versions,
            "typescript-eslint",
            "typescript",
            &["typescript"],
        )
        .await?;
    }

    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_scoped_package() {
        assert_eq!(sanitize_key("@m2s2/react-lib"), "_m2s2_react_lib");
    }

    #[test]
    fn sanitize_types_package() {
        assert_eq!(sanitize_key("@types/react"), "_types_react");
    }

    #[test]
    fn sanitize_plain_package() {
        assert_eq!(sanitize_key("typescript"), "typescript");
    }

    #[test]
    fn sanitize_hyphenated_package() {
        assert_eq!(sanitize_key("sass-embedded"), "sass_embedded");
    }

    #[test]
    fn extract_caret_range() {
        assert_eq!(extract_version("^18.3.1"), Some("18.3.1".into()));
    }

    #[test]
    fn extract_tilde_range() {
        assert_eq!(extract_version("~5.9.3"), Some("5.9.3".into()));
    }

    #[test]
    fn extract_gte_range() {
        assert_eq!(extract_version(">=3.0.0"), Some("3.0.0".into()));
    }

    #[test]
    fn extract_exact_version() {
        assert_eq!(extract_version("1.2.3"), Some("1.2.3".into()));
    }

    #[test]
    fn extract_invalid_returns_none() {
        assert_eq!(extract_version("latest"), None);
    }

    #[test]
    fn extract_star_returns_none() {
        assert_eq!(extract_version("*"), None);
    }

    #[test]
    fn npm_range_no_space_before_version() {
        let req = npm_range_to_req(">=5.9 <7.0").unwrap();
        assert!(req.matches(&Version::parse("6.5.0").unwrap()));
        assert!(!req.matches(&Version::parse("7.0.0").unwrap()));
        assert!(!req.matches(&Version::parse("5.8.9").unwrap()));
    }

    #[test]
    fn npm_range_space_before_version() {
        let req = npm_range_to_req(">= 22.0.0 < 23.0.0").unwrap();
        assert!(req.matches(&Version::parse("22.1.0").unwrap()));
        assert!(!req.matches(&Version::parse("23.0.0").unwrap()));
    }

    #[test]
    fn npm_range_caret() {
        let req = npm_range_to_req("^8.0.0").unwrap();
        assert!(req.matches(&Version::parse("8.65.0").unwrap()));
        assert!(!req.matches(&Version::parse("9.0.0").unwrap()));
    }

    #[test]
    fn version_satisfies_within_range() {
        assert!(version_satisfies("6.5.0", ">=5.9 <7.0"));
        assert!(!version_satisfies("7.0.2", ">=5.9 <7.0"));
    }

    #[test]
    fn version_satisfies_fails_open_on_unparseable_input() {
        assert!(version_satisfies("not-a-version", ">=5.9 <7.0"));
        assert!(version_satisfies("6.5.0", "not-a-range"));
    }
}
