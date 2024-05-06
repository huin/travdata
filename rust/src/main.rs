use anyhow::{Context, Result};
use clap::Parser;
use std::{path, time::Instant};

mod config;
mod extraction;
mod table;

use extraction::tabulautil;

use crate::extraction::tableextract;

/// Experimental CLI version of travdata_cli written in Rust.
#[derive(Parser, Debug)]
struct Args {
    /// Path to Tabula JAR file.
    #[arg(long)]
    tabula_libpath: String,

    /// Path to book config directory.
    book_dir: path::PathBuf,

    /// Path to input PDF.
    input_pdf: path::PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let tabula_client = tabulautil::TabulaClient::new(&args.tabula_libpath)
        .with_context(|| "initialising Tabula")?;

    let book_config_path = args.book_dir.join("book.yaml");
    let book_reader = std::fs::File::open(&book_config_path)
        .with_context(|| format!("opening book configuration {:?}", &book_config_path))?;
    let book: config::YamlGroup =
        serde_yaml_ng::from_reader(book_reader).with_context(|| "parsing book configuration")?;

    process_group(&tabula_client, &book, &args.book_dir, &args.input_pdf)
        .with_context(|| "processing book")?;

    Ok(())
}

fn process_group(
    tabula_client: &tabulautil::TabulaClient,
    grp: &config::YamlGroup,
    grp_path: &path::Path,
    input_pdf: &path::Path,
) -> Result<()> {
    for (table_name, table_cfg) in &grp.tables {
        if table_name != "encounter-modifiers" {
            continue; // Remove this when finished experimenting.
        }
        let tmpl_path = grp_path
            .join(table_name)
            .with_extension("tabula-template.json");
        process_table(tabula_client, table_cfg, &tmpl_path, input_pdf)
            .with_context(|| format!("processing table {:?}", table_name))?;
    }

    for (child_grp_name, child_grp) in &grp.groups {
        if child_grp_name != "05-encounters-and-dangers" {
            continue; // Remove this when finished experimenting.
        }
        let child_grp_path = grp_path.join(child_grp_name);
        process_group(tabula_client, child_grp, &child_grp_path, input_pdf)
            .with_context(|| format!("processing group {:?}", child_grp_name))?;
    }

    Ok(())
}

fn process_table(
    tabula_client: &tabulautil::TabulaClient,
    table_cfg: &config::YamlTable,
    tmpl_path: &path::Path,
    input_pdf: &path::Path,
) -> Result<()> {
    if !table_cfg.extraction_enabled {
        return Ok(());
    }

    println!("Extraction config: {:?}", table_cfg);
    let now = Instant::now();
    let extracted_tables = tabula_client
        .read_pdf_with_template(input_pdf, tmpl_path)
        .with_context(|| format!("extracting table from PDF {:?}", input_pdf))?;
    let table = tableextract::concat_tables(extracted_tables.tables);
    let table = tableextract::apply_transforms(&table_cfg.extraction, table)?;

    println!("Rows:");
    for row in table.0 {
        println!("{:?}", row);
    }
    println!("Extracted in: {:?}", now.elapsed());

    Ok(())
}
