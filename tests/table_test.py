# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

from typing import Any
import pytest

from travdata import tabledata


@pytest.mark.parametrize(
    "rows",
    [
        [],
        [
            [],
            [],
        ],
        [
            [""],
            [],
        ],
        [
            ["r1c1", "r1c2"],
            ["r2c1", "r2c2"],
        ],
    ],
)
def test_check_rows_type_valid(rows: list[list[str]]) -> None:
    tabledata.check_table_type(rows)


@pytest.mark.parametrize(
    "value",
    [
        "string as rows",
        ["string as row"],
        [[1]],
        [[["string in list as cell"]]],
    ],
)
def test_check_rows_type_invalid(value: Any) -> None:
    with pytest.raises(TypeError):
        tabledata.check_table_type(value)
