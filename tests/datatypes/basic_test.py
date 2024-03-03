# -*- coding: utf-8 -*-

import unittest

import testfixtures  # type: ignore[import-untyped]
from travdata.datatypes.basic import IntRange, IntRangeSet


class IntRangeTest(unittest.TestCase):

    def test_parse_roundtrip(self) -> None:
        test_cases: list[tuple[str,  IntRange]] = [
            ("", IntRange(None, None)),
            ("1+", IntRange(1, None)),
            # Regular hyphens:
            ("3-", IntRange(None, 3)),
            ("1-3", IntRange(1, 3)),
            # Hyphens as found in Traveller PDFs:
            ("3\u2013", IntRange(None, 3)),
            ("1\u20133", IntRange(1, 3)),
        ]
        for str_range, int_range in test_cases:
            with self.subTest(str_range):
                actual_parsed = IntRange.parse(str_range)
                testfixtures.compare(actual=actual_parsed, expected=int_range)
                if "\u2013" not in str_range:
                    # This won't roundtrip - it normalises to the regular hyphen.
                    self.assertEqual(str_range, str(actual_parsed))

    def test_contains(self) -> None:
        test_cases: list[tuple[str, list[int], list[int]]] = [
            ("", [-10, 0, 5, 10], []),
            ("5+", [5, 10], [-10, 0]),
            ("5-", [-10, 0, 5], [10]),
            ("0-5", [0, 5], [-10, 10]),
        ]
        for str_range, in_range, out_range in test_cases:
            r = IntRange.parse(str_range)
            for v in in_range:
                with self.subTest(f"{v} in {r}"):
                    self.assertIn(v, r)
            for v in out_range:
                with self.subTest(f"{v} not in {r}"):
                    self.assertNotIn(v, r)


class IntRangeSetTest(unittest.TestCase):

    def test_parse_roundtrip(self) -> None:
        test_cases: list[tuple[str,  IntRangeSet]] = [
            ("", IntRangeSet([IntRange(None, None)])),
            ("1-3, 7+", IntRangeSet([IntRange(1, 3), IntRange(7, None)])),
        ]
        for str_range, int_range_set in test_cases:
            with self.subTest(str_range):
                actual_parsed = IntRangeSet.parse(str_range)
                testfixtures.compare(actual=actual_parsed, expected=int_range_set)
                self.assertEqual(str_range, str(actual_parsed))

    def test_contains(self) -> None:
        test_cases: list[tuple[str, list[int], list[int]]] = [
            ("", [-10, 0, 5, 10], []),
            ("1-, 8+", [-10, 0, 10], [5]),
        ]
        for str_range, in_range, out_range in test_cases:
            r = IntRangeSet.parse(str_range)
            for v in in_range:
                with self.subTest(f"{v} in {r}"):
                    self.assertIn(v, r)
            for v in out_range:
                with self.subTest(f"{v} not in {r}"):
                    self.assertNotIn(v, r)


if __name__ == "__main__":
    unittest.main()
