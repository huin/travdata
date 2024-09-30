use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};
use googletest::{
    assert_that,
    matchers::{eq, len, ok},
};
use sha::utils::DigestExt;
use test_casing::test_casing;

use super::{CachingTableReader, HashAlgo, HashDigest};
use crate::{
    extraction::pdf::{ExtractedTable, ExtractedTables, TableReader},
    table::{Row, Table},
};

#[test]
fn hash_digest_length_is_correct() {
    let real_length = HashAlgo::default().to_bytes().len();
    let type_length = HashDigest::default().0.len();
    assert_that!(type_length, eq(real_length));
}

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

fn get_table_cache_path(tempdir: &Path) -> PathBuf {
    tempdir.join("table-cache.json")
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

fn fake_table_data(num_rows: usize, num_cols: usize, page: i32) -> ExtractedTable {
    ExtractedTable {
        page,
        table: Table(
            (1..=num_rows)
                .map(|ri| Row((1..=num_cols).map(|ci| format!("r{}c{}", ri, ci)).collect()))
                .collect(),
        ),
    }
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

    let table_cache_path = get_table_cache_path(tempdir.path());
    let mut fake_delegate = FakeTableReader::new();

    let first_original_tables = ExtractedTables(vec![fake_table_data(1, 1, 1)]);
    let first_expect_call = distinct_reads.first_call(tempdir.path())?;
    fake_delegate
        .return_tables
        .insert(first_expect_call.clone(), first_original_tables.clone());

    let second_original_tables = ExtractedTables(vec![fake_table_data(2, 2, 2)]);
    let second_expect_call = distinct_reads.second_call(tempdir.path())?;
    fake_delegate
        .return_tables
        .insert(second_expect_call.clone(), second_original_tables.clone());

    let caching_reader = CachingTableReader::load(fake_delegate, table_cache_path)?;
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
    let table_cache_path = get_table_cache_path(tempdir.path());
    let mut fake_delegate = FakeTableReader::new();
    let original_tables = ExtractedTables(vec![fake_table_data(1, 1, 1)]);

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
    let caching_reader = CachingTableReader::load(fake_delegate.clone(), table_cache_path)?;
    let actual_1 = first_expect_call.do_call(&caching_reader)?;
    let actual_2 = second_expect_call.do_call(&caching_reader)?;

    assert_that!(&actual_1, eq(&original_tables));
    assert_that!(&actual_2, eq(&original_tables));
    assert_that!(fake_delegate.calls_snapshot(), len(eq(1)));
    Ok(())
}

#[googletest::test]
fn cache_persistance() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    let table_cache_path = get_table_cache_path(tempdir.path());

    let pdf_1 = pdf_1(tempdir.path())?;
    let mut fake_delegate = FakeTableReader::new();

    let original_tables = ExtractedTables(vec![fake_table_data(1, 1, 1)]);
    let expect_call = Call {
        pdf_path: pdf_1.to_owned(),
        template_json: TEMPLATE_1.to_owned(),
    };
    fake_delegate
        .return_tables
        .insert(expect_call.clone(), original_tables.clone());

    let fake_delegate = Arc::new(fake_delegate);

    let first_caching_reader =
        CachingTableReader::load(fake_delegate.clone(), table_cache_path.clone())?;
    let actual_1 = expect_call.do_call(&first_caching_reader)?;
    assert_that!(first_caching_reader.store(), ok(eq(())));
    assert_that!(&actual_1, eq(&original_tables));
    assert_that!(
        fake_delegate.calls_snapshot(),
        eq(vec![expect_call.clone()])
    );

    let second_caching_reader = CachingTableReader::load(fake_delegate.clone(), table_cache_path)?;
    let actual_2 = expect_call.do_call(&second_caching_reader)?;
    drop(second_caching_reader);
    assert_that!(&actual_2, eq(&original_tables));
    // Should not have been called a second time.
    assert_that!(fake_delegate.calls_snapshot(), len(eq(1)));

    Ok(())
}
