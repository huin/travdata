//! Code to create/update an index of output data.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
#[cfg(test)]
use std::{collections::HashSet, io::Read};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::filesio::Reader;
use crate::{
    filesio::{FileRead, FileWrite, FilesIoError, ReadWriter},
    fmtutil,
};

#[cfg(test)]
type CsvResult<T> = std::result::Result<T, csv::Error>;

const INDEX_PATH: &str = "index.csv";
const ITEMS_DELIM: &str = ";";

#[cfg(test)]
/// Index of all extracted tables in an output.
pub struct Index {
    paths: Vec<PathBuf>,
    tags_to_paths: HashMap<String, std::collections::HashSet<PathBuf>>,
}

#[cfg(test)]
impl Index {
    /// Creates the index from `CsvRecord`s.
    fn from_records(records: impl IntoIterator<Item = CsvResult<CsvRecord>>) -> Result<Self> {
        let mut paths = Vec::default();
        let mut tags_to_paths: HashMap<String, HashSet<PathBuf>> = HashMap::default();

        for record_result in records {
            let record = record_result?;

            for tag in record.tags.split(ITEMS_DELIM) {
                tags_to_paths
                    .entry(tag.to_owned())
                    .or_default()
                    .insert(record.table_path.clone());
            }

            paths.push(record.table_path);
        }

        Ok(Index {
            paths,
            tags_to_paths,
        })
    }

    fn load_from_read<R: Read>(r: R) -> Result<Self> {
        let mut reader = csv::Reader::from_reader(r);
        let records = reader.deserialize::<CsvRecord>();
        Index::from_records(records)
    }

    /// Loads the index from a `Reader`.
    pub fn load(reader: &dyn Reader) -> Result<Self> {
        let r = reader.open_read(Path::new(INDEX_PATH))?;
        Self::load_from_read(r)
    }

    /// Returns paths to tables with all of the given tags.
    ///
    /// * `tags` Tags to select for.
    ///
    /// Returns paths of matching tables. Returns all tables if `tags` is empty.
    fn paths_with_all_tags(&self, tags: &[&str]) -> Vec<&Path> {
        let mut tags_iter = tags.iter();

        match tags_iter.next() {
            None => self.paths.iter().map(AsRef::as_ref).collect(),
            Some(&first_tag) => {
                let mut paths: Vec<&Path> = self
                    .tags_to_paths
                    .get(first_tag)
                    .map(|paths| paths.iter().map(PathBuf::as_path).collect())
                    .unwrap_or_default();

                for &tag in tags_iter {
                    if paths.is_empty() {
                        break;
                    }

                    match self.tags_to_paths.get(tag) {
                        None => {
                            paths.clear();
                            break;
                        }
                        Some(other_paths) => {
                            paths.retain(|&path| other_paths.contains(path));
                        }
                    }
                }

                paths
            }
        }
    }
}

/// Creates or updates an index.
pub struct IndexWriter<'rw> {
    write_file: FileWrite<'rw>,
    entries: HashMap<PathBuf, WriteRecord>,
}

impl<'rw> IndexWriter<'rw> {
    pub fn new(read_writer: &dyn ReadWriter<'rw>) -> Result<Self> {
        let mut entries: HashMap<PathBuf, WriteRecord> = HashMap::new();
        match read_writer.open_read(Path::new(INDEX_PATH)) {
            Ok(mut r) => Self::load_entries(&mut r, &mut entries)?,
            Err(err) if FilesIoError::NotFound.eq_anyhow(&err) => {}
            Err(err) => bail!(err),
        }

        let write_file = read_writer.open_write(Path::new(INDEX_PATH))?;

        Ok(Self {
            write_file,
            entries,
        })
    }

    fn load_entries(r: &mut FileRead, entries: &mut HashMap<PathBuf, WriteRecord>) -> Result<()> {
        let mut reader = csv::Reader::from_reader(r);
        for record_result in reader.deserialize::<CsvRecord>() {
            let record = record_result?;
            entries.insert(
                record.table_path,
                WriteRecord {
                    pages: record.pages,
                    tags: record.tags,
                },
            );
        }
        Ok(())
    }

    /// Commits entries to the index file.
    pub fn commit(mut self) -> Result<()> {
        let mut w = csv::WriterBuilder::new()
            .terminator(csv::Terminator::CRLF)
            .from_writer(&mut self.write_file);
        for (table_path, write_record) in self.entries {
            w.serialize(CsvRecord {
                table_path,
                pages: write_record.pages,
                tags: write_record.tags,
            })?;
        }
        w.flush()?;
        drop(w);
        self.write_file.commit()
    }

    /// Write an index entry.
    ///
    /// * `output_path` Path to the table file within the output.
    /// * `tags` Tags for the table being output.
    /// * `page_numbers` Page numbers that the entry was sourced from.
    pub fn add_entry<'a>(
        &mut self,
        output_path: PathBuf,
        tags: impl Iterator<Item = &'a str>,
        mut page_numbers: Vec<i32>,
    ) {
        let mut sorted_tags: Vec<String> = tags.map(str::to_string).collect();
        sorted_tags.sort();

        page_numbers.sort();

        self.entries.insert(
            output_path,
            WriteRecord {
                pages: fmtutil::join_display_slice(&page_numbers, ITEMS_DELIM),
                tags: sorted_tags.join(ITEMS_DELIM),
            },
        );
    }
}

struct WriteRecord {
    pages: String,
    tags: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CsvRecord {
    table_path: PathBuf,
    pages: String,
    tags: String,
}

#[cfg(test)]
mod tests {
    use std::{io::Write, path::Path};

    use anyhow::Result;
    use googletest::{
        assert_that,
        matchers::{eq, unordered_elements_are},
    };

    use crate::{
        extraction::index::{INDEX_PATH, Index, IndexWriter},
        filesio::{
            ReadWriter,
            mem::{MemFilesHandle, MemReadWriter},
        },
    };

    #[googletest::test]
    fn test_index_paths_with_all_tags() -> Result<()> {
        let read_writer = MemReadWriter::new(MemFilesHandle::default());

        let mut w = read_writer
            .open_write(Path::new(INDEX_PATH))
            .expect("should open");
        w.write_all(
            b"table_path,pages,tags
file-a,,tag-a;tag-d;tag-z
file-b,,tag-b;tag-d;tag-z
file-c,,tag-c;tag-z
",
        )?;
        w.commit()?;

        let index = Index::load(&read_writer)?;

        assert_that!(
            index.paths_with_all_tags(&[]),
            unordered_elements_are![
                eq(&Path::new("file-a")),
                eq(&Path::new("file-b")),
                eq(&Path::new("file-c")),
            ],
        );
        assert_that!(
            index.paths_with_all_tags(&["tag-a"]),
            unordered_elements_are![eq(&Path::new("file-a"))],
        );
        assert_that!(
            index.paths_with_all_tags(&["tag-d", "tag-z"]),
            unordered_elements_are![eq(&Path::new("file-a")), eq(&Path::new("file-b"))],
        );

        Ok(())
    }

    #[googletest::test]
    fn test_writes_new_index() -> Result<()> {
        let read_writer = MemReadWriter::new(MemFilesHandle::default());

        let mut index_writer = IndexWriter::new(&read_writer)?;
        index_writer.add_entry(
            Path::new("foo.csv").to_owned(),
            ["foo", "bar"].iter().copied(),
            vec![1, 2],
        );
        index_writer.commit()?;

        let index = Index::load(&read_writer)?;

        assert_that!(
            index.paths_with_all_tags(&["foo", "bar"]),
            unordered_elements_are![eq(&Path::new("foo.csv")),],
        );

        Ok(())
    }
}
