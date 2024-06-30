# -*- coding: utf-8 -*-
"""Extracts a single table from a PDF."""

import pathlib

from travdata import config, filesio
from travdata.extraction import transforms
from travdata.extraction.pdf import tablereader
from travdata.table import TableData


def extract_table(
    cfg_reader: filesio.Reader,
    table: config.Table,
    pdf_path: pathlib.Path,
    table_reader: tablereader.TableReader,
) -> tuple[set[int], TableData]:
    """Extracts a table from the PDF.

    :cfg_reader: Configuration file reader.
    :param table: Configuration of the table to extract. ``table.extraction``
    must not be None.
    :param pdf_path: Path to the PDF to extract from.
    :param tabula_reader: Used to read the table from the PDF.
    :returns: Set of page numbers and iterator over rows from the table.
    :raises ValueError: ``table.extraction`` is None.
    """
    if table.extraction is None:
        raise ValueError(
            f"extract_table called with table with `None` extraction: {table=}",
        )

    with cfg_reader.open_read(table.tabula_template_path) as tmpl_file:
        tables = table_reader.read_pdf_with_template(
            pdf_path=pdf_path,
            template_file=tmpl_file,
        )

    pages: set[int] = {t["page"] for t in tables}

    table_data = transforms.perform_transforms(
        cfg=table.extraction,
        tables=tables,
    )

    return pages, table_data
