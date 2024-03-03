# -*- coding: utf-8 -*-
import json
import pathlib
from typing import Iterable, Iterator, TypeAlias, TypedDict, cast

import tabula


class TablularCell(TypedDict):
    # Ignoring irrelevant fields.
    text: str


TabularRow: TypeAlias = list[TablularCell]


class TabluarTable(TypedDict):
    # Ignoring irrelevant fields.
    data: list[TabularRow]


class _TemplateEntry(TypedDict):
    page: int
    extraction_method: str
    x1: float
    x2: float
    y1: float
    y2: float
    width: float
    height: float


def read_pdf_with_template(
    *,
    pdf_path: pathlib.Path,
    template_path: pathlib.Path,
) -> list[TabluarTable]:
    result: list[TabluarTable] = []
    with template_path.open() as tf:
        template = cast(list[_TemplateEntry], json.load(tf))

    for entry in template:
        method = entry["extraction_method"]
        result.extend(
            cast(
                list[TabluarTable],
                tabula.read_pdf(  # pyright: ignore[reportPrivateImportUsage]
                    pdf_path,
                    pages=[entry["page"]],
                    java_options=["-Djava.awt.headless=true"],
                    multiple_tables=True,
                    output_format="json",
                    area=[entry["y1"], entry["x1"], entry["y2"], entry["x2"]],
                    # jpype doesn't work for me.
                    force_subprocess=True,
                    stream=method == "stream",
                    guess=method == "guess",
                    lattice=method == "lattice",
                ),
            )
        )

    return result


def table_rows_concat(tables: Iterable[TabluarTable]) -> Iterator[TabularRow]:
    for t in tables:
        yield from t["data"]


def _table_row_text(row: TabularRow) -> list[str]:
    return [cell["text"] for cell in row]


def table_rows_text(rows: Iterable[TabularRow]) -> Iterator[list[str]]:
    for row in rows:
        yield _table_row_text(row)
