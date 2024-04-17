# -*- coding: utf-8 -*-
"""Extracts multiple tables from a PDF."""

import csv
import dataclasses
import pathlib
from typing import Callable, Iterator

from travdata import config, csvutil, filesio
from travdata.extraction import tableextract


@dataclasses.dataclass
class Progress:
    """Progress report from ``extract_book``."""

    completed: int
    total: int


@dataclasses.dataclass(frozen=True)
class ExtractionConfig:
    """Extraction configuration.

    :field output_dir: Path to top level directory for output.
    :field input_pdf: Path to PDF file to extract from.
    :field group: Configuration ``Group`` of tables to extract.
    :field overwrite_existing: If true, overwrite existing CSV files.
    :field with_tags: Only extracts tables that have any of these these tags.
    :field without_tags: Only extracts tables that do not include any of these
    tags (takes precedence over with_tags).
    """

    output_dir: pathlib.Path
    input_pdf: pathlib.Path
    group: config.Group
    overwrite_existing: bool
    with_tags: frozenset[str]
    without_tags: frozenset[str]


@dataclasses.dataclass(frozen=True)
class _OutputTable:
    out_filepath: pathlib.Path
    table: config.Table


def _filter_tables(
    cfg: ExtractionConfig,
) -> Iterator[_OutputTable]:
    if cfg.group is None:
        raise RuntimeError("Book.group was not set")

    for table in cfg.group.all_tables():
        if table.extraction is None:
            continue
        out_filepath = cfg.output_dir / table.file_stem.with_suffix(".csv")

        if cfg.with_tags and not table.tags & cfg.with_tags:
            continue

        if cfg.without_tags and table.tags & cfg.without_tags:
            continue

        if not cfg.overwrite_existing and out_filepath.exists():
            continue

        yield _OutputTable(out_filepath, table)


def _extract_single_table(
    *,
    cfg_reader: filesio.Reader,
    table_reader: tableextract.TableReader,
    cfg: ExtractionConfig,
    created_directories: set[pathlib.Path],
    output_table: _OutputTable,
) -> None:
    """Helper wrapper of `extract_table` for `extract_book`."""
    group_dir = output_table.out_filepath.parent
    if group_dir not in created_directories:
        group_dir.mkdir(parents=True, exist_ok=True)
        created_directories.add(group_dir)

    rows = tableextract.extract_table(
        cfg_reader=cfg_reader,
        table=output_table.table,
        pdf_path=cfg.input_pdf,
        table_reader=table_reader,
    )
    with csvutil.open_write(output_table.out_filepath) as f:
        csv.writer(f).writerows(rows)


@dataclasses.dataclass
class ExtractEvents:
    """Extraction event callbacks.

    :field on_progress: Called at the start and after each extraction attempt.
    :field on_error: Called on any errors.
    :field do_continue: Called at intervals. If it returns False, then no
    further processing is attempted.
    """

    on_progress: Callable[[Progress], None]
    on_error: Callable[[str], None]
    do_continue: Callable[[], bool]


def extract_book(
    *,
    cfg_reader: filesio.Reader,
    table_reader: tableextract.TableReader,
    cfg: ExtractionConfig,
    events: ExtractEvents,
) -> None:
    """Extracts an entire book to CSV.

    :param cfg_reader: Configuration reader.
    :param table_reader: Extractor for individual tables from a PDF.
    :param cfg: Configuration for extraction.
    :param events: Event hooks to feed back progress, etc.
    :raises RuntimeError: If ``cfg.book_cfg.group`` was not set.
    """

    output_tables = list(_filter_tables(cfg))

    events.on_progress(Progress(0, len(output_tables)))

    created_directories: set[pathlib.Path] = set()
    for i, output_table in enumerate(output_tables, start=1):
        if not events.do_continue():
            return

        try:
            _extract_single_table(
                cfg_reader=cfg_reader,
                table_reader=table_reader,
                cfg=cfg,
                created_directories=created_directories,
                output_table=output_table,
            )
        except tableextract.ConfigurationError as exc:
            events.on_error(
                f"Configuration error while processing table "
                f"{output_table.table.file_stem}: {exc}"
            )
        finally:
            events.on_progress(Progress(i, len(output_tables)))
