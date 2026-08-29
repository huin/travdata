use googletest::prelude::*;
use testutils::DefaultForTest;

use crate::app::{
    data::{EditablePipeline, GuiNode, NodeRef, node::GuiNodeWithId, node_index::NodeIndexEntry},
    ddo,
};

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

#[gtest]
fn test_finds_indexed_node_id() {
    let mut pl = EditablePipeline::default();
    let mut index_gen = pl.node_index().generation();
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // GIVEN: node_1 is created and indexed with its initial ID.
    let node_1_id_before = "node-1-id-before";
    let node_1_ref = make_node(node_1_id_before, &mut pl);
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_1_id_before)),
        unordered_elements_are![eq(&(node_1_ref, node_1_id_before))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // WHEN: node_1 is reindexed with id changed to node_1_id_after.
    let node_1_id_after = "node-1-id-after";
    pl.with_node_ctx_by_ref_mut(node_1_ref, |node_ctx| {
        let mut node_ctx = node_ctx.unwrap();
        node_ctx.node.node_id = node_1_id_after.to_string();
        node_ctx.mark_node_changed();
    });
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));

    // THEN: no node should be found by node_1_id_before.
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_1_id_before)),
        is_empty()
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // THEN: node_1 should be found by node_1_id_after.
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_1_id_after)),
        unordered_elements_are![eq(&(node_1_ref, node_1_id_after))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));
}

#[gtest]
fn test_finds_with_node_id_collision_update() {
    let mut pl = EditablePipeline::default();
    let mut index_gen = pl.node_index().generation();
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // GIVEN: node_1 and node_2 are created and indexed.
    let node_1_id_before = "node-1-id-before";
    let node_1_ref = make_node(node_1_id_before, &mut pl);
    let node_2_id = "node-2-id-before";
    let node_2_ref = make_node(node_2_id, &mut pl);
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_1_id_before)),
        unordered_elements_are![eq(&(node_1_ref, node_1_id_before))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_2_id)),
        unordered_elements_are![eq(&(node_2_ref, node_2_id))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // WHEN: node_1 is reindexed with id changed to node_2_id (colliding with node_2).
    pl.with_node_ctx_by_ref_mut(node_1_ref, |node_ctx| {
        let mut node_ctx = node_ctx.unwrap();
        node_ctx.node.node_id = node_2_id.to_string();
        node_ctx.mark_node_changed();
    });
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));

    // THEN: no node should be found by node_1_id_before.
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_1_id_before)),
        is_empty()
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // THEN: node_1 and node_2 should be found by node_2_id.
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_2_id)),
        unordered_elements_are![eq(&(node_1_ref, node_2_id)), eq(&(node_2_ref, node_2_id))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // WHEN: node_1 is reindexed with id changed to node_1_id_after.
    let node_1_id_after = "node-1-id-after";
    pl.with_node_ctx_by_ref_mut(node_1_ref, |node_ctx| {
        let mut node_ctx = node_ctx.unwrap();
        node_ctx.node.node_id = node_1_id_after.to_string();
        node_ctx.mark_node_changed();
    });
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));

    // THEN: node_1 should be found by node_1_id_after.
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_1_id_after)),
        unordered_elements_are![eq(&(node_1_ref, node_1_id_after))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // THEN: node_2 should be found by node_2_id.
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_2_id)),
        unordered_elements_are![eq(&(node_2_ref, node_2_id)),]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));
}

#[gtest]
fn test_scan_node_id_prefix() {
    let mut pl = EditablePipeline::default();

    // GIVEN: nodes with same node ID prefix.
    let common_node_ref = make_node("common-prefix", &mut pl);
    let common_node_max_suffix_ref = make_node("common-prefix-\u{10ffff}", &mut pl);
    let common_node_max_1_ref = make_node("common-prefix-\u{10ffff}-1", &mut pl);
    let common_node_1_ref = make_node("common-prefix-1", &mut pl);
    let common_node_2_ref = make_node("common-prefix-2", &mut pl);
    let common_node_3_ref = make_node("common-prefix-3", &mut pl);

    // GIVEN: nodes with different node ID prefix.
    let common_prefiw_ref = make_node("common-prefiw", &mut pl);
    let common_prefiy_ref = make_node("common-prefiy", &mut pl);
    let other_ref = make_node("other-prefix", &mut pl);
    let another_ref = make_node("another-prefix", &mut pl);

    expect_that!(
        // WHEN: searching for nodes with ID prefix "common-prefix".
        collect_node_ref_id(pl.node_index().scan_node_id_prefix("common-prefix")),
        // THEN: only matching nodes and their IDs are returned.
        unordered_elements_are![
            eq(&(common_node_ref, "common-prefix")),
            eq(&(common_node_max_suffix_ref, "common-prefix-\u{10ffff}")),
            eq(&(common_node_max_1_ref, "common-prefix-\u{10ffff}-1")),
            eq(&(common_node_1_ref, "common-prefix-1")),
            eq(&(common_node_2_ref, "common-prefix-2")),
            eq(&(common_node_3_ref, "common-prefix-3")),
        ]
    );

    expect_that!(
        // WHEN: searching for nodes with ID prefix "common-prefix-\u{10ffff}".
        collect_node_ref_id(
            pl.node_index()
                .scan_node_id_prefix("common-prefix-\u{10ffff}")
        ),
        // THEN: only matching nodes and their IDs are returned.
        unordered_elements_are![
            eq(&(common_node_max_suffix_ref, "common-prefix-\u{10ffff}")),
            eq(&(common_node_max_1_ref, "common-prefix-\u{10ffff}-1")),
        ]
    );

    expect_that!(
        // WHEN: searching for nodes with ID prefix "".
        collect_node_ref_id(pl.node_index().scan_node_id_prefix("")),
        // THEN: all nodes and their IDs are returned.
        unordered_elements_are![
            eq(&(common_node_ref, "common-prefix")),
            eq(&(common_node_max_suffix_ref, "common-prefix-\u{10ffff}")),
            eq(&(common_node_max_1_ref, "common-prefix-\u{10ffff}-1")),
            eq(&(common_node_1_ref, "common-prefix-1")),
            eq(&(common_node_2_ref, "common-prefix-2")),
            eq(&(common_node_3_ref, "common-prefix-3")),
            eq(&(common_prefiw_ref, "common-prefiw")),
            eq(&(common_prefiy_ref, "common-prefiy")),
            eq(&(other_ref, "other-prefix")),
            eq(&(another_ref, "another-prefix")),
        ]
    );
}

#[gtest]
fn test_reindex_to_same_node_id_does_not_change_generation() {
    let mut pl = EditablePipeline::default();
    let mut index_gen = pl.node_index().generation();
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // GIVEN: node_1 is created and indexed.
    let node_1_id = "node-1-id";
    let node_1_ref = make_node(node_1_id, &mut pl);
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));

    // WHEN: node_1 is marked as changed without changing the NodeId.
    pl.with_node_ctx_by_ref_mut(node_1_ref, |node_ctx| {
        let mut node_ctx = node_ctx.unwrap();
        node_ctx.mark_node_changed();
    });

    // THEN: the index generation has not changed.
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));
}

#[gtest]
fn test_remove_node() {
    let mut pl = EditablePipeline::default();
    let mut index_gen = pl.node_index().generation();
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // GIVEN: node_1 and node_2 are created and indexed.
    let node_1_id = "node-1-id";
    let node_1_ref = make_node(node_1_id, &mut pl);
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));
    let node_2_id = "node-2-id";
    let node_2_ref = make_node(node_2_id, &mut pl);
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_1_id)),
        unordered_elements_are![eq(&(node_1_ref, node_1_id))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_2_id)),
        unordered_elements_are![eq(&(node_2_ref, node_2_id))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // WHEN: node_1 is removed.
    pl.remove_node(node_1_ref);
    expect_false!(index_gen.is_same_and_update(pl.node_index().generation()));

    // THEN: no node should be found by node_1_id.
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_1_id)),
        is_empty()
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));

    // THEN: node_2 should still be found by node_2_id.
    expect_that!(
        collect_node_ref_id(pl.node_index().lookup_exact_node_id(node_2_id)),
        unordered_elements_are![eq(&(node_2_ref, node_2_id))]
    );
    expect_true!(index_gen.is_same_and_update(pl.node_index().generation()));
}

fn collect_node_ref_id<'a>(
    iter: impl Iterator<Item = &'a NodeIndexEntry>,
) -> Vec<(NodeRef, &'a str)> {
    iter.map(|entry| (*entry.node_ref(), entry.node_id()))
        .collect()
}

fn make_node(id: &'static str, pl: &mut EditablePipeline) -> NodeRef {
    pl.add_node(GuiNodeWithId {
        node_id: id.into(),
        node: GuiNode {
            ..DefaultForTest::default_for_test()
        },
    })
}
