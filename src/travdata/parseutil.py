# -*- coding: utf-8 -*-
import re
from typing import Callable, Iterable, Iterator, Optional, TypeVar

T = TypeVar("T")


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


def clean_rows(rows: Iterable[list[str]]) -> Iterator[list[str]]:
    for row in rows:
        yield [clean_text(text) for text in row]


def map_opt_dict_key(t: Callable[[str], T], d: dict[str, str], k: str) -> Optional[T]:
    """Maps the given string value in d if present, otherwise returns None."""
    if k not in d:
        return None
    v = d[k]
    if not v:
        return None
    return t(v)


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


_EHEX_TO_INT: dict[str, int] = {}
_INT_TO_EHEX: list[str] = []


def __init_ehex() -> None:
    global _EHEX_TO_INT, _INT_TO_EHEX
    ranges: list[tuple[str, str]] = [
        ("0", "9"),
        ("A", "H"),
        ("J", "N"),
        ("P", "Z"),
    ]
    value: int = 0
    for start, end in ranges:
        istart = ord(start)
        iend = ord(end)
        for chr_ord in range(istart, iend + 1):
            c = chr(chr_ord)
            _INT_TO_EHEX.append(c)
            _EHEX_TO_INT[c] = value
            value += 1


__init_ehex()


def parse_ehex_char(c: str) -> int:
    """Parses a single Ehex digit."""
    # https://wiki.travellerrpg.com/Hexadecimal_Notation
    try:
        return _EHEX_TO_INT[c]
    except KeyError:
        raise ValueError(c)


def fmt_ehex_char(v: int) -> str:
    """Formats an integer in the Ehex digit range [0,33] to the Ehex digit."""
    if v < 0:
        raise ValueError(v)
    try:
        return _INT_TO_EHEX[v]
    except IndexError:
        raise ValueError(v)
