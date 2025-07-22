use std::{ffi::OsStr, path::Path};

use anyhow::Result;
use mockall::mock;
use serde::{Deserialize, Serialize};

use crate::{
    intermediates,
    node::{self, spec_type},
    plargs, plparams, processing, systems,
};

pub fn node_id(s: &str) -> node::NodeId {
    s.to_string().try_into().expect("expected valid Id value")
}

pub fn output_path_buf<S: AsRef<OsStr> + ?Sized>(s: &S) -> spec_type::OutputPathBuf {
    Path::new(s)
        .to_owned()
        .try_into()
        .expect("expected valid OutputPathBufValue")
}

pub fn tag(s: &str) -> node::Tag {
    s.to_string().try_into().expect("expected valid Tag value")
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

mock! {
    pub FakeSystem {}

    impl systems::GenericSystem<FakeSpec> for FakeSystem {
        fn params(&self, node: &FakeNode) -> plparams::Params;

        fn inputs(&self, node: &FakeNode) -> Vec<node::NodeId>;

        fn process(
            &self,
            node: &FakeNode,
            args: &plargs::ArgSet,
            intermediates: &intermediates::IntermediateSet,
        ) -> Result<intermediates::Intermediate>;

        fn process_multiple<'a>(
            &self,
            nodes: &'a [&'a FakeNode],
            args: &plargs::ArgSet,
            intermediates: &intermediates::IntermediateSet,
        ) -> Vec<(node::NodeId, Result<intermediates::Intermediate>)>;
    }
}

pub type TestPipeline = processing::GenericPipeline<FakeSpec>;

pub type TestProcessor = processing::GenericProcessor<FakeSpec>;
