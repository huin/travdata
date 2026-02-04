use googletest::prelude::*;
use serde::{Deserialize, Serialize};
use test_casing::{TestCases, cases, test_casing};

use crate::{node, testutil::node_id};

const CASES: TestCases<(&'static str, FakeNode)> = cases! {
    [
        (
            r#"
id: foo
type: Foo
spec:
    value: foo-value
    deps: []
            "#,
            FakeNode {
                id: node_id("foo"),
                tags: Default::default(),
                public: false,
                spec: FooSpec {
                    value: "foo-value".into(),
                    ..Default::default()
                }.into(),
            },
        ),
        (
            r#"
id: bar
type: Bar
spec:
    value: bar-value
    deps:
        - foo
            "#,
            FakeNode {
                id: node_id("bar"),
                tags: Default::default(),
                public: false,
                spec: BarSpec {
                    value: "bar-value".into(),
                    deps: vec![node_id("foo")],
                }.into(),
            },
        ),
    ]
};

#[test]
fn test_cases_len() {
    assert_eq!(2, CASES.into_iter().count());
}

#[test_casing(2, CASES)]
#[gtest]
fn test_reserialise_case(input: &'static str, expected: FakeNode) -> Result<()> {
    let got_1: FakeNode = serde_yaml_ng::from_str(input)?;
    expect_that!(got_1, eq(&expected));

    let reserialised = serde_yaml_ng::to_string(&got_1)?;
    let got_2: FakeNode = serde_yaml_ng::from_str(&reserialised)?;
    expect_that!(got_2, eq(&expected));

    Ok(())
}

// This approach may never be used (might use a more compact representation than YAML), but for now
// keeping it as a reference example.
#[gtest]
fn test_deserialise_multi_doc() -> Result<()> {
    const INPUT: &str = r#"
id: foo
type: Foo
spec:
    value: foo-value
    deps: []
---
id: bar
type: Bar
spec:
  value: bar-value
  deps:
    - foo
"#;

    for document in serde_yaml_ng::Deserializer::from_str(INPUT) {
        let _node = FakeNode::deserialize(document)?;
    }

    Ok(())
}

pub type FakeNode = node::GenericNode<FakeSpec>;

/// Per-type wrapper of a specific type of extraction configuration node.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "spec")]
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
