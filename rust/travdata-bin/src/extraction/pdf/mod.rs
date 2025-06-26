pub mod cachingreader;
pub mod pdfiumthread;
pub mod tabulareader;
pub mod tabulatmpl;

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use cachingreader::CachingTableReader;
use clap::Args;
use serde::{Deserialize, Serialize};
use tabulareader::TabulaClient;

use crate::{distpaths, table::Table, template};

/// Page numbers and tables read from a PDF.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExtractedTables(pub Vec<ExtractedTable>);

/// Page number and table read from a PDF.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExtractedTable {
    pub page: i32,
    pub data: Table,
}

pub trait TableReader {
    /// Reads table(s) from a PDF, based on the Tabula template.
    /// * `pdf_path` Path to PDF to read from.
    /// * `table_portion` region of the PDF to extract.
    fn read_table_portion(
        &self,
        pdf_path: &Path,
        table_portion: &template::TablePortion,
    ) -> Result<ExtractedTable>;

    // TODO: Add a batch read function, and make use of it in [crate::bookextract].

    /// Shuts down the [TableReader], flushing any resources that it was using.
    fn close(self: Box<Self>) -> Result<()>;
}

/// CLI arguments relating to [CachableTableReader].
#[derive(Args, Clone, Debug, Default)]
pub struct TableReaderArgs {
    /// Path to Tabula JAR file.
    #[arg(long)]
    tabula_libpath: Option<String>,

    /// Use the table cache.
    #[arg(long, default_value = "true")]
    table_cache: bool,
}

impl TableReaderArgs {
    pub fn build(&self, xdg_dirs: &xdg::BaseDirectories) -> Result<Box<dyn TableReader>> {
        let tabula_jar_path = self
            .tabula_libpath
            .clone()
            .or_else(distpaths::tabula_jar)
            .ok_or_else(|| {
                anyhow!("--tabula-libpath must be specified, as tabula.jar could not be located")
            })?;

        let tabula_reader =
            TabulaClient::new(&tabula_jar_path).with_context(|| "initialising Tabula")?;

        if !self.table_cache {
            return Ok(Box::new(tabula_reader));
        }

        let tables_cache_path = xdg_dirs.place_cache_file(Path::new("table-cache.json"))?;
        Ok(Box::new(CachingTableReader::load(
            tabula_reader,
            tables_cache_path,
        )?))
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use googletest::{assert_that, matchers::eq};

    use crate::extraction::pdf::{ExtractedTable, ExtractedTables};

    /// Test that Serde has been configured correctly for [ExtractedTables].
    #[googletest::test]
    fn extracted_tables_serialized_form() -> Result<()> {
        const SERIALIZED: &str = r#"[
            {
                "page": 1,
                "data": [
                    ["t1r1c1", "t1r1c2"],
                    ["t1r2c1", "t1r2c2"]
                ]
            },
            {
                "page": 2,
                "data": [
                    ["t2r1c1", "t2r1c2"],
                    ["t2r2c1", "t2r2c2"]
                ]
            }
        ]"#;
        let want = ExtractedTables(vec![
            ExtractedTable {
                page: 1,
                data: [["t1r1c1", "t1r1c2"], ["t1r2c1", "t1r2c2"]].into(),
            },
            ExtractedTable {
                page: 2,
                data: [["t2r1c1", "t2r1c2"], ["t2r2c1", "t2r2c2"]].into(),
            },
        ]);

        let got: ExtractedTables = serde_json::from_str(SERIALIZED)?;

        assert_that!(got, eq(&want));

        Ok(())
    }
}
