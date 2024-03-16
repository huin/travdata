# -*- coding: utf-8 -*-
import dataclasses
import pathlib

import pytest
import testfixtures  # type: ignore[import-untyped]
from travdata import config
from travdata.extraction import pdfextract, tabulautil


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
    "extraction,tables_in,expected",
    [
        (
            # Base behaviour with default config.
            config.TableExtraction(),
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
            # Concatenates input tables.
            config.TableExtraction(),
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
            # Merges specified header rows.
            config.TableExtraction(num_header_lines=2),
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
            # Adds specified leading rows.
            config.TableExtraction(add_header_row=["added header 1", "added header 2"]),
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
            # Merges rows based on configured continuation_empty_column and
            # num_header_lines.
            config.TableExtraction(num_header_lines=2, continuation_empty_column=0),
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
            # Merges rows based on configured row_num_lines and
            # num_header_lines.
            config.TableExtraction(
                num_header_lines=2,
                continuation_empty_column=None,
                row_num_lines=[2, 2, 1],
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
    ],
)
def test_extract_table(extraction: config.TableExtraction, tables_in, expected: list[list[str]]):
    config_dir = pathlib.Path("cfg_dir")
    pdf_path = pathlib.Path("some.pdf")
    file_stem = pathlib.Path("foo/bar")
    expected_template_path = pathlib.Path("cfg_dir/foo/bar.tabula-template.json")
    table_reader = FakeTableReader(tables_in=tables_in)
    actual = pdfextract.extract_table(
        config_dir=config_dir,
        pdf_path=pdf_path,
        extraction=extraction,
        file_stem=file_stem,
        table_reader=table_reader,
    )
    # Check read_pdf_with_template calls.
    testfixtures.compare(
        expected=[Call(pdf_path, expected_template_path)], actual=table_reader.calls
    )
    # Check output.
    testfixtures.compare(expected=expected, actual=actual)
