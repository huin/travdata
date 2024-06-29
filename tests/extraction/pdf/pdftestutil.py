# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import copy
import dataclasses
import pathlib
from typing import IO

from travdata.extraction.pdf import tablereader
from travdata.table import RowData, TableData


@dataclasses.dataclass(frozen=True)
class Call:
    pdf_path: pathlib.Path
    template_content: str


class FakeTableReader:
    calls: list[Call]
    return_tables: dict[Call, list[tablereader.ExtractedTable]]

    def __init__(self) -> None:
        self.calls = []
        self.return_tables = {}

    def read_pdf_with_template(
        self,
        *,
        pdf_path: pathlib.Path,
        template_file: IO[str],
    ) -> list[tablereader.ExtractedTable]:
        call = Call(pdf_path, template_file.read())
        self.calls.append(call)
        return copy.deepcopy(self.return_tables[call])


def tabula_table_from_simple(
    page: int,
    table: TableData,
) -> tablereader.ExtractedTable:
    return {"page": page, "data": table}


def fake_table_data(
    *,
    num_rows: int = 2,
    num_cols: int = 2,
    page: int = 1,
) -> tablereader.ExtractedTable:
    rows: TableData = []

    for ri in range(1, num_rows + 1):
        row: RowData = []
        for ci in range(1, num_cols + 1):
            row.append(f"r{ri}c{ci}")
        rows.append(row)

    return {"page": page, "data": rows}
