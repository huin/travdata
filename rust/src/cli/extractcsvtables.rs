use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::{
    extraction::{bookextract::Extractor, tabulautil},
    filesio,
};

/// Extracts data tables from the Mongoose Traveller 2022 core rules PDF as CSV
/// files.
#[derive(Args, Debug)]
pub struct Command {
    /// Path to the configuration.
    book_name: String,

    /// Path to input PDF.
    input_pdf: PathBuf,

    /// Path to the directory or ZIP file to output the CSV files into.
    ///
    /// Whether this is a directory or ZIP file is controlled by --output-type.
    output: PathBuf,

    /// Path to the configuration. This must be either a directory or ZIP file,
    /// directly containing a config.yaml file, book.yaml files in directories,
    /// and its required Tabula templates. Some configurations for this should
    /// be included with this program's distribution.
    #[arg(long)]
    config: PathBuf,

    /// Path to Tabula JAR file.
    #[arg(long)]
    tabula_libpath: String,

    /// Controls how data is written to the output.
    ///
    /// By default, it guesses, based on any existing file or directory at the
    /// path or the path suffix ending in ".zip".
    #[arg(long)]
    output_type: Option<crate::filesio::IoType>,

    /// Extract CSV tables that already exist in the output. This is useful when
    /// testing larger scale changes to the configuration or code.
    #[arg(long)]
    overwrite_existing: bool,
}

/// Runs the subcommand.
pub fn run(cmd: &Command) -> Result<()> {
    let tabula_client = tabulautil::TabulaClient::new(&cmd.tabula_libpath)
        .with_context(|| "initialising Tabula")?;

    let cfg_type = filesio::IoType::resolve_auto(None, &cmd.config);
    let cfg_reader = cfg_type
        .new_reader(&cmd.config)
        .with_context(|| format!("opening config path {:?} as {:?}", cmd.config, cfg_type))?;

    let output_type = filesio::IoType::resolve_auto(cmd.output_type, &cmd.output);
    let out_writer = output_type
        .new_read_writer(&cmd.output)
        .with_context(|| format!("opening output path {:?} as {:?}", cmd.output, output_type))?;

    let mut extractor = Extractor::new(tabula_client, cfg_reader, out_writer)?;

    extractor.extract_book(&cmd.book_name, &cmd.input_pdf, cmd.overwrite_existing)?;

    extractor.close()
}
