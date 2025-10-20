//! Concrete systems to act upon [crate::Node]s.

mod input_pdf_file;
mod js_context;
mod js_transform;
mod output_directory;
mod output_file_csv;
mod output_file_json;
mod tabula_pdf_extract_table;

pub use input_pdf_file::InputPdfFileSystem;
pub use js_context::JsContextSystem;
pub use js_transform::JsTransformSystem;
pub use output_directory::OutputDirectorySystem;
pub use output_file_csv::OutputFileCsvSystem;
pub use output_file_json::OutputFileJsonSystem;
pub use tabula_pdf_extract_table::TabulaPdfExtractTableSystem;
