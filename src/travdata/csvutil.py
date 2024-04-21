# -*- coding: utf-8 -*-
"""CSV utilities."""

import contextlib
import pathlib

from typing import IO

from travdata import filesio


def open_by_reader(
    reader: filesio.Reader,
    path: pathlib.PurePath,
) -> contextlib.AbstractContextManager[IO[str]]:
    """Opens a file in a manner suitable for reading CSV data from.

    :param reader: Reader to use.
    :param path: Path to the file to read.
    :return: Opened file.
    """
    return reader.open_read(path, newline="")


def open_read(
    path: pathlib.Path,
) -> contextlib.AbstractContextManager[IO[str]]:
    """Opens a file in a manner suitable for reading CSV data from.

    :param path: Path to the file to read.
    :return: Opened file.
    """
    return path.open(mode="rt", encoding="utf-8", newline="")


def open_by_writer(
    writer: filesio.Writer,
    path: pathlib.PurePath,
) -> contextlib.AbstractContextManager[IO[str]]:
    """Opens a file in a manner suitable for writing CSV data into.

    :param writer: Writer to use.
    :param path: Path to the file to write.
    :return: Opened file.
    """
    return writer.open_write(path, newline="")


def open_write(
    path: pathlib.Path,
) -> contextlib.AbstractContextManager[IO[str]]:
    """Opens a file in a manner suitable for writing CSV data into.

    :param path: Path to the file to write.
    :return: Opened file.
    """
    return path.open(mode="wt", encoding="utf-8", newline="")
