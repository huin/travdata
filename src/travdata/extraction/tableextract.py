# -*- coding: utf-8 -*-
"""Extracts a single table from a PDF."""

import pathlib
from typing import Iterable, Iterator

from travdata import config, filesio
from travdata.config import cfgerror, cfgextract
from travdata.extraction import estransform, transforms
from travdata.extraction.pdf import tablereader
from travdata.tabledata import TableData


def extract_table(
    cfg_reader: filesio.Reader,
    table: config.Table,
    pdf_path: pathlib.Path,
    table_reader: tablereader.TableReader,
    ecmas_trn: estransform.ESTransformer,
) -> tuple[set[int], TableData]:
    """Extracts a table from the PDF.

    :cfg_reader: Configuration file reader.
    :param table: Configuration of the table to extract. ``table.extraction``
    must not be None.
    :param pdf_path: Path to the PDF to extract from.
    :param tabula_reader: Used to read the table from the PDF.
    :param ecmas_trn: Used to perform ECMAScript transforms.
    :returns: Set of page numbers and iterator over rows from the table.
    :raises ValueError: ``table.extraction`` is None.
    """
    if table.transform is None:
        raise ValueError(
            f"extract_table called with table with `None` extraction: {table=}",
        )

    with cfg_reader.open_read(table.tabula_template_path) as tmpl_file:
        ext_tables = table_reader.read_pdf_with_template(
            pdf_path=pdf_path,
            template_file=tmpl_file,
        )

    pages: set[int] = {t["page"] for t in ext_tables}

    tables = list(_table_data(ext_tables))

    match table.transform:
        case None:
            table_data = transforms.perform_transforms(
                transforms=[],
                tables=tables,
            )

        case cfgextract.LegacyTransformSeq() as cfg:
            table_data = transforms.perform_transforms(
                transforms=cfg.transforms,
                tables=tables,
            )

        case cfgextract.ESTransform() as cfg:
            table_data = ecmas_trn.transform(
                tables=tables,
                source=cfg.src,
            )

        case other:
            raise cfgerror.ConfigurationError(
                f"unhandled Table.transform type: {type(other).__name__}",
            )

    return pages, table_data


def _table_data(
    tables: Iterable[tablereader.ExtractedTable],
) -> Iterator[TableData]:
    for t in tables:
        yield t["data"]
