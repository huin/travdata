//! Misc internal functions used by tableextract.

use std::{cmp::min, ops::Range};

use lazy_regex::regex;

use crate::table::Row;

pub type RowIterator = dyn Iterator<Item = Row>;

pub fn intersect_range(len: usize, from: Option<usize>, to: Option<usize>) -> Option<Range<usize>> {
    let from = min(len, from.unwrap_or(0));
    let to = min(len, to.unwrap_or(len));

    if from < to { Some(from..to) } else { None }
}

/// Replace Python `\1` style replacements with Rust regex `${1}` style.
pub fn replace_replacements(s: &str) -> String {
    let r = regex!(r"\\(g<)?([0-9]+)(?:>)?");
    r.replace_all(s, |captures: &regex::Captures| {
        match (captures.get(1), captures.get(2)) {
            (Some(_), Some(num)) => format!("${{{}}}", num.as_str()),
            (None, Some(num)) => format!("${{{}}}", num.as_str()),
            _ => panic!("should never not match one of the above cases"),
        }
    })
    .to_string()
}

pub struct CellExpansions {
    expansions: Vec<String>,
}

impl CellExpansions {
    pub fn new(srcs: &[String]) -> Self {
        Self {
            expansions: srcs.iter().map(|s| replace_replacements(s)).collect(),
        }
    }

    pub fn expand_from_capture<'a>(
        &'a self,
        captures: &'a regex::Captures,
    ) -> impl Iterator<Item = String> + 'a {
        self.expansions.iter().map(|repl| {
            let mut dst = String::default();
            captures.expand(repl, &mut dst);
            dst
        })
    }
}

#[cfg(test)]
mod tests {
    use ::googletest::{
        expect_that,
        matchers::{eq, none, some},
    };

    use super::*;

    #[googletest::test]
    fn test_replace_replacements() {
        let actual = replace_replacements(r"foo \1b bar \2 baz \123 quux");
        expect_that!(actual, eq("foo ${1}b bar ${2} baz ${123} quux"));
    }

    #[googletest::test]
    fn test_replace_replacements_g() {
        let actual = replace_replacements(r"\g<0> \g<1>");
        expect_that!(actual, eq("${0} ${1}"));
    }

    #[googletest::test]
    fn test_intersect_range() {
        expect_that!(intersect_range(10, None, None), some(eq(&(0..10))));
        expect_that!(intersect_range(10, Some(3), Some(5)), some(eq(&(3..5))));
        expect_that!(intersect_range(10, None, Some(5)), some(eq(&(0..5))));
        expect_that!(intersect_range(10, Some(3), None), some(eq(&(3..10))));
        expect_that!(intersect_range(10, Some(3), Some(12)), some(eq(&(3..10))));
        expect_that!(intersect_range(10, Some(13), Some(15)), none());
        // from == to
        expect_that!(intersect_range(10, Some(3), Some(3)), none());
        // from > to
        expect_that!(intersect_range(10, Some(5), Some(3)), none());
        // len == 0
        expect_that!(intersect_range(0, Some(1), None), none());
        expect_that!(intersect_range(0, None, Some(1)), none());
        expect_that!(intersect_range(0, None, None), none());
    }
}
