use anyhow::{Context, Result};
use toml_edit::{Item, Value};

use crate::{toml, workspace};

pub fn handle(to: &str) -> Result<()> {
    workspace::ensure()?;

    let targets = workspace::resolve_targets(to)?;
    let root = toml::load("Cargo.toml")?;
    let ws = root["workspace"]["dependencies"]
        .as_table()
        .context("No workspace.dependencies")?;

    for t in targets {
        let path = format!("{}/Cargo.toml", t);
        let mut doc = toml::load(&path)?;
        let deps_tbl = toml::deps_table(&mut doc);

        for dep in ws.iter().map(|(k, _)| k) {
            if deps_tbl.contains_key(dep) {
                deps_tbl[dep] = Item::Value(Value::from(true));
            }
        }

        toml::save(&path, &doc)?;
    }

    Ok(())
}
