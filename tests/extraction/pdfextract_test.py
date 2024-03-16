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
            config.TableExtraction(),
            [
                [
                    ["header 1", "header 2"],
                    ["row 1 cell 1", "row 2 cell 2"],
                ],
            ],
            [
                ["header 1", "header 2"],
                ["row 1 cell 1", "row 2 cell 2"],
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
