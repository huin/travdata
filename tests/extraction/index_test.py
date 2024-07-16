# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring,redefined-outer-name

import pathlib
from typing import Iterator

import hamcrest as hc
import pytest

from travdata import filesio
from travdata.extraction import index

P1 = pathlib.PurePath("foo.csv")
P2 = pathlib.PurePath("bar.csv")


@pytest.fixture
def read_writer() -> Iterator[filesio.MemReadWriter]:
    with filesio.MemReadWriter.new_read_writer({}) as rw:
        yield rw


def _paths_with_tags(idx: index.Index, *tags: str) -> list[pathlib.PurePath]:
    """Helper to avoid Mypy confusion over hc.assert_that."""
    return list(idx.paths_with_all_tags(tags))


def test_write_and_read_index(read_writer: filesio.MemReadWriter) -> None:
    with index.writer(read_writer) as w:
        w.write_entry(output_path=P1, tags=["1", "2"], pages=[1])
        w.write_entry(output_path=P2, tags=["2", "3"], pages=[5])

    idx = index.Index.read(read_writer)
    results = list(idx.paths_with_all_tags([]))
    hc.assert_that(results, hc.contains_inanyorder(P1, P2))

    hc.assert_that(_paths_with_tags(idx, "1"), hc.contains_inanyorder(P1))
    hc.assert_that(_paths_with_tags(idx, "2"), hc.contains_inanyorder(P1, P2))
    hc.assert_that(_paths_with_tags(idx, "3"), hc.contains_inanyorder(P2))


def test_rewrite_keeps_existing_entry(read_writer: filesio.MemReadWriter) -> None:
    with index.writer(read_writer) as w:
        w.write_entry(output_path=P1, tags=["1", "2"], pages=[1])
    idx = index.Index.read(read_writer)
    hc.assert_that(_paths_with_tags(idx), hc.contains_inanyorder(P1))

    with index.writer(read_writer) as w:
        w.write_entry(output_path=P2, tags=["2", "3"], pages=[5])
    idx = index.Index.read(read_writer)
    hc.assert_that(_paths_with_tags(idx), hc.contains_inanyorder(P1, P2))


def test_rewrite_replaces_existing_tags(read_writer: filesio.MemReadWriter) -> None:
    with index.writer(read_writer) as w:
        w.write_entry(output_path=P1, tags=["1", "2"], pages=[1])
    idx = index.Index.read(read_writer)
    hc.assert_that(_paths_with_tags(idx, "1", "2"), hc.contains_inanyorder(P1))

    with index.writer(read_writer) as w:
        w.write_entry(output_path=P1, tags=["3", "4"], pages=[1])
    idx = index.Index.read(read_writer)
    hc.assert_that(_paths_with_tags(idx, "1", "2"), hc.empty())
    hc.assert_that(_paths_with_tags(idx, "3", "4"), hc.contains_inanyorder(P1))
