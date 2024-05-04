# -*- coding: utf-8 -*-
"""Code to create/update an index of output data."""

import collections
import contextlib
import csv
import dataclasses
import pathlib
from typing import Iterable, Iterator, Protocol, Self

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


@dataclasses.dataclass
class _Entry:
    path: pathlib.PurePath
    tags: list[str]


class Index:
    """Index of all extracted tables in an output."""

    _paths: frozenset[pathlib.PurePath]
    _tags_to_paths: dict[str, set[pathlib.PurePath]]

    def __init__(self, entries: Iterable[_Entry]) -> None:
        """Initialises the index."""
        self._tags_to_paths = collections.defaultdict(set)
        paths: list[pathlib.PurePath] = []
        for entry in entries:
            paths.append(entry.path)
            for tag in entry.tags:
                self._tags_to_paths[tag].add(entry.path)
        self._paths = frozenset(paths)

    def paths_with_all_tags(self, tags: Iterable[str]) -> Iterable[pathlib.PurePath]:
        """Returns paths to tables with all of the given tags.

        :param tags: Tags to select for.
        :return: Paths of matching tables. Returns all tables if ``tags`` is empty.
        """
        matches: frozenset[pathlib.PurePath] = self._paths
        for tag in tags:
            matches &= self._tags_to_paths[tag]
        return matches

    @classmethod
    def read(cls, reader: filesio.Reader) -> Self:
        """Parses and returns an index from the ``Reader``.

        :param reader: Reader containing the index to read.
        :return: Parsed index.
        """

        def parse_rows(rows: Iterable[dict[str, str]]) -> Iterator[_Entry]:
            for row in rows:
                yield _Entry(
                    pathlib.PurePath(row[_INDEX_TABLE_PATH]),
                    row[_INDEX_TAGS].split(";"),
                )

        with csvutil.open_by_reader(reader, _INDEX_PATH) as read_io:
            read_csv = csv.DictReader(read_io)
            return cls(parse_rows(read_csv))


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
