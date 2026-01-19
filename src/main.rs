use clap::Parser;
use cli::Cli;

mod cli;
mod commands;
mod registry;
mod toml;
mod util;
mod workspace;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let cli = if args.len() > 1 && args[1] == "work" {
        Cli::parse_from(std::iter::once(args[0].clone()).chain(args.into_iter().skip(2)))
    } else {
        Cli::parse()
    };

    if let Err(_) = commands::dispatch(cli) {
        eprintln!("\x1b[31mABORT PROGRAM\x1b[0m");
        std::process::exit(1);
    }
}
