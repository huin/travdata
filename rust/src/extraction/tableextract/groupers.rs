//! Grouping configuration and functions used by `fold_rows`.

use serde::Deserialize;

use super::RowIterator;
use crate::table::Row;

#[derive(Deserialize, Debug)]
/// Suported configuring row grouping operations.
pub enum RowGrouper {
    AllRows(AllRows),
    EmptyColumn(EmptyColumn),
    StaticRowCounts(StaticRowCounts),
}

pub fn group_rows(cfg: &RowGrouper, rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    use RowGrouper::*;
    match cfg {
        AllRows(_) => all_rows(rows),
        StaticRowCounts(cfg) => static_row_counts(cfg, rows),
        EmptyColumn(cfg) => empty_column(cfg, rows),
    }
}

#[derive(Deserialize, Debug)]
/// Specifies to group all remaining rows.
pub struct AllRows {}

fn all_rows(rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    vec![rows.collect()]
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Specifies to group rows by when a given column is empty.
pub struct EmptyColumn {
    pub column_index: usize,
}

fn empty_column(cfg: &EmptyColumn, rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
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

#[derive(Deserialize, Debug)]
#[serde(transparent)]
/// Specifies explicit input row counts for output grouped rows.
pub struct StaticRowCounts {
    pub row_counts: Vec<usize>,
}

fn static_row_counts(cfg: &StaticRowCounts, rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
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
