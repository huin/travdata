# -*- coding: utf-8 -*-
"""
Extracts data tables from the Mongoose Traveller 2022 core rules PDF as
CSV files.
"""

import argparse
import contextlib
import pathlib
import sys
import textwrap
from typing import Callable, Iterator

from progress import bar as progress  # type: ignore[import-untyped]
from travdata import config
from travdata.extraction import pdfextract, tabulautil


def add_subparser(subparsers) -> None:
    """Adds a subcommand parser to ``subparsers``."""
    argparser: argparse.ArgumentParser = subparsers.add_parser(
        "extractcsvtables",
        description=__doc__,
        formatter_class=argparse.RawTextHelpFormatter,
    )
    argparser.set_defaults(run=run)

    argparser.add_argument(
        "book_name",
        help=textwrap.dedent(
            """
            Name identifier of the PDF file to extract.

            Use `travdata_cli -c CONFIG_DIR listbooks` to list accepted values
            for this argument.
            """
        ),
        metavar="BOOK",
    )
    argparser.add_argument(
        "input_pdf",
        help="Path to the PDF file to read tables from.",
        type=pathlib.Path,
        metavar="INPUT.PDF",
    )
    argparser.add_argument(
        "output_dir",
        help="Path to the directory to output the CSV files into.",
        type=pathlib.Path,
        metavar="OUT_DIR",
        default=pathlib.Path("./csv-tables"),
    )

    config.add_config_flag(argparser)
    argparser.add_argument(
        "--overwrite-existing",
        help=textwrap.dedent(
            """
            Extract CSV tables that already exist in the
            output. This is useful when testing larger scale changes to the
            configuration or code.
            """
        ),
        action="store_true",
        default=False,
    )
    argparser.add_argument(
        "--no-progress",
        help="""Disable progress bar.""",
        action="store_true",
        default=False,
    )

    tab_grp = argparser.add_argument_group("Tabula")
    tab_grp.add_argument(
        "--tabula-force-subprocess",
        help=textwrap.dedent(
            """
            If jpype cannot use libjvm, try seting this flag to use a slower
            path that uses Java as a subprocess.
            """
        ),
        action="store_true",
        default=False,
    )


@contextlib.contextmanager
def _progress_reporter(no_progress: bool) -> Iterator[Callable[[pdfextract.Progress], None]]:
    if no_progress:
        progress_bar = None

        def on_progress(p: pdfextract.Progress) -> None:
            del p  # unused

    else:
        progress_bar = progress.Bar("Extracting tables")
        progress_bar.start()

        def on_progress(p: pdfextract.Progress) -> None:
            progress_bar.index = p.completed
            progress_bar.max = p.total
            progress_bar.update()

    try:
        yield on_progress
    finally:
        if progress_bar is not None:
            progress_bar.finish()


def run(args: argparse.Namespace) -> int:
    """CLI entry point."""

    cfg = config.load_config_from_flag(args, [args.book_name])
    try:
        book_cfg = cfg.books[args.book_name]
    except KeyError:
        print(f"Book {args.book_name} is unknown.", file=sys.stderr)
        return 1
    if book_cfg.group is None:
        raise RuntimeError("book_cfg.group should have been loaded, but is None")

    def on_error(error: str) -> None:
        print(error, file=sys.stderr)

    with (
        tabulautil.TabulaClient(force_subprocess=args.tabula_force_subprocess) as tabula_client,
        _progress_reporter(args.no_progress) as on_progress,
    ):
        pdfextract.extract_book(
            table_reader=tabula_client,
            cfg=pdfextract.ExtractionConfig(
                config_dir=args.config_dir,
                output_dir=args.output_dir,
                input_pdf=args.input_pdf,
                book_cfg=book_cfg,
                overwrite_existing=args.overwrite_existing,
            ),
            events=pdfextract.ExtractEvents(
                on_progress=on_progress,
                on_error=on_error,
                do_continue=lambda: True,
            ),
        )

    return 0
