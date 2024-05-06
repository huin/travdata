use std::ops::{Deref, DerefMut};

use crate::extraction::tabulautil;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Table(pub Vec<Row>);

impl Deref for Table {
    type Target = Vec<Row>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Table {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<tabulautil::JsonTable> for Table {
    fn from(value: tabulautil::JsonTable) -> Self {
        Table(value.data.into_iter().map(Into::into).collect())
    }
}

impl<C, R> From<C> for Table
where
    C: IntoIterator<Item = R>,
    R: Into<Row>,
{
    fn from(value: C) -> Self {
        Table(value.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Row(pub Vec<String>);

impl Deref for Row {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Row {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<tabulautil::JsonRow> for Row {
    fn from(value: tabulautil::JsonRow) -> Self {
        Row(value.0.into_iter().map(|cell| cell.text).collect())
    }
}

impl<C, S> From<C> for Row
where
    C: IntoIterator<Item = S>,
    S: Into<String>,
{
    fn from(value: C) -> Self {
        Row(value.into_iter().map(Into::into).collect())
    }
}
