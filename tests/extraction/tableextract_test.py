# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import dataclasses
import pathlib

import pytest
import testfixtures  # type: ignore[import-untyped]
from travdata import config
from travdata.config import cfgextract
from travdata.extraction import tableextract, tabulautil


@dataclasses.dataclass(frozen=True)
class Call:
    pdf_path: pathlib.Path
    template_path: pathlib.Path


class FakeTableReader:
    calls: list[Call]
    return_tables: list[tabulautil.TabulaTable]

    def __init__(self, tables_in: list[list[list[str]]]) -> None:
        self.calls = []

        tables_out: list[tabulautil.TabulaTable] = []
        for table_in in tables_in:
            rows_out: list[tabulautil.TabulaRow] = []
            tables_out.append({"data": rows_out})
            for row_in in table_in:
                cells_out: tabulautil.TabulaRow = []
                rows_out.append(cells_out)
                for cell_in in row_in:
                    cells_out.append({"text": cell_in})
        self.return_tables = tables_out

    def read_pdf_with_template(
        self,
        *,
        pdf_path: pathlib.Path,
        template_path: pathlib.Path,
    ) -> list[tabulautil.TabulaTable]:
        self.calls.append(Call(pdf_path, template_path))
        return self.return_tables


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
    name: str,
    extract_cfg: cfgextract.TableExtraction,
    tables_in,
    expected: list[list[str]],
):
    print(name)
    config_dir = pathlib.Path("cfg_dir")
    pdf_path = pathlib.Path("some.pdf")
    file_stem = pathlib.Path("foo/bar")
    expected_template_path = pathlib.Path("cfg_dir/foo/bar.tabula-template.json")
    table_reader = FakeTableReader(tables_in=tables_in)
    actual = tableextract.extract_table(
        table=config.Table(
            cfg_dir=config_dir,
            file_stem=file_stem,
            extraction=extract_cfg,
        ),
        pdf_path=pdf_path,
        table_reader=table_reader,
    )
    # Check read_pdf_with_template calls.
    testfixtures.compare(
        expected=[Call(pdf_path, expected_template_path)], actual=table_reader.calls
    )
    # Check output.
    testfixtures.compare(expected=expected, actual=actual)
