use std::{
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};

use crate::{
    config::{
        self,
        root::{load_config, Config},
    },
    extraction::pdf::tabulareader,
    filesio::{ReadWriter, Reader},
    table::Table,
};

use super::{
    index::IndexWriter,
    pdf::TableReader,
    tableextract::{
        estransform::{ESScript, ESScriptOrigin, ESTransformer, TransformFn},
        legacy_transform, TableTransform,
    },
};

/// Encapsulates the values required to extract tables from book(s).
pub struct Extractor<'a> {
    tabula_client: tabulareader::TabulaClient,
    estrn: ESTransformer,
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
    fn on_progress(&mut self, path: &Path, completed: usize, total: usize);
    fn on_error(&mut self, err: anyhow::Error);
    fn on_end(&mut self);
    fn do_continue(&self) -> bool;
}

impl<'a> Extractor<'a> {
    /// Create a new [Extractor].
    pub fn new(
        tabula_client: tabulareader::TabulaClient,
        cfg_reader: Box<dyn Reader<'a>>,
        out_writer: Box<dyn ReadWriter<'a>>,
    ) -> Result<Self> {
        let cfg = load_config(cfg_reader.as_ref()).with_context(|| "loading configuration")?;

        let index_writer =
            IndexWriter::new(out_writer.as_ref()).with_context(|| "opening index for update")?;

        let mut estrn = ESTransformer::new();
        run_ecma_scripts(cfg_reader.as_ref(), &mut estrn, &cfg.ecma_script_modules)?;

        Ok(Self {
            tabula_client,
            estrn,
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

        if let Err(err) = run_ecma_scripts(
            self.cfg_reader.as_ref(),
            &mut self.estrn,
            &book_cfg.ecma_script_modules,
        ) {
            events.on_error(err.context("running ECMA scripts for book"));
            events.on_end();
            return;
        }

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
            .filter(|&table_cfg| !table_cfg.disable_extraction)
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
            let extract_result = self
                .extract_table(out_table.table_cfg, spec.input_pdf)
                .with_context(|| format!("processing table {:?}", out_table.out_filepath));

            match extract_result {
                Err(err) => {
                    events.on_error(err);
                }
                Ok((table, mut page_numbers)) => {
                    page_numbers
                        .iter_mut()
                        .for_each(|page_number| *page_number += book_cfg.page_offset);

                    let write_result = Self::write_table(
                        self.out_writer.as_ref(),
                        &mut self.index_writer,
                        out_table,
                        table,
                        page_numbers,
                    );
                    if let Err(err) = write_result {
                        events.on_error(err);
                    }
                }
            }

            events.on_progress(&out_table.out_filepath, i + 1, output_tables.len());
            if !events.do_continue() {
                break;
            }
        }

        events.on_end();
    }

    fn write_table(
        out_writer: &dyn ReadWriter<'a>,
        index_writer: &mut IndexWriter,
        out_table: &OutputTable,
        table: Table,
        page_numbers: Vec<i32>,
    ) -> Result<()> {
        let csv_path = out_table.table_cfg.file_stem.with_extension("csv");
        let mut csv_file = out_writer.open_write(&csv_path)?;

        table.write_csv(&mut csv_file)?;

        // Check for error rather than implicitly flushing and ignoring.
        csv_file.commit().with_context(|| "committing CSV file")?;

        index_writer.add_entry(
            csv_path,
            out_table.table_cfg.tags.iter().map(String::as_ref),
            page_numbers,
        );

        Ok(())
    }

    /// Extracts a single table into a CSV file.
    fn extract_table(
        &self,
        table_cfg: &config::book::Table,
        input_pdf: &Path,
    ) -> Result<(Table, Vec<i32>)> {
        let tmpl_path = table_cfg.tabula_template_path();

        let extracted_tables = self
            .tabula_client
            .read_pdf_with_template(self.cfg_reader.as_ref(), input_pdf, &tmpl_path)
            .with_context(|| format!("extracting table from PDF {:?}", input_pdf))?;

        let table = match &table_cfg.transform {
            None => Table::concatenated(extracted_tables.tables),
            Some(TableTransform::LegacyTransformSeq(legacy_transform)) => {
                let mut table = Table::concatenated(extracted_tables.tables);
                table = legacy_transform::apply_transforms(&legacy_transform.transforms, table)?;
                table.clean();
                table
            }
            Some(TableTransform::ESTransform(es_transform)) => {
                let func = TransformFn {
                    function_body: es_transform.src.clone(),
                    origin: ESScriptOrigin {
                        resource_name: format!("{:?}", table_cfg.file_stem),
                        resource_line_offset: 0,
                        resource_column_offset: 0,
                        script_id: 0,
                    },
                };
                self.estrn
                    .transform(func, extracted_tables.tables)
                    .with_context(|| "applying ESTransform")?
            }
        };

        let page_numbers: Vec<i32> = extracted_tables.source_pages.into_iter().collect();

        Ok((table, page_numbers))
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

fn run_ecma_scripts(
    cfg_reader: &dyn Reader,
    estrn: &mut ESTransformer,
    script_paths: &[PathBuf],
) -> Result<()> {
    for (i, path) in script_paths.iter().enumerate() {
        let path_str = path.to_str().ok_or_else(|| {
            anyhow!(
                "ecma_script_modules[{}] ({:?}) is not a valid UTF-8 path",
                i,
                path
            )
        })?;

        let mut source = String::new();
        cfg_reader
            .open_read(path)
            .with_context(|| format!("opening ecma_script_modules[{}] ({:?})", i, path))?
            .read_to_string(&mut source)
            .with_context(|| format!("reading ecma_script_modules[{}] ({:?})", i, path))?;

        estrn
            .run_script(ESScript {
                source,
                origin: ESScriptOrigin {
                    resource_name: path_str.to_owned(),
                    resource_line_offset: 0,
                    resource_column_offset: 0,
                    script_id: 0,
                },
            })
            .with_context(|| format!("running ecma_script_modules[{}] ({:?})", i, path))?;
    }

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
