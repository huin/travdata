#![allow(dead_code)]

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::{
    index::IndexWriter,
    pdf::{ExtractedTable, ExtractedTables, TableReader},
    tableextract::{
        TableTransform,
        estransform::{ESScript, ESTransformer, TransformFn},
        legacy_transform,
    },
};
use crate::{filesio::ReadWriter, table::Table, template};

/// Encapsulates the values required to extract tables from book(s).
pub struct Extractor<'a> {
    tabula_client: &'a dyn TableReader,
    estrn: ESTransformer,
    tmpl: &'a template::Book,
}

/// Specifies a book's tables to be extracted by [Extractor::extract_book].
pub struct ExtractSpec<'a> {
    pub input_pdf: &'a Path,
    pub overwrite_existing: bool,
    pub with_tags: &'a [String],
    pub without_tags: &'a [String],
}

impl ExtractSpec<'_> {
    /// Returns `true` if `tags` is allowed by both `with_tags` and `without_tags`.
    fn allows(&self, tags: &HashSet<String>) -> bool {
        self.with_tags_allows(tags) && self.without_tags_allows(tags)
    }

    /// Returns `true` if `tags` is allowed by `self.with_tags`.
    fn with_tags_allows(&self, tags: &HashSet<String>) -> bool {
        self.with_tags.is_empty() || tags.iter().any(|tag| self.with_tags.contains(tag))
    }

    /// Returns `true` if `tags` is allowed by `self.without_tags`.
    fn without_tags_allows(&self, tags: &HashSet<String>) -> bool {
        self.without_tags.is_empty() || !tags.iter().any(|tag| self.without_tags.contains(tag))
    }
}

/// Extraction event emitted to track progress.
#[derive(Debug)]
pub enum ExtractEvent {
    /// Indicates successful progress of extraction of a single output file.
    Progress {
        path: PathBuf,
        completed: usize,
        total: usize,
    },
    /// Indicates error with some portion of the extraction process.
    /// If `true`, `terminal` indicates that the error prevents any progress being made.
    Error { err: anyhow::Error, terminal: bool },
    /// Indicates that extraction has completed and that no more events will follow.
    Completed,
    /// Indicates that extraction has been cancelled, and than no more events will follow.
    Cancelled,
}

/// Trait to implement to receive notifications about extraction events, or to
/// cancel extraction early.
pub trait ExtractEvents {
    fn on_event(&mut self, event: ExtractEvent);
    fn do_continue(&self) -> bool;
}

impl<'a> Extractor<'a> {
    /// Create a new [Extractor].
    pub fn new(tmpl: &'a template::Book, tabula_client: &'a dyn TableReader) -> Result<Self> {
        let mut estrn = ESTransformer::new().context("initialising ESTransformer")?;
        run_ecma_scripts(&mut estrn, &tmpl.scripts)?;

        Ok(Self {
            tabula_client,
            estrn,
            tmpl,
        })
    }

    /// Extracts tables from a single book.
    pub fn extract_book(
        &self,
        spec: ExtractSpec,
        events: &mut dyn ExtractEvents,
        out_writer: &dyn ReadWriter<'a>,
    ) {
        let mut index_writer =
            match IndexWriter::new(out_writer).with_context(|| "opening index for update") {
                Ok(index_writer) => index_writer,
                Err(err) => {
                    events.on_event(ExtractEvent::Error {
                        err,
                        terminal: true,
                    });
                    return;
                }
            };

        let output_tables = self.collect_output_tables(&spec, out_writer);

        for (i, out_table) in output_tables.iter().enumerate() {
            let extract_result = self
                .extract_table(out_table, spec.input_pdf)
                .with_context(|| format!("processing table {:?}", out_table.out_filepath));

            match extract_result {
                Err(err) => {
                    events.on_event(ExtractEvent::Error {
                        err,
                        terminal: false,
                    });
                }
                Ok((table, page_numbers_set)) => {
                    let page_numbers = page_numbers_set
                        .into_iter()
                        .map(|page_number| page_number + self.tmpl.page_offset)
                        .collect();

                    let write_result = Self::write_table(
                        out_writer,
                        &mut index_writer,
                        out_table,
                        table,
                        page_numbers,
                    );
                    if let Err(err) = write_result {
                        events.on_event(ExtractEvent::Error {
                            err,
                            terminal: false,
                        });
                    }
                }
            }

            events.on_event(ExtractEvent::Progress {
                path: out_table.out_filepath.clone(),
                completed: i + 1,
                total: output_tables.len(),
            });
            if !events.do_continue() {
                events.on_event(ExtractEvent::Cancelled);
                return;
            }
        }

        if let Err(err) = index_writer
            .commit()
            .with_context(|| "commiting changes to the index")
        {
            events.on_event(ExtractEvent::Error {
                err,
                terminal: false,
            });
        }

        events.on_event(ExtractEvent::Completed);
    }

    fn collect_output_tables<'tmpl>(
        &'tmpl self,
        spec: &ExtractSpec,
        out_writer: &dyn ReadWriter<'a>,
    ) -> Vec<OutputTable<'tmpl>> {
        let mut output_tables: Vec<OutputTable<'tmpl>> = Vec::new();
        let empty_tags = HashSet::new();
        collect_output_tables_from_group(
            out_writer,
            &self.tmpl.group,
            Path::new(""),
            "".to_string(),
            spec,
            &mut output_tables,
            &empty_tags,
        );
        output_tables
    }

    fn write_table(
        out_writer: &dyn ReadWriter<'a>,
        index_writer: &mut IndexWriter,
        out_table: &OutputTable,
        table: Table,
        page_numbers: Vec<i32>,
    ) -> Result<()> {
        let mut csv_file = out_writer.open_write(&out_table.out_filepath)?;

        table.write_csv(&mut csv_file)?;

        // Check for error rather than implicitly flushing and ignoring.
        csv_file.commit().with_context(|| "committing CSV file")?;

        index_writer.add_entry(
            out_table.out_filepath.clone(),
            out_table.table_tmpl.tags.iter().map(String::as_ref),
            page_numbers,
        );

        Ok(())
    }

    /// Extracts a single table into a CSV file.
    fn extract_table(
        &self,
        out_table: &OutputTable,
        input_pdf: &Path,
    ) -> Result<(Table, HashSet<i32>)> {
        let extracted_tables = ExtractedTables(
            out_table
                .table_tmpl
                .portions
                .iter()
                .map(|table_portion| {
                    self.tabula_client
                        .read_table_portion(input_pdf, table_portion)
                        .with_context(|| format!("extracting table from PDF {:?}", input_pdf))
                })
                .collect::<Result<Vec<ExtractedTable>>>()?,
        );

        let page_numbers: HashSet<i32> = extracted_tables
            .0
            .iter()
            .map(|ext_table| ext_table.page)
            .collect();

        let tables_iter = extracted_tables
            .0
            .into_iter()
            .map(|ext_table| ext_table.data);

        let table = match &out_table.table_tmpl.transform {
            None => Table::concatenated(tables_iter),
            Some(TableTransform::LegacyTransformSeq(legacy_transform)) => {
                let mut table = Table::concatenated(tables_iter);
                table = legacy_transform::apply_transforms(&legacy_transform.transforms, table)?;
                table.clean();
                table
            }
            Some(TableTransform::ESTransform(es_transform)) => {
                let func = TransformFn {
                    function_body: es_transform.src.clone(),
                    origin: v8wrapper::ESScriptOrigin {
                        resource_name: out_table.transform_name.clone(),
                        is_module: false,
                        ..Default::default()
                    },
                };
                self.estrn
                    .transform(func, tables_iter.collect())
                    .with_context(|| "applying ESTransform")?
            }
        };

        Ok((table, page_numbers))
    }
}

fn run_ecma_scripts(estrn: &mut ESTransformer, scripts: &[template::Script]) -> Result<()> {
    for script in scripts.iter() {
        estrn
            .run_script(ESScript {
                source: script.code.clone(),
                origin: v8wrapper::ESScriptOrigin {
                    resource_name: script.name.clone(),
                    is_module: false,
                    ..Default::default()
                },
            })
            .with_context(|| format!("running script {:?}", script.name))?;
    }

    Ok(())
}

struct OutputTable<'tmpl> {
    out_filepath: PathBuf,
    /// Provides a name for an ECMAScript transform.
    transform_name: String,
    tags: HashSet<String>,
    table_tmpl: &'tmpl template::Table,
}

/// Collects the set of [template::Table]s that are to be included in the extration by `spec`.
fn collect_output_tables_from_group<'tmpl>(
    out_writer: &dyn ReadWriter<'_>,
    group: &'tmpl template::Group,
    out_path: &Path,
    transform_name: String,
    spec: &ExtractSpec,
    output_tables: &mut Vec<OutputTable<'tmpl>>,
    parent_tags: &HashSet<String>,
) {
    if !spec.without_tags_allows(&group.tags) {
        return;
    }

    // Inherit parent tags and own tags.
    let mut tags = parent_tags.clone();
    tags.extend(group.tags.iter().cloned());

    for (child_group_name, child_group) in group.groups.iter() {
        let child_path = out_path.join(child_group_name);
        collect_output_tables_from_group(
            out_writer,
            child_group,
            &child_path,
            format!("{}/{}", transform_name, child_group_name),
            spec,
            output_tables,
            &tags,
        );
    }

    for (child_table_name, child_table) in group.tables.iter() {
        let mut table_path = out_path.join(child_table_name);
        table_path.set_extension("csv");
        collect_output_table(
            out_writer,
            child_table,
            table_path,
            format!("{}/{}.js", transform_name, child_table_name),
            spec,
            output_tables,
            &tags,
        );
    }
}

fn collect_output_table<'tmpl>(
    out_writer: &dyn ReadWriter<'_>,
    table: &'tmpl template::Table,
    out_path: PathBuf,
    transform_name: String,
    spec: &ExtractSpec,
    output_tables: &mut Vec<OutputTable<'tmpl>>,
    parent_tags: &HashSet<String>,
) {
    // Inherit parent tags and own tags.
    let mut tags = parent_tags.clone();
    tags.extend(table.tags.iter().cloned());

    if !spec.allows(&tags) {
        return;
    }

    if !spec.overwrite_existing && out_writer.exists(&out_path) {
        return;
    }

    output_tables.push(OutputTable::<'tmpl> {
        out_filepath: out_path,
        transform_name,
        tags,
        table_tmpl: table,
    });
}
