use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::{
    config::{self, root::Config},
    extraction::tabulautil,
    filesio::{ReadWriter, Reader},
};

use super::tableextract::extract_table;

pub fn extract_book(
    tabula_client: &tabulautil::TabulaClient,
    cfg: &Config,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    book_name: &str,
    input_pdf: &Path,
) -> Result<()> {
    let top_group = cfg
        .books
        .get(book_name)
        .ok_or_else(|| anyhow!("book {:?} does not exist in the configuration", book_name))?
        .load_group(cfg_reader)?;

    process_group(tabula_client, cfg_reader, out_writer, &top_group, input_pdf)
}

fn process_group(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    grp: &config::book::Group,
    input_pdf: &Path,
) -> Result<()> {
    for (table_name, table_cfg) in &grp.tables {
        extract_table(tabula_client, cfg_reader, out_writer, table_cfg, input_pdf)
            .with_context(|| format!("processing table {:?}", table_name))?;
    }

    for (child_grp_name, child_grp) in &grp.groups {
        process_group(tabula_client, cfg_reader, out_writer, child_grp, input_pdf)
            .with_context(|| format!("processing group {:?}", child_grp_name))?;
    }

    Ok(())
}
