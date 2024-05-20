//! Code to create/update an index of output data.

use std::path::Path;

use anyhow::Result;

use crate::filesio::ReadWriter;

pub struct Writer {}

const INDEX_PATH: &str = "index.csv";

impl Writer {
    pub fn new(read_writer: &dyn ReadWriter) -> Result<Self> {
        // Read in any existing index so that we can append new entries.
        let _ = read_writer.open_read(Path::new(INDEX_PATH));
        todo!()
    }
}
