//! Extracts a single table from a PDF.

use crate::config::extract;
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

pub fn apply_transforms(cfg: &extract::TableExtraction, table: Table) -> Table {
    let mut rows = table;
    for trn in &cfg.transforms {
        rows = transform(trn, rows);
    }
    rows
}

fn transform(cfg: &extract::TableTransform, table: Table) -> Table {
    use extract::TableTransform::*;
    match cfg {
        PrependRow(cfg) => prepend_row_iter(cfg, table),
        FoldRows(cfg) => fold_rows(cfg, table),
        other => {
            // Make the match exhaustive, rather than have this placeholder
            // default case.
            eprintln!(
                "Transform {:?} unhandled, passing through as identity.",
                other
            );
            table
        }
    }
}

fn prepend_row_iter(cfg: &extract::PrependRow, mut table: Table) -> Table {
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

#[test]
fn test_static_row_counts_with_shortfall() {
    let cfg = extract::StaticRowCounts {
        row_counts: vec![2, 1, 2],
    };
    let rows_in: Vec<Row> = vec![Row(vec!["first".to_string()])];
    let mut rows_iter: Box<RowIterator> = Box::new(rows_in.into_iter());

    let groups = static_row_counts(&cfg, &mut rows_iter);

    assert_eq!(groups, vec![vec![Row(vec!["first".to_string()]),],],);

    let remainder = vec![];
    assert_eq!(rows_iter.collect::<Vec<Row>>(), remainder,);
}

#[test]
fn test_static_row_counts_with_remainder() {
    let cfg = extract::StaticRowCounts {
        row_counts: vec![2, 1, 2],
    };
    let rows_in: Vec<Row> = vec![
        Row(vec!["first".to_string()]),
        Row(vec!["second".to_string()]),
        Row(vec!["third".to_string()]),
        Row(vec!["fourth".to_string()]),
        Row(vec!["fifth".to_string()]),
        Row(vec!["sixth".to_string()]),
    ];
    let mut rows_iter: Box<RowIterator> = Box::new(rows_in.into_iter());

    let groups = static_row_counts(&cfg, &mut rows_iter);

    assert_eq!(
        groups,
        vec![
            vec![
                Row(vec!["first".to_string()]),
                Row(vec!["second".to_string()]),
            ],
            vec![Row(vec!["third".to_string()])],
            vec![
                Row(vec!["fourth".to_string()]),
                Row(vec!["fifth".to_string()]),
            ],
        ],
    );

    let remainder = vec![Row(vec!["sixth".to_string()])];
    assert_eq!(rows_iter.collect::<Vec<Row>>(), remainder,);
}

fn empty_column(cfg: &extract::EmptyColumn, rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    let mut groups: Vec<Vec<Row>> = Vec::new();
    let mut group: Vec<Row> = Vec::new();
    for row in rows {
        match row.0.get(cfg.column_index).map(String::as_str) {
            // Cell is effectively empty in both cases:
            Some("") => {
                group.push(row);
            }
            None => {
                group.push(row);
            }
            // Cell is non-empty, starts new group.
            Some(_) => {
                groups.push(group);
                group = Vec::new();
            }
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
            // `cells` is used to join together the contents of each resulting cell.
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
                        cells.push(cell.as_str());
                    }
                }
                row_out.0.push(cells.join(" "));
            }

            table_out.0.push(row_out);
        }
    }

    table_out
}
