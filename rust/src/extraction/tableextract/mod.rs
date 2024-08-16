//! Extracts a single table from a PDF.

mod estransform;
pub mod groupers;
mod internal;
pub mod legacy_transform;

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config;
use crate::extraction::parseutil::clean_text;
use crate::filesio::{ReadWriter, Reader};
use crate::table::Table;

use super::index::IndexWriter;
use super::tabulautil;

#[derive(Deserialize, Debug)]
/// Configures transformation of the raw data from Tabula into the output structured data.
pub enum TableTransform {
    LegacyTransformSeq(LegacyTransformSeq),
    ESTransform(ESTransform),
}

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
/// Configures the specifics of extracting the CSV from the PDF.
pub struct LegacyTransformSeq {
    pub transforms: Vec<legacy_transform::TableTransform>,
}

#[derive(Deserialize, Debug)]
/// ECMAScript based table transformation.
pub struct ESTransform {
    pub src: String,
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
    if table_cfg.disable_extraction {
        return Ok(());
    }

    let tmpl_path = table_cfg.tabula_template_path();

    let extracted_tables = tabula_client
        .read_pdf_with_template(cfg_reader, input_pdf, &tmpl_path)
        .with_context(|| format!("extracting table from PDF {:?}", input_pdf))?;

    let mut table = match &table_cfg.transform {
        None => concat_tables(extracted_tables.tables),
        Some(TableTransform::LegacyTransformSeq(legacy_transform)) => {
            let table = concat_tables(extracted_tables.tables);
            legacy_transform::apply_transforms(&legacy_transform.transforms, table)?
        }
        Some(TableTransform::ESTransform(es_transform)) => {
            todo!("XXX")
        }
    };

    clean_table(&mut table);

    let csv_path = table_cfg.file_stem.with_extension("csv");
    let mut csv_file = out_writer.open_write(&csv_path)?;

    table.write_csv(&mut csv_file)?;

    // Check for error rather than implicitly flushing and ignoring.
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
