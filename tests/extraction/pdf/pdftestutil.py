# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import dataclasses
import pathlib
from typing import IO

from travdata.extraction.pdf import tablereader


@dataclasses.dataclass(frozen=True)
class Call:
    pdf_path: pathlib.Path
    template_content: str


class FakeTableReader:
    calls: list[Call]
    return_tables: list[tablereader.TabulaTable]

    def __init__(self) -> None:
        self.calls = []
        self.return_tables = []

    def read_pdf_with_template(
        self,
        *,
        pdf_path: pathlib.Path,
        template_file: IO[str],
    ) -> list[tablereader.TabulaTable]:
        self.calls.append(Call(pdf_path, template_file.read()))
        return self.return_tables


def tabula_table_from_simple(
    page_number: int,
    rows: list[list[str]],
) -> tablereader.TabulaTable:
    rows_out: list[tablereader.TabulaRow] = []

    for row_in in rows:
        cells_out: tablereader.TabulaRow = []
        rows_out.append(cells_out)
        for cell_in in row_in:
            cells_out.append({"text": cell_in})

    return {"page_number": page_number, "data": rows_out}


def fake_table_data(
    *,
    num_rows: int = 2,
    num_cols: int = 2,
    page_number: int = 1,
) -> tablereader.TabulaTable:
    rows: list[tablereader.TabulaRow] = []

    for ri in range(1, num_rows + 1):
        row: tablereader.TabulaRow = []
        for ci in range(1, num_cols + 1):
            row.append({"text": f"r{ri}c{ci}"})
        rows.append(row)

    return {"page_number": page_number, "data": rows}
