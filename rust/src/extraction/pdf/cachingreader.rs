use std::path::Path;

use crate::extraction::pdf::TableReader;

pub struct CachingTableReader<T> {
    _delegate: T,
}

impl<T> CachingTableReader<T> {
    pub fn new(_delegate: T, _cache_path: &Path) -> Self {
        Self { _delegate }
    }
}

impl<T> TableReader for CachingTableReader<T>
where
    T: TableReader,
{
    fn read_pdf_with_template(
        &self,
        _pdf_path: &std::path::Path,
        _template_json: &str,
    ) -> anyhow::Result<super::ExtractedTables> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };

    use anyhow::{anyhow, Result};
    use googletest::{
        assert_that,
        matchers::{eq, len},
    };
    use test_casing::test_casing;

    use super::CachingTableReader;
    use crate::{
        extraction::pdf::{ExtractedTables, TableReader},
        table::{Row, Table},
    };

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    struct Call {
        pdf_path: PathBuf,
        template_json: String,
    }

    impl Call {
        fn do_call(&self, reader: &dyn TableReader) -> Result<ExtractedTables> {
            reader.read_pdf_with_template(&self.pdf_path, &self.template_json)
        }
    }

    struct FakeTableReader {
        calls: Mutex<Vec<Call>>,
        return_tables: HashMap<Call, ExtractedTables>,
    }

    impl FakeTableReader {
        fn new() -> Self {
            FakeTableReader {
                calls: Mutex::new(Vec::new()),
                return_tables: HashMap::new(),
            }
        }

        fn calls_snapshot(&self) -> Vec<Call> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl TableReader for FakeTableReader {
        fn read_pdf_with_template(
            &self,
            pdf_path: &std::path::Path,
            template_json: &str,
        ) -> anyhow::Result<ExtractedTables> {
            let call = Call {
                pdf_path: pdf_path.to_owned(),
                template_json: template_json.to_owned(),
            };

            let tables_opt = self.return_tables.get(&call).cloned();

            let result =
                tables_opt.ok_or_else(|| anyhow!("could not find `return_tables` for {:?}", call));

            self.calls
                .lock()
                .expect("failed to lock `FakeTableReader::calls`")
                .push(call);

            result
        }
    }

    impl TableReader for Arc<FakeTableReader> {
        fn read_pdf_with_template(
            &self,
            pdf_path: &std::path::Path,
            template_json: &str,
        ) -> Result<ExtractedTables> {
            self.as_ref()
                .read_pdf_with_template(pdf_path, template_json)
        }
    }

    fn pdf_1(tempdir: &Path) -> Result<PathBuf> {
        let path = tempdir.join("1.pdf");
        std::fs::write(&path, "PDF 1 data")?;
        Ok(path)
    }

    fn pdf_1_copy(tempdir: &Path) -> Result<PathBuf> {
        let path = tempdir.join("copy-of-1.pdf");
        std::fs::write(&path, "PDF 1 data")?;
        Ok(path)
    }

    fn pdf_2(tempdir: &Path) -> Result<PathBuf> {
        let path = tempdir.join("2.pdf");
        std::fs::write(&path, "PDF 2 data")?;
        Ok(path)
    }

    const TEMPLATE_1: &str = "\"template 1\"";
    const TEMPLATE_2: &str = "\"template 2\"";

    struct TwoReadsCase {
        name: &'static str,
        first_template_json: &'static str,
        second_template_json: &'static str,
        first_pdf: &'static dyn Fn(&Path) -> Result<PathBuf>,
        second_pdf: &'static dyn Fn(&Path) -> Result<PathBuf>,
    }

    impl TwoReadsCase {
        fn first_call(&self, tempdir_path: &Path) -> Result<Call> {
            Ok(Call {
                pdf_path: (self.first_pdf)(tempdir_path)?,
                template_json: self.first_template_json.to_owned(),
            })
        }

        fn second_call(&self, tempdir_path: &Path) -> Result<Call> {
            Ok(Call {
                pdf_path: (self.second_pdf)(tempdir_path)?,
                template_json: self.second_template_json.to_owned(),
            })
        }
    }

    impl std::fmt::Debug for TwoReadsCase {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
            write!(f, "TwoReadsCase{{name: {} ...}}", self.name)
        }
    }

    fn fake_table_data(num_rows: usize, num_cols: usize) -> Table {
        Table(
            (1..=num_rows)
                .map(|ri| Row((1..=num_cols).map(|ci| format!("r{}c{}", ri, ci)).collect()))
                .collect(),
        )
    }

    fn page_number_set(page_number: i32) -> HashSet<i32> {
        let mut pages = HashSet::new();
        pages.insert(page_number);
        pages
    }

    const DISTINCT_READS_CASES: [TwoReadsCase; 3] = [
        TwoReadsCase {
            name: "same_template_different_pdf",
            first_template_json: TEMPLATE_1,
            second_template_json: TEMPLATE_1,
            first_pdf: &pdf_1,
            second_pdf: &pdf_2,
        },
        TwoReadsCase {
            name: "different_template_same_pdf",
            first_template_json: TEMPLATE_1,
            second_template_json: TEMPLATE_2,
            first_pdf: &pdf_1,
            second_pdf: &pdf_1,
        },
        TwoReadsCase {
            name: "different_template_different_pdf",
            first_template_json: TEMPLATE_1,
            second_template_json: TEMPLATE_2,
            first_pdf: &pdf_1,
            second_pdf: &pdf_2,
        },
    ];

    #[test_casing(3, DISTINCT_READS_CASES)]
    fn does_not_cache_distinct_reads(distinct_reads: TwoReadsCase) -> Result<()> {
        assert_that!(DISTINCT_READS_CASES, len(eq(3)));

        let tempdir = tempfile::tempdir()?;

        let cache_dir = tempdir.path().join("cache");
        let mut fake_delegate = FakeTableReader::new();

        let first_original_tables = ExtractedTables {
            source_pages: page_number_set(1),
            tables: vec![fake_table_data(1, 1)],
        };
        let first_expect_call = distinct_reads.first_call(tempdir.path())?;
        fake_delegate
            .return_tables
            .insert(first_expect_call.clone(), first_original_tables.clone());

        let second_original_tables = ExtractedTables {
            source_pages: page_number_set(2),
            tables: vec![fake_table_data(2, 2)],
        };
        let second_expect_call = distinct_reads.second_call(tempdir.path())?;
        fake_delegate
            .return_tables
            .insert(second_expect_call.clone(), second_original_tables.clone());

        let caching_reader = CachingTableReader::new(fake_delegate, &cache_dir);
        let actual_1 = first_expect_call.do_call(&caching_reader)?;
        assert_that!(actual_1, eq(first_original_tables));
        let actual_2 = second_expect_call.do_call(&caching_reader)?;
        assert_that!(actual_2, eq(second_original_tables));

        Ok(())
    }

    const CACHE_HIT_TWO_READS_CASES: [TwoReadsCase; 5] = [
        TwoReadsCase {
            name: "same_template_1_same_pdf_path_1",
            first_template_json: TEMPLATE_1,
            second_template_json: TEMPLATE_1,
            first_pdf: &pdf_1,
            second_pdf: &pdf_1,
        },
        TwoReadsCase {
            name: "same_template_1_same_pdf_path_2",
            first_template_json: TEMPLATE_1,
            second_template_json: TEMPLATE_1,
            first_pdf: &pdf_2,
            second_pdf: &pdf_2,
        },
        TwoReadsCase {
            name: "same_template_2_same_pdf_path_1",
            first_template_json: TEMPLATE_2,
            second_template_json: TEMPLATE_2,
            first_pdf: &pdf_1,
            second_pdf: &pdf_1,
        },
        TwoReadsCase {
            name: "same_template_2_same_pdf_path_2",
            first_template_json: TEMPLATE_2,
            second_template_json: TEMPLATE_2,
            first_pdf: &pdf_2,
            second_pdf: &pdf_2,
        },
        // Support hashing the PDF and getting a hit on a copy of the PDF at a different path.
        TwoReadsCase {
            name: "same_template_1_same_pdf_content",
            first_template_json: TEMPLATE_1,
            second_template_json: TEMPLATE_1,
            first_pdf: &pdf_1,
            second_pdf: &pdf_1_copy,
        },
    ];

    #[test_casing(5, CACHE_HIT_TWO_READS_CASES)]
    fn cache_hit_two_reads(cache_hit_read: TwoReadsCase) -> Result<()> {
        assert_that!(CACHE_HIT_TWO_READS_CASES, len(eq(5)));

        let tempdir = tempfile::tempdir()?;
        let cache_dir = tempdir.path().join("cache");
        let mut fake_delegate = FakeTableReader::new();
        let original_tables = ExtractedTables {
            source_pages: page_number_set(1),
            tables: vec![fake_table_data(1, 1), fake_table_data(2, 1)],
        };

        let first_expect_call = cache_hit_read.first_call(tempdir.path())?;
        fake_delegate
            .return_tables
            .insert(first_expect_call.clone(), original_tables.clone());
        // This may or may not be a duplicate of first_expect_call.
        let second_expect_call = cache_hit_read.second_call(tempdir.path())?;
        fake_delegate
            .return_tables
            .insert(second_expect_call.clone(), original_tables.clone());

        let fake_delegate = Arc::new(fake_delegate);
        let caching_reader = CachingTableReader::new(fake_delegate.clone(), &cache_dir);
        let actual_1 = first_expect_call.do_call(&caching_reader)?;
        let actual_2 = second_expect_call.do_call(&caching_reader)?;

        assert_that!(&actual_1, eq(&original_tables));
        assert_that!(&actual_2, eq(&original_tables));
        assert_that!(fake_delegate.calls_snapshot(), len(eq(1)));
        Ok(())
    }

    // TODO: test cache_persistance
}
