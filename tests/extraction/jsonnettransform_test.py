# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring


import hamcrest as hc

from travdata.config import cfgextract
from travdata.extraction import jsonnettransform
from travdata.tabledata import TableData
from .pdf import pdftestutil


def test_perform_transform() -> None:
    """Uses a Jsonnet expression to produce a valid output."""
    extracted_tables = [
        pdftestutil.tabula_table_from_simple(
            page=1,
            table=[
                ["Number"],
                ["1"],
                ["2"],
            ],
        ),
        pdftestutil.tabula_table_from_simple(
            page=1,
            table=[
                ["3"],
                ["4"],
            ],
        ),
    ]

    cfg = cfgextract.JsonnetExtraction(
        code="""
        function(tables)
            local rows = std.flattenArrays(tables);
            local header = rows[0];
            local tail = rows[1:];
            [header + ["Square"]] +
            [
                local v = std.parseInt(row[0]);
                [row[0], std.toString(v * v)]
                for row in tail
            ]
        """,
    )

    result: TableData = jsonnettransform.perform_transforms(
        cfg=cfg,
        extracted_tables=extracted_tables,
    )

    hc.assert_that(
        result,
        hc.equal_to(
            [
                ["Number", "Square"],
                ["1", "1"],
                ["2", "4"],
                ["3", "9"],
                ["4", "16"],
            ]
        ),
    )
