use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_yaml_ng::with;
use simple_bar::ProgressBar;

use crate::{
    config::{
        self,
        root::{load_config, Config},
    },
    extraction::tabulautil,
    filesio::{ReadWriter, Reader},
};

use super::{index::IndexWriter, tableextract::extract_table};

/// Encapsulates the values required to extract tables from book(s).
pub struct Extractor<'a> {
    tabula_client: tabulautil::TabulaClient,
    cfg: Config,
    cfg_reader: Box<dyn Reader<'a>>,
    out_writer: Box<dyn ReadWriter<'a>>,
    index_writer: IndexWriter<'a>,
}

/// Specifies a book's tables to be extracted by [Extractor::extract_book].
pub struct ExtractSpec<'a> {
    pub book_name: &'a str,
    pub input_pdf: &'a Path,
    pub overwrite_existing: bool,
    pub with_tags: &'a [String],
    pub without_tags: &'a [String],
}

impl<'a> Extractor<'a> {
    /// Create a new [Extractor].
    pub fn new(
        tabula_client: tabulautil::TabulaClient,
        cfg_reader: Box<dyn Reader<'a>>,
        out_writer: Box<dyn ReadWriter<'a>>,
    ) -> Result<Self> {
        let cfg = load_config(cfg_reader.as_ref()).with_context(|| "loading configuration")?;

        let index_writer =
            IndexWriter::new(out_writer.as_ref()).with_context(|| "opening index for update")?;

        Ok(Self {
            tabula_client,
            cfg,
            cfg_reader,
            out_writer,
            index_writer,
        })
    }

    /// Extracts tables from a single book.
    pub fn extract_book(&mut self, spec: ExtractSpec) -> Result<()> {
        let book_cfg = self.cfg.books.get(spec.book_name).ok_or_else(|| {
            anyhow!(
                "book {:?} does not exist in the configuration",
                spec.book_name
            )
        })?;

        let top_group = book_cfg.load_group(self.cfg_reader.as_ref())?;

        let output_tables: Vec<OutputTable<'_>> = top_group
            .iter_tables()
            .filter(|&table_cfg| {
                spec.with_tags
                    .iter()
                    .any(|with_tag| table_cfg.tags.contains(with_tag))
                    && !spec
                        .without_tags
                        .iter()
                        .any(|without_tag| table_cfg.tags.contains(without_tag))
            })
            .map(OutputTable::from_table_cfg)
            .filter(|out_table| {
                spec.overwrite_existing || !self.out_writer.exists(&out_table.out_filepath)
            })
            .collect();

        let mut progress_bar = ProgressBar::cargo_style(output_tables.len() as u32, 80, true);

        for out_table in &output_tables {
            extract_table(
                &self.tabula_client,
                self.cfg_reader.as_ref(),
                self.out_writer.as_ref(),
                &mut self.index_writer,
                book_cfg,
                out_table.table_cfg,
                spec.input_pdf,
            )
            .with_context(|| format!("processing table {:?}", out_table.out_filepath))?;

            progress_bar.update();
        }

        Ok(())
    }

    /// Completes any extractions performed. Any extracted data may or may not
    /// be complete if this is not called.
    pub fn close(self) -> Result<()> {
        self.index_writer
            .commit()
            .with_context(|| "commiting changes to the index")?;

        self.out_writer
            .close()
            .with_context(|| "closing out written files")
    }
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
