use super::*;

impl testutils::DefaultForTest for NodeId {
    fn default_for_test() -> Self {
        Self::new_unchecked("test-default-node-id".into())
    }
}
