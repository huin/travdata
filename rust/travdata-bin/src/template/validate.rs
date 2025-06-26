//! Validation functions for [crate::template] types.

use anyhow::{Context, Result, bail};
use lazy_regex::regex;

/// Validates a name for a [crate::template::Group].
pub fn group_name(name: &str) -> Result<()> {
    as_base_file_name(name).context("group name")
}

/// Validates a name for a [crate::template::Table].
pub fn table_name(name: &str) -> Result<()> {
    as_base_file_name(name).context("table name")
}

/// Validates a tag for a [crate::template::Group] or [crate::template::Table].
fn as_base_file_name(name: &str) -> Result<()> {
    let re = regex!(r#"^[a-z0-9_-]+$"#);
    if !re.is_match(name) {
        bail!("must only contain characters a-Z, 0-9, -, and _");
    }
    Ok(())
}

pub fn tag(tag: &str) -> Result<()> {
    let re = regex!(r#"^[a-z0-9_/-]+$"#);
    if !re.is_match(tag) {
        bail!("must only contain characters a-Z, 0-9, /, -, and _");
    }
    if tag.starts_with('/') || tag.ends_with('/') {
        bail!("must not start or end with '/'");
    }
    if tag.contains("//") {
        bail!("must not contain more than one consecutive '/'");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;

    const STRINGS_WITH_PUNCTUATION: &[&str] = &[
        "with,punctuation",
        "with\"punctuation",
        "with\\punctuation",
        "with'punctuation",
        "with punctuation",
        "with{punctuation",
        "with(punctuation",
        "with#punctuation",
        "with&punctuation",
        "with=punctuation",
        "with+punctuation",
        "with.punctuation",
    ];

    #[googletest::test]
    fn test_table_name() {
        expect_that!(table_name("valid-name-123"), ok(()));
        expect_that!(table_name("valid_name_123"), ok(()));
        expect_that!(table_name("invalid/name"), err(anything()));
        for s in STRINGS_WITH_PUNCTUATION {
            expect_that!(table_name(s), err(anything()));
        }
    }

    #[googletest::test]
    fn test_tag() {
        expect_that!(tag("valid_tag"), ok(()));
        expect_that!(tag("valid_tag_123"), ok(()));
        expect_that!(tag("valid-tag"), ok(()));
        expect_that!(tag("valid/tag"), ok(()));
        expect_that!(tag("another/valid/tag"), ok(()));
        expect_that!(tag("another/valid_tag"), ok(()));
        expect_that!(tag("/invalid/tag"), err(anything()));
        expect_that!(tag("invalid/tag/"), err(anything()));
        expect_that!(tag("invalid//tag"), err(anything()));
        for s in STRINGS_WITH_PUNCTUATION {
            expect_that!(tag(s), err(anything()));
        }
    }
}
