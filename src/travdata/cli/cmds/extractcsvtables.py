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
from travdata import config, filesio
from travdata.extraction import bookextract, tabulautil


def add_subparser(subparsers) -> None:
    """Adds a subcommand parser to ``subparsers``."""
    argparser: argparse.ArgumentParser = subparsers.add_parser(
        "extractcsvtables",
        description=__doc__,
        formatter_class=argparse.RawTextHelpFormatter,
        prefix_chars="-+",
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
        "output",
        help=textwrap.dedent(
            """
            Path to the directory or ZIP file to output the CSV files into.

            Whether this is a directory or ZIP file is controlled by
            --output-type.
            """
        ),
        type=pathlib.Path,
        metavar="OUTPUT_PATH",
    )

    config.add_config_flag(argparser)

    argparser.add_argument(
        "--no-progress",
        help="""Disable progress bar.""",
        action="store_true",
        default=False,
    )

    argparser.add_argument(
        "--output-type",
        help=textwrap.dedent(
            """
            Controls how data is output to the OUTPUT_PATH.

            * AUTO guesses, based on any existing file or directory at the path
              or the path suffix ending in ".zip".
            * DIR writes as a directory.
            * ZIP writes as a ZIP file.
            """
        ),
        type=filesio.IOType,
        choices=filesio.IOType,
        default=filesio.IOType.AUTO,
    )

    outsel_grp = argparser.add_argument_group(
        "Output selection",
        description="Controls which data is extracted from the book.",
    )
    outsel_grp.add_argument(
        "--overwrite-existing",
        help=textwrap.dedent(
            """
            Extract CSV tables that already exist in the output. This is useful
            when testing larger scale changes to the configuration or code.
            """
        ),
        action="store_true",
        default=False,
    )
    outsel_grp.add_argument(
        "+t",
        "--with-tag",
        dest="with_tag",
        nargs="*",
        metavar="TAG",
        default=[],
        help=textwrap.dedent(
            """
            Only extract tables that have any of these tags. --without-tag takes
            precedence over this.
            """
        ),
    )
    outsel_grp.add_argument(
        "-t",
        "--without-tag",
        dest="without_tag",
        nargs="*",
        metavar="TAG",
        default=[],
        help=textwrap.dedent(
            """
            Only extract tables that do not have any of these tags. This takes
            precedence over --with-tag.
            """
        ),
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
def _progress_reporter(no_progress: bool) -> Iterator[Callable[[bookextract.Progress], None]]:
    if no_progress:
        progress_bar = None

        def on_progress(p: bookextract.Progress) -> None:
            del p  # unused

    else:
        progress_bar = progress.Bar("Extracting tables")
        progress_bar.start()

        def on_progress(p: bookextract.Progress) -> None:
            progress_bar.index = p.completed
            progress_bar.max = p.total
            progress_bar.update()

    try:
        yield on_progress
    finally:
        if progress_bar is not None:
            progress_bar.finish()


def _create_read_writer(
    args: argparse.Namespace,
) -> contextlib.AbstractContextManager[filesio.ReadWriter]:
    output: pathlib.Path = args.output
    output_type: filesio.IOType = args.output_type
    output_type = output_type.resolve_auto(output)
    return output_type.new_read_writer(output)


def run(args: argparse.Namespace) -> int:
    """CLI entry point."""

    with_tags = frozenset(args.with_tag)
    without_tags = frozenset(args.without_tag)
    if intersection := with_tags & without_tags:
        fmt_inter = ", ".join(sorted(intersection))
        print(
            f"Tags have been specified for both inclusion and exclusion: {fmt_inter}.",
            file=sys.stderr,
        )
        return 1

    ext_cfg = bookextract.ExtractionConfig(
        cfg_reader_ctx=config.config_reader(args),
        out_writer_ctx=_create_read_writer(args),
        input_pdf=args.input_pdf,
        book_id=args.book_name,
        overwrite_existing=args.overwrite_existing,
        with_tags=with_tags,
        without_tags=without_tags,
    )

    def on_error(error: str) -> None:
        print(error, file=sys.stderr)

    with (
        tabulautil.TabulaClient(force_subprocess=args.tabula_force_subprocess) as tabula_client,
        _progress_reporter(args.no_progress) as on_progress,
    ):
        bookextract.extract_book(
            table_reader=tabula_client,
            ext_cfg=ext_cfg,
            events=bookextract.ExtractEvents(
                on_progress=on_progress,
                on_error=on_error,
                do_continue=lambda: True,
            ),
        )

    return 0
