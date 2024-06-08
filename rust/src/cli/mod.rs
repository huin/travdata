use anyhow::Result;
use clap::{Parser, Subcommand};

mod extractcsvtables;
mod gui;

/// Experimental CLI version of travdata_cli written in Rust.
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ExtractCsvTables(extractcsvtables::Command),
    Gui,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    use Command::*;
    match &args.command {
        ExtractCsvTables(cmd) => extractcsvtables::run(cmd),
        Gui => gui::run(),
    }
}
