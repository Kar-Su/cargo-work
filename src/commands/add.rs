use anyhow::{anyhow, bail, Result};
use std::{collections::HashMap, fs, path::Path};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};

use crate::registry::{resolve_crate, ResolvedCrate};

fn log_adding(msg: &str) {
    println!("\x1b[32m[adding]\x1b[0m {}", msg);
}

fn log_matching(msg: &str) {
    println!("\x1b[36m[matching]\x1b[0m {}", msg);
}

fn log_error(msg: &str) {
    eprintln!("\x1b[31m[error]\x1b[0m {}", msg);
}

pub fn handle(deps: &[String], raw_features: &[String], to: &str) -> Result<()> {
    log_adding("Starting dependency add process");

    let targets = resolve_targets(to)?;
    log_matching(&format!("Target crates resolved: {}", targets.join(", ")));

    log_matching("Parsing feature definitions");
    let feature_map = parse_features_bracket(raw_features, deps)?;
    log_matching(&format!(
        "Feature map built for {} crate(s)",
        feature_map.len()
    ));

    let mut resolved = Vec::new();
    for dep in deps {
        let (name, version_req) = parse_dep_spec(dep);

        log_matching(&format!(
            "Resolving crate '{}' with version spec '{}'",
            name,
            version_req.unwrap_or("latest")
        ));

        let requested_features = feature_map.get(name).cloned().unwrap_or_default();

        let krate = resolve_crate(name, version_req, &requested_features).map_err(|e| {
            log_error(&e.to_string());
            e
        })?;

        log_matching(&format!(
            "Matched crate '{}' -> version {}",
            krate.name, krate.version
        ));

        if !krate.features.is_empty() {
            log_matching(&format!(
                "Validated features for '{}': {}",
                krate.name,
                krate.features.join(", ")
            ));
        }

        resolved.push(krate);
    }

    log_adding("Updating [workspace.dependencies]");
    update_workspace_dependencies(&resolved)?;

    for t in &targets {
        log_adding(&format!("Adding dependencies to crate '{}'", t));
    }
    update_target_crates(&resolved, &targets)?;

    log_adding("Dependency add process completed successfully");
    Ok(())
}

fn resolve_targets(to: &str) -> Result<Vec<String>> {
    if to == "all" {
        return workspace_members();
    }

    let targets: Vec<String> = to
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    if targets.is_empty() {
        bail!("No target crates specified");
    }

    if targets.iter().any(|t| t == "all") {
        bail!("'all' cannot be combined with specific crate names");
    }

    Ok(targets)
}

fn workspace_members() -> Result<Vec<String>> {
    let doc = load_toml("Cargo.toml")?;
    let members = doc["workspace"]["members"]
        .as_array()
        .ok_or_else(|| anyhow!("workspace.members not found"))?;

    Ok(members
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}

pub fn parse_features_bracket(
    raw: &[String],
    deps: &[String],
) -> Result<HashMap<String, Vec<String>>> {
    let input = raw.join(" ");
    let mut chars = input.chars().peekable();

    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    while let Some(c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        let mut lib = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == '[' {
                break;
            }
            lib.push(ch);
            chars.next();
        }

        let lib = lib.trim().to_string();
        if lib.is_empty() {
            bail!("Expected library name before '['");
        }

        let base = lib.split('@').next().unwrap();

        if !deps.iter().any(|d| d.split('@').next().unwrap() == base) {
            bail!("Feature specified for unknown dependency '{}'", base);
        }

        if chars.next() != Some('[') {
            bail!("Expected '[' after '{}'", base);
        }

        let mut feats = String::new();
        while let Some(ch) = chars.next() {
            if ch == ']' {
                break;
            }
            feats.push(ch);
        }

        let features: Vec<String> = feats
            .split(',')
            .map(|f| f.trim())
            .filter(|f| !f.is_empty())
            .map(String::from)
            .collect();

        if features.is_empty() {
            bail!("No features specified for '{}'", base);
        }

        map.entry(base.to_string())
            .and_modify(|v| v.extend(features.clone()))
            .or_insert(features);
    }

    Ok(map)
}

fn parse_dep_spec(s: &str) -> (&str, Option<&str>) {
    match s.split_once('@') {
        Some((name, ver)) => (name, Some(ver)),
        None => (s, None),
    }
}

fn update_workspace_dependencies(crates: &[ResolvedCrate]) -> Result<()> {
    let mut doc = load_toml("Cargo.toml")?;

    let deps_table = doc["workspace"]["dependencies"]
        .as_table_mut()
        .ok_or_else(|| anyhow!("[workspace.dependencies] must exist (init in create)"))?;

    for krate in crates {
        let mut it = InlineTable::new();
        it.insert("version", Value::from(krate.version.clone()));

        if !krate.features.is_empty() {
            let mut arr = Array::new();
            for f in &krate.features {
                arr.push(Value::from(f.clone()));
            }
            it.insert("features", Value::Array(arr));
        }

        deps_table.insert(&krate.name, Item::Value(Value::InlineTable(it)));
    }

    save_toml("Cargo.toml", &doc)?;
    Ok(())
}

fn update_target_crates(crates: &[ResolvedCrate], targets: &[String]) -> Result<()> {
    for crate_name in targets {
        let path = Path::new(crate_name).join("Cargo.toml");

        if !path.exists() {
            bail!("Target crate '{}' not found", crate_name);
        }

        let mut doc = load_toml(path.to_str().unwrap())?;

        let deps = doc["dependencies"]
            .or_insert(Item::Table(Table::new()))
            .as_table_mut()
            .unwrap();

        for krate in crates {
            if deps.get(&krate.name).is_some() {
                continue;
            }

            let mut it = InlineTable::new();
            it.insert("workspace", Value::from(true));

            deps.insert(&krate.name, Item::Value(Value::InlineTable(it)));
        }

        save_toml(path.to_str().unwrap(), &doc)?;
    }

    Ok(())
}

fn load_toml(path: &str) -> Result<DocumentMut> {
    let content = fs::read_to_string(path)?;
    Ok(content.parse::<DocumentMut>()?)
}

fn save_toml(path: &str, doc: &DocumentMut) -> Result<()> {
    fs::write(path, doc.to_string())?;
    Ok(())
}
