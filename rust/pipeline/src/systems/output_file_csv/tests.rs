use std::env::temp_dir;

use anyhow::Result;
use googletest::prelude::*;
use serde_json::Value;
use testutils::DefaultForTest;

use crate::{
    intermediates, plargs, specs,
    testutil::{node_id, output_path_buf},
};

use super::*;

#[gtest]
fn test_process_writes_file() -> Result<()> {
    let system = OutputFileCsvSystem;

    const FILENAME: &str = "bar/foo.csv";

    let node = crate::Node {
        spec: specs::OutputFileCsv {
            input_data: node_id("input-data"),
            directory: node_id("output-directory"),
            filename: output_path_buf(FILENAME),
        }
        .into(),
        ..DefaultForTest::default_for_test()
    };

    let mut intermediates = intermediates::IntermediateSet::new();
    intermediates.set(
        node_id("input-data"),
        intermediates::JsonData(Value::Array(vec![
            Value::Array(vec![
                Value::String("header1".into()),
                Value::String("header2".into()),
            ]),
            Value::Array(vec![
                Value::String("r1c1".into()),
                Value::String("r1c2".into()),
            ]),
            Value::Array(vec![Value::String("r2c1".into())]),
            Value::Array(vec![
                Value::String("r3c1".into()),
                Value::String("r3c2".into()),
                Value::String("r3c3".into()),
            ]),
            Value::Array(vec![]),
        ]))
        .into(),
    );

    let output_dir = temp_dir();
    intermediates.set(
        node_id("output-directory"),
        intermediates::OutputDirectory(output_dir.clone()).into(),
    );
    let args = plargs::ArgSet::default();

    expect_that!(
        system.process(&node, &args, &intermediates)?,
        eq(&intermediates::NoData.into()),
    );

    let records = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(output_dir.join(FILENAME))?
        .records()
        .map(|str_record_result| {
            str_record_result.map(|str_record| {
                str_record
                    .iter()
                    .map(str::to_string)
                    .collect::<Vec<String>>()
            })
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let expect_records: Vec<Vec<String>> = vec![
        vec!["header1".into(), "header2".into()],
        vec!["r1c1".into(), "r1c2".into()],
        vec!["r2c1".into()],
        vec!["r3c1".into(), "r3c2".into(), "r3c3".into()],
        // Unlike the original data, CSV does not replicate an empty row.
        vec!["".into()],
    ];

    expect_eq!(records, expect_records);

    Ok(())
}
