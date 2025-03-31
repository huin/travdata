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
    extraction::pdf::{ExtractedTable, TableReader},
    table::{Row, Table},
    template,
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
    table_portion: template::TablePortion,
}

impl Call {
    fn do_call(&self, reader: &dyn TableReader) -> Result<ExtractedTable> {
        reader.read_table_portion(&self.pdf_path, &self.table_portion)
    }
}

struct FakeTableReaderCalls {
    calls: Mutex<Vec<Call>>,
}
impl FakeTableReaderCalls {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }

    fn add_call(&self, call: Call) {
        self.calls
            .lock()
            .expect("failed to lock `FakeTableReader::calls`")
            .push(call);
    }

    fn calls_snapshot(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }
}

#[derive(Clone)]
struct FakeTableReader {
    calls: Arc<FakeTableReaderCalls>,
    return_table: HashMap<Call, ExtractedTable>,
}

impl FakeTableReader {
    fn new() -> Self {
        FakeTableReader {
            calls: Arc::new(FakeTableReaderCalls::new()),
            return_table: HashMap::new(),
        }
    }

    fn calls(&self) -> Arc<FakeTableReaderCalls> {
        self.calls.clone()
    }
}

impl TableReader for FakeTableReader {
    fn read_table_portion(
        &self,
        pdf_path: &Path,
        table_portion: &template::TablePortion,
    ) -> Result<ExtractedTable> {
        let call = Call {
            pdf_path: pdf_path.to_owned(),
            table_portion: table_portion.clone(),
        };

        let tables_opt = self.return_table.get(&call).cloned();

        let result =
            tables_opt.ok_or_else(|| anyhow!("could not find `return_table` for {:?}", call));

        self.calls.add_call(call);

        result
    }

    fn close(self: Box<Self>) -> Result<()> {
        Ok(())
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

const TABLE_PORTION_1: template::TablePortion = template::TablePortion {
    extraction_method: template::TabulaExtractionMethod::Lattice,
    page: 1,
    rect: template::PDFRect {
        left: template::PDFPoints::from_quantised(5),
        top: template::PDFPoints::from_quantised(30),
        right: template::PDFPoints::from_quantised(20),
        bottom: template::PDFPoints::from_quantised(10),
    },
};

const TABLE_PORTION_2: template::TablePortion = template::TablePortion {
    extraction_method: template::TabulaExtractionMethod::Stream,
    page: 2,
    rect: template::PDFRect {
        left: template::PDFPoints::from_quantised(15),
        top: template::PDFPoints::from_quantised(40),
        right: template::PDFPoints::from_quantised(30),
        bottom: template::PDFPoints::from_quantised(20),
    },
};

struct TwoReadsCase {
    name: &'static str,
    first_table_portion: template::TablePortion,
    second_table_portion: template::TablePortion,
    first_pdf: &'static dyn Fn(&Path) -> Result<PathBuf>,
    second_pdf: &'static dyn Fn(&Path) -> Result<PathBuf>,
}

impl TwoReadsCase {
    fn first_call(&self, tempdir_path: &Path) -> Result<Call> {
        Ok(Call {
            pdf_path: (self.first_pdf)(tempdir_path)?,
            table_portion: self.first_table_portion.clone(),
        })
    }

    fn second_call(&self, tempdir_path: &Path) -> Result<Call> {
        Ok(Call {
            pdf_path: (self.second_pdf)(tempdir_path)?,
            table_portion: self.second_table_portion.clone(),
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
        data: Table(
            (1..=num_rows)
                .map(|ri| Row((1..=num_cols).map(|ci| format!("r{}c{}", ri, ci)).collect()))
                .collect(),
        ),
    }
}

const DISTINCT_READS_CASES: [TwoReadsCase; 3] = [
    TwoReadsCase {
        name: "same_template_different_pdf",
        first_table_portion: TABLE_PORTION_1,
        second_table_portion: TABLE_PORTION_1,
        first_pdf: &pdf_1,
        second_pdf: &pdf_2,
    },
    TwoReadsCase {
        name: "different_template_same_pdf",
        first_table_portion: TABLE_PORTION_1,
        second_table_portion: TABLE_PORTION_2,
        first_pdf: &pdf_1,
        second_pdf: &pdf_1,
    },
    TwoReadsCase {
        name: "different_template_different_pdf",
        first_table_portion: TABLE_PORTION_1,
        second_table_portion: TABLE_PORTION_2,
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

    let first_original_table = fake_table_data(1, 1, 1);
    let first_expect_call = distinct_reads.first_call(tempdir.path())?;
    fake_delegate
        .return_table
        .insert(first_expect_call.clone(), first_original_table.clone());

    let second_original_table = fake_table_data(2, 2, 2);
    let second_expect_call = distinct_reads.second_call(tempdir.path())?;
    fake_delegate
        .return_table
        .insert(second_expect_call.clone(), second_original_table.clone());

    let caching_reader = CachingTableReader::load(fake_delegate, table_cache_path)?;
    let actual_1 = first_expect_call.do_call(&caching_reader)?;
    assert_that!(actual_1, eq(&first_original_table));
    let actual_2 = second_expect_call.do_call(&caching_reader)?;
    assert_that!(actual_2, eq(&second_original_table));

    Ok(())
}

const CACHE_HIT_TWO_READS_CASES: [TwoReadsCase; 5] = [
    TwoReadsCase {
        name: "same_template_1_same_pdf_path_1",
        first_table_portion: TABLE_PORTION_1,
        second_table_portion: TABLE_PORTION_1,
        first_pdf: &pdf_1,
        second_pdf: &pdf_1,
    },
    TwoReadsCase {
        name: "same_template_1_same_pdf_path_2",
        first_table_portion: TABLE_PORTION_1,
        second_table_portion: TABLE_PORTION_1,
        first_pdf: &pdf_2,
        second_pdf: &pdf_2,
    },
    TwoReadsCase {
        name: "same_template_2_same_pdf_path_1",
        first_table_portion: TABLE_PORTION_2,
        second_table_portion: TABLE_PORTION_2,
        first_pdf: &pdf_1,
        second_pdf: &pdf_1,
    },
    TwoReadsCase {
        name: "same_template_2_same_pdf_path_2",
        first_table_portion: TABLE_PORTION_2,
        second_table_portion: TABLE_PORTION_2,
        first_pdf: &pdf_2,
        second_pdf: &pdf_2,
    },
    // Support hashing the PDF and getting a hit on a copy of the PDF at a different path.
    TwoReadsCase {
        name: "same_template_1_same_pdf_content",
        first_table_portion: TABLE_PORTION_1,
        second_table_portion: TABLE_PORTION_1,
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
    let original_table = fake_table_data(1, 1, 1);

    let first_expect_call = cache_hit_read.first_call(tempdir.path())?;
    fake_delegate
        .return_table
        .insert(first_expect_call.clone(), original_table.clone());
    // This may or may not be a duplicate of first_expect_call.
    let second_expect_call = cache_hit_read.second_call(tempdir.path())?;
    fake_delegate
        .return_table
        .insert(second_expect_call.clone(), original_table.clone());
    let delegate_calls = fake_delegate.calls();

    let caching_reader = CachingTableReader::load(fake_delegate, table_cache_path)?;
    let actual_1 = first_expect_call.do_call(&caching_reader)?;
    let actual_2 = second_expect_call.do_call(&caching_reader)?;

    assert_that!(&actual_1, eq(&original_table));
    assert_that!(&actual_2, eq(&original_table));
    assert_that!(delegate_calls.calls_snapshot(), len(eq(1)));
    Ok(())
}

#[googletest::test]
fn cache_persistance() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    let table_cache_path = get_table_cache_path(tempdir.path());

    let pdf_1 = pdf_1(tempdir.path())?;
    let mut fake_delegate = FakeTableReader::new();

    let original_table = fake_table_data(1, 1, 1);
    let expect_call = Call {
        pdf_path: pdf_1.to_owned(),
        table_portion: TABLE_PORTION_1,
    };
    fake_delegate
        .return_table
        .insert(expect_call.clone(), original_table.clone());
    let delegate_calls = fake_delegate.calls();

    let first_caching_reader =
        CachingTableReader::load(fake_delegate.clone(), table_cache_path.clone())?;
    let actual_1 = expect_call.do_call(&first_caching_reader)?;
    assert_that!(first_caching_reader.store(), ok(eq(&())));
    assert_that!(&actual_1, eq(&original_table));
    assert_that!(
        delegate_calls.calls_snapshot(),
        eq(&vec![expect_call.clone()])
    );

    let second_caching_reader = CachingTableReader::load(fake_delegate, table_cache_path)?;
    let actual_2 = expect_call.do_call(&second_caching_reader)?;
    drop(second_caching_reader);
    assert_that!(&actual_2, eq(&original_table));
    // Should not have been called a second time.
    assert_that!(delegate_calls.calls_snapshot(), len(eq(1)));

    Ok(())
}
