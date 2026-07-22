use crate::spec_types::OutputPathBuf;

use super::*;

impl testutils::DefaultForTest for Spec {
    fn default_for_test() -> Self {
        Spec::InputPdfFile(InputPdfFile::default_for_test())
    }
}

impl testutils::DefaultForTest for InputPdfFile {
    fn default_for_test() -> Self {
        Self {
            description: "default input PDF description".into(),
        }
    }
}

impl testutils::DefaultForTest for OutputFileCsv {
    fn default_for_test() -> Self {
        Self {
            input_data: "input-id".into(),
            directory: "directory-id".into(),
            filename: OutputPathBuf::new_for_test("output.csv"),
        }
    }
}

impl testutils::DefaultForTest for OutputFileJson {
    fn default_for_test() -> Self {
        Self {
            input_data: "input-id".into(),
            directory: "directory-id".into(),
            filename: OutputPathBuf::new_for_test("output.json"),
        }
    }
}
