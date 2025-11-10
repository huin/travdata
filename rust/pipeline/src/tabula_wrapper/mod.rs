pub mod singlethreaded;

use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

/// Required trait for making a single batch extraction call to Tabula for table(s) extraction.
pub trait TabulaExtractor {
    fn extract_tables(&self, request: TabulaExtractionRequest) -> Result<JsonTableSet>;
}

/// Single request to a [TabulaExtractor] to batch extract tables from a PDF file.
pub struct TabulaExtractionRequest {
    pub pdf_path: PathBuf,
    pub password: Option<String>,
    pub page: i32,
    pub guess: bool,
    pub use_returns: bool,
    pub page_areas: Vec<(i32, tabula::Rectangle)>,
    pub method: tabula::ExtractionMethod,
}

/// A sequence of extracted tables from a PDF file.
#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct JsonTableSet(pub Vec<JsonTable>);

/// A single extracted table from a PDF file.
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct JsonTable {
    pub extraction_method: String,
    pub page_number: i32,
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
    pub right: f32,
    pub bottom: f32,
    pub data: Vec<JsonRow>,
}

/// A single extracted table row from a PDF file.
#[derive(Deserialize, Debug)]
pub struct JsonRow(pub Vec<JsonCell>);

/// A single extracted table cell from a PDF file.
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct JsonCell {
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
}
