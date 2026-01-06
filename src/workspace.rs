use anyhow::{Context, Result};
use std::fs;
use toml_edit::DocumentMut;

pub fn ensure() -> Result<()> {
    if !std::path::Path::new("Cargo.toml").exists() {
        anyhow::bail!("Run inside workspace root");
    }
    Ok(())
}

pub fn members() -> Result<Vec<String>> {
    let doc: DocumentMut = fs::read_to_string("Cargo.toml")?.parse()?;
    let arr = doc["workspace"]["members"]
        .as_array()
        .context("No workspace.members")?;

    Ok(arr
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}

pub fn resolve_targets(input: &str) -> Result<Vec<String>> {
    if input == "all" {
        members()
    } else {
        Ok(input.split(',').map(String::from).collect())
    }
}
