# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import pathlib

import hamcrest as hc
import pytest
import testfixtures  # type: ignore[import-untyped]

from travdata import config, filesio
from travdata import tabledata
from travdata.config import cfgextract
from travdata.extraction import tableextract
from .pdf import pdftestutil


@pytest.mark.parametrize(
    "name,extract_cfg,tables_in,expected",
    [
        (
            "Base behaviour with default config.",
            cfgextract.TableExtraction(),
            [
                [
                    ["header 1", "header 2"],
                    ["r1c1", "r1c2"],
                ],
            ],
            [
                ["header 1", "header 2"],
                ["r1c1", "r1c2"],
            ],
        ),
        (
            "Concatenates input tables.",
            cfgextract.TableExtraction(),
            [
                [
                    ["header 1", "header 2"],
                    ["r1c1", "r1c2"],
                ],
                [
                    ["r2c1", "r2c2"],
                    ["r3c1", "r3c2"],
                ],
            ],
            [
                ["header 1", "header 2"],
                ["r1c1", "r1c2"],
                ["r2c1", "r2c2"],
                ["r3c1", "r3c2"],
            ],
        ),
        (
            "Adds specified leading row.",
            cfgextract.TableExtraction(
                transforms=[cfgextract.PrependRow(["added header 1", "added header 2"])]
            ),
            [
                [
                    ["r1c1", "r1c2"],
                    ["r2c1", "r2c2"],
                ],
            ],
            [
                ["added header 1", "added header 2"],
                ["r1c1", "r1c2"],
                ["r2c1", "r2c2"],
            ],
        ),
        (
            "Merges specified header rows, and keeps individual rows thereafter.",
            cfgextract.TableExtraction(
                transforms=[
                    cfgextract.FoldRows(
                        [
                            cfgextract.StaticRowCounts([2]),
                        ]
                    ),
                ],
            ),
            [
                [
                    ["header 1-1", "header 2-1"],
                    ["header 1-2", "header 2-2"],
                    ["r1c1", "r1c2"],
                    ["r2c1", "r2c2"],
                ],
            ],
            [
                ["header 1-1 header 1-2", "header 2-1 header 2-2"],
                ["r1c1", "r1c2"],
                ["r2c1", "r2c2"],
            ],
        ),
        (
            "Merges rows based on configured StaticRowLengths.",
            cfgextract.TableExtraction(
                transforms=[
                    cfgextract.FoldRows(
                        [
                            cfgextract.StaticRowCounts([2, 2, 2]),
                        ]
                    ),
                ],
            ),
            [
                [
                    ["", "header 2-1"],
                    ["header 1", "header 2-2"],
                    ["r1c1", "r1c2"],
                    ["", "r2c2"],
                    ["r3c1", "r3c2"],
                    ["r4c1", ""],
                    ["r5c1", "r5c2"],
                ],
            ],
            [
                ["header 1", "header 2-1 header 2-2"],
                ["r1c1", "r1c2 r2c2"],
                ["r3c1 r4c1", "r3c2"],
                ["r5c1", "r5c2"],
            ],
        ),
        (
            "Merges rows based on configured leading StaticRowLengths and EmptyColumn thereafter.",
            cfgextract.TableExtraction(
                transforms=[
                    cfgextract.FoldRows(
                        [
                            cfgextract.StaticRowCounts([2]),
                            cfgextract.EmptyColumn(0),
                        ]
                    ),
                ],
            ),
            [
                [
                    ["", "header 2-1"],
                    ["header 1", "header 2-2"],
                    ["r1c1", "r1c2"],
                    ["", "r2c2"],
                    ["r3c1", "r3c2"],
                    ["r4c1", ""],
                    ["r5c1", "r5c2"],
                ],
            ],
            [
                ["header 1", "header 2-1 header 2-2"],
                ["r1c1", "r1c2 r2c2"],
                ["r3c1", "r3c2"],
                ["r4c1", ""],
                ["r5c1", "r5c2"],
            ],
        ),
        (
            "Fold all rows.",
            cfgextract.TableExtraction(
                transforms=[cfgextract.FoldRows([cfgextract.AllRows()])],
            ),
            [
                [
                    ["r1c1", "r1c2", "r1c3"],
                    ["r2c1", "r2c2"],
                    ["r3c1", "r3c2", "r3c3"],
                ],
            ],
            [
                ["r1c1 r2c1 r3c1", "r1c2 r2c2 r3c2", "r1c3 r3c3"],
            ],
        ),
        (
            "Splits a column by the matches of a regex.",
            cfgextract.TableExtraction(
                transforms=[
                    cfgextract.ExpandColumnOnRegex(
                        column=1,
                        pattern=r"[*] +([^:]+): +(.+)",
                        on_match=[r"\1", r"\2"],
                        default=[r"", r"\g<0>"],
                    ),
                ],
            ),
            [
                [
                    ["r1c1", "* label 1: text 1", "last col"],
                    ["r2c1", "* label 2: text 2", "last col"],
                    ["r3c1", "continuation text", "last col"],
                    ["r4c1", "* text 4: without last col"],
                    ["r5c1"],  # Row without split column.
                    [],  # empty row
                ],
            ],
            [
                ["r1c1", "label 1", "text 1", "last col"],
                ["r2c1", "label 2", "text 2", "last col"],
                ["r3c1", "", "continuation text", "last col"],
                ["r4c1", "text 4", "without last col"],
                ["r5c1"],
                [],  # empty row
            ],
        ),
        (
            "Joins a range of columns - from+to set.",
            cfgextract.TableExtraction(
                transforms=[cfgextract.JoinColumns(from_=1, to=3, delim=" ")],
            ),
            [
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    ["r2c1", "r2c2", "r2c3", "r2c4"],
                    ["r3c1", "r3c2", "r3c3"],
                    ["r4c1", "r4c2"],
                    ["r5c1"],
                    [],
                ],
            ],
            [
                ["r1c1", "r1c2 r1c3", "r1c4", "r1c5"],
                ["r2c1", "r2c2 r2c3", "r2c4"],
                ["r3c1", "r3c2 r3c3"],
                ["r4c1", "r4c2"],
                ["r5c1"],
                [],
            ],
        ),
        (
            "Joins a range of columns - from set.",
            cfgextract.TableExtraction(
                transforms=[cfgextract.JoinColumns(from_=1, delim=" ")],
            ),
            [
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    ["r2c1", "r2c2", "r2c3", "r2c4"],
                    ["r3c1", "r3c2", "r3c3"],
                    ["r4c1", "r4c2"],
                    ["r5c1"],
                    [],
                ],
            ],
            [
                ["r1c1", "r1c2 r1c3 r1c4 r1c5"],
                ["r2c1", "r2c2 r2c3 r2c4"],
                ["r3c1", "r3c2 r3c3"],
                ["r4c1", "r4c2"],
                ["r5c1"],
                [],
            ],
        ),
        (
            "Joins a range of columns - to set.",
            cfgextract.TableExtraction(
                transforms=[cfgextract.JoinColumns(to=3, delim=" ")],
            ),
            [
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    ["r2c1", "r2c2", "r2c3", "r2c4"],
                    ["r3c1", "r3c2", "r3c3"],
                    ["r4c1", "r4c2"],
                    ["r5c1"],
                    [],
                ],
            ],
            [
                ["r1c1 r1c2 r1c3", "r1c4", "r1c5"],
                ["r2c1 r2c2 r2c3", "r2c4"],
                ["r3c1 r3c2 r3c3"],
                ["r4c1 r4c2"],
                ["r5c1"],
                [],
            ],
        ),
        (
            "Joins a range of columns - neither from/to set set.",
            cfgextract.TableExtraction(
                transforms=[cfgextract.JoinColumns(delim=" ")],
            ),
            [
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    ["r2c1", "r2c2", "r2c3", "r2c4"],
                    ["r3c1", "r3c2", "r3c3"],
                    ["r4c1", "r4c2"],
                    ["r5c1"],
                    [],
                ],
            ],
            [
                ["r1c1 r1c2 r1c3 r1c4 r1c5"],
                ["r2c1 r2c2 r2c3 r2c4"],
                ["r3c1 r3c2 r3c3"],
                ["r4c1 r4c2"],
                ["r5c1"],
                [],
            ],
        ),
        (
            "Splits a column on a pattern.",
            cfgextract.TableExtraction(
                transforms=[
                    cfgextract.SplitColumn(
                        column=1,
                        pattern=r",\s*",
                    )
                ],
            ),
            [
                [
                    ["0", "a, b,c"],
                    ["0", "a, b,c", "d"],
                    ["0", "a"],
                    ["0"],
                    [],
                ],
            ],
            [
                ["0", "a", "b", "c"],
                ["0", "a", "b", "c", "d"],
                ["0", "a"],
                ["0"],
                [],
            ],
        ),
        (
            "Wraps a row every N columns.",
            cfgextract.TableExtraction(
                transforms=[
                    cfgextract.WrapRowEveryN(columns=2),
                ],
            ),
            [
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4"],
                    ["r2c1", "r2c2", "r2c3", "r2c4", "r2c5"],
                    ["r3c1", "r3c2", "r3c3"],
                    [],
                    ["r5c1"],
                ],
            ],
            [
                ["r1c1", "r1c2"],
                ["r1c3", "r1c4"],
                ["r2c1", "r2c2"],
                ["r2c3", "r2c4"],
                ["r2c5", "r3c1"],
                ["r3c2", "r3c3"],
                ["r5c1"],
            ],
        ),
        (
            "Transposes a table.",
            cfgextract.TableExtraction(
                transforms=[cfgextract.Transpose()],
            ),
            [
                [
                    ["r1c1", "r1c2", "r1c3"],
                    ["r2c1", "r2c2"],
                    ["r3c1", "r3c2", "r3c3"],
                ],
            ],
            [
                ["r1c1", "r2c1", "r3c1"],
                ["r1c2", "r2c2", "r3c2"],
                ["r1c3", "", "r3c3"],
            ],
        ),
    ],
)
def test_extract_table(
    record_property,
    name: str,
    extract_cfg: cfgextract.TableExtraction,
    tables_in: list[tabledata.TableData],
    expected: tabledata.TableData,
) -> None:
    # pylint: disable=too-many-locals
    record_property("name", name)

    # Self-check the inputs.
    for table_in in tables_in:
        tabledata.check_table_type(table_in)
    tabledata.check_table_type(expected)

    tmpl_path = pathlib.PurePath("foo/bar.tabula-template.json")
    tmpl_content = '{"fake": "json"}'
    files = {tmpl_path: tmpl_content}
    pdf_path = pathlib.Path("some.pdf")
    file_stem = pathlib.Path("foo/bar")
    with filesio.MemReadWriter.new_reader(files) as cfg_reader:
        table_reader = pdftestutil.FakeTableReader()
        expect_call = pdftestutil.Call(pdf_path, tmpl_content)
        table_reader.return_tables = {
            expect_call: [pdftestutil.tabula_table_from_simple(1, table) for table in tables_in]
        }
        actual_pages, actual = tableextract.extract_table(
            cfg_reader=cfg_reader,
            table=config.Table(
                file_stem=file_stem,
                extraction=extract_cfg,
            ),
            pdf_path=pdf_path,
            table_reader=table_reader,
        )
    assert actual_pages == {1}
    # Check read_pdf_with_template calls.
    hc.assert_that(table_reader.calls, hc.contains_exactly(hc.equal_to(expect_call)))
    # Check output.
    testfixtures.compare(expected=expected, actual=actual)
    # pylint: enable=too-many-locals
