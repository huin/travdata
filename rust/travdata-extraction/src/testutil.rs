use std::{ffi::OsStr, path::Path};

use anyhow::Result;
use mockall::mock;
use serde::{Deserialize, Serialize};

use crate::{
    intermediates,
    node::{self, spec_type},
    processargs, processparams, systems,
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

impl FakeSpec {
    pub fn default_foo() -> Self {
        FakeSpec::Foo(FooSpec {
            value: "foo-value".into(),
        })
    }

    pub fn default_bar() -> Self {
        FakeSpec::Bar(BarSpec {
            value: "bar-value".into(),
        })
    }
}

impl node::SpecTrait for FakeSpec {
    type Discrim = FakeSpecDiscriminants;

    fn discriminant(&self) -> Self::Discrim {
        self.into()
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FooSpec {
    value: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BarSpec {
    value: String,
}

pub type FakeNode = node::GenericNode<FakeSpec>;

impl node::GenericNode<FakeSpec> {
    pub fn default_with_spec(spec: FakeSpec) -> Self {
        Self {
            id: node_id("foo"),
            tags: Default::default(),
            public: false,
            spec,
        }
    }
}

mock! {
    pub FakeSystem {}

    impl systems::GenericSystem<FakeSpec> for FakeSystem {
        fn params(&self, node: &FakeNode) -> Option<processparams::NodeParams>;

        fn inputs(&self, node: &FakeNode) -> Vec<node::NodeId>;

        fn process(
            &self,
            node: &FakeNode,
            args: &processargs::ArgSet,
            intermediates: &intermediates::IntermediateSet,
        ) -> Result<intermediates::Intermediate>;

        fn process_multiple<'a>(
            &self,
            nodes: &'a [&'a FakeNode],
            args: &processargs::ArgSet,
            intermediates: &intermediates::IntermediateSet,
        ) -> Vec<(node::NodeId, Result<intermediates::Intermediate>)>;
    }
}
