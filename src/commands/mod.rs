use crate::cli::{Cli, Commands};
use anyhow::Result;

pub mod add;
pub mod create;
pub mod list;
pub mod remove;
pub mod sync;

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.cmd {
        Commands::Create { project, lib, bin } => create::handle(&project, &lib, &bin),
        Commands::Add { deps, features, to } => add::handle(&deps, &features, &to),
        Commands::Remove { deps, from } => remove::handle(&deps, &from),
        Commands::List => list::handle(),
        Commands::Sync { to } => sync::handle(&to),
    }
}
