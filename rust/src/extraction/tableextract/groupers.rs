//! Grouping functions used by `fold_rows`.

use crate::{config::extract, table::Row};

use super::RowIterator;

pub fn group_rows(cfg: &extract::RowGrouper, rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    use extract::RowGrouper::*;
    match cfg {
        AllRows(_) => all_rows(rows),
        StaticRowCounts(cfg) => static_row_counts(cfg, rows),
        EmptyColumn(cfg) => empty_column(cfg, rows),
    }
}

fn all_rows(rows: &mut Box<RowIterator>) -> Vec<Vec<Row>> {
    vec![rows.collect()]
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
