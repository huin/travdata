# -*- coding: utf-8 -*-
"""Extracts multiple tables from a PDF."""

import contextlib
import csv
import dataclasses
import pathlib
from typing import Callable, Iterable, Iterator, Optional, Self

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

    cfg_reader_ctx: contextlib.AbstractContextManager[filesio.Reader]
    out_writer_ctx: contextlib.AbstractContextManager[filesio.ReadWriter]
    input_pdf: pathlib.Path
    book_id: str
    overwrite_existing: bool
    with_tags: frozenset[str]
    without_tags: frozenset[str]


@dataclasses.dataclass(frozen=True)
class _OutputTable:
    out_filepath: pathlib.PurePath
    table: config.Table


def _filter_tables(
    ext_cfg: ExtractionConfig,
    book_group: config.Group,
    out_writer: filesio.ReadWriter,
) -> Iterator[_OutputTable]:
    for table in book_group.all_tables():
        if table.extraction is None:
            continue
        out_filepath = table.file_stem.with_suffix(".csv")

        if ext_cfg.with_tags and not table.tags & ext_cfg.with_tags:
            continue

        if ext_cfg.without_tags and table.tags & ext_cfg.without_tags:
            continue

        if not ext_cfg.overwrite_existing and out_writer.exists(out_filepath):
            continue

        yield _OutputTable(out_filepath, table)


def _extract_single_table(
    *,
    cfg_reader: filesio.Reader,
    out_writer: filesio.ReadWriter,
    table_reader: tableextract.TableReader,
    input_pdf: pathlib.Path,
    output_table: _OutputTable,
) -> set[int]:
    """Helper wrapper of `extract_table` for `extract_book`, returning page numbers."""
    pages, rows = tableextract.extract_table(
        cfg_reader=cfg_reader,
        table=output_table.table,
        pdf_path=input_pdf,
        table_reader=table_reader,
    )
    with csvutil.open_by_read_writer(out_writer, output_table.out_filepath) as f:
        csv.writer(f).writerows(rows)
    return pages


@dataclasses.dataclass
class ExtractEvents:
    """Extraction event callbacks.

    :field on_progress: Called at the start and after each extraction attempt.
    :field on_error: Called on any errors.
    :field do_continue: Called at intervals. If it returns False, then no
    further processing is attempted.
    """

    on_progress: Optional[Callable[[Progress], None]] = None
    on_output: Optional[Callable[[pathlib.PurePath], None]] = None
    on_error: Optional[Callable[[str], None]] = None
    do_continue: Optional[Callable[[], bool]] = None


# Columns/record field names in the output index file.
_INDEX_TABLE_PATH = "table_path"
_INDEX_PAGES = "pages"
_INDEX_TAGS = "tags"
_INDEX_COLUMNS = [
    _INDEX_TABLE_PATH,
    _INDEX_PAGES,
    _INDEX_TAGS,
]

_INDEX_PATH = pathlib.PurePath("index.csv")


class _Indexer:
    _write_csv: csv.DictWriter
    _seen_paths: set[str]

    def __init__(self, write_csv: csv.DictWriter) -> None:
        self._write_csv = write_csv
        self._seen_paths = set()

    @classmethod
    @contextlib.contextmanager
    def for_read_writer(cls, read_writer: filesio.ReadWriter) -> Iterator[Self]:
        """Manages an ``_Indexer``.

        :param read_writer: ReadWriter containing the index to create or update.
        :yield: The ``_Indexer``.
        """
        # Read in any existing index so that we can append new entries.
        existing_rows: list[dict[str, str]] = []
        prior_field_names: set[str] = set()
        try:
            with csvutil.open_by_reader(read_writer, _INDEX_PATH) as read_io:
                read_csv = csv.DictReader(read_io)
                existing_rows.extend(read_csv)
                if read_csv.fieldnames:
                    prior_field_names = set(read_csv.fieldnames)
        except filesio.NotFoundError:
            # No existing index, no existing entries to copy over.
            pass

        # Retain unknown columns, merge known existing.
        fieldnames = _INDEX_COLUMNS + sorted(prior_field_names - set(_INDEX_COLUMNS))

        with csvutil.open_by_read_writer(read_writer, _INDEX_PATH) as write_io:
            write_csv = csv.DictWriter(write_io, fieldnames=fieldnames)
            write_csv.writeheader()

            self = cls(write_csv)
            yield self

            for row in existing_rows:
                if row.get(_INDEX_TABLE_PATH, None) in self._seen_paths:
                    continue
                write_csv.writerows(existing_rows)

    def write_entry(
        self,
        output_table: _OutputTable,
        book_cfg: config.Book,
        pages: Iterable[int],
    ) -> None:
        """Write an index entry.

        :param output_table: Table being output.
        :param book_cfg: Book configuration.
        :param pages: Page numbers that the entry was sourced from.
        """
        path = str(output_table.out_filepath)
        self._write_csv.writerow(
            {
                _INDEX_TABLE_PATH: str(output_table.out_filepath),
                _INDEX_PAGES: ";".join(str(book_cfg.page_offset + page) for page in sorted(pages)),
                _INDEX_TAGS: ";".join(sorted(output_table.table.tags)),
            }
        )
        self._seen_paths.add(path)


def extract_book(
    *,
    table_reader: tableextract.TableReader,
    ext_cfg: ExtractionConfig,
    events: ExtractEvents,
) -> None:
    """Extracts an entire book to CSV.

    :param table_reader: Extractor for individual tables from a PDF.
    :param cfg: Configuration for extraction.
    :param events: Event hooks to feed back progress, etc.
    :raises RuntimeError: If ``cfg.book_cfg.group`` was not set.
    """

    with (
        ext_cfg.cfg_reader_ctx as cfg_reader,
        ext_cfg.out_writer_ctx as out_writer,
        _Indexer.for_read_writer(out_writer) as indexer,
    ):
        cfg = config.load_config(cfg_reader)
        try:
            book_cfg = cfg.books[ext_cfg.book_id]
        except KeyError:
            if events.on_error:
                events.on_error(
                    f"Book {ext_cfg.book_id} not found in configuration.",
                )
            return

        book_group = book_cfg.load_group(cfg_reader)

        output_tables = sorted(
            _filter_tables(ext_cfg, book_group, out_writer),
            key=lambda ft: ft.out_filepath,
        )

        if events.on_progress:
            events.on_progress(Progress(0, len(output_tables)))

        for i, output_table in enumerate(output_tables, start=1):
            if events.do_continue and not events.do_continue():
                return

            try:
                pages = _extract_single_table(
                    cfg_reader=cfg_reader,
                    out_writer=out_writer,
                    table_reader=table_reader,
                    input_pdf=ext_cfg.input_pdf,
                    output_table=output_table,
                )
            except tableextract.ConfigurationError as exc:
                if events.on_error:
                    events.on_error(
                        f"Configuration error while processing table "
                        f"{output_table.table.file_stem}: {exc}"
                    )
            else:
                if events.on_output:
                    events.on_output(output_table.out_filepath)

                indexer.write_entry(
                    output_table=output_table,
                    book_cfg=book_cfg,
                    pages=pages,
                )
            finally:
                if events.on_progress:
                    events.on_progress(Progress(i, len(output_tables)))
