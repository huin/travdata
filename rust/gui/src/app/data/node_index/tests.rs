use pipeline::NodeId;

use crate::app::data::NodeRef;

use super::*;

mod test_by_node_id {
    use googletest::prelude::*;
    use pipeline::NodeId;
    use testutils::DefaultForTest;

    use crate::app::data::NodeRef;

    use super::super::*;
    use super::collect_node_ref_id;

    #[gtest]
    fn test_finds_indexed_node_id() {
        let mut next_ref = NodeRef::default();

        let mut index = NodeIndex::default();
        let mut index_gen = index.generation();
        expect_true!(index_gen.is_same_and_update(index.generation));

        let node_1_id_before = NodeId::from("node-1-id-before");
        let node_1_ref = next_ref.next_and_inc_for_test();
        let mut node_1 = pipeline::Node {
            meta: pipeline::NodeMeta {
                id: node_1_id_before.clone(),
            },
            ..DefaultForTest::default_for_test()
        };

        // GIVEN: node_1 is indexed with its initial ID node_1_id_before.
        index.index_node(node_1_ref, &node_1);
        expect_false!(index_gen.is_same_and_update(index.generation));
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id_before)),
            unordered_elements_are![eq(&(node_1_ref, node_1_id_before.clone()))]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // WHEN: node_1 is reindexed with id changed to node_1_id_after.
        let node_1_id_after = NodeId::from("node-1-id-after");
        node_1.meta.id = node_1_id_after.clone();
        index.index_node(node_1_ref, &node_1);
        expect_false!(index_gen.is_same_and_update(index.generation));

        // THEN: no node should be found by node_1_id_before.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id_before)),
            is_empty()
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // THEN: node_1 should be found by node_1_id_after.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id_after)),
            unordered_elements_are![eq(&(node_1_ref, node_1_id_after.clone()))]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));
    }

    #[gtest]
    fn test_finds_with_node_id_collision_update() {
        let mut next_ref = NodeRef::default();

        let mut index = NodeIndex::default();
        let mut index_gen = index.generation();
        expect_true!(index_gen.is_same_and_update(index.generation));

        let node_1_id_before = NodeId::from("node-1-id-before");
        let node_1_ref = next_ref.next_and_inc_for_test();
        let mut node_1 = pipeline::Node {
            meta: pipeline::NodeMeta {
                id: node_1_id_before.clone(),
            },
            ..DefaultForTest::default_for_test()
        };

        let node_2_id = NodeId::from("node-2-id");
        let node_2_ref = next_ref.next_and_inc_for_test();
        let node_2 = pipeline::Node {
            meta: pipeline::NodeMeta {
                id: node_2_id.clone(),
            },
            ..DefaultForTest::default_for_test()
        };

        // GIVEN: node_1 and node_2 are indexed.
        index.index_node(node_1_ref, &node_1);
        expect_false!(index_gen.is_same_and_update(index.generation));
        index.index_node(node_2_ref, &node_2);
        expect_false!(index_gen.is_same_and_update(index.generation));
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id_before)),
            unordered_elements_are![eq(&(node_1_ref, node_1_id_before.clone()))]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_2_id)),
            unordered_elements_are![eq(&(node_2_ref, node_2_id.clone()))]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // WHEN: node_1 is reindexed with id changed to node_2_id (colliding with node_2).
        node_1.meta.id = node_2_id.clone();
        index.index_node(node_1_ref, &node_1);
        expect_false!(index_gen.is_same_and_update(index.generation));

        // THEN: no node should be found by node_1_id_before.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id_before)),
            is_empty()
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // THEN: node_1 and node_2 should be found by node_2_id.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_2_id)),
            unordered_elements_are![
                eq(&(node_1_ref, node_2_id.clone())),
                eq(&(node_2_ref, node_2_id.clone())),
            ]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // WHEN: node_1 is reindexed with id changed to node_1_id_after.
        let node_1_id_after = NodeId::from("node-1-id-after");
        node_1.meta.id = node_1_id_after.clone();
        index.index_node(node_1_ref, &node_1);
        expect_false!(index_gen.is_same_and_update(index.generation));

        // THEN: node_1 should be found by node_1_id_after.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id_after)),
            unordered_elements_are![eq(&(node_1_ref, node_1_id_after.clone())),]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // THEN: node_2 should be found by node_2_id.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_2_id)),
            unordered_elements_are![eq(&(node_2_ref, node_2_id.clone())),]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));
    }

    #[gtest]
    fn test_reindex_to_same_node_id_does_not_change_generation() {
        let mut next_ref = NodeRef::default();

        let mut index = NodeIndex::default();
        let mut index_gen = index.generation();
        expect_true!(index_gen.is_same_and_update(index.generation));

        let node_1_id = NodeId::from("node-1-id");
        let node_1_ref = next_ref.next_and_inc_for_test();
        let node_1 = pipeline::Node {
            meta: pipeline::NodeMeta {
                id: node_1_id.clone(),
            },
            ..DefaultForTest::default_for_test()
        };

        // GIVEN: node_1 is indexed.
        index.index_node(node_1_ref, &node_1);
        expect_false!(index_gen.is_same_and_update(index.generation));

        // WHEN: node_1 is re-indexed without changing the NodeId.
        index.index_node(node_1_ref, &node_1);

        // THEN: the index generation has not changed.
        expect_true!(index_gen.is_same_and_update(index.generation));
    }

    #[gtest]
    fn test_deindex_node() {
        let mut next_ref = NodeRef::default();

        let mut index = NodeIndex::default();
        let mut index_gen = index.generation();
        expect_true!(index_gen.is_same_and_update(index.generation));

        let node_1_id = NodeId::from("node-1-id");
        let node_1_ref = next_ref.next_and_inc_for_test();
        let mut node_1 = pipeline::Node {
            meta: pipeline::NodeMeta {
                id: node_1_id.clone(),
            },
            ..DefaultForTest::default_for_test()
        };

        let node_2_id = NodeId::from("node-2-id");
        let node_2_ref = next_ref.next_and_inc_for_test();
        let node_2 = pipeline::Node {
            meta: pipeline::NodeMeta {
                id: node_2_id.clone(),
            },
            ..DefaultForTest::default_for_test()
        };

        // GIVEN: node_1 and node_2 are indexed.
        index.index_node(node_1_ref, &node_1);
        expect_false!(index_gen.is_same_and_update(index.generation));
        index.index_node(node_2_ref, &node_2);
        expect_false!(index_gen.is_same_and_update(index.generation));
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id)),
            unordered_elements_are![eq(&(node_1_ref, node_1_id.clone()))]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_2_id)),
            unordered_elements_are![eq(&(node_2_ref, node_2_id.clone()))]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // WHEN: node_1 is deindexed.
        node_1.meta.id = node_2_id.clone();
        index.deindex_node(node_1_ref);
        expect_false!(index_gen.is_same_and_update(index.generation));

        // THEN: no node should be found by node_1_id.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id)),
            is_empty()
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // THEN: node_2 should still be found by node_2_id.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_2_id)),
            unordered_elements_are![eq(&(node_2_ref, node_2_id.clone())),]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));
    }

    #[gtest]
    fn test_deindex_non_existing_node() {
        let mut next_ref = NodeRef::default();

        let mut index = NodeIndex::default();
        let mut index_gen = index.generation();
        expect_true!(index_gen.is_same_and_update(index.generation));

        let node_1_id = NodeId::from("node-1-id");
        let node_1_ref = next_ref.next_and_inc_for_test();
        let node_1 = pipeline::Node {
            meta: pipeline::NodeMeta {
                id: node_1_id.clone(),
            },
            ..DefaultForTest::default_for_test()
        };

        let node_non_exist_ref = next_ref.next_and_inc_for_test();

        // GIVEN: node_1 is indexed.
        index.index_node(node_1_ref, &node_1);
        expect_false!(index_gen.is_same_and_update(index.generation));
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id)),
            unordered_elements_are![eq(&(node_1_ref, node_1_id.clone()))]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));

        // WHEN: a non-existing node ref is deindexed.
        index.deindex_node(node_non_exist_ref);

        // THEN: the generation does not change.
        expect_true!(index_gen.is_same_and_update(index.generation));

        // THEN: node_1 should still be found by node_1_id.
        expect_that!(
            collect_node_ref_id(index.by_node_id(&node_1_id)),
            unordered_elements_are![eq(&(node_1_ref, node_1_id.clone()))]
        );
        expect_true!(index_gen.is_same_and_update(index.generation));
    }
}

fn collect_node_ref_id<'a>(
    iter: impl Iterator<Item = &'a NodeIndexEntry>,
) -> Vec<(NodeRef, NodeId)> {
    iter.map(|entry| (*entry.node_ref(), entry.node_id().clone()))
        .collect()
}
