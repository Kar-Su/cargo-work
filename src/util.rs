use anyhow::{bail, Result};
use std::process::Command;

pub fn exec_cmd(cmd: &[&str]) -> Result<()> {
    let status = Command::new(cmd[0]).args(&cmd[1..]).status()?;

    if !status.success() {
        bail!("Command failed: {:?}", cmd);
    }
    Ok(())
}
