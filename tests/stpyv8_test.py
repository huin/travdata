# -*- coding: utf-8 -*-
"""Experiments with stpyv8."""
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import json
import hamcrest as hc
import STPyV8  # type: ignore[import-untyped]

from .extraction.pdf import pdftestutil


def test_simple_string_roundtrip() -> None:
    with STPyV8.JSContext() as ctx:
        upcase = ctx.eval(
            """
            ((lowerString) => {
                return lowerString.toUpperCase();
            })
            """
        )
        v = upcase("hello world!")
    hc.assert_that(v, hc.equal_to("HELLO WORLD!"))


def test_table_roundtrip() -> None:
    with STPyV8.JSContext() as ctx:
        concat_tables = ctx.eval(
            """
            ((ext_tables_json) => {
                const ext_tables = JSON.parse(ext_tables_json);
                const result = [];
                for (const ext_table of ext_tables) {
                    for (const row of ext_table.data) {
                        result.push(row);
                    }
                }
                return JSON.stringify(result);
            })
            """
        )
        result = json.loads(
            concat_tables(
                json.dumps(
                    [
                        pdftestutil.fake_table_data(),
                        pdftestutil.fake_table_data(),
                    ],
                )
            )
        )
    expected = pdftestutil.fake_table_data()["data"] + pdftestutil.fake_table_data()["data"]
    hc.assert_that(
        result,
        hc.equal_to(expected),
    )
