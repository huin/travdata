use super::*;

impl<S> testutils::DefaultForTest for GenericNode<S>
where
    S: testutils::DefaultForTest,
{
    fn default_for_test() -> Self {
        Self {
            id: NodeId::default_for_test(),
            tags: Default::default(),
            public: true,
            spec: S::default_for_test(),
        }
    }
}
