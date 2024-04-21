# -*- coding: utf-8 -*-
"""Tests for travdata.filesio."""
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

from contextlib import AbstractContextManager
import pathlib
import tempfile

import pytest
import testfixtures  # type: ignore[import-untyped]

from travdata import filesio


def test_dir_roundtrip() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        zip_path = pathlib.Path(tmpdir) / "dir"
        _roundtrip_test(
            reader_ctx=filesio.DirReader.open(zip_path),
            writer_ctx=filesio.DirWriter.create(zip_path),
        )


def test_mem_roundtrip() -> None:
    files: dict[pathlib.PurePath, str] = {}
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
        (pathlib.PurePath("foo.txt"), "foo contents"),
        (pathlib.PurePath("bar/baz.txt"), "baz contents"),
    ]
    not_exist_files = [
        pathlib.PurePath("not-exist.txt"),
        pathlib.PurePath("no-dir/not-exist.txt"),
    ]

    expected_paths: list[pathlib.PurePath] = []
    with writer_ctx as writer:
        for path, contents in files:
            expected_paths.append(path)
            with writer.open_write(path) as fw:
                fw.write(contents)
            assert writer.exists(path)

        for path in not_exist_files:
            assert not writer.exists(pathlib.PurePath(path))

    expected_paths.sort()

    with reader_ctx as reader:
        for path_str, want_contents in files:
            path = pathlib.PurePath(path_str)
            with reader.open_read(path) as fr:
                got_contents = fr.read()
                assert want_contents == got_contents

        for path in not_exist_files:
            with pytest.raises(filesio.NotFoundError):
                with reader.open_read(path):
                    pass

        actual_paths = sorted(reader.iter_files())

    testfixtures.compare(expected=expected_paths, actual=actual_paths)
