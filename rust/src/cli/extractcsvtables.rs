use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Args;

use crate::{
    config::{self, root::load_config},
    extraction::{tableextract, tabulautil},
    filesio::{self, DirReadWriter, ReadWriter, Reader},
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

    run_impl(
        &tabula_client,
        cfg_reader.as_ref(),
        out_writer,
        &cmd.input_pdf,
        &cmd.book_name,
    )
}

fn run_impl(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    out_writer: Box<dyn ReadWriter>,
    input_pdf: &Path,
    book_name: &str,
) -> Result<()> {
    let cfg = load_config(cfg_reader)?;
    let book = cfg
        .books
        .get(book_name)
        .ok_or_else(|| anyhow!("book {:?} does not exist in the configuration", book_name))?
        .load_group(cfg_reader)?;

    process_group(
        &tabula_client,
        cfg_reader,
        out_writer.as_ref(),
        &book,
        input_pdf,
    )
    .with_context(|| "processing book")?;

    out_writer.close()?;

    Ok(())
}

fn process_group(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    grp: &config::book::Group,
    input_pdf: &Path,
) -> Result<()> {
    for (table_name, table_cfg) in &grp.tables {
        process_table(tabula_client, cfg_reader, out_writer, table_cfg, input_pdf)
            .with_context(|| format!("processing table {:?}", table_name))?;
    }

    for (child_grp_name, child_grp) in &grp.groups {
        process_group(tabula_client, cfg_reader, out_writer, child_grp, input_pdf)
            .with_context(|| format!("processing group {:?}", child_grp_name))?;
    }

    Ok(())
}

fn process_table(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    table_cfg: &config::book::Table,
    input_pdf: &Path,
) -> Result<()> {
    if !table_cfg.extraction_enabled {
        return Ok(());
    }

    let mut csv_file = out_writer.open_write(&table_cfg.file_stem.with_extension("csv"))?;
    let mut csv_writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(&mut csv_file);

    let tmpl_path = table_cfg.tabula_template_path();

    let extracted_tables = tabula_client
        .read_pdf_with_template(cfg_reader, input_pdf, &tmpl_path)
        .with_context(|| format!("extracting table from PDF {:?}", input_pdf))?;
    let table = tableextract::concat_tables(extracted_tables.tables);
    let table = tableextract::apply_transforms(&table_cfg.extraction, table)?;

    for row in table.0 {
        csv_writer
            .write_record(&row.0)
            .with_context(|| "writing record")?;
    }

    // Check for error rather than implicitly flushing and ignoring.
    csv_writer.flush().with_context(|| "flushing to CSV")?;
    drop(csv_writer);
    csv_file.commit().with_context(|| "committing CSV file")?;

    Ok(())
}
