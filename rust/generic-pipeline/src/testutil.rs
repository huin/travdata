use mockall::mock;

use crate::{
    intermediates,
    node::{self, NodeId},
    pipeline, plargs, plinputs, plparams, processing, systems,
};

pub fn node_id(s: &str) -> node::NodeId {
    NodeId::test_node_id(s)
}

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug, Eq, PartialEq)]
pub enum FakeSpec {
    Foo(FooSpec),
    Bar(BarSpec),
}

impl From<FooSpec> for FakeSpec {
    fn from(value: FooSpec) -> Self {
        Self::Foo(value)
    }
}

impl From<BarSpec> for FakeSpec {
    fn from(value: BarSpec) -> Self {
        Self::Bar(value)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FooSpec {
    pub value: String,
    pub deps: Vec<node::NodeId>,
}

impl Default for FooSpec {
    fn default() -> Self {
        Self {
            value: "foo-value".into(),
            deps: Default::default(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BarSpec {
    pub value: String,
    pub deps: Vec<node::NodeId>,
}

impl Default for BarSpec {
    fn default() -> Self {
        Self {
            value: "bar-value".into(),
            deps: Default::default(),
        }
    }
}

pub type FakeNode = node::GenericNode<FakeSpec>;

impl Default for FakeNode {
    fn default() -> Self {
        Self {
            id: node_id("default-node-id"),
            tags: Default::default(),
            public: Default::default(),
            spec: FooSpec::default().into(),
        }
    }
}

impl node::GenericNode<FakeSpec> {
    pub fn default_with_spec<S>(spec: S) -> Self
    where
        S: Into<FakeSpec>,
    {
        Self {
            id: node_id("foo"),
            tags: Default::default(),
            public: false,
            spec: spec.into(),
        }
    }

    pub fn add_inputs<'a>(
        &self,
        reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
    ) -> Result<(), TestSystemError> {
        let deps = match &self.spec {
            FakeSpec::Foo(foo_spec) => &foo_spec.deps,
            FakeSpec::Bar(bar_spec) => &bar_spec.deps,
        };

        for dep in deps {
            reg.add_input(dep);
        }

        Ok(())
    }
}

pub fn foo_node(id: &str, deps: &[&str]) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: FooSpec {
            deps: deps.iter().map(|s| node_id(s)).collect(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

pub fn bar_node(id: &str, deps: &[&str]) -> FakeNode {
    FakeNode {
        id: node_id(id),
        spec: BarSpec {
            deps: deps.iter().map(|s| node_id(s)).collect(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TestParamType {}

#[derive(Debug, Eq, PartialEq)]
pub struct TestArgValue {}

pub type TestArgSet = plargs::GenericArgSet<TestArgValue>;

#[derive(Debug, Eq, PartialEq)]
pub enum TestIntermediateValue {
    NoData,
    ValueOne(u16),
}

pub type TestIntermediateSet = intermediates::GenericIntermediateSet<TestIntermediateValue>;

pub type TestPipeline = pipeline::GenericPipeline<FakeSpec>;

pub struct TestPipelineTypes;

impl crate::PipelineTypes for TestPipelineTypes {
    type Spec = FakeSpec;

    type ParamType = TestParamType;

    type ArgValue = TestArgValue;

    type IntermediateValue = TestIntermediateValue;

    type SystemError = TestSystemError;
}

#[derive(Debug, Eq, PartialEq)]
pub enum TestSystemError {
    ErrorOne,
    ErrorTwo,
}

impl std::fmt::Display for TestSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TestSystemError {}

pub type TestProcessor = processing::GenericProcessor<TestPipelineTypes>;

mock! {
    pub FakeSystem {}

    impl systems::GenericSystem<TestPipelineTypes> for FakeSystem {
        fn params<'a>(
            &self,
            node: &FakeNode,
            params: &'a mut plparams::GenericNodeParamsRegistrator<'a, TestParamType>,
        ) -> Result<(), TestSystemError>;

        fn inputs<'a>(
            &self,
            node: &FakeNode,
            reg: &'a mut plinputs::NodeInputsRegistrator<'a>,
        ) -> Result<(), TestSystemError>;

        fn process(
            &self,
            node: &FakeNode,
            args: &TestArgSet,
            intermediates: &TestIntermediateSet,
        ) -> Result<TestIntermediateValue, TestSystemError>;

        fn process_multiple<'a>(
            &self,
            nodes: &'a [&'a FakeNode],
            args: &plargs::GenericArgSet<TestArgValue>,
            intermediates: &TestIntermediateSet,
        ) -> Vec<systems::NodeResult<TestPipelineTypes>>;
    }
}
