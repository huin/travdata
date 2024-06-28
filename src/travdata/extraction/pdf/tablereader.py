# -*- coding: utf-8 -*-
"""Defines the ``TableReader`` protocol and related data types."""

import pathlib
from typing import IO, Protocol, TypeAlias, TypedDict


class TabulaCell(TypedDict):
    """Type of table cells emitted by tabula-py."""

    # Ignoring irrelevant fields.
    text: str


# Type of table rows emitted by tabula-py.
TabulaRow: TypeAlias = list[TabulaCell]


class TabulaTable(TypedDict):
    """Type of tables emitted by tabula-py."""

    # Ignoring irrelevant fields.
    data: list[TabulaRow]


class TableReader(Protocol):
    """Required interface to extract a table from a PDF file.

    :param Protocol: _description_
    """

    def read_pdf_with_template(
        self,
        *,
        pdf_path: pathlib.Path,
        template_file: IO[str],
    ) -> tuple[set[int], list[TabulaTable]]:
        """Reads tables from a PDF file, using the named template file.

        :param pdf_path: Path to the PDF file.
        :param template_file: File-like reader for the Tabula template JSON
        file.
        :return: Set of page numbers and list of extracted tables.
        """
        raise NotImplementedError
