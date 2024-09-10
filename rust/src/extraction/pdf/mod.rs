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
