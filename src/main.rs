use clap::Parser;

use crate::cli::{Cli, cli_init};
mod cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli_init(&cli) {
        eprintln!("Application Error: {e}");
        std::process::exit(1);
    }
}
