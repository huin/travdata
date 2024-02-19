# -*- coding: utf-8 -*-
import re
from typing import Callable, Iterable, Iterator


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
