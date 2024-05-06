//! Extracts a single table from a PDF.

use std::cmp::min;
use std::ops::Range;

use anyhow::Result;
use lazy_regex::regex;

use crate::config::extract;
use crate::extraction::parseutil::clean_text;
use crate::table::{Row, Table};

type RowIterator = dyn Iterator<Item = Row>;

pub fn concat_tables(tables: Vec<Table>) -> Table {
    Table(
        tables
            .into_iter()
            .flat_map(|table| table.0.into_iter())
            .collect(),
    )
}

pub fn apply_transforms(cfg: &extract::TableExtraction, table: Table) -> Result<Table> {
    let mut rows = table;
    for trn in &cfg.transforms {
        rows = transform(trn, rows)?;
    }

    clean_table(&mut rows);

    Ok(rows)
}

/// Clean leading, trailing, and redundant sequences of whitespace within the
/// `Table`, in-place.
fn clean_table(table: &mut Table) {
    for row in table.iter_mut() {
        for cell in row.iter_mut() {
            clean_text(cell);
        }
    }
}

fn transform(cfg: &extract::TableTransform, table: Table) -> Result<Table> {
    use extract::TableTransform::*;
    match cfg {
        ExpandColumnOnRegex(cfg) => expand_column_on_regex(cfg, table),
        FoldRows(cfg) => Ok(fold_rows(cfg, table)),
        JoinColumns(cfg) => Ok(join_columns(cfg, table)),
        PrependRow(cfg) => Ok(prepend_row(cfg, table)),
        Transpose(_) => Ok(transpose(table)),
        WrapRowEveryN(cfg) => Ok(wrap_row_every_n(cfg, table)),
    }
}

/// Replace Python `\1` style replacements with Rust regex `${1}` style.
fn replace_replacements(s: &str) -> String {
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

struct CellExpansions {
    expansions: Vec<String>,
}

impl CellExpansions {
    fn new(srcs: &[String]) -> Self {
        Self {
            expansions: srcs.iter().map(|s| replace_replacements(s)).collect(),
        }
    }

    fn expand_from_capture<'a>(
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

fn expand_column_on_regex(cfg: &extract::ExpandColumnOnRegex, mut table: Table) -> Result<Table> {
    let pattern = regex::Regex::new(&cfg.pattern)?;

    let on_match = CellExpansions::new(&cfg.on_match);
    let default = CellExpansions::new(&cfg.default);

    for row in &mut table.0 {
        let cell = match row.get_mut(cfg.column) {
            None => {
                // Specified column not present. Leave as-is.
                continue;
            }
            Some(vec_cell) => {
                // Steal the original value from the vector for replacement.
                // This allows a later splice on the row vector without keeping
                // a borrowed ref to a member inside it, which we're replacing
                // anyway.
                let mut cell = String::default();
                std::mem::swap(&mut cell, vec_cell);
                cell
            }
        };

        match pattern.captures(&cell) {
            Some(captures) => {
                // Replace with the expansions in `on_match`.
                row.splice(
                    cfg.column..cfg.column + 1,
                    on_match.expand_from_capture(&captures),
                );
            }
            None => {
                // Fall back to expansions in `default`.
                let captures = regex!(r".*")
                    .captures(&cell)
                    .expect("must match any string");
                row.splice(
                    cfg.column..cfg.column + 1,
                    default.expand_from_capture(&captures),
                );
            }
        }
    }

    Ok(table)
}

fn prepend_row(cfg: &extract::PrependRow, mut table: Table) -> Table {
    table.0.insert(0, cfg.0.clone().into());
    table
}

fn all_rows(rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    vec![rows.collect()]
}

fn static_row_counts(cfg: &extract::StaticRowCounts, rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    let mut groups = Vec::with_capacity(cfg.row_counts.len());
    for count in &cfg.row_counts {
        let mut group = Vec::with_capacity(*count);
        for _ in 0..*count {
            match rows.next() {
                Some(row) => group.push(row),
                None => {
                    groups.push(group);
                    return groups;
                }
            }
        }
        groups.push(group);
    }

    groups
}

fn empty_column(cfg: &extract::EmptyColumn, rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    let mut groups: Vec<Vec<Row>> = Vec::new();
    let mut group: Vec<Row> = Vec::new();
    for row in rows {
        if row.0.get(cfg.column_index).map_or(true, String::is_empty) {
            // Cell is empty or absent - continues the group:
            group.push(row);
        } else {
            // Cell is non-empty, starts new group.
            if !group.is_empty() {
                groups.push(group);
            }
            group = vec![row];
        }
    }
    if !group.is_empty() {
        groups.push(group);
    }
    groups
}

fn group_rows(cfg: &extract::RowGrouper, rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    use extract::RowGrouper::*;
    match cfg {
        AllRows(_) => all_rows(rows),
        StaticRowCounts(cfg) => static_row_counts(cfg, rows),
        EmptyColumn(cfg) => empty_column(cfg, rows),
    }
}

fn fold_rows(cfg: &extract::FoldRows, table: Table) -> Table {
    let mut table_out = Table::default();

    // Can we get away without boxing this?
    let mut rows_iter: Box<RowIterator> = Box::new(table.0.into_iter());

    // Could we change groupers from returning vec of groups to instead taking a
    // FnMut that pushes each group to the table? (fewer allocations)

    for group_cfg in &cfg.group_by {
        let groups = group_rows(group_cfg, &mut rows_iter);
        for group in groups {
            // `cells` is used to join together the contents of each resulting
            // cell. Keep it here to reuse the allocation.
            let mut cells: Vec<&str> = Vec::with_capacity(group.len());

            let row_len = match group.iter().map(|row| row.0.len()).max() {
                Some(max) => max,
                None => continue,
            };

            // Compose a new row from the joined cells of the respective columns
            // in the group.
            let mut row_out = Row(Vec::with_capacity(row_len));
            for col in 0..row_len {
                for row_idx in 0..group.len() {
                    let row_in = group.get(row_idx).expect("must be in bounds");
                    if let Some(cell) = row_in.0.get(col) {
                        cells.push(cell.as_str().trim());
                    }
                }
                row_out.0.push(cells.join(" "));
                cells.clear();
            }

            table_out.0.push(row_out);
        }
    }

    // Pass through everything else without folding.
    table_out.0.extend(rows_iter);

    table_out
}

fn intersect_range(len: usize, from: Option<usize>, to: Option<usize>) -> Option<Range<usize>> {
    let from = min(len, from.unwrap_or(0));
    let to = min(len, to.unwrap_or(len));

    if from < to {
        Some(from..to)
    } else {
        None
    }
}

fn join_columns(cfg: &extract::JoinColumns, mut table: Table) -> Table {
    // `joiner`'s allocation is reused to join cells.
    let mut joiner: Vec<String> = Vec::new();

    for row in &mut table.0 {
        match intersect_range(row.len(), cfg.from, cfg.to) {
            None => {
                // Range does not affect any columns. Leave as-is.
                continue;
            }
            Some(rng) => {
                joiner.extend(row.splice(rng.clone(), ["".to_string()]));
                row[rng.start] = joiner.join(&cfg.delim);
                joiner.clear();
            }
        }
    }

    table
}

fn transpose(table: Table) -> Table {
    let orig_num_cols: usize = table.iter().map(|row| row.len()).max().unwrap_or(0);
    let orig_num_rows: usize = table.len();

    let mut out_table = Table(Vec::with_capacity(orig_num_cols));

    // Preallocate in a rectangle of empty strings.
    for _ in 0..orig_num_cols {
        let mut row: Vec<String> = Vec::with_capacity(orig_num_rows);
        row.resize(orig_num_rows, String::new());
        out_table.push(Row(row))
    }

    for (orig_row, row) in table.0.into_iter().enumerate() {
        for (orig_col, cell) in row.0.into_iter().enumerate() {
            out_table[orig_col][orig_row] = cell;
        }
    }

    out_table
}

fn wrap_row_every_n(cfg: &extract::WrapRowEveryN, table: Table) -> Table {
    let num_cells: usize = table.iter().map(|row| row.len()).sum();
    let num_out_rows = num_cells / cfg.num_columns
        + if num_cells % cfg.num_columns > 0 {
            1
        } else {
            0
        };

    let mut out_table = Table(Vec::with_capacity(num_out_rows));
    let mut out_row = Vec::with_capacity(cfg.num_columns);
    for row in table.0 {
        for cell in row.0 {
            out_row.push(cell);
            if out_row.len() >= cfg.num_columns {
                out_table.0.push(Row(out_row));
                out_row = Vec::with_capacity(cfg.num_columns);
            }
        }
    }

    if !out_row.is_empty() {
        out_table.push(Row(out_row));
    }

    out_table
}

#[cfg(test)]
mod tests {
    use googletest::{
        expect_that,
        matchers::{eq, none, some},
    };

    use crate::extraction::tableextract::{intersect_range, replace_replacements};

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
        expect_that!(intersect_range(10, None, None), some(eq(0..10)));
        expect_that!(intersect_range(10, Some(3), Some(5)), some(eq(3..5)));
        expect_that!(intersect_range(10, None, Some(5)), some(eq(0..5)));
        expect_that!(intersect_range(10, Some(3), None), some(eq(3..10)));
        expect_that!(intersect_range(10, Some(3), Some(12)), some(eq(3..10)));
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

    mod test_apply_transforms {
        use googletest::matchers::ok;
        use googletest::{expect_that, matchers::eq};

        use crate::{config::extract::*, table::Table};

        use crate::extraction::tableextract::apply_transforms;

        #[googletest::test]
        /// Base behaviour with default config.
        fn with_default_config() {
            test_apply_transforms_case(
                r#"[]"#,
                &[&["header 1", "header 2"], &["r1c1", "r1c2"]],
                &[&["header 1", "header 2"], &["r1c1", "r1c2"]],
            );
        }

        #[googletest::test]
        fn adds_specified_leading_row() {
            test_apply_transforms_case(
                r#"
                - !PrependRow
                    - "added header 1"
                    - "added header 2"
                "#,
                &[&["r1c1", "r1c2"], &["r2c1", "r2c2"]],
                &[
                    &["added header 1", "added header 2"],
                    &["r1c1", "r1c2"],
                    &["r2c1", "r2c2"],
                ],
            );
        }

        #[googletest::test]
        /// Merges specified header rows, and keeps individual rows thereafter.
        fn merges_static_header_rows_keeps_individual_rows_thereafter() {
            test_apply_transforms_case(
                r#"
                - !FoldRows
                    - !StaticRowCounts [2]
                "#,
                &[
                    &["header 1-1", "header 2-1"],
                    &["header 1-2", "header 2-2"],
                    &["r1c1", "r1c2"],
                    &["r2c1", "r2c2"],
                ],
                &[
                    &["header 1-1 header 1-2", "header 2-1 header 2-2"],
                    &["r1c1", "r1c2"],
                    &["r2c1", "r2c2"],
                ],
            );
        }

        #[googletest::test]
        /// Merges rows based on configured StaticRowCounts.
        fn merges_rows_based_on_static_row_counts() {
            test_apply_transforms_case(
                r#"
                - !FoldRows
                    - !StaticRowCounts [2, 2, 2]
                "#,
                &[
                    &["", "header 2-1"],
                    &["header 1", "header 2-2"],
                    &["r1c1", "r1c2"],
                    &["", "r2c2"],
                    &["r3c1", "r3c2"],
                    &["r4c1", ""],
                    &["r5c1", "r5c2"],
                ],
                &[
                    &["header 1", "header 2-1 header 2-2"],
                    &["r1c1", "r1c2 r2c2"],
                    &["r3c1 r4c1", "r3c2"],
                    &["r5c1", "r5c2"],
                ],
            )
        }

        #[googletest::test]
        /// Merges rows based on configured leading StaticRowLengths and EmptyColumn thereafter.
        fn merges_leading_static_row_counts_and_then_empty_column_thereafter() {
            test_apply_transforms_case(
                r#"
                - !FoldRows
                    - !StaticRowCounts [2]
                    - !EmptyColumn 0
                "#,
                &[
                    &["", "header 2-1"],
                    &["header 1", "header 2-2"],
                    &["r1c1", "r1c2"],
                    &["", "r2c2"],
                    &["r3c1", "r3c2"],
                    &["r4c1", ""],
                    &["r5c1", "r5c2"],
                ],
                &[
                    &["header 1", "header 2-1 header 2-2"],
                    &["r1c1", "r1c2 r2c2"],
                    &["r3c1", "r3c2"],
                    &["r4c1", ""],
                    &["r5c1", "r5c2"],
                ],
            );
        }

        #[googletest::test]
        /// Fold all rows.
        fn fold_all_rows() {
            test_apply_transforms_case(
                r#"
                - !FoldRows
                    - !AllRows {}
                "#,
                &[
                    &["r1c1", "r1c2", "r1c3"],
                    &["r2c1", "r2c2"],
                    &["r3c1", "r3c2", "r3c3"],
                ],
                &[&["r1c1 r2c1 r3c1", "r1c2 r2c2 r3c2", "r1c3 r3c3"]],
            );
        }

        #[googletest::test]
        /// Splits a column by the matches of a regex.
        fn split_column_by_regex_matches() {
            test_apply_transforms_case(
                r#"
                - !ExpandColumnOnRegex
                    column: 1
                    pattern: '[*] +([^:]+): +(.+)'
                    on_match: ['\1', '\2']
                    default: ['', '\g<0>']
                "#,
                &[
                    &["r1c1", "* label 1: text 1", "last col"],
                    &["r2c1", "* label 2: text 2", "last col"],
                    &["r3c1", "continuation text", "last col"],
                    &["r4c1", "* text 4: without last col"],
                    &["r5c1"], // Row without split column.
                    &[],       // Empty row.
                ],
                &[
                    &["r1c1", "label 1", "text 1", "last col"],
                    &["r2c1", "label 2", "text 2", "last col"],
                    &["r3c1", "", "continuation text", "last col"],
                    &["r4c1", "text 4", "without last col"],
                    &["r5c1"],
                    &[], // Empty row.
                ],
            );
        }

        #[googletest::test]
        /// Joins a range of columns - from+to set.
        fn join_columns_from_to_set() {
            test_apply_transforms_case(
                r#"
                - !JoinColumns
                    from: 1
                    to: 3
                    delim: " "
                "#,
                &[
                    &["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    &["r2c1", "r2c2", "r2c3", "r2c4"],
                    &["r3c1", "r3c2", "r3c3"],
                    &["r4c1", "r4c2"],
                    &["r5c1"],
                    &[],
                ],
                &[
                    &["r1c1", "r1c2 r1c3", "r1c4", "r1c5"],
                    &["r2c1", "r2c2 r2c3", "r2c4"],
                    &["r3c1", "r3c2 r3c3"],
                    &["r4c1", "r4c2"],
                    &["r5c1"],
                    &[],
                ],
            );
        }

        #[googletest::test]
        /// Joins a range of columns - from set.
        fn joins_columns_from_set() {
            test_apply_transforms_case(
                r#"
                - !JoinColumns
                    from: 1
                    delim: " "
                "#,
                &[
                    &["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    &["r2c1", "r2c2", "r2c3", "r2c4"],
                    &["r3c1", "r3c2", "r3c3"],
                    &["r4c1", "r4c2"],
                    &["r5c1"],
                    &[],
                ],
                &[
                    &["r1c1", "r1c2 r1c3 r1c4 r1c5"],
                    &["r2c1", "r2c2 r2c3 r2c4"],
                    &["r3c1", "r3c2 r3c3"],
                    &["r4c1", "r4c2"],
                    &["r5c1"],
                    &[],
                ],
            );
        }

        #[googletest::test]
        /// Joins a range of columns - to set.
        fn joins_columns_to_set() {
            test_apply_transforms_case(
                r#"
                - !JoinColumns
                    to: 3
                    delim: " "
                "#,
                &[
                    &["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    &["r2c1", "r2c2", "r2c3", "r2c4"],
                    &["r3c1", "r3c2", "r3c3"],
                    &["r4c1", "r4c2"],
                    &["r5c1"],
                    &[],
                ],
                &[
                    &["r1c1 r1c2 r1c3", "r1c4", "r1c5"],
                    &["r2c1 r2c2 r2c3", "r2c4"],
                    &["r3c1 r3c2 r3c3"],
                    &["r4c1 r4c2"],
                    &["r5c1"],
                    &[],
                ],
            );
        }

        #[googletest::test]
        /// Joins a range of columns - neither from/to set set.
        fn joins_columns_neither_set() {
            test_apply_transforms_case(
                r#"
                - !JoinColumns
                    delim: " "
                "#,
                &[
                    &["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    &["r2c1", "r2c2", "r2c3", "r2c4"],
                    &["r3c1", "r3c2", "r3c3"],
                    &["r4c1", "r4c2"],
                    &["r5c1"],
                    &[],
                ],
                &[
                    &["r1c1 r1c2 r1c3 r1c4 r1c5"],
                    &["r2c1 r2c2 r2c3 r2c4"],
                    &["r3c1 r3c2 r3c3"],
                    &["r4c1 r4c2"],
                    &["r5c1"],
                    &[],
                ],
            );
        }

        #[googletest::test]
        /// Transposes a table.
        fn transpose() {
            test_apply_transforms_case(
                r#"
                - !Transpose {}
                "#,
                &[
                    &["r1c1", "r1c2", "r1c3"],
                    &["r2c1", "r2c2"],
                    &["r3c1", "r3c2", "r3c3"],
                ],
                &[
                    &["r1c1", "r2c1", "r3c1"],
                    &["r1c2", "r2c2", "r3c2"],
                    &["r1c3", "", "r3c3"],
                ],
            );
        }

        #[googletest::test]
        /// Wraps a row every N columns.
        fn wraps_row_every_n() {
            test_apply_transforms_case(
                r#"
                - !WrapRowEveryN 2
                "#,
                &[
                    &["r1c1", "r1c2", "r1c3", "r1c4"],
                    &["r2c1", "r2c2", "r2c3", "r2c4", "r2c5"],
                    &["r3c1", "r3c2", "r3c3"],
                    &[],
                    &["r5c1"],
                ],
                &[
                    &["r1c1", "r1c2"],
                    &["r1c3", "r1c4"],
                    &["r2c1", "r2c2"],
                    &["r2c3", "r2c4"],
                    &["r2c5", "r3c1"],
                    &["r3c2", "r3c3"],
                    &["r5c1"],
                ],
            );
        }

        fn test_apply_transforms_case(
            cfg_str: &str,
            table_in_str: &[&[&str]],
            table_expected_str: &[&[&str]],
        ) {
            let cfg: TableExtraction = serde_yaml_ng::from_str(cfg_str).unwrap();

            let table_in: Table = table_in_str.iter().map(|r| r.into_iter().copied()).into();
            let table_expected: Table = table_expected_str
                .iter()
                .map(|r| r.into_iter().copied())
                .into();

            let table_out = apply_transforms(&cfg, table_in);

            expect_that!(table_out, ok(eq(table_expected)));
        }
    }
}
