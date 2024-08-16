use std::{
    io::Write,
    ops::{Deref, DerefMut},
};

use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};

use crate::extraction::tabulautil;

#[allow(dead_code)]
type CsvResult<T> = std::result::Result<T, csv::Error>;

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Table(pub Vec<Row>);

impl Table {
    #[allow(dead_code)]
    /// Reads a [Table] encoded as CSV.
    pub fn read_csv(r: &mut dyn std::io::Read) -> Result<Self> {
        let mut csv_reader = csv::ReaderBuilder::new().flexible(true).from_reader(r);
        let rows: Vec<Row> = csv_reader
            .records()
            .map(|record_result| {
                record_result.map(|record| Row(record.iter().map(str::to_owned).collect()))
            })
            .collect::<CsvResult<Vec<Row>>>()?;

        Ok(Self(rows))
    }

    /// Writes the [Table], encoding as CSV.
    pub fn write_csv(&self, w: &mut dyn Write) -> Result<()> {
        let mut csv_writer = csv::WriterBuilder::new().flexible(true).from_writer(w);
        for row in &self.0 {
            csv_writer
                .write_record(&row.0)
                .with_context(|| "writing record")?;
        }
        csv_writer.flush().with_context(|| "flushing to CSV")?;
        drop(csv_writer);
        Ok(())
    }
}

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

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
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
