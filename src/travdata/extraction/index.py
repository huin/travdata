# -*- coding: utf-8 -*-
"""Code to create/update an index of output data."""

import contextlib
import csv
import pathlib
from typing import Iterable, Iterator, Protocol

from travdata import config, csvutil, filesio


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


class Writer(Protocol):
    """Creates or updates an index."""

    def write_entry(
        self,
        output_path: pathlib.PurePath,
        table: config.Table,
        book_cfg: config.Book,
        pages: Iterable[int],
    ) -> None:
        """Write an index entry.

        :param output_path: Path to the table file within the output.
        :param table: Table being output.
        :param book_cfg: Book configuration.
        :param pages: Page numbers that the entry was sourced from.
        """


@contextlib.contextmanager
def writer(read_writer: filesio.ReadWriter) -> Iterator[Writer]:
    """Creates a context manager for an ``Indexer``.

    :param read_writer: ReadWriter containing the index to create or update.
    :yield: The ``Indexer``.
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

        self = _WriterImpl(write_csv)
        yield self

        for row in existing_rows:
            if row.get(_INDEX_TABLE_PATH, None) in self.seen_paths:
                continue
            write_csv.writerows(existing_rows)


class _WriterImpl:
    _write_csv: csv.DictWriter
    seen_paths: set[str]

    def __init__(self, write_csv: csv.DictWriter) -> None:
        self._write_csv = write_csv
        self.seen_paths = set()

    def write_entry(
        self,
        output_path: pathlib.PurePath,
        table: config.Table,
        book_cfg: config.Book,
        pages: Iterable[int],
    ) -> None:
        """Write an index entry."""
        path = str(output_path)
        self._write_csv.writerow(
            {
                _INDEX_TABLE_PATH: path,
                _INDEX_PAGES: ";".join(str(book_cfg.page_offset + page) for page in sorted(pages)),
                _INDEX_TAGS: ";".join(sorted(table.tags)),
            }
        )
        self.seen_paths.add(path)
