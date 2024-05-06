use std::path;

use anyhow::{Context, Result};
use clap::Args;

use crate::{
    config,
    extraction::{tableextract, tabulautil},
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
    config: path::PathBuf,

    /// Path to book config directory.
    book_dir: path::PathBuf,

    /// Path to input PDF.
    input_pdf: path::PathBuf,

    /// Path to Tabula JAR file.
    #[arg(long)]
    tabula_libpath: String,
}

/// Runs the subcommand.
pub fn run(cmd: &Command) -> Result<()> {
    let _cfg = config::root::load_config(&cmd.config)?;

    let tabula_client = tabulautil::TabulaClient::new(&cmd.tabula_libpath)
        .with_context(|| "initialising Tabula")?;

    let book_config_path = cmd.book_dir.join("book.yaml");
    let book_reader = std::fs::File::open(&book_config_path)
        .with_context(|| format!("opening book configuration {:?}", &book_config_path))?;
    let book: config::book::YamlGroup =
        serde_yaml_ng::from_reader(book_reader).with_context(|| "parsing book configuration")?;

    process_group(&tabula_client, &book, &cmd.book_dir, &cmd.input_pdf)
        .with_context(|| "processing book")?;

    Ok(())
}

fn process_group(
    tabula_client: &tabulautil::TabulaClient,
    grp: &config::book::YamlGroup,
    grp_path: &path::Path,
    input_pdf: &path::Path,
) -> Result<()> {
    for (table_name, table_cfg) in &grp.tables {
        let tmpl_path = grp_path
            .join(table_name)
            .with_extension("tabula-template.json");
        process_table(tabula_client, table_cfg, &tmpl_path, input_pdf)
            .with_context(|| format!("processing table {:?}", table_name))?;
    }

    for (child_grp_name, child_grp) in &grp.groups {
        let child_grp_path = grp_path.join(child_grp_name);
        process_group(tabula_client, child_grp, &child_grp_path, input_pdf)
            .with_context(|| format!("processing group {:?}", child_grp_name))?;
    }

    Ok(())
}

fn process_table(
    tabula_client: &tabulautil::TabulaClient,
    table_cfg: &config::book::YamlTable,
    tmpl_path: &path::Path,
    input_pdf: &path::Path,
) -> Result<()> {
    if !table_cfg.extraction_enabled {
        return Ok(());
    }

    let extracted_tables = tabula_client
        .read_pdf_with_template(input_pdf, tmpl_path)
        .with_context(|| format!("extracting table from PDF {:?}", input_pdf))?;
    let table = tableextract::concat_tables(extracted_tables.tables);
    let table = tableextract::apply_transforms(&table_cfg.extraction, table)?;

    for row in table.0 {
        println!("{:?}", row);
    }

    Ok(())
}
