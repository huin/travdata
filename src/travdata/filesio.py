# -*- coding: utf-8 -*-
"""Provides types to read and write a collection of related files."""

# pylint: disable=too-few-public-methods

import contextlib
import enum
import io
import os
import pathlib
import shutil
import tempfile
from typing import IO, Iterator, Optional, Protocol, Self
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

    def new_reader(
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
                return DirReader.new_reader(path)
            case IOType.ZIP:
                return ZipReader.new_reader(path)
            case _:
                raise Error(f"cannot open output type {self} with a path")

    def new_read_writer(
        self,
        path: pathlib.Path,
    ) -> contextlib.AbstractContextManager["ReadWriter"]:
        """Creates a context manager for a ``ReadWriter``.

        :param path: Path for the writer to create.
        :raises Error: If ``self`` is an unknown or unresolved value like
        ``AUTO``.
        :return: Context manager for a ``ReadWriter``.
        """
        match self:
            case IOType.DIR:
                return DirReadWriter.new_read_writer(path)
            case IOType.ZIP:
                return ZipReadWriter.new_read_writer(path)
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

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Return ``True`` if the file exists.

        :param path: Path to the file.
        :return: ``True`` if the file exists, otherwise ``False``.
        """
        ...


class ReadWriter(Reader, Protocol):
    """Protocol for reading and writing files in the collection."""

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


class DirReader:
    """Reads files in a local filesystem directory."""

    _dir_path: pathlib.Path
    _created_dirs: set[pathlib.Path]

    def __init__(self, dir_path: pathlib.Path) -> None:
        """Initialise the DirReadWriter to read/write in the given directory."""
        self._dir_path = dir_path
        self._created_dirs = set()

    @classmethod
    @contextlib.contextmanager
    def new_reader(cls, dir_path: pathlib.Path) -> Iterator[Self]:
        """Create a DirReader to read in the given directory."""
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

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Implements Reader.exists."""
        return (self._dir_path / path).exists()


class DirReadWriter(DirReader):
    """Reads and writes files in a local filesystem directory."""

    @classmethod
    @contextlib.contextmanager
    def new_read_writer(cls, dir_path: pathlib.Path) -> Iterator[Self]:
        """Create a DirReadWriter to read and write in the given directory."""
        yield cls(dir_path)

    def open_write(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> contextlib.AbstractContextManager[IO[str]]:
        """Implements ReadWriter.open_write."""
        full_path = self._dir_path / path
        parent_dir = full_path.parent
        if parent_dir not in self._created_dirs:
            parent_dir.mkdir(parents=True, exist_ok=True)
            self._created_dirs.add(parent_dir)
        return full_path.open("wt", encoding=_ENCODING, newline=newline)


class MemReader:
    """Reads in-memory files."""

    _files: dict[pathlib.PurePath, str]

    def __init__(self, files: dict[pathlib.PurePath, str]) -> None:
        """Initialise the MemReader to read from the given files."""
        self._files = files

    @classmethod
    @contextlib.contextmanager
    def new_reader(cls, files: dict[pathlib.PurePath, str]) -> Iterator[Self]:
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

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Implements Reader.exists."""
        return path in self._files


class MemReadWriter(MemReader):
    """Reads and writes in-memory files."""

    @classmethod
    @contextlib.contextmanager
    def new_read_writer(cls, files: dict[pathlib.PurePath, str]) -> Iterator[Self]:
        """Create a MemReadWriter to read and write in the given files."""
        yield cls(files)

    @contextlib.contextmanager
    def open_write(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> Iterator[IO[str]]:
        """Implements ReadWriter.open_write."""
        f = io.StringIO(newline=newline)
        try:
            yield f
        finally:
            f.seek(0, io.SEEK_SET)
            self._files[path] = f.read()


class ZipReader(Reader):
    """Reads files from a ZIP file."""

    _zip_file: Optional[zipfile.ZipFile]

    def __init__(self, zip_file: Optional[zipfile.ZipFile]) -> None:
        """Initialise the ZipReader to read from the opened ZIP file."""
        self._zip_file = zip_file

    @classmethod
    @contextlib.contextmanager
    def new_reader(
        cls,
        zip_path: pathlib.Path,
        ignore_not_exist: bool = False,
    ) -> Iterator[Self]:
        """Create a ZipReader to write to the ZIP file at the path.

        :param zip_path: Path to the ZIP file to open for reading.
        :param ignore_not_exist: If True, then ignore ``zip_path`` not existing
        on the filesystem. If the ZIP file does not exist, then the produced
        ``ZipReader`` will behave as if no files exist within it.
        :raises NotFoundError: If ``zip_path`` does not exist, and
        ``ignore_not_exist`` is False.
        :returns: Context managed ZipReader.
        """
        try:
            zip_file = zipfile.ZipFile(zip_path, "r")
        except FileNotFoundError:
            if not ignore_not_exist:
                raise
            zip_file = None

        try:
            yield cls(zip_file)
        finally:
            if zip_file is not None:
                zip_file.close()

    @contextlib.contextmanager
    def open_read(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> Iterator[IO[str]]:
        """Implements Reader.open_read."""
        if self._zip_file is None:
            raise NotFoundError(path)

        try:
            f = self._zip_file.open(str(path), "r")
        except KeyError as exc:
            raise NotFoundError(path) from exc
        with f:
            yield io.TextIOWrapper(f, encoding=_ENCODING, newline=newline)

    def iter_files(self) -> Iterator[pathlib.PurePath]:
        """Implements Reader.iter_files."""
        if self._zip_file is None:
            return

        for info in self._zip_file.infolist():
            yield pathlib.PurePath(info.filename)

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Implements Reader.exists."""
        if self._zip_file is None:
            return False

        try:
            self._zip_file.getinfo(str(path))
        except KeyError:
            return False
        return True


class ZipReadWriter:
    """Reads and writes files in a ZIP file."""

    # _zip_reader is a reader for existing files from an existing Zip file being
    # updated, if one exists.
    _zip_reader: ZipReader
    # _read_writer is a staging area for new files.
    _read_writer: ReadWriter

    def __init__(self, zip_reader: ZipReader, read_writer: ReadWriter) -> None:
        """Initialise the ZipReadWriter to write to the opened ZIP file."""
        self._zip_reader = zip_reader
        self._read_writer = read_writer

    @classmethod
    @contextlib.contextmanager
    def new_read_writer(cls, zip_path: pathlib.Path) -> Iterator[Self]:
        """Create a ZipReadWriter to write to a new ZIP file at the path."""
        with (
            tempfile.TemporaryDirectory() as tmpdir,
            ZipReader.new_reader(zip_path, ignore_not_exist=True) as zip_reader,
            DirReadWriter.new_reader(pathlib.Path(tmpdir)) as read_writer,
        ):
            zrw = cls(zip_reader, read_writer)
            yield zrw

            if next(read_writer.iter_files(), None) is None:
                # No files changed, so can leave the existing ZIP file as-is.
                return

            dest_dir = zip_path.parent
            dest_dir.mkdir(parents=True, exist_ok=True)
            new_zip_path = _copy_reader_into_zipfile(zrw, dest_dir)

        shutil.move(new_zip_path, zip_path)

    @contextlib.contextmanager
    def open_read(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> Iterator[IO[str]]:
        """Implements Reader.open_read."""
        try:
            with self._read_writer.open_read(path, newline) as r:
                yield r
        except NotFoundError:
            with self._zip_reader.open_read(path, newline) as r:
                yield r

    def iter_files(self) -> Iterator[pathlib.PurePath]:
        """Implements Reader.iter_files."""
        seen: set[pathlib.PurePath] = set()
        for path in self._read_writer.iter_files():
            yield path
            seen.add(path)
        for path in self._zip_reader.iter_files():
            if path not in seen:
                yield path

    def exists(
        self,
        path: pathlib.PurePath,
    ) -> bool:
        """Implements Reader.exists."""
        return self._read_writer.exists(path) or self._zip_reader.exists(path)

    @contextlib.contextmanager
    def open_write(
        self,
        path: pathlib.PurePath,
        newline: str = _NEWLINE,
    ) -> Iterator[IO[str]]:
        """Implements ReadWriter.open_write."""
        with self._read_writer.open_write(path, newline) as w:
            yield w


def _copy_reader_into_zipfile(
    reader: Reader,
    dest_dir: pathlib.Path,
) -> pathlib.Path:
    # mkstemp creates a new uniquely named file, but does not delete when we're
    # done with it, which works nicely to create the new ZIP file without
    # overwriting any existing file.
    zf_fd, zf_path_str = tempfile.mkstemp(suffix=".zip", dir=dest_dir)
    with (
        os.fdopen(zf_fd, mode="wb") as zf,
        zipfile.ZipFile(zf, mode="w") as zw,
    ):
        for path in reader.iter_files():
            with (
                reader.open_read(path, newline="") as r,
                zw.open(str(path), mode="w") as w,
            ):
                wb = io.TextIOWrapper(w, encoding=_ENCODING, newline="")
                n = 0
                while data := r.read(64 * 1024):
                    wb.write(data)
                    n += len(data)
                wb.flush()

    return pathlib.Path(zf_path_str)
