//! Process entry point: parse, run, render error, exit.

mod cli;
mod commands;
mod config;
mod error;
mod output;
mod resolve;
mod slack;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();
    if let Err(e) = commands::run(args) {
        eprintln!("error: {e}");
        std::process::exit(e.exit_code());
    }
}
