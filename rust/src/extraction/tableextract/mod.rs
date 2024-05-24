//! Extracts a single table from a PDF.

pub mod groupers;
mod internal;
pub mod transform;

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config;
use crate::extraction::parseutil::clean_text;
use crate::filesio::{ReadWriter, Reader};
use crate::table::Table;

use super::index::IndexWriter;
use super::tabulautil;

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
/// Configures the specifics of extracting the CSV from the PDF.
pub struct TableExtraction {
    pub transforms: Vec<transform::TableTransform>,
}

/// Extracts a single table into a CSV file.
pub fn extract_table(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    index_writer: &mut IndexWriter,
    book_cfg: &config::root::Book,
    table_cfg: &config::book::Table,
    input_pdf: &Path,
) -> Result<()> {
    if !table_cfg.extraction_enabled {
        return Ok(());
    }

    let csv_path = table_cfg.file_stem.with_extension("csv");
    let mut csv_file = out_writer.open_write(&csv_path)?;
    let mut csv_writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(&mut csv_file);

    let tmpl_path = table_cfg.tabula_template_path();

    let extracted_tables = tabula_client
        .read_pdf_with_template(cfg_reader, input_pdf, &tmpl_path)
        .with_context(|| format!("extracting table from PDF {:?}", input_pdf))?;

    let table = concat_tables(extracted_tables.tables);
    let mut table = transform::apply_transforms(&table_cfg.extraction.transforms, table)?;

    clean_table(&mut table);

    for row in table.0 {
        csv_writer
            .write_record(&row.0)
            .with_context(|| "writing record")?;
    }

    // Check for error rather than implicitly flushing and ignoring.
    csv_writer.flush().with_context(|| "flushing to CSV")?;
    drop(csv_writer);
    csv_file.commit().with_context(|| "committing CSV file")?;

    let page_numbers: Vec<i32> = extracted_tables.source_pages.into_iter().collect();
    index_writer.add_entry(csv_path, book_cfg, table_cfg, &page_numbers);

    Ok(())
}

/// Concatenates the given tables into a single `Table`.
fn concat_tables(tables: Vec<Table>) -> Table {
    Table(
        tables
            .into_iter()
            .flat_map(|table| table.0.into_iter())
            .collect(),
    )
}

/// Clean leading, trailing, and redundant sequences of whitespace within the
/// `Table`, in-place.
fn clean_table(table: &mut Table) {
    for row in table.iter_mut() {
        for cell in row.iter_mut() {
            clean_text(cell);
        }
    }
}
