# -*- coding: utf-8 -*-
"""Utility wrapper for tabular-py."""

import json
import pathlib
from typing import IO, Self, TypeAlias, TypedDict, cast

import jpype  # type: ignore[import-untyped]
import tabula
from travdata.extraction.pdf import tablereader
from travdata.tabledata import TableData


class _TemplateEntry(TypedDict):
    """JSON type for a Tabula template entry."""

    page: int
    extraction_method: str
    x1: float
    x2: float
    y1: float
    y2: float
    width: float
    height: float


class _TabulaCell(TypedDict):
    """Subset of a JSON type for a table cell from Tabula."""

    text: str


# JSON type for a table row from Tabula.
_TabulaRow: TypeAlias = list[_TabulaCell]


class _TabulaTable(TypedDict):
    """Subset of a JSON type for a table from Tabula."""

    data: list[_TabulaRow]


class TabulaClient:
    """Client wrapper around Tabula.

    Note that:

    * This should only be created and/or shutdown on the main thread.
    * Only one instance can ever exist for the program lifetime, due to
    limitations in JPype.
    """

    _force_subprocess: bool

    def __init__(self, force_subprocess: bool) -> None:
        """Initialise the ``TabulaClient``.

        :param force_subprocess: Should Tabula be run as a child process, versus
        using the faster jpype.
        """
        self._force_subprocess = force_subprocess
        self._needs_shutdown = False

    def __enter__(self) -> Self:
        return self

    def __exit__(self, *args) -> None:
        del args  # unused
        self.shutdown()

    def shutdown(self) -> None:
        """Shutdown any resources being used.

        This must only be called from the main thread, this is also true for
        using a TabulaClient as a context manager.
        """
        if self._needs_shutdown:
            jpype.shutdownJVM()
            self._needs_shutdown = False

    def read_pdf_with_template(
        self,
        *,
        pdf_path: pathlib.Path,
        template_file: IO[str],
    ) -> list[tablereader.ExtractedTable]:
        """Reads table(s) from a PDF, based on the Tabula template.

        :param pdf_path: Path to PDF to read from.
        :param template_file: File-like reader for the Tabula template JSON
        file.
        :return: Tables read from the PDF.
        """
        self._needs_shutdown = not self._force_subprocess

        result: list[tablereader.ExtractedTable] = []
        template = cast(list[_TemplateEntry], json.load(template_file))

        for entry in template:
            method = entry["extraction_method"]

            raw_tables = self._read_pdf(
                input_path=pdf_path,
                pages=[entry["page"]],
                multiple_tables=True,
                area=[entry["y1"], entry["x1"], entry["y2"], entry["x2"]],
                force_subprocess=self._force_subprocess,
                stream=method == "stream",
                guess=method == "guess",
                lattice=method == "lattice",
            )

            # The raw tables from Tabula contain extraneous information that
            # isn't used elsewhere. Trim it down before returning it.

            table: TableData = []
            # Typically there should only be one entry per call to _read_pdf,
            # but flattening is easier than checking.
            for raw_table in raw_tables:
                for raw_row in raw_table["data"]:
                    table.append([raw_cell["text"] for raw_cell in raw_row])

            result.append({"page": entry["page"], "data": table})

        return result

    def _read_pdf(self, **kwargs) -> list[_TabulaTable]:
        return cast(
            list[_TabulaTable],
            tabula.read_pdf(  # pyright: ignore[reportPrivateImportUsage]
                java_options=["-Djava.awt.headless=true"],
                output_format="json",
                **kwargs,
            ),
        )
