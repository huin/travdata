use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::{
    config::{self, root::Config},
    extraction::tabulautil,
    filesio::{ReadWriter, Reader},
};

use super::{index::IndexWriter, tableextract::extract_table};

pub fn extract_book(
    tabula_client: &tabulautil::TabulaClient,
    cfg: &Config,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    book_name: &str,
    input_pdf: &Path,
) -> Result<()> {
    let book_cfg = cfg
        .books
        .get(book_name)
        .ok_or_else(|| anyhow!("book {:?} does not exist in the configuration", book_name))?;

    let top_group = book_cfg.load_group(cfg_reader)?;

    let mut index_writer =
        IndexWriter::new(out_writer).with_context(|| "opening index for update")?;

    process_group(
        tabula_client,
        cfg_reader,
        out_writer,
        &mut index_writer,
        book_cfg,
        &top_group,
        input_pdf,
    )?;

    index_writer
        .commit()
        .with_context(|| "commiting changes to the index")?;

    Ok(())
}

fn process_group(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    index_writer: &mut IndexWriter,
    book_cfg: &config::root::Book,
    grp: &config::book::Group,
    input_pdf: &Path,
) -> Result<()> {
    for (table_name, table_cfg) in &grp.tables {
        extract_table(
            tabula_client,
            cfg_reader,
            out_writer,
            index_writer,
            book_cfg,
            table_cfg,
            input_pdf,
        )
        .with_context(|| format!("processing table {:?}", table_name))?;
    }

    for (child_grp_name, child_grp) in &grp.groups {
        process_group(
            tabula_client,
            cfg_reader,
            out_writer,
            index_writer,
            book_cfg,
            child_grp,
            input_pdf,
        )
        .with_context(|| format!("processing group {:?}", child_grp_name))?;
    }

    Ok(())
}
