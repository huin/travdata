# -*- coding: utf-8 -*-
"""CSV utilities."""

import pathlib

from typing import TextIO


def open_read(path: pathlib.Path) -> TextIO:
    """Opens a file in a manner suitable for reading CSV data from.

    :param path: Path to the file to read.
    :return: Opened file.
    """
    return open(path, "rt", encoding="utf-8", newline="")


def open_write(path: pathlib.Path) -> TextIO:
    """Opens a file in a manner suitable for writing CSV data into.

    :param path: Path to the file to write.
    :return: Opened file.
    """
    return open(path, "wt", encoding="utf-8", newline="")
