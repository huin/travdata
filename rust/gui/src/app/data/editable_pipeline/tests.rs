use googletest::prelude::*;

use crate::app::{data::EditablePipeline, ddo};

#[gtest]
fn test_roundtrip_valid_pipeline() -> Result<()> {
    // GIVEN: a valid ddo::PipelineNodes.
    let original: ddo::PipelineNodes = serde_json::from_reader(std::fs::File::open(
        "test_data/minimal-valid-pipeline.json",
    )?)?;

    // GIVEN: an EditablePipeline created from the original pipeline.
    let editable = EditablePipeline::try_from(original.clone())?;

    // WHEN: the EditablePipeline is converted back to the ddo::PipelineNodes.
    let actual = editable.to_pipeline();

    // THEN: the conversion was successful and is equal to the original.
    expect_that!(actual, ok(eq(&original)));

    Ok(())
}
