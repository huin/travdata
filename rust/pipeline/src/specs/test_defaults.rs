use super::*;
use crate::testutil::*;

impl testutils::DefaultForTest for Spec {
    fn default_for_test() -> Self {
        Spec::InputPdfFile(InputPdfFile::default_for_test())
    }
}

impl testutils::DefaultForTest for InputPdfFile {
    fn default_for_test() -> Self {
        Self
    }
}

impl testutils::DefaultForTest for OutputFileCsv {
    fn default_for_test() -> Self {
        Self {
            input_data: node_id("input-id"),
            filename: output_path_buf("output.csv"),
        }
    }
}

impl testutils::DefaultForTest for OutputFileJson {
    fn default_for_test() -> Self {
        Self {
            input_data: node_id("input-id"),
            filename: output_path_buf("output.json"),
        }
    }
}
