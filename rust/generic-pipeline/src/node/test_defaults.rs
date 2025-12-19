use map_macro::hash_set;

use super::*;

impl<S> testutils::DefaultForTest for GenericNode<S>
where
    S: testutils::DefaultForTest,
{
    fn default_for_test() -> Self {
        Self {
            id: NodeId::default_for_test(),
            tags: hash_set![],
            public: true,
            spec: S::default_for_test(),
        }
    }
}
