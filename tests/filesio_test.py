# -*- coding: utf-8 -*-
"""Tests for travdata.filesio."""
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

from contextlib import AbstractContextManager
import contextlib
import dataclasses
import functools
import pathlib
import tempfile
from typing import Callable, Iterator, TypeAlias

import pytest
from pytest_subtests import SubTests

from travdata import filesio


ReadWriterFactory: TypeAlias = Callable[[], AbstractContextManager[filesio.ReadWriter]]
ReaderFactory: TypeAlias = Callable[[], AbstractContextManager[filesio.Reader]]


@dataclasses.dataclass
class IOContext:
    """Creates Readers and ReadWriters for a given state shared between any created instances."""

    reader_factory: ReaderFactory
    read_writer_factory: ReadWriterFactory

    def reader_factories(self) -> Iterator[tuple[str, ReaderFactory]]:
        yield "Reader", self.reader_factory
        yield "ReadWriter", self.read_writer_factory


MakeIoContext: TypeAlias = Callable[
    [],
    AbstractContextManager[IOContext],
]


def test_dir(subtests: SubTests) -> None:

    @contextlib.contextmanager
    def make_dir_context() -> Iterator[IOContext]:
        with tempfile.TemporaryDirectory() as tmpdir:
            dir_path = pathlib.Path(tmpdir) / "dir"

            yield IOContext(
                reader_factory=functools.partial(filesio.DirReadWriter.new_reader, dir_path),
                read_writer_factory=functools.partial(filesio.DirReadWriter.new_reader, dir_path),
            )

    _test_io(subtests, make_dir_context)


def test_mem(subtests: SubTests) -> None:
    @contextlib.contextmanager
    def make_mem_context() -> Iterator[IOContext]:
        files: dict[pathlib.PurePath, str] = {}

        yield IOContext(
            reader_factory=functools.partial(filesio.MemReadWriter.new_reader, files),
            read_writer_factory=functools.partial(filesio.MemReadWriter.new_reader, files),
        )

    _test_io(subtests, make_mem_context)


def test_zip(subtests: SubTests) -> None:
    @contextlib.contextmanager
    def make_zip_context() -> Iterator[IOContext]:
        with (tempfile.TemporaryDirectory() as tmpdir,):
            zip_path = pathlib.Path(tmpdir) / "dir" / "files.zip"

            yield IOContext(
                reader_factory=functools.partial(
                    filesio.ZipReader.new_reader, zip_path, ignore_not_exist=True
                ),
                read_writer_factory=functools.partial(
                    filesio.ZipReadWriter.new_read_writer, zip_path
                ),
            )

    _test_io(subtests, make_zip_context)


def _test_io(
    subtests: SubTests,
    make_io_context: MakeIoContext,
) -> None:
    """Reusable test that applies to all implementations of Reader/ReadWriter."""
    # Tests that apply to anything implementing filesio.Reader - that is: both
    # Reader and ReadWriter.
    empty_reader_cases: list[Callable[[ReaderFactory], None]] = [
        _empty_reader_has_no_files,
        _empty_reader_not_exists,
        _empty_reader_open_read_raises_not_found,
    ]
    for reader_case in empty_reader_cases:
        with make_io_context() as io_ctx:
            for reader_desc, reader_factory in io_ctx.reader_factories():
                with subtests.test(f"{reader_case.__name__} {reader_desc}"):
                    reader_case(reader_factory)

    # Tests that apply to a combination of Reader and ReadWriter.
    full_cases: list[Callable[[IOContext], None]] = [
        _read_writer_reads_own_file,
        _reads_created_files,
        _readers_iter_files,
        _read_writer_overwrites_file,
    ]
    for full_case in full_cases:
        with (
            subtests.test(full_case.__name__),
            make_io_context() as io_ctx,
        ):
            full_case(io_ctx)


def _empty_reader_has_no_files(rf: ReaderFactory) -> None:
    with rf() as reader:
        assert not sorted(reader.iter_files())


def _empty_reader_not_exists(rf: ReaderFactory) -> None:
    with rf() as reader:
        path = pathlib.PurePath("not-exist")
        assert not reader.exists(path)


def _empty_reader_open_read_raises_not_found(rf: ReaderFactory) -> None:
    with rf() as reader:
        path = pathlib.PurePath("not-exist")
        with pytest.raises(filesio.NotFoundError):
            with reader.open_read(path):
                pass


def _read_writer_reads_own_file(io_ctx: IOContext) -> None:
    with io_ctx.read_writer_factory() as read_writer:
        path = pathlib.PurePath("file.txt")
        contents = "contents"

        with read_writer.open_write(path) as w:
            w.write(contents)

        with read_writer.open_read(path) as r:
            got_contents = r.read()
            assert contents == got_contents


def _reads_created_files(io_ctx: IOContext) -> None:
    files = [
        (pathlib.PurePath("file.txt"), "file contents"),
        (pathlib.PurePath("subdir/other.txt"), "other contents"),
    ]

    with io_ctx.read_writer_factory() as read_writer:
        for path, contents in files:
            with read_writer.open_write(path) as w:
                w.write(contents)

        # Should be present in ReadWriter that created them.
        for path, contents in files:
            with read_writer.open_read(path) as r:
                assert r.read() == contents

    # Should be present in Reader implementations.
    for reader_desc, reader_factory in io_ctx.reader_factories():
        with reader_factory() as reader:
            for path, contents in files:
                with reader.open_read(path) as r:
                    got_contents = r.read()
                    assert contents == got_contents, f"{path=} {reader_desc=}"


def _readers_iter_files(io_ctx: IOContext) -> None:
    paths: list[pathlib.PurePath] = sorted(
        [
            pathlib.PurePath("file.txt"),
            pathlib.PurePath("subdir/other.txt"),
        ]
    )

    with io_ctx.read_writer_factory() as read_writer:
        for path in paths:
            with read_writer.open_write(path) as w:
                w.write("ignored content")

        # Should be present in ReadWriter that created them.
        assert sorted(read_writer.iter_files()) == paths

    # Should be present in Reader implementations.
    for reader_desc, reader_factory in io_ctx.reader_factories():
        with reader_factory() as reader:
            assert sorted(reader.iter_files()) == paths, reader_desc


def _read_writer_overwrites_file(io_ctx: IOContext) -> None:
    path = pathlib.PurePath("file.txt")
    v1 = "content v1"
    v2 = "content v2"
    v3 = "content v3"

    with io_ctx.read_writer_factory() as read_writer:
        with read_writer.open_write(path) as w:
            w.write(v1)
        with read_writer.open_read(path) as r:
            assert v1 == r.read()

    with io_ctx.read_writer_factory() as read_writer:
        with read_writer.open_read(path) as r:
            assert v1 == r.read()
        with read_writer.open_write(path) as w:
            w.write(v2)
        with read_writer.open_read(path) as r:
            assert v2 == r.read()
        with read_writer.open_write(path) as w:
            w.write(v3)
        with read_writer.open_read(path) as r:
            assert v3 == r.read()

    with io_ctx.reader_factory() as reader:
        with reader.open_read(path) as r:
            assert v3 == r.read()
