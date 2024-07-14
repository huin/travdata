# -*- coding: utf-8 -*-
"""Python-based extracted table transformations."""

import functools
import itertools
import re
from typing import Iterable, Iterator, Protocol, TypeAlias

from travdata.config import cfgerror, cfgextract
from travdata.extraction import parseutil
from travdata.tabledata import RowData, TableData


_RX_ANYTHING = re.compile(".*")


# _RowGroup represents a consecutive set of rows within a Table.
_RowGroup: TypeAlias = list[RowData]


def perform_transforms(
    transforms: Iterable[cfgextract.LegacyTransform],
    tables: list[TableData],
) -> TableData:
    """Transforms the rows based on the given transform configurations."""
    rows: Iterator[RowData] = _table_rows_concat(tables)

    for transform_cfg in transforms:
        rows = _transform(transform_cfg, rows)

    return list(_clean_rows(rows))


def _transform(
    cfg: cfgextract.LegacyTransform,
    rows: Iterable[RowData],
) -> Iterator[RowData]:
    # pylint: disable=too-many-return-statements
    match cfg:
        case cfgextract.ExpandColumnOnRegex():
            return _expand_column_on_regex(cfg, rows)
        case cfgextract.JoinColumns():
            return _join_columns(cfg, rows)
        case cfgextract.PrependRow():
            return _prepend_row(cfg, rows)
        case cfgextract.FoldRows():
            return _fold_rows(cfg, rows)
        case cfgextract.SplitColumn():
            return _split_column(cfg, rows)
        case cfgextract.Transpose():
            return _transpose(rows)
        case cfgextract.WrapRowEveryN():
            return _wrap_row_every_n(cfg, rows)
        case _:
            raise cfgerror.ConfigurationError(
                f"{type(cfg).__name__} is an unknown type of TableTransform",
            )
    # pylint: enable=too-many-return-statements


def _expand_column_on_regex(
    cfg: cfgextract.ExpandColumnOnRegex,
    rows: Iterable[RowData],
) -> Iterator[RowData]:
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


def _join_columns(
    cfg: cfgextract.JoinColumns,
    rows: Iterable[RowData],
) -> Iterator[RowData]:
    delim = cfg.delim
    from_, to = cfg.from_, cfg.to
    for row in rows:
        out_row = []

        if from_ is not None:
            out_row.extend(row[:from_])

        if to_join := row[from_:to]:
            out_row.append(delim.join(to_join))

        if to is not None:
            out_row.extend(row[to:])

        yield out_row


def _prepend_row(cfg: cfgextract.PrependRow, rows: Iterable[RowData]) -> Iterator[RowData]:
    """Implements the config.PrependRow transformation."""
    return itertools.chain([cfg.row], rows)


class _LineGrouper(Protocol):

    def __call__(self, lines: Iterable[RowData]) -> Iterator[_RowGroup]:
        """Group input rows into groups, according to the implementation.

        :param lines: Input rows.
        :yield: Row groups.
        """
        raise NotImplementedError


def _all_rows(lines: Iterable[RowData]) -> Iterator[_RowGroup]:
    yield list(lines)


def _static_row_lengths(
    cfg: cfgextract.StaticRowCounts, lines: Iterable[RowData]
) -> Iterator[_RowGroup]:
    for num_lines in cfg.row_counts:
        yield list(itertools.islice(lines, num_lines))


def _empty_column(cfg: cfgextract.EmptyColumn, lines: Iterable[RowData]) -> Iterator[_RowGroup]:
    group: _RowGroup = []
    for line in lines:
        if line[cfg.column_index] == "":
            group.append(line)
        else:
            if group:
                yield group
            group = [line]
    if group:
        yield group


def _make_line_grouper(cfg: cfgextract.RowGrouper) -> _LineGrouper:
    match cfg:
        case cfgextract.AllRows():
            return _all_rows
        case cfgextract.StaticRowCounts():
            return functools.partial(_static_row_lengths, cfg)
        case cfgextract.EmptyColumn():
            return functools.partial(_empty_column, cfg)
        case _:
            raise cfgerror.ConfigurationError(
                f"{type(cfg).__name__} is an unknown type of row folder",
            )


def _multi_grouper(groupers: list[_LineGrouper], lines: Iterable[RowData]) -> Iterator[_RowGroup]:
    for grouper in groupers:
        yield from grouper(lines)
    # Everything remaining is in individual groups.
    for line in lines:
        yield [line]


def _fold_rows(
    cfg: cfgextract.FoldRows,
    rows: Iterable[RowData],
) -> Iterator[RowData]:
    """Implements the config.FoldRows transformation."""

    grouper = _multi_grouper([_make_line_grouper(folder) for folder in cfg.group_by], rows)

    for line_group in grouper:
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

        row: RowData = [" ".join(cell) for cell in row_accum]
        yield row


def _split_column(
    cfg: cfgextract.SplitColumn,
    rows: Iterable[RowData],
) -> Iterator[RowData]:
    pattern = re.compile(cfg.pattern)
    for row in rows:
        if len(row) <= cfg.column:
            yield row
            continue
        new_row = row[: cfg.column]
        new_row.extend(pattern.split(row[cfg.column]))
        new_row.extend(row[cfg.column + 1 :])
        yield new_row


def _transpose(
    rows: Iterable[RowData],
) -> Iterator[RowData]:
    orig_rows = list(rows)
    orig_num_cols = max(len(row) for row in orig_rows)
    orig_num_rows = len(orig_rows)

    for i in range(orig_num_cols):
        row: RowData = []
        for j in range(orig_num_rows):
            try:
                cell = orig_rows[j][i]
            except IndexError:
                cell = ""
            row.append(cell)
        yield row


def _wrap_row_every_n(
    cfg: cfgextract.WrapRowEveryN,
    rows: Iterable[RowData],
) -> Iterator[RowData]:
    if cfg.columns < 1:
        raise cfgerror.ConfigurationError(
            f"{cfg.yaml_tag}.columns must be at least 1, but is {cfg.columns}"
        )
    accum: RowData = []
    for row in rows:
        for cell in row:
            accum.append(cell)
            l = len(accum)
            if l == cfg.columns:
                yield accum
                accum = []
            elif l < cfg.columns:
                continue
            else:
                raise RuntimeError(
                    f"too many items {l} in accumulated row versus maximum" f" of {cfg.columns}"
                )
    if accum:
        yield accum


def _clean_rows(rows: Iterable[RowData]) -> Iterator[RowData]:
    for row in rows:
        yield [parseutil.clean_text(text) for text in row]


def _table_rows_concat(
    tables: Iterable[TableData],
) -> Iterator[RowData]:
    """Concatenates rows from multiple tables into a single row iterator.

    :param tables: Tables to concatenate rows from.
    :yield: Rows from the tables.
    """
    for t in tables:
        yield from iter(t)
