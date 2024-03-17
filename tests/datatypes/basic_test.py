# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import pytest
import testfixtures  # type: ignore[import-untyped]
from travdata.datatypes.basic import IntRange, IntRangeSet


@pytest.mark.parametrize(
    "str_range,int_range",
    [
        ("", IntRange(None, None)),
        ("1+", IntRange(1, None)),
        # Regular hyphens:
        ("3-", IntRange(None, 3)),
        ("1-3", IntRange(1, 3)),
        # Hyphens as found in Traveller PDFs:
        ("3\u2013", IntRange(None, 3)),
        ("1\u20133", IntRange(1, 3)),
    ],
)
def test_int_range_parse_roundtrip(str_range: str, int_range: IntRange) -> None:
    actual_parsed = IntRange.parse(str_range)
    testfixtures.compare(actual=actual_parsed, expected=int_range)
    if "\u2013" not in str_range:
        # This won't roundtrip - it normalises to the regular hyphen.
        assert str_range == str(actual_parsed)


@pytest.mark.parametrize(
    "str_range,in_range,out_range",
    [
        ("", [-10, 0, 5, 10], []),
        ("5+", [5, 10], [-10, 0]),
        ("5-", [-10, 0, 5], [10]),
        ("0-5", [0, 5], [-10, 10]),
    ],
)
def test_int_range_contains(str_range: str, in_range: list[int], out_range: list[int]) -> None:
    r = IntRange.parse(str_range)
    for v in in_range:
        assert v in r
    for v in out_range:
        assert v not in r


@pytest.mark.parametrize(
    "str_range,int_range_set",
    [
        ("", IntRangeSet([IntRange(None, None)])),
        ("1-3, 7+", IntRangeSet([IntRange(1, 3), IntRange(7, None)])),
    ],
)
def test_int_range_set_parse_roundtrip(str_range: str, int_range_set: IntRangeSet) -> None:
    actual_parsed = IntRangeSet.parse(str_range)
    testfixtures.compare(actual=actual_parsed, expected=int_range_set)
    assert str_range == str(actual_parsed)


@pytest.mark.parametrize(
    "str_range,in_range,out_range",
    [
        ("", [-10, 0, 5, 10], []),
        ("1-, 8+", [-10, 0, 10], [5]),
    ],
)
def test_int_range_set_contains(str_range: str, in_range: list[int], out_range: list[int]) -> None:
    r = IntRangeSet.parse(str_range)
    for v in in_range:
        assert v in r
    for v in out_range:
        assert v not in r
