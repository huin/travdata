use std::path::Path;

use anyhow::{Context, Result};

use crate::{
    config::self,
    extraction::{tableextract, tabulautil},
    filesio::{ReadWriter, Reader},
};

pub fn process_group(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    grp: &config::book::Group,
    input_pdf: &Path,
) -> Result<()> {
    for (table_name, table_cfg) in &grp.tables {
        process_table(tabula_client, cfg_reader, out_writer, table_cfg, input_pdf)
            .with_context(|| format!("processing table {:?}", table_name))?;
    }

    for (child_grp_name, child_grp) in &grp.groups {
        process_group(tabula_client, cfg_reader, out_writer, child_grp, input_pdf)
            .with_context(|| format!("processing group {:?}", child_grp_name))?;
    }

    Ok(())
}

fn process_table(
    tabula_client: &tabulautil::TabulaClient,
    cfg_reader: &dyn Reader,
    out_writer: &dyn ReadWriter,
    table_cfg: &config::book::Table,
    input_pdf: &Path,
) -> Result<()> {
    if !table_cfg.extraction_enabled {
        return Ok(());
    }

    let mut csv_file = out_writer.open_write(&table_cfg.file_stem.with_extension("csv"))?;
    let mut csv_writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(&mut csv_file);

    let tmpl_path = table_cfg.tabula_template_path();

    let extracted_tables = tabula_client
        .read_pdf_with_template(cfg_reader, input_pdf, &tmpl_path)
        .with_context(|| format!("extracting table from PDF {:?}", input_pdf))?;
    let table = tableextract::concat_tables(extracted_tables.tables);
    let table = tableextract::apply_transforms(&table_cfg.extraction, table)?;

    for row in table.0 {
        csv_writer
            .write_record(&row.0)
            .with_context(|| "writing record")?;
    }

    // Check for error rather than implicitly flushing and ignoring.
    csv_writer.flush().with_context(|| "flushing to CSV")?;
    drop(csv_writer);
    csv_file.commit().with_context(|| "committing CSV file")?;

    Ok(())
}
