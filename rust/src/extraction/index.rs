//! Code to create/update an index of output data.

use std::{
    collections::{HashMap, HashSet},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    config::{book::Table, root::Book},
    filesio::{FileWrite, ReadWriter, Reader},
};

type CsvResult<T> = std::result::Result<T, csv::Error>;

const INDEX_PATH: &str = "index.csv";

/// Index of all extracted tables in an output.
pub struct Index {
    paths: Vec<PathBuf>,
    tags_to_paths: HashMap<String, HashSet<PathBuf>>,
}

impl Index {
    /// Creates the index from `CsvRecord`s.
    fn from_records(records: impl IntoIterator<Item = CsvResult<CsvRecord>>) -> Result<Self> {
        let mut paths = Vec::default();
        let mut tags_to_paths: HashMap<String, HashSet<PathBuf>> = HashMap::default();

        for record_result in records {
            let record = record_result?;

            for tag in record.tags.split(";") {
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
        let records = reader
            .deserialize::<CsvRecord>();
        Index::from_records(records)
    }

    /// Loads the index from a `Reader`.
    pub fn load(reader: &dyn Reader) -> Result<Self> {
        let r = reader.open_read(Path::new(INDEX_PATH))?;
        Self::load_from_read(r)
    }

    /// Returns paths to tables with all of the given tags.
    ///
    /// * `tags`` Tags to select for.
    ///
    /// Returns paths of matching tables. Returns all tables if `tags` is empty.
    fn paths_with_all_tags(&self, tags: &[&str]) -> Vec<&Path> {
        let mut tags_iter = tags.into_iter();

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
        todo!()
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

    use googletest::{
        assert_that,
        matchers::{eq, unordered_elements_are},
    };

    use crate::{extraction::index::INDEX_PATH, filesio::{mem::{MemFilesHandle, MemReadWriter}, ReadWriter}};

    use super::Index;

    #[test]
    fn test_index_paths_with_all_tags() {
        let read_writer = MemReadWriter::new(MemFilesHandle::default());

        let mut w = read_writer.open_write(Path::new(INDEX_PATH)).expect("should open");
        w.write_all(b"table_path,pages,tags
file-a,,tag-a;tag-d;tag-z
file-b,,tag-b;tag-d;tag-z
file-c,,tag-c;tag-z
").expect("should write");
        w.commit().expect("should commit");

        let index = Index::load(&read_writer).expect("should create from reader");

        assert_that!(
            index.paths_with_all_tags(&vec![]),
            unordered_elements_are![
                eq(Path::new("file-a")),
                eq(Path::new("file-b")),
                eq(Path::new("file-c")),
            ],
        );
        assert_that!(
            index.paths_with_all_tags(&vec!["tag-a"]),
            unordered_elements_are![eq(Path::new("file-a"))],
        );
        assert_that!(
            index.paths_with_all_tags(&vec!["tag-d", "tag-z"]),
            unordered_elements_are![eq(Path::new("file-a")), eq(Path::new("file-b"))],
        );
    }
}
