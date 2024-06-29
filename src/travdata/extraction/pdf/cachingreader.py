# -*- coding: utf-8 -*-
"""Provides a caching wrapper around a ``TableReader``"""

from collections.abc import MutableMapping
import contextlib
import dataclasses
import hashlib
import io
import json
import pathlib
from typing import IO, Iterator, Optional, Self, TypedDict

import cachetools
import xdg_base_dirs

from travdata.extraction.pdf import tablereader


# 1MiB max size (of cached values).
_MAX_CACHE_SIZE = 1 << 20

_PERSIST_CACHE_VERSION = "1"


@contextlib.contextmanager
def optional_table_cache(
    delegate: tablereader.TableReader,
    disable: bool,
    cache_path: Optional[pathlib.Path] = None,
) -> Iterator[tablereader.TableReader]:
    """Conditionally injects a CachingTableReader."""
    if disable:
        yield delegate
    else:
        if cache_path is None:
            cache_path = xdg_base_dirs.xdg_cache_home() / "travdata" / "table-cache.json"
        with CachingTableReader(
            delegate=delegate,
            cache_path=cache_path,
        ) as caching_reader:
            yield caching_reader


def _key(
    pdf_hash: bytes,
    template_file: IO[str],
) -> tuple[str, IO[str]]:
    template_content = template_file.read()
    template_content_reader = io.StringIO(template_content)

    hasher = hashlib.sha256(usedforsecurity=False)
    hasher.update(pdf_hash)
    hasher.update(template_content.encode())
    return hasher.hexdigest(), template_content_reader


@dataclasses.dataclass
class _FileHashEntry:
    digest: bytes
    st_size: int
    st_mtime: int


class _PersistantCache(TypedDict):
    version: str
    entries: dict[str, list[tablereader.ExtractedTable]]


class CachingTableReader:
    """Caching wrapper around a ``TableReader``.

    Particularly useful when delegating to ``TabulaClient``, which can be quite
    slow.
    """

    _delegate: tablereader.TableReader
    _file_hash_cache: MutableMapping[pathlib.Path, _FileHashEntry]
    _tables_cache_path: pathlib.Path
    _tables_cache: MutableMapping[str, list[tablereader.ExtractedTable]]

    def __init__(
        self,
        delegate: tablereader.TableReader,
        cache_path: pathlib.Path,
    ) -> None:
        self._delegate = delegate
        self._file_hash_cache = cachetools.LRUCache(maxsize=100)
        self._tables_cache_path = cache_path
        self._tables_cache = cachetools.LRUCache(maxsize=_MAX_CACHE_SIZE, getsizeof=len)

    def __enter__(self) -> Self:
        try:
            with self._tables_cache_path.open("rt") as f:
                loaded_cache: _PersistantCache = json.load(f)
        except (IOError, json.JSONDecodeError):
            # Failed to load cache, carry on anyway.
            return self

        try:
            if loaded_cache["version"] != _PERSIST_CACHE_VERSION:
                # Incompatible version.
                return self

            for key, value in loaded_cache["entries"].items():
                self._tables_cache[key] = value
        except (KeyError, TypeError):
            # Failed to load cache, carry on anyway.
            return self

        return self

    def __exit__(self, *args) -> None:
        del args  # unused

        dumped_cache: _PersistantCache = {
            "version": _PERSIST_CACHE_VERSION,
            "entries": {},
        }
        dumped_cache["entries"].update(self._tables_cache)

        self._tables_cache_path.parent.mkdir(parents=True, exist_ok=True)

        with self._tables_cache_path.open("wt") as f:
            json.dump(dumped_cache, f)

    def _hash_file(
        self,
        path: pathlib.Path,
    ) -> bytes:
        abs_path = path.absolute()
        file_stat = abs_path.stat()
        size = file_stat.st_size
        mtime = int(file_stat.st_mtime)
        try:
            hash_entry = self._file_hash_cache[abs_path]
        except KeyError:
            # Cache miss.
            pass
        else:
            if (
                hash_entry.st_size == file_stat.st_size
                and hash_entry.st_mtime == file_stat.st_mtime
            ):
                return hash_entry.digest
            # Cache entry invalid.

        with path.open("rb") as f:
            digest = hashlib.file_digest(f, hashlib.sha256).digest()

        self._file_hash_cache[abs_path] = _FileHashEntry(
            digest=digest,
            st_size=size,
            st_mtime=mtime,
        )

        return digest

    def read_pdf_with_template(
        self,
        *,
        pdf_path: pathlib.Path,
        template_file: IO[str],
    ) -> list[tablereader.ExtractedTable]:
        """Caching implementation of ``TableReader.read_pdf_with_templates``."""
        pdf_hash = self._hash_file(pdf_path)

        key, template_mem_file = _key(pdf_hash, template_file)
        try:
            return self._tables_cache[key]
        except KeyError:
            pass

        tables = self._delegate.read_pdf_with_template(
            pdf_path=pdf_path,
            template_file=template_mem_file,
        )
        self._tables_cache[key] = tables

        return tables
