pub mod cachingreader;
pub mod tabulareader;

use std::path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::table::Table;

/// Page numbers and tables read from a PDF.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExtractedTables(pub Vec<ExtractedTable>);

/// Page number and table read from a PDF.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExtractedTable {
    pub page: i32,
    pub table: Table,
}

pub trait TableReader {
    /// Reads table(s) from a PDF, based on the Tabula template.
    /// * `pdf_path` Path to PDF to read from.
    /// * `template_json` Raw JSON-encoded contents of the Tabula template file.
    fn read_pdf_with_template(
        &self,
        pdf_path: &path::Path,
        template_json: &str,
    ) -> Result<ExtractedTables>;
}

impl<T> TableReader for &T
where
    T: TableReader,
{
    fn read_pdf_with_template(
        &self,
        pdf_path: &path::Path,
        template_json: &str,
    ) -> Result<ExtractedTables> {
        T::read_pdf_with_template(self, pdf_path, template_json)
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
                "table": [
                    ["t1r1c1", "t1r1c2"],
                    ["t1r2c1", "t1r2c2"]
                ]
            },
            {
                "page": 2,
                "table": [
                    ["t2r1c1", "t2r1c2"],
                    ["t2r2c1", "t2r2c2"]
                ]
            }
        ]"#;
        let want = ExtractedTables(vec![
            ExtractedTable {
                page: 1,
                table: [["t1r1c1", "t1r1c2"], ["t1r2c1", "t1r2c2"]].into(),
            },
            ExtractedTable {
                page: 2,
                table: [["t2r1c1", "t2r1c2"], ["t2r2c1", "t2r2c2"]].into(),
            },
        ]);

        let got: ExtractedTables = serde_json::from_str(SERIALIZED)?;

        assert_that!(got, eq(want));

        Ok(())
    }
}
