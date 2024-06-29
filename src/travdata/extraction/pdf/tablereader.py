# -*- coding: utf-8 -*-
"""Defines the ``TableReader`` protocol and related data types."""

import pathlib
from typing import IO, Protocol, TypedDict

from travdata import table


class ExtractedTable(TypedDict):
    """Type of tables emitted by TableReader."""

    page: int
    data: table.TableData


class TableReader(Protocol):
    """Required interface to extract a table from a PDF file."""

    def read_pdf_with_template(
        self,
        *,
        pdf_path: pathlib.Path,
        template_file: IO[str],
    ) -> list[ExtractedTable]:
        """Reads tables from a PDF file, using the named template file.

        :param pdf_path: Path to the PDF file.
        :param template_file: File-like reader for the Tabula template JSON
        file.
        :return: List of extracted tables.
        """
        raise NotImplementedError
