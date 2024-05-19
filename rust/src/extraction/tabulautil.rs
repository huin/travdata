use std::{collections::HashSet, path};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{filesio::Reader, table::Table};

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Template(pub Vec<TemplateEntry>);

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

fn load_tabula_tmpl(cfg_reader: &dyn Reader, path: &path::Path) -> Result<Template> {
    let tmpl_reader = cfg_reader.open_read(path)?;
    let tmpl = serde_json::from_reader(tmpl_reader)?;
    Ok(tmpl)
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct JsonTableSet(pub Vec<JsonTable>);

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

#[derive(Deserialize, Debug)]
pub struct JsonCell {
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
}

/// Page numbers and tables read from a PDF.
#[derive(Debug)]
pub struct ExtractedTables {
    pub source_pages: HashSet<i32>,
    pub tables: Vec<Table>,
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

    /// Reads table(s) from a PDF, based on the Tabula template.
    /// * `cfg_reader` a `Reader` for the configuration.
    /// * `pdf_path` Path to PDF to read from.
    /// * `template_file` Path to the Tabula template JSON
    /// file.
    pub fn read_pdf_with_template(
        &self,
        cfg_reader: &dyn Reader,
        pdf_path: &path::Path,
        template_file: &path::Path,
    ) -> Result<ExtractedTables> {
        let template = load_tabula_tmpl(cfg_reader, template_file)
            .with_context(|| format!("loading Tabula template in {:?}", template_file))?;
        let env = self.vm.attach().with_context(|| "attaching to TabulaVM")?;

        let mut source_pages: HashSet<i32> = HashSet::new();
        let mut tables: Vec<Table> = Vec::with_capacity(template.0.len());

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

            let extracted_file = tabula.parse_document(pdf_path, "some-table")?;
            let result: JsonTableSet = serde_json::from_reader(extracted_file)
                .with_context(|| "parsing JSON output from Tabula")?;

            source_pages.insert(entry.page);

            for json_table in result.0 {
                tables.push(json_table.into());
            }
        }

        Ok(ExtractedTables {
            source_pages,
            tables,
        })
    }
}
