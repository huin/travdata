use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use super::ExtractedTable;
use crate::{extraction::pdf::TableReader, template};

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct JsonTableSet(pub Vec<JsonTable>);

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

#[derive(Deserialize, Debug)]
pub struct JsonRow(pub Vec<JsonCell>);

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct JsonCell {
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
}

/// Client wrapper around Tabula.
pub struct TabulaClient {
    vm: tabula::TabulaVM,
}

impl TabulaClient {
    pub fn new(libpath: &str) -> Result<Self> {
        let vm = tabula::TabulaVM::new(libpath, false)?;
        Ok(TabulaClient { vm })
    }
}

impl TableReader for TabulaClient {
    fn read_table_portion(
        &self,
        pdf_path: &Path,
        table_portion: &template::TablePortion,
    ) -> Result<ExtractedTable> {
        let pages = [table_portion.page];
        let page_areas = [(
            tabula::ABSOLUTE_AREA_CALCULATION_MODE,
            tabula::Rectangle::from_coords(
                table_portion.rect.left.to_f32(),
                table_portion.rect.bottom.to_f32(),
                table_portion.rect.right.to_f32(),
                table_portion.rect.top.to_f32(),
            ),
        )];

        let extraction_method = match table_portion.extraction_method {
            template::TabulaExtractionMethod::Stream => tabula::ExtractionMethod::Basic,
            template::TabulaExtractionMethod::Guess => tabula::ExtractionMethod::Decide,
            template::TabulaExtractionMethod::Lattice => tabula::ExtractionMethod::Spreadsheet,
        };

        let env = self.vm.attach().with_context(|| "attaching to TabulaVM")?;

        let tabula = env
            .configure_tabula(
                Some(&page_areas),
                Some(&pages),
                tabula::OutputFormat::Json,
                false,
                extraction_method,
                false,
                None,
            )
            .with_context(|| "configuring Tabula to extract table")?;

        let extracted_file = tempfile::NamedTempFile::new()?;
        tabula.parse_document_into(pdf_path, extracted_file.path())?;
        let mut result: JsonTableSet = serde_json::from_reader(extracted_file)
            .with_context(|| "parsing JSON output from Tabula")?;

        Ok(ExtractedTable {
            page: table_portion.page,
            data: result
                .0
                .pop()
                .ok_or_else(|| {
                    anyhow!(
                        "expected exactly one table to have been extracted, got {}",
                        result.0.len()
                    )
                })?
                .into(),
        })
    }

    fn close(self: Box<Self>) -> Result<()> {
        Ok(())
    }
}
