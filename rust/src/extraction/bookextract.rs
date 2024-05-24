use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use simple_bar::ProgressBar;

use crate::{
    config::{self, root::Config},
    extraction::tabulautil,
    filesio::{ReadWriter, Reader},
};

use super::{index::IndexWriter, tableextract::extract_table};

pub fn extract_book(
    tabula_client: &tabulautil::TabulaClient,
    cfg: &Config,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    book_name: &str,
    input_pdf: &Path,
) -> Result<()> {
    let book_cfg = cfg
        .books
        .get(book_name)
        .ok_or_else(|| anyhow!("book {:?} does not exist in the configuration", book_name))?;

    let top_group = book_cfg.load_group(cfg_reader)?;

    let mut index_writer =
        IndexWriter::new(out_writer).with_context(|| "opening index for update")?;

    let output_tables: Vec<OutputTable<'_>> = top_group.iter_tables()
        .map(OutputTable::from_table_cfg)
        .filter(|out_table| !out_writer.exists(&out_table.out_filepath))
        .collect();

    let mut progress_bar = ProgressBar::cargo_style(output_tables.len() as u32, 80, true);

    for out_table in &output_tables {
        extract_table(
            tabula_client,
            cfg_reader,
            out_writer,
            &mut index_writer,
            book_cfg,
            out_table.table_cfg,
            input_pdf,
        )
        .with_context(|| format!("processing table {:?}", out_table.out_filepath))?;

        progress_bar.update();
    }

    index_writer
        .commit()
        .with_context(|| "commiting changes to the index")?;

    Ok(())
}

struct OutputTable<'cfg> {
    out_filepath: PathBuf,
    table_cfg: &'cfg config::book::Table,
}

impl<'cfg> OutputTable<'cfg> {
    fn from_table_cfg(table_cfg: &'cfg config::book::Table) -> Self {
        Self {
            out_filepath: table_cfg.file_stem.with_extension("csv"),
            table_cfg,
        }
    }
}
