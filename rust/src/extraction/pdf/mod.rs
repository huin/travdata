pub mod tabulareader;

use std::{collections::HashSet, path};

use anyhow::Result;

use crate::table::Table;

/// Page numbers and tables read from a PDF.
#[derive(Debug)]
pub struct ExtractedTables {
    pub source_pages: HashSet<i32>,
    pub tables: Vec<Table>,
}

pub trait TableReader {
    /// Reads table(s) from a PDF, based on the Tabula template.
    /// * `cfg_reader` a `Reader` for the configuration.
    /// * `pdf_path` Path to PDF to read from.
    /// * `template_json` Raw JSON-encoded contents of the Tabula template file.
    fn read_pdf_with_template(
        &self,
        pdf_path: &path::Path,
        template_json: &str,
    ) -> Result<ExtractedTables>;
}
