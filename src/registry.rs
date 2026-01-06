use anyhow::{anyhow, bail, Result};
use crates_io_api::SyncClient;
use semver::{Version, VersionReq};
use std::time::Duration;

#[derive(Debug)]
pub struct ResolvedCrate {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}

/// Resolve crate name, version (support semver range), and validate features
pub fn resolve_crate(
    name: &str,
    version_req: Option<&str>,
    requested_features: &[String],
) -> Result<ResolvedCrate> {
    let client = SyncClient::new("cargo-setup", Duration::from_millis(500))?;

    let krate = client
        .get_crate(name)
        .map_err(|_| anyhow!("Crate '{}' not found on crates.io", name))?;

    let mut versions: Vec<Version> = krate
        .versions
        .iter()
        .filter(|v| !v.yanked)
        .filter_map(|v| Version::parse(&v.num).ok())
        .collect();

    if versions.is_empty() {
        bail!("No valid versions for '{}'", name);
    }

    versions.sort_by(|a, b| b.cmp(a));

    let resolved_version = match version_req {
        None => versions[0].clone(), // latest

        Some(raw) => {
            if raw.starts_with('=') {
                let v = &raw[1..];
                let ver = Version::parse(v).map_err(|_| anyhow!("Invalid version '{}'", v))?;

                if versions.contains(&ver) {
                    ver
                } else {
                    bail!("Version '{}' of '{}' not found on crates.io", v, name);
                }
            } else if is_exact_version(raw) {
                let ver = Version::parse(raw).map_err(|_| anyhow!("Invalid version '{}'", raw))?;

                if versions.contains(&ver) {
                    ver
                } else {
                    bail!("Version '{}' of '{}' not found on crates.io", raw, name);
                }
            } else {
                let range = normalize_range(raw)?;
                let req = VersionReq::parse(&range)
                    .map_err(|_| anyhow!("Invalid semver range '{}'", raw))?;

                versions
                    .into_iter()
                    .find(|v| req.matches(v))
                    .ok_or_else(|| {
                        anyhow!("No version of '{}' satisfies semver range '{}'", name, raw)
                    })?
            }
        }
    };

    let version_str = resolved_version.to_string();

    println!(
        "\x1b[36m[matching]\x1b[0m Semver matched '{}' -> {}",
        version_req.unwrap_or("latest"),
        version_str
    );

    let ver_info = krate
        .versions
        .iter()
        .find(|v| v.num == version_str)
        .ok_or_else(|| anyhow!("Version '{}' not found for '{}'", version_str, name))?;

    let available_features: Vec<String> = ver_info.features.keys().cloned().collect();

    for f in requested_features {
        if !available_features.contains(f) {
            bail!(
                "Feature '{}' not found in crate '{}' version {}",
                f,
                name,
                version_str
            );
        }
    }

    Ok(ResolvedCrate {
        name: name.to_string(),
        version: version_str,
        features: requested_features.to_vec(),
    })
}

fn is_exact_version(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

fn normalize_range(raw: &str) -> Result<String> {
    // if user already used operator
    if raw.starts_with('^') || raw.starts_with('~') || raw.starts_with('>') || raw.starts_with('<')
    {
        return Ok(raw.to_string());
    }

    let parts: Vec<&str> = raw.split('.').collect();

    if parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
        Ok(format!("^{}", raw))
    } else {
        bail!("Invalid version or range '{}'", raw);
    }
}
