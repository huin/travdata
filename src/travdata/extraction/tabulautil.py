# -*- coding: utf-8 -*-
"""Utility wrapper for tabular-py."""

import json
import pathlib
import tempfile
from typing import IO, Iterable, Iterator, TypeAlias, TypedDict, cast

import jpype  # type: ignore[import-untyped]
import tabula


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


class _TemplateEntry(TypedDict):
    page: int
    extraction_method: str
    x1: float
    x2: float
    y1: float
    y2: float
    width: float
    height: float


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

    def __enter__(self) -> "TabulaClient":
        # Hack to get tabula initialised from the main thread, otherwise the
        # application may not quit when multi-threaded (such as in a GUI).
        with tempfile.NamedTemporaryFile() as tmpfile:
            tmpfile.write(b"dummy data")
            tmpfile.flush()
            try:
                _ = self._read_pdf(input_path=tmpfile.name, pages=[1])
            except Exception:  # pylint: disable=broad-exception-caught
                # Expected failure due to dummy file not being a real PDF.
                pass

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
    ) -> tuple[set[int], list[TabulaTable]]:
        """Reads table(s) from a PDF, based on the Tabula template.

        :param pdf_path: Path to PDF to read from.
        :param template_file: File-like reader for the Tabula template JSON
        file.
        :return: Page numbers and tables read from the PDF.
        """
        self._needs_shutdown = not self._force_subprocess

        result: list[TabulaTable] = []
        template = cast(list[_TemplateEntry], json.load(template_file))

        pages: set[int] = set()

        for entry in template:
            method = entry["extraction_method"]
            pages.add(int(entry["page"]))
            result.extend(
                cast(
                    list[TabulaTable],
                    self._read_pdf(
                        input_path=pdf_path,
                        pages=[entry["page"]],
                        multiple_tables=True,
                        area=[entry["y1"], entry["x1"], entry["y2"], entry["x2"]],
                        force_subprocess=self._force_subprocess,
                        stream=method == "stream",
                        guess=method == "guess",
                        lattice=method == "lattice",
                    ),
                )
            )

        return pages, result

    def _read_pdf(self, **kwargs) -> list[TabulaTable]:
        return cast(
            list[TabulaTable],
            tabula.read_pdf(  # pyright: ignore[reportPrivateImportUsage]
                java_options=["-Djava.awt.headless=true"], output_format="json", **kwargs
            ),
        )


def table_rows_concat(tables: Iterable[TabulaTable]) -> Iterator[TabulaRow]:
    """Concatenates rows from multiple Tabula tables into a single row iterator.

    :param tables: Tables to concatenate rows from.
    :yield: Rows from the tables.
    """
    for t in tables:
        yield from t["data"]


def _table_row_text(row: TabulaRow) -> list[str]:
    return [cell["text"] for cell in row]


def table_rows_text(rows: Iterable[TabulaRow]) -> Iterator[list[str]]:
    """Converts Tabula row dictionaries into simple lists of cells.

    :param rows: Tabula rows to read from.
    :yield: Lists of strings, each presenting the text from the cell.
    """
    for row in rows:
        yield _table_row_text(row)
