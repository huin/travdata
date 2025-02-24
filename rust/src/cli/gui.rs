use anyhow::Result;
use clap::Args;

use crate::{
    extraction::pdf::TableReaderArgs,
    gui::{self},
};

/// Runs a GUI to perform table extractions from PDF files.
#[derive(Args, Debug, Default)]
pub struct Command {
    /// Options relating to configuring the table reader.
    #[command(flatten)]
    table_reader: TableReaderArgs,

    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    gtk_options: Vec<String>,
}

pub fn run(cmd: &Command, xdg_dirs: xdg::BaseDirectories) -> Result<()> {
    let table_reader = cmd.table_reader.build(&xdg_dirs)?;

    gui::run(table_reader, &cmd.gtk_options, xdg_dirs)
}
