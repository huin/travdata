use googletest::prelude::*;
use test_casing::{TestCases, cases, test_casing};

use super::*;

const NODE_ID_VALID_CASES: TestCases<&'static str> = cases! {
    [
        "foo",
        "foo-bar",
        "foo-123",
        "123",
    ]
};

#[test]
fn test_node_id_valid_cases_len() {
    assert_eq!(4, NODE_ID_VALID_CASES.into_iter().count());
}

#[test_casing(4, NODE_ID_VALID_CASES)]
#[gtest]
fn test_node_id_valid(input: &'static str) -> anyhow::Result<()> {
    let expected = NodeId::new_unchecked(input.into());

    expect_that!(NodeId::try_from(input), ok(eq(&expected)));
    expect_that!(NodeId::try_from(input.to_string()), ok(eq(&expected)));

    let input_json = serde_json::to_string(input)?;
    expect_that!(
        serde_json::from_str::<NodeId>(&input_json),
        ok(eq(&expected))
    );

    Ok(())
}

const NODE_ID_INVALID_CASES: TestCases<&'static str> = cases! {
    [
        "",
        "&foo",
        "fo o",
        "fo_o",
        "fo+o",
        "fo:o",
        "foo-",
        "-foo",
    ]
};

#[test]
fn test_node_id_invalid_cases_len() {
    assert_eq!(8, NODE_ID_INVALID_CASES.into_iter().count());
}

#[test_casing(8, NODE_ID_INVALID_CASES)]
#[gtest]
fn test_node_id_invalid(input: &'static str) -> anyhow::Result<()> {
    expect_that!(NodeId::try_from(input), err(anything()));
    expect_that!(NodeId::try_from(input.to_string()), err(anything()));

    let input_json = serde_json::to_string(input)?;
    expect_that!(serde_json::from_str::<NodeId>(&input_json), err(anything()));

    Ok(())
}

const TAG_VALID_CASES: TestCases<&'static str> = cases! {
    [
        "foo",
        "foo-bar",
        "foo-123",
        "123",
        "foo/bar-123/baz",
    ]
};

#[test]
fn test_tag_valid_cases_len() {
    assert_eq!(5, TAG_VALID_CASES.into_iter().count());
}

#[test_casing(5, TAG_VALID_CASES)]
#[gtest]
fn test_tag_valid(input: &'static str) -> anyhow::Result<()> {
    let expected = Tag::new_unchecked(input.into());

    expect_that!(Tag::try_from(input), ok(eq(&expected)));
    expect_that!(Tag::try_from(input.to_string()), ok(eq(&expected)));

    let input_json = serde_json::to_string(input)?;
    expect_that!(serde_json::from_str::<Tag>(&input_json), ok(eq(&expected)));

    Ok(())
}

const TAG_INVALID_CASES: TestCases<&'static str> = cases! {
    [
        "",
        "&foo",
        "fo o",
        "fo_o",
        "fo+o",
        "fo:o",
        "foo-",
        "-foo",
        "/foo",
        "foo/",
        "foo//bar",
    ]
};

#[test]
fn test_tag_invalid_cases_len() {
    assert_eq!(11, TAG_INVALID_CASES.into_iter().count());
}

#[test_casing(11, TAG_INVALID_CASES)]
#[gtest]
fn test_tag_invalid(input: &'static str) -> anyhow::Result<()> {
    expect_that!(Tag::try_from(input), err(anything()));
    expect_that!(Tag::try_from(input.to_string()), err(anything()));

    let input_json = serde_json::to_string(input)?;
    expect_that!(serde_json::from_str::<Tag>(&input_json), err(anything()));

    Ok(())
}
