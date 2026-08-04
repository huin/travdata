use std::env::temp_dir;

use googletest::prelude::*;
use serde_json::Value;
use testutils::DefaultForTest;

use crate::{intermediates, specs, testutil::output_path_buf};

use super::*;

#[gtest]
fn test_process_writes_file() -> Result<()> {
    let system = OutputFileJsonSystem;

    const FILENAME: &str = "bar/foo.json";

    let node = crate::Node {
        meta: DefaultForTest::default_for_test(),
        spec: specs::OutputFileJson {
            input_data: "input-data".into(),
            directory: "output-directory".into(),
            filename: output_path_buf(FILENAME),
        }
        .into(),
    };

    let mut intermediates = intermediates::IntermediateSet::new();

    let original_data = {
        let mut map = serde_json::Map::new();
        map.insert("foo".into(), Value::String("foo_value".into()));
        map.insert("bar".into(), Value::Bool(true));
        Value::Object(map)
    };
    intermediates.set(
        "input-data".into(),
        intermediates::JsonData(original_data.clone()).into(),
    );

    let output_dir = temp_dir();
    intermediates.set(
        "output-directory".into(),
        intermediates::OutputDirectory(output_dir.clone()).into(),
    );
    let args = ArgSet::default();

    expect_that!(
        system.process(&node, &args, &intermediates)?,
        eq(&intermediates::NoData.into()),
    );

    let written_file = std::fs::File::open(output_dir.join(FILENAME))?;
    let written_data: Value = serde_json::from_reader(&written_file)?;

    expect_eq!(written_data, original_data);

    Ok(())
}
