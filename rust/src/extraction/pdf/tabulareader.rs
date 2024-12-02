use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::extraction::pdf::{ExtractedTables, TableReader};

use super::ExtractedTable;

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Template(pub Vec<TemplateEntry>);

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TemplateEntry {
    pub page: i32,
    pub extraction_method: String,
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
    pub width: f32,
    pub height: f32,
}

fn load_tabula_tmpl(template_json: &str) -> Result<Template> {
    let tmpl = serde_json::from_str(template_json)?;
    Ok(tmpl)
}

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
    fn read_pdf_with_template(
        &self,
        pdf_path: &Path,
        template_json: &str,
    ) -> Result<ExtractedTables> {
        let template = load_tabula_tmpl(template_json)
            .with_context(|| format!("loading Tabula template in {:?}", template_json))?;
        let env = self.vm.attach().with_context(|| "attaching to TabulaVM")?;

        let mut tables: Vec<ExtractedTable> = Vec::with_capacity(template.0.len());

        for entry in &template.0 {
            let pages = [entry.page];
            let page_areas = [(
                tabula::ABSOLUTE_AREA_CALCULATION_MODE,
                tabula::Rectangle::from_coords(entry.x1, entry.y1, entry.x2, entry.y2),
            )];

            let extraction_method = match entry.extraction_method.as_str() {
                "stream" => tabula::ExtractionMethod::Basic,
                "guess" => tabula::ExtractionMethod::Decide,
                "lattice" => tabula::ExtractionMethod::Spreadsheet,
                other => anyhow::bail!("unknown extraction_method: {:?}", other),
            };

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
            let result: JsonTableSet = serde_json::from_reader(extracted_file)
                .with_context(|| "parsing JSON output from Tabula")?;

            for json_table in result.0 {
                tables.push(ExtractedTable {
                    page: entry.page,
                    data: json_table.into(),
                });
            }
        }

        Ok(ExtractedTables(tables))
    }

    fn close(self: Box<Self>) -> Result<()> {
        Ok(())
    }
}
