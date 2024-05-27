use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

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

/// Trait to implement to receive notifications about extraction events, or to
/// cancel extraction early.
pub trait ExtractEvents {
    fn on_progress(&mut self, completed: usize, total: usize);
    fn on_output(&mut self, path: &Path);
    fn on_error(&mut self, err: anyhow::Error);
    fn on_end(&mut self);
    fn do_continue(&self) -> bool;
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
    pub fn extract_book(&mut self, spec: ExtractSpec, events: &mut dyn ExtractEvents) {
        let book_cfg = match self.cfg.books.get(spec.book_name) {
            Some(book_cfg) => book_cfg,
            None => {
                events.on_error(anyhow!(
                    "book {:?} does not exist in the configuration",
                    spec.book_name
                ));
                events.on_end();
                return;
            }
        };

        let top_group = match book_cfg.load_group(self.cfg_reader.as_ref()) {
            Ok(top_group) => top_group,
            Err(err) => {
                events.on_error(err);
                events.on_end();
                return;
            }
        };

        let output_tables: Vec<OutputTable<'_>> = top_group
            .iter_tables()
            .filter(|&table_cfg| {
                spec.with_tags.is_empty()
                    || spec
                        .with_tags
                        .iter()
                        .any(|with_tag| table_cfg.tags.contains(with_tag))
            })
            .filter(|&table_cfg| {
                spec.without_tags.is_empty()
                    || !spec
                        .without_tags
                        .iter()
                        .any(|without_tag| table_cfg.tags.contains(without_tag))
            })
            .map(OutputTable::from_table_cfg)
            .filter(|out_table| {
                spec.overwrite_existing || !self.out_writer.exists(&out_table.out_filepath)
            })
            .collect();

        for (i, out_table) in output_tables.iter().enumerate() {
            let extract_result = extract_table(
                &self.tabula_client,
                self.cfg_reader.as_ref(),
                self.out_writer.as_ref(),
                &mut self.index_writer,
                book_cfg,
                out_table.table_cfg,
                spec.input_pdf,
            )
            .with_context(|| format!("processing table {:?}", out_table.out_filepath));
            if let Err(err) = extract_result {
                events.on_error(err);
            }

            events.on_progress(i + 1, output_tables.len());
            if !events.do_continue() {
                break;
            }
        }

        events.on_end();
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
