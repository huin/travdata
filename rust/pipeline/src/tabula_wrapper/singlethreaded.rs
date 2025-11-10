use anyhow::Context;

use crate::tabula_wrapper;

/// Single threaded implementation of [tabula_wrapper::TabulaExtractor].
///
/// Must be created and run on the main thread.
pub struct SingleThreadedTabulaExtractor<'env> {
    tabula_env: tabula::TabulaEnv<'env>,
}

impl<'env> SingleThreadedTabulaExtractor<'env> {
    /// Creates a [SingleThreadedTabulaExtractor] with the given [tabula::TabulaEnv].
    pub fn new(tabula_env: tabula::TabulaEnv<'env>) -> Self {
        Self { tabula_env }
    }
}

impl<'env> tabula_wrapper::TabulaExtractor for SingleThreadedTabulaExtractor<'env> {
    fn extract_tables(
        &self,
        request: super::TabulaExtractionRequest,
    ) -> anyhow::Result<tabula_wrapper::JsonTableSet> {
        let tabula = self
            .tabula_env
            .configure_tabula(
                Some(&request.page_areas),
                Some(&[request.page]),
                tabula::OutputFormat::Json,
                request.guess,
                request.method,
                request.use_returns,
                request.password.as_deref(),
            )
            .context("configuring Tabula to extract table")?;

        let extracted_file = tempfile::NamedTempFile::new()
            .context("creating temporary file for extracting PDF table data")?;
        tabula
            .parse_document_into(&request.pdf_path, extracted_file.path())
            .context("extracting PDF table data")?;

        serde_json::from_reader(extracted_file).context("parsing extracted PDF table data")
    }
}
