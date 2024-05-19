use std::{
    io::stdout,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use clap::Args;

use crate::{
    config::{self, book::load_book, root::load_config},
    extraction::{tableextract, tabulautil},
    filesio::{DirReadWriter, Reader},
};

/// Extracts data tables from the Mongoose Traveller 2022 core rules PDF as CSV
/// files.
#[derive(Args, Debug)]
pub struct Command {
    /// Path to the configuration. This must be either a directory or ZIP file,
    /// directly containing a config.yaml file, book.yaml files in directories,
    /// and its required Tabula templates. Some configurations for this should
    /// be included with this program's distribution.
    ///
    /// TODO: Support reading from ZIP file as well.
    #[arg(long)]
    config: PathBuf,

    /// Path to the configuration.
    book_name: String,

    /// Path to input PDF.
    input_pdf: PathBuf,

    /// Path to Tabula JAR file.
    #[arg(long)]
    tabula_libpath: String,
}

/// Runs the subcommand.
pub fn run(cmd: &Command) -> Result<()> {
    let tabula_client = tabulautil::TabulaClient::new(&cmd.tabula_libpath)
        .with_context(|| "initialising Tabula")?;

    let cfg_reader = DirReadWriter::new(&cmd.config);

    let cfg = load_config(&cfg_reader)?;
    let book = cfg
        .books
        .get(&cmd.book_name)
        .ok_or_else(|| {
            anyhow!(
                "book {:?} does not exist in the configuration",
                cmd.book_name
            )
        })?
        .load_group(&cfg_reader)?;

    process_group(
        &tabula_client,
        &cfg_reader,
        &book,
        &cmd.config,
        &cmd.input_pdf,
    )
    .with_context(|| "processing book")?;

    Ok(())
}

fn process_group(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    grp: &config::book::Group,
    grp_path: &Path,
    input_pdf: &Path,
) -> Result<()> {
    for (table_name, table_cfg) in &grp.tables {
        process_table(tabula_client, cfg_reader, table_cfg, input_pdf)
            .with_context(|| format!("processing table {:?}", table_name))?;
    }

    for (child_grp_name, child_grp) in &grp.groups {
        let child_grp_path = grp_path.join(child_grp_name);
        process_group(
            tabula_client,
            cfg_reader,
            child_grp,
            &child_grp_path,
            input_pdf,
        )
        .with_context(|| format!("processing group {:?}", child_grp_name))?;
    }

    Ok(())
}

fn process_table(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    table_cfg: &config::book::Table,
    input_pdf: &Path,
) -> Result<()> {
    if !table_cfg.extraction_enabled {
        return Ok(());
    }

    let mut csv_writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(stdout());

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

    Ok(())
}
