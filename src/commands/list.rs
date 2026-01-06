use anyhow::{anyhow, Result};
use std::fs;
use toml_edit::DocumentMut;

pub fn handle() -> Result<()> {
    let root = load_toml("Cargo.toml")?;

    let ws_deps = root["workspace"]["dependencies"]
        .as_table()
        .ok_or_else(|| anyhow!("[workspace.dependencies] not found"))?;

    let members = root["workspace"]["members"]
        .as_array()
        .ok_or_else(|| anyhow!("[workspace.members] not found"))?
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();

    println!("workspace dependencies:");

    for (name, item) in ws_deps.iter() {
        let (version, features) = match item.as_value().and_then(|v| v.as_inline_table()) {
            Some(it) => {
                let ver = it.get("version").and_then(|v| v.as_str()).unwrap_or("-");

                let feats = it
                    .get("features")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(String::from)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                (ver.to_string(), feats)
            }
            None => ("-".to_string(), vec![]),
        };

        let mut used_by = Vec::new();

        for m in &members {
            let path = format!("{}/Cargo.toml", m);
            let doc = match fs::read_to_string(&path) {
                Ok(c) => c.parse::<DocumentMut>()?,
                Err(_) => continue,
            };

            let deps = doc["dependencies"].as_table();
            if let Some(deps) = deps {
                if let Some(dep) = deps.get(name) {
                    if dep
                        .as_value()
                        .and_then(|v| v.as_inline_table())
                        .and_then(|it| it.get("workspace"))
                        .and_then(|v| v.as_bool())
                        == Some(true)
                    {
                        used_by.push(m.to_string());
                    }
                }
            }
        }

        let feat_str = if features.is_empty() {
            "-".to_string()
        } else {
            format!("[{}]", features.join(", "))
        };

        println!(
            "- {:<8} {:<12} {:<30} → {}",
            name,
            version,
            feat_str,
            used_by.join(", ")
        );
    }

    Ok(())
}

fn load_toml(path: &str) -> Result<DocumentMut> {
    let content = fs::read_to_string(path)?;
    Ok(content.parse::<DocumentMut>()?)
}
