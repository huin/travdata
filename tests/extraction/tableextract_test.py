# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import dataclasses
import pathlib

import hamcrest as hc
import pytest
import testfixtures  # type: ignore[import-untyped]

from travdata import config, filesio
from travdata import tabledata
from travdata.config import cfgextract
from travdata.extraction import estransform, tableextract
from .pdf import pdftestutil


@dataclasses.dataclass(frozen=True)
class Case:
    name: str
    extract_cfg: cfgextract.TableTransform
    tables_in: list[tabledata.TableData]
    expected: tabledata.TableData

    def test_id(self) -> str:
        return self.name


@pytest.mark.parametrize(
    "case",
    [
        Case(
            name="Base behaviour with default config.",
            extract_cfg=cfgextract.LegacyTransformSeq(),
            tables_in=[
                [
                    ["header 1", "header 2"],
                    ["r1c1", "r1c2"],
                ],
            ],
            expected=[
                ["header 1", "header 2"],
                ["r1c1", "r1c2"],
            ],
        ),
        Case(
            name="Concatenates input tables.",
            extract_cfg=cfgextract.LegacyTransformSeq(),
            tables_in=[
                [
                    ["header 1", "header 2"],
                    ["r1c1", "r1c2"],
                ],
                [
                    ["r2c1", "r2c2"],
                    ["r3c1", "r3c2"],
                ],
            ],
            expected=[
                ["header 1", "header 2"],
                ["r1c1", "r1c2"],
                ["r2c1", "r2c2"],
                ["r3c1", "r3c2"],
            ],
        ),
        Case(
            name="Adds specified leading row.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[cfgextract.PrependRow(["added header 1", "added header 2"])]
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2"],
                    ["r2c1", "r2c2"],
                ],
            ],
            expected=[
                ["added header 1", "added header 2"],
                ["r1c1", "r1c2"],
                ["r2c1", "r2c2"],
            ],
        ),
        Case(
            name="Merges specified header rows, and keeps individual rows thereafter.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[
                    cfgextract.FoldRows(
                        [
                            cfgextract.StaticRowCounts([2]),
                        ]
                    ),
                ],
            ),
            tables_in=[
                [
                    ["header 1-1", "header 2-1"],
                    ["header 1-2", "header 2-2"],
                    ["r1c1", "r1c2"],
                    ["r2c1", "r2c2"],
                ],
            ],
            expected=[
                ["header 1-1 header 1-2", "header 2-1 header 2-2"],
                ["r1c1", "r1c2"],
                ["r2c1", "r2c2"],
            ],
        ),
        Case(
            name="Merges rows based on configured StaticRowLengths.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[
                    cfgextract.FoldRows(
                        [
                            cfgextract.StaticRowCounts([2, 2, 2]),
                        ]
                    ),
                ],
            ),
            tables_in=[
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
            expected=[
                ["header 1", "header 2-1 header 2-2"],
                ["r1c1", "r1c2 r2c2"],
                ["r3c1 r4c1", "r3c2"],
                ["r5c1", "r5c2"],
            ],
        ),
        Case(
            name="fold_rows_leading_static_and_empty",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[
                    cfgextract.FoldRows(
                        [
                            cfgextract.StaticRowCounts([2]),
                            cfgextract.EmptyColumn(0),
                        ]
                    ),
                ],
            ),
            tables_in=[
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
            expected=[
                ["header 1", "header 2-1 header 2-2"],
                ["r1c1", "r1c2 r2c2"],
                ["r3c1", "r3c2"],
                ["r4c1", ""],
                ["r5c1", "r5c2"],
            ],
        ),
        Case(
            name="Fold all rows.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[cfgextract.FoldRows([cfgextract.AllRows()])],
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2", "r1c3"],
                    ["r2c1", "r2c2"],
                    ["r3c1", "r3c2", "r3c3"],
                ],
            ],
            expected=[
                ["r1c1 r2c1 r3c1", "r1c2 r2c2 r3c2", "r1c3 r3c3"],
            ],
        ),
        Case(
            name="Splits a column by the matches of a regex.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[
                    cfgextract.ExpandColumnOnRegex(
                        column=1,
                        pattern=r"[*] +([^:]+): +(.+)",
                        on_match=[r"\1", r"\2"],
                        default=[r"", r"\g<0>"],
                    ),
                ],
            ),
            tables_in=[
                [
                    ["r1c1", "* label 1: text 1", "last col"],
                    ["r2c1", "* label 2: text 2", "last col"],
                    ["r3c1", "continuation text", "last col"],
                    ["r4c1", "* text 4: without last col"],
                    ["r5c1"],  # Row without split column.
                    [],  # empty row
                ],
            ],
            expected=[
                ["r1c1", "label 1", "text 1", "last col"],
                ["r2c1", "label 2", "text 2", "last col"],
                ["r3c1", "", "continuation text", "last col"],
                ["r4c1", "text 4", "without last col"],
                ["r5c1"],
                [],  # empty row
            ],
        ),
        Case(
            name="Joins a range of columns - from+to set.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[cfgextract.JoinColumns(from_=1, to=3, delim=" ")],
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    ["r2c1", "r2c2", "r2c3", "r2c4"],
                    ["r3c1", "r3c2", "r3c3"],
                    ["r4c1", "r4c2"],
                    ["r5c1"],
                    [],
                ],
            ],
            expected=[
                ["r1c1", "r1c2 r1c3", "r1c4", "r1c5"],
                ["r2c1", "r2c2 r2c3", "r2c4"],
                ["r3c1", "r3c2 r3c3"],
                ["r4c1", "r4c2"],
                ["r5c1"],
                [],
            ],
        ),
        Case(
            name="Joins a range of columns - from set.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[cfgextract.JoinColumns(from_=1, delim=" ")],
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    ["r2c1", "r2c2", "r2c3", "r2c4"],
                    ["r3c1", "r3c2", "r3c3"],
                    ["r4c1", "r4c2"],
                    ["r5c1"],
                    [],
                ],
            ],
            expected=[
                ["r1c1", "r1c2 r1c3 r1c4 r1c5"],
                ["r2c1", "r2c2 r2c3 r2c4"],
                ["r3c1", "r3c2 r3c3"],
                ["r4c1", "r4c2"],
                ["r5c1"],
                [],
            ],
        ),
        Case(
            name="Joins a range of columns - to set.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[cfgextract.JoinColumns(to=3, delim=" ")],
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    ["r2c1", "r2c2", "r2c3", "r2c4"],
                    ["r3c1", "r3c2", "r3c3"],
                    ["r4c1", "r4c2"],
                    ["r5c1"],
                    [],
                ],
            ],
            expected=[
                ["r1c1 r1c2 r1c3", "r1c4", "r1c5"],
                ["r2c1 r2c2 r2c3", "r2c4"],
                ["r3c1 r3c2 r3c3"],
                ["r4c1 r4c2"],
                ["r5c1"],
                [],
            ],
        ),
        Case(
            name="Joins a range of columns - neither from/to set set.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[cfgextract.JoinColumns(delim=" ")],
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4", "r1c5"],
                    ["r2c1", "r2c2", "r2c3", "r2c4"],
                    ["r3c1", "r3c2", "r3c3"],
                    ["r4c1", "r4c2"],
                    ["r5c1"],
                    [],
                ],
            ],
            expected=[
                ["r1c1 r1c2 r1c3 r1c4 r1c5"],
                ["r2c1 r2c2 r2c3 r2c4"],
                ["r3c1 r3c2 r3c3"],
                ["r4c1 r4c2"],
                ["r5c1"],
                [],
            ],
        ),
        Case(
            name="Splits a column on a pattern.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[
                    cfgextract.SplitColumn(
                        column=1,
                        pattern=r",\s*",
                    )
                ],
            ),
            tables_in=[
                [
                    ["0", "a, b,c"],
                    ["0", "a, b,c", "d"],
                    ["0", "a"],
                    ["0"],
                    [],
                ],
            ],
            expected=[
                ["0", "a", "b", "c"],
                ["0", "a", "b", "c", "d"],
                ["0", "a"],
                ["0"],
                [],
            ],
        ),
        Case(
            name="Wraps a row every N columns.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[
                    cfgextract.WrapRowEveryN(columns=2),
                ],
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2", "r1c3", "r1c4"],
                    ["r2c1", "r2c2", "r2c3", "r2c4", "r2c5"],
                    ["r3c1", "r3c2", "r3c3"],
                    [],
                    ["r5c1"],
                ],
            ],
            expected=[
                ["r1c1", "r1c2"],
                ["r1c3", "r1c4"],
                ["r2c1", "r2c2"],
                ["r2c3", "r2c4"],
                ["r2c5", "r3c1"],
                ["r3c2", "r3c3"],
                ["r5c1"],
            ],
        ),
        Case(
            name="Transposes a table.",
            extract_cfg=cfgextract.LegacyTransformSeq(
                transforms=[cfgextract.Transpose()],
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2", "r1c3"],
                    ["r2c1", "r2c2"],
                    ["r3c1", "r3c2", "r3c3"],
                ],
            ],
            expected=[
                ["r1c1", "r2c1", "r3c1"],
                ["r1c2", "r2c2", "r3c2"],
                ["r1c3", "", "r3c3"],
            ],
        ),
        Case(
            name="Uses ECMAScript to transform a table.",
            extract_cfg=cfgextract.ESTransform(
                src="return concatExtTables(tables);",
            ),
            tables_in=[
                [
                    ["r1c1", "r1c2"],
                    ["r2c1", "r2c2"],
                ],
                [
                    ["r3c1", "r3c2"],
                    ["r4c1", "r4c2"],
                ],
            ],
            expected=[
                ["r1c1", "r1c2"],
                ["r2c1", "r2c2"],
                ["r3c1", "r3c2"],
                ["r4c1", "r4c2"],
            ],
        ),
    ],
    ids=Case.test_id,
)
def test_extract_table(case: Case) -> None:
    # pylint: disable=too-many-locals

    # Self-check the inputs.
    for table_in in case.tables_in:
        tabledata.check_table_type(table_in)
    tabledata.check_table_type(case.expected)

    tmpl_path = pathlib.PurePath("foo/bar.tabula-template.json")
    tmpl_content = '{"fake": "json"}'
    es_module = pathlib.PurePath("lib.js")
    files = {
        tmpl_path: tmpl_content,
        es_module: """
            function concatExtTables(tables) {
                const result = [];
                for (const table of tables) {
                    for (const row of table) {
                        result.push(row);
                    }
                }
                return result;
            }
        """,
    }
    pdf_path = pathlib.Path("some.pdf")
    file_stem = pathlib.Path("foo/bar")
    with (
        filesio.MemReadWriter.new_reader(files) as cfg_reader,
        estransform.transformer(cfg_reader) as estrn,
    ):
        estrn.load_module(es_module)
        table_reader = pdftestutil.FakeTableReader()
        table_extractor = tableextract.TableExtractor(
            cfg_reader=cfg_reader,
            table_reader=table_reader,
            estrn=estrn,
        )
        expect_call = pdftestutil.Call(pdf_path, tmpl_content)
        table_reader.return_tables = {
            expect_call: [
                pdftestutil.tabula_table_from_simple(1, table) for table in case.tables_in
            ]
        }
        actual_pages, actual = table_extractor.extract_table(
            table=config.Table(
                file_stem=file_stem,
                transform=case.extract_cfg,
            ),
            pdf_path=pdf_path,
        )
    assert actual_pages == {1}
    # Check read_pdf_with_template calls.
    hc.assert_that(table_reader.calls, hc.contains_exactly(hc.equal_to(expect_call)))
    # Check output.
    testfixtures.compare(expected=case.expected, actual=actual)
    # pylint: enable=too-many-locals
