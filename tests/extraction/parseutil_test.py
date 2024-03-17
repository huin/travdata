# -*- coding: utf-8 -*-
# pylint: disable=missing-function-docstring,missing-module-docstring

import pytest
from travdata.extraction import parseutil


@pytest.mark.parametrize(
    "inp,want",
    [
        ("0", 0),
        ("5", 5),
        ("9", 9),
        ("A", 10),
        ("H", 17),
        ("J", 18),
        ("N", 22),
        ("P", 23),
        ("Z", 33),
    ],
)
def test_parse_ehex_char_parse_valid(inp: str, want: int) -> None:
    got = parseutil.parse_ehex_char(inp)
    assert got == want


@pytest.mark.parametrize(
    "inp",
    [
        "I",
        "O",
        "a",
        "h",
        ".",
    ],
)
def test_parse_ehex_char_parse_invalid(inp: str) -> None:
    with pytest.raises(ValueError):
        parseutil.parse_ehex_char(inp)


@pytest.mark.parametrize(
    "inp,want",
    [
        (0, "0"),
        (5, "5"),
        (9, "9"),
        (10, "A"),
        (17, "H"),
        (18, "J"),
        (22, "N"),
        (23, "P"),
        (33, "Z"),
    ],
)
def test_parse_ehex_char_fmt_valid(inp: int, want: str) -> None:
    got = parseutil.fmt_ehex_char(inp)
    assert got == want


@pytest.mark.parametrize(
    "inp",
    [
        -3,
        34,
    ],
)
def test_parse_ehex_char_fmt_invalid(inp: int) -> None:
    with pytest.raises(ValueError):
        parseutil.fmt_ehex_char(inp)
