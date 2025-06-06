use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use simplelog::LevelFilter;

mod extractcsvtables;
mod gui;

/// Experimental CLI version of travdata_cli written in Rust.
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Logging level.
    #[arg(long, default_value = "Warn")]
    log_level: LevelFilter,
}

#[derive(Subcommand)]
enum Command {
    ExtractCsvTables(extractcsvtables::Command),
    Gui(gui::Command),
}

impl Default for Command {
    fn default() -> Self {
        Self::Gui(gui::Command::default())
    }
}

pub fn run() -> Result<()> {
    let args = Args::parse();
    let xdg_dirs = xdg::BaseDirectories::with_prefix("travdata");

    simplelog::SimpleLogger::init(args.log_level, simplelog::Config::default())
        .with_context(|| "configuring logging")?;

    let cmd = args.command.unwrap_or_default();

    use Command::*;
    match &cmd {
        ExtractCsvTables(cmd) => extractcsvtables::run(cmd, xdg_dirs),
        Gui(cmd) => gui::run(cmd, xdg_dirs),
    }
}
