use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo-work")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Create {
        project: String,
        #[arg(long)]
        lib: String,
        #[arg(long)]
        bin: String,
    },
    Add {
        deps: Vec<String>,
        #[arg(long)]
        features: Vec<String>,
        #[arg(long)]
        to: String,
    },
    Remove {
        deps: Vec<String>,
        #[arg(long)]
        from: String,
    },
    List,
    Sync {
        #[arg(long)]
        to: String,
    },
}
