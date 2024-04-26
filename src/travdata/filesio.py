# -*- coding: utf-8 -*-
"""Provides types to read and write a collection of related files."""

# pylint: disable=too-few-public-methods

import contextlib
import enum
import io
import os
import pathlib
from typing import IO, Iterator, Protocol, Self
import zipfile


_ENCODING = "utf-8"
_NEWLINE = "\n"


class Error(Exception):
    """Base exception emitted by filesio."""


class NotFoundError(Error):
    """Attempted to read a file that does not exist."""


class IOType(enum.StrEnum):
    """
    Enumeration of readers/writer types that can be constructed with a
    filesystem path.
    """

    # Resolution needed:
    AUTO = "AUTO"
    # Concrete values:
    DIR = "DIR"
    ZIP = "ZIP"

    @classmethod
    def from_int_id(cls, id_: int) -> "IOType":
        """Converts from an integer ID to IOType."""
        match id_:
            case 1:
                return IOType.AUTO
            case 2:
                return IOType.DIR
            case 3:
                return IOType.ZIP
            case _:
                raise ValueError(id_)

    def to_int_id(self) -> int:
        """Converts from an IOType to integer ID."""
        match self:
            case IOType.AUTO:
                return 1
            case IOType.DIR:
                return 2
            case IOType.ZIP:
                return 3
            case _:
                raise ValueError(self)

    def open(
        self,
        path: pathlib.Path,
    ) -> contextlib.AbstractContextManager["Reader"]:
        """Creates a context manager for a ``Reader``.

        :param path: Path for the reader to read.
        :raises Error: If ``self`` is an unknown or unresolved value like
        ``AUTO``.
        :return: Context manager for a ``Reader``.
        """
        match self:
            case IOType.DIR:
                return DirReader.open(path)
            case IOType.ZIP:
                return ZipReader.open(path)
            case _:
                raise Error(f"cannot open output type {self} with a path")

    def create(
        self,
        path: pathlib.Path,
    ) -> contextlib.AbstractContextManager["Writer"]:
        """Creates a context manager for a ``Writer``.

        :param path: Path for the writer to create.
        :raises Error: If ``self`` is an unknown or unresolved value like
        ``AUTO``.
        :return: Context manager for a ``Writer``.
        """
        match self:
            case IOType.DIR:
                return DirWriter.create(path)
            case IOType.ZIP:
                return ZipWriter.create(path)
            case _:
                raise Error(f"cannot open output type {self} with a path")

    def resolve_auto(self, path: pathlib.Path) -> "IOType":
        """Returns a concrete IOType for the given filesystem ``path``.

        :param path: Filesystem path to aid resolution.
        :return: If ``self`` is ``AUTO``, then a concrete value or ``self`` if
        concrete.
        """
        if self != IOType.AUTO:
            return self

        if not path.exists():
            if path.suffix == ".zip":
                return IOType.ZIP
            return IOType.DIR
        if path.is_file():
            return IOType.ZIP
        # Fall back to guessing as directory.
        return IOType.DIR


class Reader(Protocol):
    """Protocol for reading files from the collection."""

    def open_read(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> contextlib.AbstractContextManager[IO[str]]:
        """Open a text file for reading.

        :param path: Path of the file to read.
        :param newline: Newline sequence to use.
        :return: Context-managed readable file-like object.
        :raises NotFoundError: If the ``path`` does not exist.
        """
        ...

    def iter_files(self) -> Iterator[pathlib.PurePath]:
        """Iterates over all files that the reader has.

        :yield: Paths of individual files. The order is undefined.
        """
        ...


class Writer(Protocol):
    """Protocol for writing files to the collection."""

    def open_write(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> contextlib.AbstractContextManager[IO[str]]:
        """Open a text file for writing.

        :param path: Path of the file to write.
        :param newline: Newline sequence to use.
        :return: Context-managed writable file-like object.
        """
        ...

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Return ``True`` if the file exists.

        :param path: Path to the file.
        :return: ``True`` if the file exists, otherwise ``False``.
        """
        ...


class DirReader(Reader):
    """Reads files from a local filesystem directory."""

    _dir_path: pathlib.Path

    def __init__(self, dir_path: pathlib.Path) -> None:
        """Initialise the DirReader to read from the given directory."""
        self._dir_path = dir_path

    @classmethod
    @contextlib.contextmanager
    def open(cls, dir_path: pathlib.Path) -> Iterator[Self]:
        """Create a DirReader to read from the given directory."""
        yield cls(dir_path)

    @property
    def dir_path(self) -> pathlib.Path:
        """Returns the directory path."""
        return self._dir_path

    def open_read(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> contextlib.AbstractContextManager[IO[str]]:
        """Implements Reader.open_read."""
        full_path = self._dir_path / path
        try:
            f = full_path.open("rt", encoding=_ENCODING, newline=newline)
        except FileNotFoundError as exc:
            raise NotFoundError(path) from exc
        return f

    def iter_files(self) -> Iterator[pathlib.PurePath]:
        """Implements Reader.iter_files."""
        for root, _, files in os.walk(self._dir_path):
            for filename in files:
                full_path = pathlib.PurePath(root) / filename
                yield full_path.relative_to(self._dir_path)


class DirWriter(Writer):
    """Writes files to a local filesystem directory."""

    _dir_path: pathlib.Path
    _created_dirs: set[pathlib.Path]

    def __init__(self, dir_path: pathlib.Path) -> None:
        """Initialise the DirWriter to write to the given directory."""
        self._dir_path = dir_path
        self._created_dirs = set()

    @classmethod
    @contextlib.contextmanager
    def create(cls, dir_path: pathlib.Path) -> Iterator[Self]:
        """Create a DirWriter to write to the given directory."""
        yield cls(dir_path)

    def open_write(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> contextlib.AbstractContextManager[IO[str]]:
        """Implements Writer.open_write."""
        full_path = self._dir_path / path
        parent_dir = full_path.parent
        if parent_dir not in self._created_dirs:
            parent_dir.mkdir(parents=True, exist_ok=True)
            self._created_dirs.add(parent_dir)
        return full_path.open("wt", encoding=_ENCODING, newline=newline)

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Implements Writer.exists."""
        return (self._dir_path / path).exists()


class MemReader(Reader):
    """Reads files from in-memory files."""

    _files: dict[pathlib.PurePath, str]

    def __init__(self, files: dict[pathlib.PurePath, str]) -> None:
        """Initialise the MemReader to read from the given files."""
        self._files = files

    @classmethod
    @contextlib.contextmanager
    def open(cls, files: dict[pathlib.PurePath, str]) -> Iterator[Self]:
        """Create a MemReader to read from the given files."""
        yield cls(files)

    @contextlib.contextmanager
    def open_read(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> Iterator[IO[str]]:
        """Implements Reader.open_read."""
        try:
            contents = self._files[path]
        except KeyError as exc:
            raise NotFoundError(path) from exc
        yield io.StringIO(contents, newline=newline)

    def iter_files(self) -> Iterator[pathlib.PurePath]:
        """Implements Reader.iter_files."""
        return iter(self._files.keys())


class MemWriter(Writer):
    """Writes files to in-memory files."""

    _files: dict[pathlib.PurePath, str]

    def __init__(self, files: dict[pathlib.PurePath, str]) -> None:
        """Initialise the MemWriter to write to the given files."""
        self._files = files

    @classmethod
    @contextlib.contextmanager
    def create(cls, files: dict[pathlib.PurePath, str]) -> Iterator[Self]:
        """Create a MemReader to write to the given files."""
        yield cls(files)

    @contextlib.contextmanager
    def open_write(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> Iterator[IO[str]]:
        """Implements Writer.open_write."""
        f = io.StringIO(newline=newline)
        try:
            yield f
        finally:
            f.seek(0, io.SEEK_SET)
            self._files[path] = f.read()

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Implements Writer.exists."""
        return path in self._files


class ZipReader(Reader):
    """Reads files from a ZIP file."""

    _zip_file: zipfile.ZipFile

    def __init__(self, zip_file: zipfile.ZipFile) -> None:
        """Initialise the ZipReader to read from the opened ZIP file."""
        self._zip_file = zip_file

    @classmethod
    @contextlib.contextmanager
    def open(cls, zip_path: pathlib.Path) -> Iterator[Self]:
        """Create a ZipReader to write to the ZIP file at the path."""
        zip_file = zipfile.ZipFile(zip_path, "r")
        try:
            yield cls(zip_file)
        finally:
            zip_file.close()

    @contextlib.contextmanager
    def open_read(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> Iterator[IO[str]]:
        """Implements Reader.open_read."""
        try:
            f = self._zip_file.open(str(path), "r")
        except KeyError as exc:
            raise NotFoundError(path) from exc
        with f:
            yield io.TextIOWrapper(f, encoding=_ENCODING, newline=newline)

    def iter_files(self) -> Iterator[pathlib.PurePath]:
        """Implements Reader.iter_files."""
        for info in self._zip_file.infolist():
            yield pathlib.PurePath(info.filename)


class ZipWriter(Writer):
    """Writes files to a ZIP file."""

    _zip_file: zipfile.ZipFile

    def __init__(self, zip_file: zipfile.ZipFile) -> None:
        """Initialise the ZipWriter to write to the opened ZIP file."""
        self._zip_file = zip_file

    @classmethod
    @contextlib.contextmanager
    def create(cls, zip_path: pathlib.Path) -> Iterator[Self]:
        """Create a ZipWriter to write to a new ZIP file at the path."""
        zip_path.parent.mkdir(parents=True, exist_ok=True)
        zip_file = zipfile.ZipFile(zip_path, "w")
        try:
            yield cls(zip_file)
        finally:
            zip_file.close()

    @contextlib.contextmanager
    def open_write(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> Iterator[IO[str]]:
        """Implements Writer.open_write."""
        with self._zip_file.open(str(path), "w") as f:
            fw = io.TextIOWrapper(f, encoding=_ENCODING, newline=newline)
            try:
                yield fw
            finally:
                fw.flush()

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Implements Writer.exists."""
        try:
            self._zip_file.getinfo(str(path))
        except KeyError:
            return False
        return True
