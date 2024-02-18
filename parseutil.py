# -*- coding: utf-8 -*-
import pathlib
import re
from typing import Iterator, TypeAlias, TypedDict, cast

import tabula


class TablularCell(TypedDict):
    # Ignoring irrelevant fields.
    text: str


TabularRow: TypeAlias = list[TablularCell]


class TabluarTable(TypedDict):
    # Ignoring irrelevant fields.
    data: list[TabularRow]


_WHITESPACE_RUN_RX = re.compile(r"\s+")


def clean_text(s: str) -> str:
    return _WHITESPACE_RUN_RX.sub(" ", s.strip())


def parse_set(s: str) -> set[str]:
    return {clean_text(v) for v in s.split(",")}


def parse_credits(s: str) -> int:
    return int(s.removeprefix("Cr"))


def d66_enum() -> Iterator[str]:
    for i in range(36):
        yield f"{1 + i // 6}{1 + i % 6}"


def read_pdf(*, pdf_path: pathlib.Path, pages: list[int]) -> list[TabluarTable]:
    return cast(list[TabluarTable], tabula.read_pdf(
        pdf_path,
        pages=pages,
        java_options=["-Djava.awt.headless=true"],
        multiple_tables=True,
        output_format="json",
        # jpype doesn't work for me.
        force_subprocess=True,
    ))
