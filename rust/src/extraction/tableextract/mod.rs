//! Extracts a single table from a PDF.

pub mod bookextract;
pub mod groupers;
mod internal;

use anyhow::Result;
use lazy_regex::regex;
use serde::Deserialize;

use crate::extraction::parseutil::clean_text;
use crate::table::{Row, Table};

type RowIterator = dyn Iterator<Item = Row>;

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
/// Configures the specifics of extracting the CSV from the PDF.
pub struct TableExtraction {
    pub transforms: Vec<TableTransform>,
}

/// Applies the transformations specified in `cfg`.
pub fn apply_transforms(cfg: &TableExtraction, table: Table) -> Result<Table> {
    let mut rows = table;
    for trn in &cfg.transforms {
        rows = transform(trn, rows)?;
    }

    clean_table(&mut rows);

    Ok(rows)
}

/// Concatenates the given tables into a single `Table`.
pub fn concat_tables(tables: Vec<Table>) -> Table {
    Table(
        tables
            .into_iter()
            .flat_map(|table| table.0.into_iter())
            .collect(),
    )
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

#[derive(Deserialize, Debug)]
/// Supported table transformation operations.
pub enum TableTransform {
    ExpandColumnOnRegex(ExpandColumnOnRegex),
    FoldRows(FoldRows),
    JoinColumns(JoinColumns),
    PrependRow(PrependRow),
    Transpose(Transpose),
    WrapRowEveryN(WrapRowEveryN),
}

fn transform(cfg: &TableTransform, table: Table) -> Result<Table> {
    use TableTransform::*;
    match cfg {
        ExpandColumnOnRegex(cfg) => expand_column_on_regex(cfg, table),
        FoldRows(cfg) => Ok(fold_rows(cfg, table)),
        JoinColumns(cfg) => Ok(join_columns(cfg, table)),
        PrependRow(cfg) => Ok(prepend_row(cfg, table)),
        Transpose(_) => Ok(transpose(table)),
        WrapRowEveryN(cfg) => Ok(wrap_row_every_n(cfg, table)),
    }
}

#[derive(Deserialize, Debug)]
/// Splits a column by the matches of a regex.
pub struct ExpandColumnOnRegex {
    pub column: usize,
    pub pattern: String,
    // When `pattern` matches (using `Pattern.fullmatch`), `on_match` produces
    // the resulting cells, using groups from the match to the pattern. Each
    // string is expanded using `Match.expand`, see
    // https://docs.python.org/3/library/re.html#match-objects.
    pub on_match: Vec<String>,
    // When `pattern` does not match, default produces cells as if matching on a
    // regex ".*" when `pattern` does not match. Similarly, each string is
    // expanded using `Match.expand` (using \g<0> makes sense here to extract the
    // entire original text into a cell).
    pub default: Vec<String>,
}
fn expand_column_on_regex(cfg: &ExpandColumnOnRegex, mut table: Table) -> Result<Table> {
    let pattern = regex::Regex::new(&cfg.pattern)?;

    let on_match = internal::CellExpansions::new(&cfg.on_match);
    let default = internal::CellExpansions::new(&cfg.default);

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

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Folds rows, according to the given sequence of groupings.
pub struct FoldRows {
    pub group_by: Vec<groupers::RowGrouper>,
}

fn fold_rows(cfg: &FoldRows, table: Table) -> Table {
    let mut table_out = Table::default();

    // Can we get away without boxing this?
    let mut rows_iter: Box<RowIterator> = Box::new(table.0.into_iter());

    // Could we change groupers from returning vec of groups to instead taking a
    // FnMut that pushes each group to the table? (fewer allocations)

    for group_cfg in &cfg.group_by {
        let groups = groupers::group_rows(group_cfg, &mut rows_iter);
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

#[derive(Deserialize, Debug)]
/// Joins a range of columns.
pub struct JoinColumns {
    #[serde(default = "Default::default")]
    pub from: Option<usize>,
    #[serde(default = "Default::default")]
    pub to: Option<usize>,
    #[serde(default = "Default::default")]
    pub delim: String,
}

fn join_columns(cfg: &JoinColumns, mut table: Table) -> Table {
    // `joiner`'s allocation is reused to join cells.
    let mut joiner: Vec<String> = Vec::new();

    for row in &mut table.0 {
        match internal::intersect_range(row.len(), cfg.from, cfg.to) {
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

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Appends given literal row values to the start of a table.
pub struct PrependRow(pub Vec<String>);

fn prepend_row(cfg: &PrependRow, mut table: Table) -> Table {
    table.0.insert(0, cfg.0.clone().into());
    table
}

#[derive(Deserialize, Debug)]
/// Transposes the table (rows become columns and vice versa).
pub struct Transpose {}

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

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Wraps a row every N columns.
pub struct WrapRowEveryN {
    pub num_columns: usize,
}

fn wrap_row_every_n(cfg: &WrapRowEveryN, table: Table) -> Table {
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
    #[cfg(test)]
    mod apply_transforms {
        use googletest::matchers::ok;
        use googletest::{expect_that, matchers::eq};

        use crate::extraction::tableextract::TableExtraction;
        use crate::table::Table;

        use super::super::apply_transforms;

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
