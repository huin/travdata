use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use simplelog::LevelFilter;

mod extractcsvtables;
mod gui;

/// Experimental CLI version of travdata_cli written in Rust.
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Logging level.
    #[arg(long, default_value = "Warn")]
    log_level: LevelFilter,
}

#[derive(Subcommand)]
enum Command {
    ExtractCsvTables(extractcsvtables::Command),
    Gui,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    simplelog::SimpleLogger::init(args.log_level, simplelog::Config::default())
        .with_context(|| "configuring logging")?;

    use Command::*;
    match &args.command {
        ExtractCsvTables(cmd) => extractcsvtables::run(cmd),
        Gui => gui::run(),
    }
}
