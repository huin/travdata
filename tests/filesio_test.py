# -*- coding: utf-8 -*-
"""Tests for travdata.filesio."""
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

from contextlib import AbstractContextManager
import pathlib
import tempfile

from travdata import filesio


def test_dir_roundtrip() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        zip_path = pathlib.Path(tmpdir) / "dir"
        _roundtrip_test(
            reader_ctx=filesio.DirReader.open(zip_path),
            writer_ctx=filesio.DirWriter.create(zip_path),
        )


def test_mem_roundtrip() -> None:
    files: dict[pathlib.PurePath, bytes] = {}
    _roundtrip_test(
        reader_ctx=filesio.MemReader.open(files),
        writer_ctx=filesio.MemWriter.create(files),
    )


def test_zip_roundtrip() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        zip_path = pathlib.Path(tmpdir) / "files.zip"
        _roundtrip_test(
            reader_ctx=filesio.ZipReader.open(zip_path),
            writer_ctx=filesio.ZipWriter.create(zip_path),
        )


def _roundtrip_test(
    reader_ctx: AbstractContextManager[filesio.Reader],
    writer_ctx: AbstractContextManager[filesio.Writer],
) -> None:
    files = [
        ("foo.txt", "foo contents"),
        ("bar/baz.txt", "baz contents"),
    ]

    with writer_ctx as writer:
        for path_str, contents in files:
            path = pathlib.PurePath(path_str)
            with writer.open_write(path) as fw:
                fw.write(contents.encode("utf-8"))

    with reader_ctx as reader:
        for path_str, want_contents in files:
            path = pathlib.PurePath(path_str)
            with reader.open_read(path) as fr:
                got_contents = fr.read().decode("utf-8")
                assert want_contents == got_contents
