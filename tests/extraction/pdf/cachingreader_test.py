# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring,redefined-outer-name

import dataclasses
import json
import pathlib
import tempfile
from typing import Iterator, Self

import hamcrest as hc
import pytest

from travdata.extraction.pdf import cachingreader
from . import pdftestutil


@pytest.fixture
def tempdir() -> Iterator[pathlib.Path]:
    with tempfile.TemporaryDirectory() as tempdir_str:
        yield pathlib.Path(tempdir_str)


@pytest.fixture
def fake_delegate() -> pdftestutil.FakeTableReader:
    return pdftestutil.FakeTableReader()


@pytest.fixture
def caching_reader(
    tempdir: pathlib.Path,
    fake_delegate: pdftestutil.FakeTableReader,
) -> Iterator[cachingreader.CachingTableReader]:
    with cachingreader.CachingTableReader(
        delegate=fake_delegate,
        cache_path=tempdir / "cache",
    ) as caching_reader:
        yield caching_reader


@dataclasses.dataclass(frozen=True)
class File:
    path: pathlib.Path
    original_content: str

    @classmethod
    def create(cls, path: pathlib.Path, content: str) -> Self:
        path.write_text(content)
        return cls(path, original_content=content)


@pytest.fixture
def template_1(tempdir: pathlib.Path) -> File:
    return File.create(tempdir / "template-1.json", json.dumps("template 1"))


@pytest.fixture
def template_2(tempdir: pathlib.Path) -> File:
    return File.create(tempdir / "template-2.json", json.dumps("template 2"))


@pytest.fixture
def pdf_1(tempdir: pathlib.Path) -> File:
    return File.create(tempdir / "1.pdf", "PDF 1 data")


@pytest.fixture
def pdf_1_copy(tempdir: pathlib.Path, pdf_1: File) -> File:
    return File.create(tempdir / "copy-of-1.pdf", pdf_1.original_content)


@pytest.fixture
def pdf_2(tempdir: pathlib.Path) -> File:
    return File.create(tempdir / "2.pdf", "PDF 2 data")


@pytest.fixture
def files_by_name(
    template_1: File,
    template_2: File,
    pdf_1: File,
    pdf_1_copy: File,
    pdf_2: File,
) -> dict[str, File]:
    return {
        "template_1": template_1,
        "template_2": template_2,
        "pdf_1": pdf_1,
        "pdf_1_copy": pdf_1_copy,
        "pdf_2": pdf_2,
    }


@dataclasses.dataclass
class TwoReadsCase:
    first_template: File
    second_template: File
    first_pdf: File
    second_pdf: File


@pytest.fixture(
    params=[
        ("template_1", "template_1", "pdf_1", "pdf_2"),
        ("template_1", "template_1", "pdf_1", "pdf_2"),
        ("template_1", "template_2", "pdf_1", "pdf_1"),
        ("template_1", "template_2", "pdf_1", "pdf_2"),
    ]
)
def distinct_reads(
    request: pytest.FixtureRequest,
    files_by_name: dict[str, File],
) -> TwoReadsCase:
    """Creates permutations for test_does_not_cache_distinct_reads."""
    return TwoReadsCase(
        first_template=files_by_name[request.param[0]],
        second_template=files_by_name[request.param[1]],
        first_pdf=files_by_name[request.param[2]],
        second_pdf=files_by_name[request.param[3]],
    )


def test_does_not_cache_distinct_reads(
    fake_delegate: pdftestutil.FakeTableReader,
    caching_reader: cachingreader.CachingTableReader,
    distinct_reads: TwoReadsCase,
) -> None:
    first_original_tables = [
        pdftestutil.fake_table_data(num_rows=1, num_cols=1, page_number=1),
    ]
    first_expect_call = pdftestutil.Call(
        pdf_path=distinct_reads.first_pdf.path,
        template_content=distinct_reads.first_template.original_content,
    )
    fake_delegate.return_tables[first_expect_call] = first_original_tables

    second_original_tables = [
        pdftestutil.fake_table_data(num_rows=2, num_cols=1, page_number=2),
    ]
    second_expect_call = pdftestutil.Call(
        pdf_path=distinct_reads.second_pdf.path,
        template_content=distinct_reads.second_template.original_content,
    )
    fake_delegate.return_tables[second_expect_call] = second_original_tables

    with distinct_reads.first_template.path.open("rt") as f:
        actual_1 = caching_reader.read_pdf_with_template(
            pdf_path=distinct_reads.first_pdf.path,
            template_file=f,
        )
    hc.assert_that(actual_1, hc.equal_to(first_original_tables))

    with distinct_reads.second_template.path.open("rt") as f:
        actual_2 = caching_reader.read_pdf_with_template(
            pdf_path=distinct_reads.second_pdf.path,
            template_file=f,
        )
    hc.assert_that(actual_2, hc.equal_to(second_original_tables))


@pytest.fixture(
    params=[
        ("template_1", "template_1", "pdf_1", "pdf_1"),
        ("template_1", "template_1", "pdf_2", "pdf_2"),
        ("template_2", "template_2", "pdf_1", "pdf_1"),
        ("template_2", "template_2", "pdf_2", "pdf_2"),
        # Support hashing the PDF and getting a hit on a copy of the PDF at a
        # different path.
        ("template_1", "template_1", "pdf_1", "pdf_1_copy"),
    ]
)
def cache_hit_read(
    request: pytest.FixtureRequest,
    files_by_name: dict[str, File],
) -> TwoReadsCase:
    """Creates permutations for test_cache_hit_two_reads."""
    return TwoReadsCase(
        first_template=files_by_name[request.param[0]],
        second_template=files_by_name[request.param[1]],
        first_pdf=files_by_name[request.param[2]],
        second_pdf=files_by_name[request.param[3]],
    )


def test_cache_hit_two_reads(
    fake_delegate: pdftestutil.FakeTableReader,
    caching_reader: cachingreader.CachingTableReader,
    cache_hit_read: TwoReadsCase,
) -> None:
    original_tables = [
        pdftestutil.fake_table_data(num_rows=1),
        pdftestutil.fake_table_data(num_rows=2),
    ]
    first_expect_call = pdftestutil.Call(
        pdf_path=cache_hit_read.first_pdf.path,
        template_content=cache_hit_read.first_template.original_content,
    )
    # This may or may not be a duplicate of first_expect_call.
    second_expect_call = pdftestutil.Call(
        pdf_path=cache_hit_read.second_pdf.path,
        template_content=cache_hit_read.second_template.original_content,
    )
    fake_delegate.return_tables[first_expect_call] = original_tables
    fake_delegate.return_tables[second_expect_call] = original_tables

    with cache_hit_read.first_template.path.open("rt") as f:
        actual_1 = caching_reader.read_pdf_with_template(
            pdf_path=cache_hit_read.first_pdf.path,
            template_file=f,
        )
    hc.assert_that(actual_1, hc.equal_to(original_tables))

    with cache_hit_read.second_template.path.open("rt") as f:
        actual_2 = caching_reader.read_pdf_with_template(
            pdf_path=cache_hit_read.second_pdf.path,
            template_file=f,
        )
    hc.assert_that(actual_2, hc.equal_to(original_tables))

    hc.assert_that(
        fake_delegate.calls,
        hc.contains_exactly(hc.equal_to(first_expect_call)),
        "Delegate should only have been called once.",
    )


def test_cache_persistance(
    tempdir: pathlib.Path,
    fake_delegate: pdftestutil.FakeTableReader,
    pdf_1: File,
    template_1: File,
) -> None:
    original_tables = [
        pdftestutil.fake_table_data(num_rows=1),
        pdftestutil.fake_table_data(num_rows=2),
    ]
    expect_call = pdftestutil.Call(
        pdf_path=pdf_1.path,
        template_content=template_1.original_content,
    )
    fake_delegate.return_tables[expect_call] = original_tables

    with (
        cachingreader.CachingTableReader(
            delegate=fake_delegate,
            cache_path=tempdir / "cache",
        ) as first_caching_reader,
        template_1.path.open("rt") as f,
    ):
        actual_1 = first_caching_reader.read_pdf_with_template(
            pdf_path=pdf_1.path,
            template_file=f,
        )
    hc.assert_that(actual_1, hc.equal_to(original_tables))
    hc.assert_that(fake_delegate.calls, hc.has_length(1))

    with (
        cachingreader.CachingTableReader(
            delegate=fake_delegate,
            cache_path=tempdir / "cache",
        ) as second_caching_reader,
        template_1.path.open("rt") as f,
    ):
        actual_2 = second_caching_reader.read_pdf_with_template(
            pdf_path=pdf_1.path,
            template_file=f,
        )
    hc.assert_that(actual_2, hc.equal_to(original_tables))
    hc.assert_that(
        fake_delegate.calls, hc.has_length(1), "Delegate should only have been called once."
    )
