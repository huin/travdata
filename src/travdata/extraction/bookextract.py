# -*- coding: utf-8 -*-
"""Extracts multiple tables from a PDF."""

import abc
import csv
import dataclasses
import pathlib
import traceback
from typing import Callable, Iterable, Iterator, Self

from travdata import config, csvutil, filesio
from travdata.config import cfgerror
from travdata.extraction import ecmastransform, index, tableextract
from travdata.extraction.pdf import tablereader


@dataclasses.dataclass(frozen=True)
class ExtractionConfig:
    """Extraction configuration.

    :field cfg_reader_type_path: File IO type and path to use in reading
    configuration.
    :field out_writer_type_path: File IO type and path to use in writing output.
    :field input_pdf: Path to PDF file to extract from.
    :field book_id: ID of book to extract.
    :field overwrite_existing: If true, overwrite existing CSV files.
    :field with_tags: Only extracts tables that have any of these these tags.
    :field without_tags: Only extracts tables that do not include any of these
    tags (takes precedence over with_tags).
    """

    cfg_reader_type_path: filesio.IOTypePath
    out_writer_type_path: filesio.IOTypePath
    input_pdf: pathlib.Path
    book_id: str
    overwrite_existing: bool
    with_tags: frozenset[str]
    without_tags: frozenset[str]


@dataclasses.dataclass(frozen=True)
class _OutputTable:
    out_filepath: pathlib.PurePath
    table: config.Table


def _filter_tables(
    ext_cfg: ExtractionConfig,
    book_group: config.Group,
    out_writer: filesio.ReadWriter,
) -> Iterator[_OutputTable]:
    for table in book_group.all_tables():
        if table.disable_extraction:
            continue
        out_filepath = table.file_stem.with_suffix(".csv")

        if ext_cfg.with_tags and not table.tags & ext_cfg.with_tags:
            continue

        if ext_cfg.without_tags and table.tags & ext_cfg.without_tags:
            continue

        if not ext_cfg.overwrite_existing and out_writer.exists(out_filepath):
            continue

        yield _OutputTable(out_filepath, table)


def _init_ecmas_trn(
    modules: Iterable[pathlib.PurePath],
    ecmas_trn: ecmastransform.Transformer,
) -> None:
    for module_path in modules:
        ecmas_trn.load_module(module_path)


def _extract_single_table(  # pylint: disable=too-many-arguments
    *,
    cfg_reader: filesio.Reader,
    out_writer: filesio.ReadWriter,
    table_reader: tablereader.TableReader,
    ecmas_trn: ecmastransform.Transformer,
    input_pdf: pathlib.Path,
    output_table: _OutputTable,
) -> set[int]:
    """Helper wrapper of `extract_table` for `extract_book`, returning page numbers."""
    pages, rows = tableextract.extract_table(
        cfg_reader=cfg_reader,
        table=output_table.table,
        pdf_path=input_pdf,
        table_reader=table_reader,
        ecmas_trn=ecmas_trn,
    )
    with csvutil.open_by_read_writer(out_writer, output_table.out_filepath) as f:
        csv.writer(f).writerows(rows)
    return pages


class ExtractEvent(abc.ABC):
    """Abstract marker baseclass for extraction events."""


@dataclasses.dataclass(frozen=True)
class EndedEvent(ExtractEvent):
    """Reports that extraction has ended. This will be the terminal event."""

    abnormal: bool


@dataclasses.dataclass(frozen=True)
class ErrorEvent(ExtractEvent):
    """Report of an error during extraction."""

    message: str


@dataclasses.dataclass(frozen=True)
class FileOutputEvent(ExtractEvent):
    """Report of an output file sucessfully output."""

    path: pathlib.PurePath


@dataclasses.dataclass(frozen=True)
class ProgressEvent(ExtractEvent):
    """Progress report from ``extract_book``."""

    completed: int
    total: int


@dataclasses.dataclass
class _Configs:
    cfg: config.Config
    book_cfg: config.Book
    book_grp: config.Group

    @classmethod
    def load(cls, cfg_reader: filesio.Reader, book_id: str) -> Self:
        """Loads configuration from reader."""
        cfg = config.load_config(cfg_reader)
        try:
            book_cfg = cfg.books[book_id]
        except KeyError as exc:
            raise cfgerror.ConfigurationError(
                f"Book {book_id} not found in configuration."
            ) from exc

        book_grp = book_cfg.load_group(cfg_reader)

        return cls(cfg=cfg, book_cfg=book_cfg, book_grp=book_grp)


def _extract_book_core(
    *,
    table_reader: tablereader.TableReader,
    ext_cfg: ExtractionConfig,
    do_continue: Callable[[], bool],
) -> Iterator[ExtractEvent]:
    with (
        ext_cfg.cfg_reader_type_path.new_reader() as cfg_reader,
        ext_cfg.out_writer_type_path.new_read_writer() as out_writer,
        index.writer(out_writer) as indexer,
        ecmastransform.transformer(cfg_reader) as ecmas_trn,
    ):
        try:
            cfgs = _Configs.load(cfg_reader, ext_cfg.book_id)
        except cfgerror.ConfigurationError as exc:
            yield ErrorEvent(
                message=(f"Error reading configuration: {exc}"),
            )
            return

        output_tables = sorted(
            _filter_tables(ext_cfg, cfgs.book_grp, out_writer),
            key=lambda ft: ft.out_filepath,
        )

        _init_ecmas_trn(cfgs.cfg.ecma_script_modules, ecmas_trn)

        yield ProgressEvent(completed=0, total=len(output_tables))

        for i, output_table in enumerate(output_tables, start=1):
            if not do_continue():
                return

            try:
                pages = _extract_single_table(
                    cfg_reader=cfg_reader,
                    out_writer=out_writer,
                    table_reader=table_reader,
                    ecmas_trn=ecmas_trn,
                    input_pdf=ext_cfg.input_pdf,
                    output_table=output_table,
                )
            except cfgerror.ConfigurationError as exc:
                yield ErrorEvent(
                    message=(
                        f"Configuration error while processing table "
                        f"{output_table.table.file_stem}: {exc}"
                    ),
                )
            else:
                yield FileOutputEvent(output_table.out_filepath)

                indexer.write_entry(
                    output_path=output_table.out_filepath,
                    table=output_table.table,
                    book_cfg=cfgs.book_cfg,
                    pages=pages,
                )
            finally:
                yield ProgressEvent(completed=i, total=len(output_tables))


def extract_book(
    *,
    table_reader: tablereader.TableReader,
    ext_cfg: ExtractionConfig,
    do_continue: Callable[[], bool],
) -> Iterator[ExtractEvent]:
    """Extracts an entire book to CSV.

    :param table_reader: Extractor for individual tables from a PDF.
    :param ext_cfg: Configuration for extraction.
    :param do_contiune: Periodically called to check if extraction should
    continue.
    :yields: Events about the extraction process.
    """

    abnormal: bool = False
    try:
        yield from _extract_book_core(
            table_reader=table_reader,
            ext_cfg=ext_cfg,
            do_continue=do_continue,
        )
    except Exception as exc:  # pylint: disable=broad-exception-caught
        details = "".join(traceback.format_exception(exc))
        abnormal = True
        yield ErrorEvent(
            message=f"Unhandled exception during extraction: {details}",
        )
    finally:
        yield EndedEvent(abnormal=abnormal)
