pub mod tabulareader;

use std::{collections::HashSet, path};

use anyhow::Result;

use crate::{filesio::Reader, table::Table};

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
    /// * `template_file` Path to the Tabula template JSON file.
    fn read_pdf_with_template(
        &self,
        cfg_reader: &dyn Reader,
        pdf_path: &path::Path,
        template_file: &path::Path,
    ) -> Result<ExtractedTables>;
}
