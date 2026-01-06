use anyhow::{Context, Result};
use std::collections::HashSet;

use crate::{toml, workspace};

pub fn handle(deps: &[String], from: &str) -> Result<()> {
    workspace::ensure()?;

    let targets = workspace::resolve_targets(from)?;
    let mut still_used = HashSet::new();

    // 1. Remove deps from target crates
    for t in &targets {
        let path = format!("{}/Cargo.toml", t);
        let mut doc = toml::load(&path)?;

        if let Some(tbl) = doc["dependencies"].as_table_mut() {
            for dep in deps {
                tbl.remove(dep);
            }
        }

        toml::save(&path, &doc)?;
    }

    // 2. Detect remaining usage across workspace
    for member in workspace::members()? {
        let doc = toml::load(&format!("{}/Cargo.toml", member))?;
        if let Some(tbl) = doc["dependencies"].as_table() {
            for dep in deps {
                if tbl.contains_key(dep) {
                    still_used.insert(dep.clone());
                }
            }
        }
    }

    // 3. Remove from workspace.dependencies if unused
    let mut root = toml::load("Cargo.toml")?;
    let ws = root["workspace"]["dependencies"]
        .as_table_mut()
        .context("No workspace.dependencies")?;

    for dep in deps {
        if !still_used.contains(dep) {
            ws.remove(dep);
        }
    }

    toml::save("Cargo.toml", &root)?;
    Ok(())
}
