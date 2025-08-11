use anyhow::Result;
use mockall::mock;
use serde::{Deserialize, Serialize};

use crate::{
    intermediates,
    node::{self, NodeId},
    pipeline, plargs, plparams, processing, systems,
};

pub fn node_id(s: &str) -> node::NodeId {
    NodeId::test_node_id(s)
}

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[serde(tag = "type", content = "spec")]
pub enum FakeSpec {
    Foo(FooSpec),
    Bar(BarSpec),
}

impl node::SpecTrait for FakeSpec {
    type Discrim = FakeSpecDiscriminants;

    fn discriminant(&self) -> Self::Discrim {
        self.into()
    }
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

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
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

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
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

    pub fn modified<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        let mut s = self;
        f(&mut s);
        s
    }

    pub fn deps(&self) -> Vec<node::NodeId> {
        match &self.spec {
            FakeSpec::Foo(foo_spec) => foo_spec.deps.clone(),
            FakeSpec::Bar(bar_spec) => bar_spec.deps.clone(),
        }
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
pub enum TestParamType {
    TypeOne,
    TypeTwo,
}

pub type TestParam = plparams::GenericParam<TestParamType>;
pub type TestParams = plparams::GenericParams<TestParamType>;

#[derive(Debug, Eq, PartialEq)]
pub enum TestArgValue {
    TypeOne(u16),
    TypeTwo(u32),
}

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
}

pub type TestSystemMap = hashbrown::HashMap<
    FakeSpecDiscriminants,
    std::rc::Rc<dyn systems::GenericSystem<TestPipelineTypes>>,
>;

pub type TestProcessor = processing::GenericProcessor<TestPipelineTypes>;

mock! {
    pub FakeSystem {}

    impl systems::GenericSystem<TestPipelineTypes> for FakeSystem {
        fn params(&self, node: &FakeNode) -> TestParams;

        fn inputs(&self, node: &FakeNode) -> Vec<node::NodeId>;

        fn process(
            &self,
            node: &FakeNode,
            args: &TestArgSet,
            intermediates: &TestIntermediateSet,
        ) -> Result<TestIntermediateValue>;

        fn process_multiple<'a>(
            &self,
            nodes: &'a [&'a FakeNode],
            args: &plargs::GenericArgSet<TestArgValue>,
            intermediates: &TestIntermediateSet,
        ) -> Vec<systems::NodeResult<TestIntermediateValue>>;
    }
}
