# -*- coding: utf-8 -*-
"""
Extracts data tables from the Mongoose Traveller 2022 core rules PDF as
CSV files.
"""

import argparse
import csv
import pathlib
import sys
import textwrap
from typing import cast

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
        "config_dir",
        help=textwrap.dedent(
            """
            Path to the extraction configuration directory for the given PDF.
            This must contain a config.yaml file, and its required Tabula
            templates. Some configurations for this should be included with this
            program's distribution.
            """
        ),
        type=pathlib.Path,
        metavar="CONFIG_DIR",
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


def run(args: argparse.Namespace) -> None:
    """CLI entry point."""

    tabula_client = tabulautil.TabulaClient(
        force_subprocess=args.tabula_force_subprocess,
    )

    group = config.load_config(args.config_dir)

    output_tables: list[tuple[pathlib.Path, config.Table]] = []
    for table in group.all_tables():
        if table.extraction is None:
            continue
        out_filepath = args.output_dir / table.file_stem.with_suffix(".csv")

        if args.overwrite_existing or not out_filepath.exists():
            output_tables.append((out_filepath, table))

    if not args.no_progress:
        monitored_output_tables = progress.Bar(
            "Extracting tables",
            max=len(output_tables),
        ).iter(output_tables)
    else:
        monitored_output_tables = iter(output_tables)

    created_directories: set[pathlib.Path] = set()
    for out_filepath, table in monitored_output_tables:
        if table.extraction is None:
            continue
        extraction = table.extraction

        out_filepath = cast(pathlib.Path, out_filepath)
        table = cast(config.Table, table)

        group_dir = out_filepath.parent
        if group_dir not in created_directories:
            group_dir.mkdir(parents=True, exist_ok=True)
            created_directories.add(group_dir)

        try:
            try:
                rows = pdfextract.extract_table(
                    config_dir=args.config_dir,
                    pdf_path=args.input_pdf,
                    file_stem=table.file_stem,
                    extraction=extraction,
                    table_reader=tabula_client,
                )
                with open(out_filepath, "wt", encoding="utf-8") as f:
                    csv.writer(f).writerows(rows)
            except Exception as e:
                e.add_note(f"Error while processing table {table.file_stem}: {e}")
                raise
        except pdfextract.ConfigurationError as e:
            print(e, file=sys.stdout)
