# -*- coding: utf-8 -*-
"""Extracts tables from a PDF."""

import itertools
import pathlib
from typing import Iterable, Iterator, Protocol, TypeAlias

from travdata import config
from travdata.extraction import parseutil, tabulautil


class TableReader(Protocol):
    """Required interface to extract a table from a PDF file.

    :param Protocol: _description_
    """

    def read_pdf_with_template(
        self,
        *,
        pdf_path: pathlib.Path,
        template_path: pathlib.Path,
    ) -> list[tabulautil.TabulaTable]:
        """Reads tables from a PDF file, using the named template file.

        :param pdf_path: Path to the PDF file.
        :param template_path: Path to the tabula-template.json file.
        :return: List of extracted tables.
        """
        raise NotImplementedError


class ConfigurationError(Exception):
    """Exception indication error in the given configuration."""


def extract_table(
    config_dir: pathlib.Path,
    pdf_path: pathlib.Path,
    extraction: config.TableExtraction,
    file_stem: pathlib.Path,
    table_reader: TableReader,
) -> Iterator[list[str]]:
    """Extracts a table from the PDF.

    :param config_dir: Config directory containing the config.yaml file.
    :param pdf_path: Path to the PDF to extract from.
    :param file_stem: Path of the Tabula table template configuration.
    :param extraction: Table configuration configuration.
    :param tabula_cfg: Configuration for Tabula extractor.
    :returns: Iterator over rows from the table.
    """
    tabula_rows: Iterator[tabulautil.TabulaRow] = tabulautil.table_rows_concat(
        table_reader.read_pdf_with_template(
            pdf_path=pdf_path,
            template_path=config_dir / file_stem.with_suffix(".tabula-template.json"),
        )
    )
    text_rows = tabulautil.table_rows_text(tabula_rows)

    if extraction.row_folding:
        text_rows = _fold_rows(
            lines=text_rows,
            grouper=_MultiGrouper(
                [_make_line_grouper(folder) for folder in extraction.row_folding]
            ),
        )

    text_rows = _clean_rows(text_rows)

    if extraction.add_header_row is not None:
        text_rows = itertools.chain([extraction.add_header_row], text_rows)

    return text_rows


_Line: TypeAlias = list[str]
_LineGroup: TypeAlias = list[_Line]
_Row: TypeAlias = list[str]


class _LineGrouper(Protocol):

    def group_lines(self, lines: Iterable[_Line]) -> Iterator[_LineGroup]:
        """Group input rows into groups, according to the implementation.

        :param lines: Input rows.
        :yield: Row groups.
        """
        raise NotImplementedError


class _StaticRowLengths(_LineGrouper):
    _line_counts: list[int]

    def __init__(self, cfg: config.StaticRowCounts) -> None:
        self._line_counts = list(cfg.row_counts)

    def group_lines(self, lines: Iterable[_Line]) -> Iterator[_LineGroup]:
        for num_lines in self._line_counts:
            yield list(itertools.islice(lines, num_lines))


class _EmptyColumn(_LineGrouper):
    _column_index: int

    def __init__(self, cfg: config.EmptyColumn) -> None:
        self._column_index = cfg.column_index

    def group_lines(self, lines: Iterable[_Line]) -> Iterator[_LineGroup]:
        group: _LineGroup = []
        for line in lines:
            if line[self._column_index] == "":
                group.append(line)
            else:
                if group:
                    yield group
                group = [line]
        if group:
            yield group


def _make_line_grouper(cfg: config.RowFolder) -> _LineGrouper:
    match cfg:
        case config.StaticRowCounts():
            return _StaticRowLengths(cfg)
        case config.EmptyColumn():
            return _EmptyColumn(cfg)
        case _:
            raise ConfigurationError(f"{type(cfg).__name__} is an unknown type of row folder")


class _MultiGrouper(_LineGrouper):
    _groupers: list[_LineGrouper]

    def __init__(self, groupers: list[_LineGrouper]) -> None:
        self._groupers = groupers

    def group_lines(self, lines: Iterable[_Line]) -> Iterator[_LineGroup]:
        for grouper in self._groupers:
            yield from grouper.group_lines(lines)
        # Everything remaining is in individual groups.
        for line in lines:
            yield [line]


def _fold_rows(
    lines: Iterable[_Line],
    grouper: _LineGrouper,
) -> Iterator[_Row]:
    for line_group in grouper.group_lines(lines):
        # List of cell texts, each of which contain the sequence of strings that
        # make up the resulting row's cells. The following is essentially a
        # transpose operation.
        row_accum: list[list[str]] = []
        for line in line_group:
            missing_count = len(line) - len(row_accum)
            if missing_count > 0:
                for _ in range(missing_count):
                    row_accum.append([])
            for acc, text in zip(row_accum, line):
                if text:
                    acc.append(text)

        row: _Row = [" ".join(cell) for cell in row_accum]
        yield row


def _clean_rows(rows: Iterable[list[str]]) -> Iterator[list[str]]:
    for row in rows:
        yield [parseutil.clean_text(text) for text in row]
