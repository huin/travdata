# -*- coding: utf-8 -*-
import collections
import dataclasses
import json
import pathlib
import re
from typing import Callable, Iterable, Iterator, TypeAlias, TypedDict, cast

import tabula


class TablularCell(TypedDict):
    # Ignoring irrelevant fields.
    text: str


TabularRow: TypeAlias = list[TablularCell]


class TabluarTable(TypedDict):
    # Ignoring irrelevant fields.
    data: list[TabularRow]


def amalgamate_streamed_rows(
    rows: Iterable[dict[str, TablularCell]],
    continuation: Callable[[dict[str, TablularCell]], bool],
    join: str = "\n",
) -> Iterator[dict[str, str]]:
    row_accum: dict[str, list[str]] = collections.defaultdict(list)

    def form_row():
        return {label: join.join(cell) for label, cell in row_accum.items()}

    try:
        for row in rows:
            if not continuation(row) and row_accum:
                yield form_row()
                row_accum = collections.defaultdict(list)
            for label, cell in row.items():
                if text := cell["text"]:
                    row_accum[label].append(text)

        if row_accum:
            yield form_row()
    except Exception as e:
        e.add_note(f"for {row=}")
        raise


def headers_and_iter_rows(
    rows: Iterable[TabularRow],
) -> tuple[list[str], Iterator[TabularRow]]:
    rows_iter = iter(rows)
    header = row_text(next(rows_iter))
    return header, rows_iter


def clean_labelled_rows(rows: Iterable[dict[str, str]]) -> Iterator[dict[str, str]]:
    for row in rows:
        yield {label: clean_text(text) for label, text in row.items()}


def concat_rows(tables: list[TabluarTable]) -> list[TabularRow]:
    rows: list[TabularRow] = []
    for t in tables:
        rows.extend(t["data"])
    return rows


def label_rows(
    rows: Iterable[TabularRow],
    header: list[str],
) -> Iterator[dict[str, TablularCell]]:
    try:
        for row in rows:
            yield {label: cell for label, cell in zip(header, row)}
    except Exception as e:
        e.add_note(f"for {row=}")
        raise


def row_text(row: TabularRow) -> list[str]:
    return [cell["text"] for cell in row]


_WHITESPACE_RUN_RX = re.compile(r"\s+")


def clean_text(s: str) -> str:
    return _WHITESPACE_RUN_RX.sub(" ", s.strip())


def parse_set(s: str) -> set[str]:
    return {clean_text(v) for v in s.split(",")}


def parse_credits(s: str) -> int:
    if s.startswith("MCr"):
        return 1_000_000 * int(s.removeprefix("MCr"))
    elif s.startswith("Cr"):
        return int(s.removeprefix("Cr"))
    else:
        raise ValueError(s)


def d66_enum() -> Iterator[str]:
    for i in range(36):
        yield f"{1 + i // 6}{1 + i % 6}"


def read_pdf(*, pdf_path: pathlib.Path, pages: list[int]) -> list[TabluarTable]:
    return cast(
        list[TabluarTable],
        tabula.read_pdf(
            pdf_path,
            pages=pages,
            java_options=["-Djava.awt.headless=true"],
            multiple_tables=True,
            output_format="json",
            # jpype doesn't work for me.
            force_subprocess=True,
        ),
    )


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
                tabula.read_pdf(
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
