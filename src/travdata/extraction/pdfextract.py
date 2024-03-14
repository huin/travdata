# -*- coding: utf-8 -*-
import itertools
import pathlib
from typing import Callable, Iterable, Iterator

from travdata import config
from travdata.extraction import parseutil, tabulautil


class ConfigurationError(Exception):
    pass


def _iter_num_rows_continuations(row_num_lines: list[int]) -> Iterator[bool]:
    for num_lines in row_num_lines:
        yield False
        for _ in range(num_lines - 1):
            yield True


def extract_table(
    config_dir: pathlib.Path,
    pdf_path: pathlib.Path,
    extraction: config.TableExtraction,
    file_stem: pathlib.Path,
    tabula_cfg: tabulautil.TabulaConfig,
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
        tabulautil.read_pdf_with_template(
            pdf_path=pdf_path,
            template_path=config_dir / file_stem.with_suffix(".tabula-template.json"),
            config=tabula_cfg,
        )
    )

    if extraction.row_num_lines is not None:
        iter_num_rows_continuations = _iter_num_rows_continuations(extraction.row_num_lines)
    else:
        iter_num_rows_continuations = None

    def continuation(i: int, row: list[str]) -> bool:
        if extraction.add_header_row is None:
            if i == 0:
                return False
            elif i < extraction.num_header_lines:
                return True

        if extraction.continuation_empty_column is not None:
            return row[extraction.continuation_empty_column] == ""
        elif iter_num_rows_continuations is not None:
            try:
                return next(iter_num_rows_continuations)
            except StopIteration:
                raise ConfigurationError("Not enough total lines specified in row_num_lines.")
        else:
            return False

    text_rows = tabulautil.table_rows_text(tabula_rows)
    text_rows = _fold_rows(
        rows=text_rows,
        continuation=continuation,
    )
    text_rows = _clean_rows(text_rows)
    if extraction.add_header_row is not None:
        text_rows = itertools.chain([extraction.add_header_row], text_rows)
    return text_rows


def _fold_rows(
    rows: Iterable[list[str]],
    continuation: Callable[[int, list[str]], bool],
    join: str = "\n",
) -> Iterator[list[str]]:
    row_accum: list[list[str]] = []

    def form_row():
        return [join.join(cell) for cell in row_accum]

    for i, row in enumerate(rows):
        try:
            if not continuation(i, row) and row_accum:
                yield form_row()
                row_accum = []
            missing_count = len(row) - len(row_accum)
            if missing_count > 0:
                for _ in range(missing_count):
                    row_accum.append([])
            for acc, text in zip(row_accum, row):
                if text:
                    acc.append(text)
        except Exception as e:
            e.add_note(f"for {row=}")
            raise

    if row_accum:
        yield form_row()


def _clean_rows(rows: Iterable[list[str]]) -> Iterator[list[str]]:
    for row in rows:
        yield [parseutil.clean_text(text) for text in row]
