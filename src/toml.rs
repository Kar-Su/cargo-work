use anyhow::Result;
use std::fs;
use toml_edit::{DocumentMut, Item, Table};

pub fn load(path: &str) -> Result<DocumentMut> {
    Ok(fs::read_to_string(path)?.parse::<DocumentMut>()?)
}

pub fn save(path: &str, doc: &DocumentMut) -> Result<()> {
    fs::write(path, doc.to_string())?;
    Ok(())
}

pub fn deps_table(doc: &mut DocumentMut) -> &mut Table {
    doc["dependencies"]
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .unwrap()
}
