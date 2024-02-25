#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import unittest

from travellerutil import parseutil


class ParseEhex(unittest.TestCase):

    def test_parse_valid(self) -> None:
        cases: list[tuple[str, int]] = [
            # (input, want)
            ("0", 0),
            ("5", 5),
            ("9", 9),
            ("A", 10),
            ("H", 17),
            ("J", 18),
            ("N", 22),
            ("P", 23),
            ("Z", 33),
        ]
        for inp, want in cases:
            with self.subTest(inp):
                got = parseutil.parse_ehex_char(inp)
                self.assertEqual(got, want)

    def test_parse_invalid(self) -> None:
        cases: list[str] = [
            "I",
            "O",
            "a",
            "h",
            ".",
        ]
        for inp in cases:
            with self.assertRaises(ValueError):
                parseutil.parse_ehex_char(inp)

    def test_fmt_valid(self) -> None:
        cases: list[tuple[int, str]] = [
            # (input, want)
            (0 , "0"),
            (5 , "5"),
            (9 , "9"),
            (10, "A"),
            (17, "H"),
            (18, "J"),
            (22, "N"),
            (23, "P"),
            (33, "Z"),
        ]
        for inp, want in cases:
            with self.subTest(inp):
                got = parseutil.fmt_ehex_char(inp)
                self.assertEqual(got, want)

    def test_fmt_invalid(self) -> None:
        cases: list[int] = [
            -3,
            34,
        ]
        for inp in cases:
            with self.assertRaises(ValueError):
                parseutil.fmt_ehex_char(inp)


if __name__ == "__main__":
    unittest.main()
