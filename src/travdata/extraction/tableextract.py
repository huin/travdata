# -*- coding: utf-8 -*-
"""Extracts a single table from a PDF."""

import itertools
import pathlib
import re
from typing import Iterable, Iterator, Protocol, TypeAlias

from travdata import config
from travdata.extraction import parseutil, tabulautil


_RX_ANYTHING = re.compile(".*")


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
    table: config.Table,
    pdf_path: pathlib.Path,
    table_reader: TableReader,
) -> Iterator[list[str]]:
    """Extracts a table from the PDF.

    :param table: Configuration of the table to extract. ``table.extraction``
    must not be None.
    :param pdf_path: Path to the PDF to extract from.
    :param tabula_reader: Used to read the table from the PDF.
    :returns: Iterator over rows from the table.
    :raises ValueError: ``table.extraction`` is None.
    """
    if table.extraction is None:
        raise ValueError(
            f"extract_table called with table with `None` extraction: {table=}",
        )

    tabula_rows: Iterator[tabulautil.TabulaRow] = tabulautil.table_rows_concat(
        table_reader.read_pdf_with_template(
            pdf_path=pdf_path,
            template_path=table.tabula_template_path,
        )
    )
    rows = tabulautil.table_rows_text(tabula_rows)

    for transform_cfg in table.extraction.transforms:
        rows = _transform(transform_cfg, rows)

    return _clean_rows(rows)


_Row: TypeAlias = list[str]
_RowGroup: TypeAlias = list[_Row]


def _transform(cfg: config.TableTransform, rows: Iterable[_Row]) -> Iterator[_Row]:
    match cfg:
        case config.ExpandColumnOnRegex():
            return _expand_column_on_regex(cfg, rows)
        case config.PrependRow():
            return _prepend_row(cfg, rows)
        case config.FoldRows():
            return _fold_rows(cfg, rows)
        case _:
            raise ConfigurationError(
                f"{type(cfg).__name__} is an unknown type of TableTransform",
            )


def _expand_column_on_regex(
    cfg: config.ExpandColumnOnRegex,
    rows: Iterable[_Row],
) -> Iterator[_Row]:
    rx = re.compile(cfg.pattern)
    for row in rows:
        try:
            prior, to_match, following = row[: cfg.column], row[cfg.column], row[cfg.column + 1 :]
        except IndexError:
            # Specified column not present. Pass-through as-is.
            yield row
            continue

        new_row = prior

        if rx_match := rx.fullmatch(to_match):
            for cell_tmpl in cfg.on_match:
                new_row.append(rx_match.expand(cell_tmpl))
        elif rx_match := _RX_ANYTHING.fullmatch(to_match):
            for cell_tmpl in cfg.default:
                new_row.append(rx_match.expand(cell_tmpl))
        else:
            # Should never happen.
            raise RuntimeError(f"{_RX_ANYTHING} failed to match {to_match!r}")

        new_row.extend(following)
        yield new_row


def _prepend_row(cfg: config.PrependRow, rows: Iterable[_Row]) -> Iterator[_Row]:
    """Implements the config.PrependRow transformation."""
    return itertools.chain([cfg.row], rows)


class _LineGrouper(Protocol):

    def group_lines(self, lines: Iterable[_Row]) -> Iterator[_RowGroup]:
        """Group input rows into groups, according to the implementation.

        :param lines: Input rows.
        :yield: Row groups.
        """
        raise NotImplementedError


class _StaticRowLengths(_LineGrouper):
    _line_counts: list[int]

    def __init__(self, cfg: config.StaticRowCounts) -> None:
        self._line_counts = list(cfg.row_counts)

    def group_lines(self, lines: Iterable[_Row]) -> Iterator[_RowGroup]:
        for num_lines in self._line_counts:
            yield list(itertools.islice(lines, num_lines))


class _EmptyColumn(_LineGrouper):
    _column_index: int

    def __init__(self, cfg: config.EmptyColumn) -> None:
        self._column_index = cfg.column_index

    def group_lines(self, lines: Iterable[_Row]) -> Iterator[_RowGroup]:
        group: _RowGroup = []
        for line in lines:
            if line[self._column_index] == "":
                group.append(line)
            else:
                if group:
                    yield group
                group = [line]
        if group:
            yield group


def _make_line_grouper(cfg: config.RowGrouper) -> _LineGrouper:
    match cfg:
        case config.StaticRowCounts():
            return _StaticRowLengths(cfg)
        case config.EmptyColumn():
            return _EmptyColumn(cfg)
        case _:
            raise ConfigurationError(
                f"{type(cfg).__name__} is an unknown type of row folder",
            )


class _MultiGrouper(_LineGrouper):
    _groupers: list[_LineGrouper]

    def __init__(self, groupers: list[_LineGrouper]) -> None:
        self._groupers = groupers

    def group_lines(self, lines: Iterable[_Row]) -> Iterator[_RowGroup]:
        for grouper in self._groupers:
            yield from grouper.group_lines(lines)
        # Everything remaining is in individual groups.
        for line in lines:
            yield [line]


def _fold_rows(
    cfg: config.FoldRows,
    rows: Iterable[_Row],
) -> Iterator[_Row]:
    """Implements the config.FoldRows transformation."""

    grouper = _MultiGrouper([_make_line_grouper(folder) for folder in cfg.group_by])

    for line_group in grouper.group_lines(rows):
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
