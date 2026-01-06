use anyhow::Result;
use std::fs;
use toml_edit::{Array, DocumentMut, Item};
use which::which;

use crate::util::exec_cmd;

pub fn handle(project: &str, libs: &str, bins: &str) -> Result<()> {
    fs::create_dir(project)?;
    std::env::set_current_dir(project)?;

    let libs: Vec<&str> = libs.split(',').filter(|s| !s.is_empty()).collect();
    let bins: Vec<&str> = bins.split(',').filter(|s| !s.is_empty()).collect();

    for l in &libs {
        exec_cmd(&["cargo", "new", l, "--lib", "--vcs", "none"])?;
    }

    for b in &bins {
        exec_cmd(&["cargo", "new", b, "--vcs", "none"])?;
    }

    let mut doc = DocumentMut::new();
    let mut members = Array::new();

    for m in libs.iter().chain(bins.iter()) {
        members.push(*m);
    }

    doc["workspace"] = Item::Table(toml_edit::Table::new());

    doc["workspace"]["members"] = members.into();
    doc["workspace"]["resolver"] = "3".into();

    doc["workspace"]["dependencies"] = Item::Table(toml_edit::Table::new());

    fs::write("Cargo.toml", doc.to_string())?;

    if which("git").is_ok() {
        exec_cmd(&["git", "init"])?;
        fs::write(".gitignore", "/target\n**/target\nCargo.lock\n")?;
    }
    exec_cmd(&["cargo", "build"])?;

    Ok(())
}
