# -*- coding: utf-8 -*-
import json
import pathlib
import re
from typing import Callable, Iterable, Iterator, TypeAlias, TypedDict, cast

import tabula


def amalgamate_streamed_rows(
    rows: Iterable[list[str]],
    continuation: Callable[[int, list[str]], bool],
    join: str = "\n",
) -> Iterator[list[str]]:
    row_accum: list[list[str]] = []
 
    def form_row():
        return [join.join(cell) for cell in row_accum]
 
    try:
        for i, row in enumerate(rows):
            if not continuation(i, row) and row_accum:
                yield form_row()
                row_accum = []
            missing_count = len(row) - len(row_accum)
            if missing_count > 0:
                for _ in range(missing_count):
                    row_accum.append([])
            for acc, text in zip(row_accum, row):
                if text:
                    acc.append(text)

        if row_accum:
            yield form_row()
    except Exception as e:
        e.add_note(f"for {row=}")
        raise


def headers_and_iter_rows(
    rows: Iterable[list[str]],
) -> tuple[list[str], Iterator[list[str]]]:
    rows_iter = iter(rows)
    header = next(rows_iter)
    return header, rows_iter


def clean_rows(rows: Iterable[list[str]]) -> Iterator[list[str]]:
    for row in rows:
        yield [clean_text(text) for text in row]


def label_rows(
    rows: Iterable[list[str]],
    header: list[str],
) -> Iterator[dict[str, str]]:
    try:
        for row in rows:
            yield {label: text for label, text in zip(header, row)}
    except Exception as e:
        e.add_note(f"for {row=}")
        raise


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


class TablularCell(TypedDict):
    # Ignoring irrelevant fields.
    text: str


TabularRow: TypeAlias = list[TablularCell]


class TabluarTable(TypedDict):
    # Ignoring irrelevant fields.
    data: list[TabularRow]


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


def table_rows_concat(tables: list[TabluarTable]) -> list[TabularRow]:
    rows: list[TabularRow] = []
    for t in tables:
        rows.extend(t["data"])
    return rows


def table_row_text(row: TabularRow) -> list[str]:
    return [cell["text"] for cell in row]


def table_rows_text(rows: Iterable[TabularRow]) -> Iterator[list[str]]:
    for row in rows:
        yield table_row_text(row)
