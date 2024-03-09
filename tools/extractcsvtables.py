#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import csv
import dataclasses
import pathlib

from progress import bar as progress  # type: ignore[import-untyped]
from travdata import pdfextract, tabulautil


def main() -> None:
    argparser = argparse.ArgumentParser(
        description="""
        Extracts data tables from the Mongoose Traveller 2022 core rules PDF as
        CSV files.

        The extracted data is *not* for redistribution, as it is almost
        certainly subject to copyright. This utility (and its output) is
        intended as an aid to those who legally own a copy of the Mongoose
        Traveller materials, and wish to make use of the data for their own
        purposes.

        It is the sole responsibility of the user of this program to use the
        extracted data in a manner that respects the publisher's IP rights.
        """,
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    argparser.add_argument(
        "config_dir",
        help="""Path to the extraction configuration directory for the given
        PDF. This must contain a config.yaml file, and its required Tabula
        templates. Some configurations for this should be included with this
        program's distribution.""",
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
        help="""Extract CSV tables that already exist in the output. This is
        useful when testing larger scale changes to the configuration or
        code.""",
        action="store_true",
        default=False,
    )

    tab_grp = argparser.add_argument_group("Tabula")
    tab_grp.add_argument(
        "--tabula-force-subprocess",
        help="""If jpype cannot use libjvm, try seting this flag to use a slower
        path that uses Java as a subprocess.""",
        action="store_true",
        default=False,
    )

    args = argparser.parse_args()

    tabula_cfg = tabulautil.TabulaConfig(
        force_subprocess=args.tabula_force_subprocess,
    )

    group = pdfextract.load_config(args.config_dir)
    table_extractors = pdfextract.extract_tables(
        group=group,
        config_dir=args.config_dir,
        pdf_path=args.input_pdf,
        tabula_cfg=tabula_cfg,
    )

    outputs = [
        _OutputTable(_output_filepath(args.output_dir, ext_table), ext_table)
        for ext_table in table_extractors
    ]
    if not args.overwrite_existing:
        outputs = [o for o in outputs if not o.filepath.exists()]

    progress_bar = progress.Bar("Extracting tables", max=len(outputs))
    created_directories: set[pathlib.Path] = set()
    for output in progress_bar.iter(outputs):
        group_dir = output.filepath.parent
        if group_dir not in created_directories:
            group_dir.mkdir(parents=True, exist_ok=True)
        with open(output.filepath, "wt") as f:
            csv.writer(f).writerows(output.ext_table.extract_rows())


@dataclasses.dataclass
class _OutputTable:
    filepath: pathlib.Path
    ext_table: pdfextract.TableExtractor


def _output_filepath(
    output_dir: pathlib.Path,
    ext_table: pdfextract.TableExtractor,
) -> pathlib.Path:
    return output_dir / ext_table.table_cfg.file_stem.with_suffix(".csv")


if __name__ == "__main__":
    main()
