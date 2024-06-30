# -*- coding: utf-8 -*-
"""Table-related type aliases used throughout the codebase."""

from typing import Any, TypeAlias

RowData: TypeAlias = list[str]
TableData: TypeAlias = list[RowData]


def check_table_type(rows: TableData) -> None:
    """Checks the ``TableData`` type at runtime."""
    want_type = "Array[Array[string]]"
    if not isinstance(rows, list):
        raise TypeError(
            f"want {want_type}, got . with type {_json_type_name(rows)}",
        )
    for i, row in enumerate(rows):
        if not isinstance(row, list):
            raise TypeError(
                f"want {want_type}, got .[{i}] with type {_json_type_name(row)}",
            )
        for j, cell in enumerate(row):
            if not isinstance(cell, str):
                raise TypeError(
                    f"want {want_type}, got .[{i}][{j}] with type {_json_type_name(cell)}",
                )


def _json_type_name(value: Any) -> str:
    # pylint: disable=too-many-return-statements
    match value:
        case dict():
            return "Object"
        case list():
            return "Array"
        case float() | int():
            return "number"
        case str():
            return "string"
        case bool():
            return "boolean"
        case None:
            return "null"
        case _:
            # Shouldn't happen from a plain json.loads.
            return f"Python type {type(value).__name__}"
