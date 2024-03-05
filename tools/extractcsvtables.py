#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import csv
import pathlib

from travdata import pdfextract


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

    args = argparser.parse_args()

    cfg = pdfextract.load_config(args.config_dir)
    extracted_tables = pdfextract.extract_tables(
        cfg=cfg,
        pdf_path=args.input_pdf,
    )

    created_directories: set[pathlib.Path] = set()
    for table in extracted_tables:
        group_dir = args.output_dir / table.group_name
        if group_dir not in created_directories:
            group_dir.mkdir(parents=True, exist_ok=True)
        with open(group_dir / f"{table.table_name}.csv", "wt") as f:
            csv.writer(f).writerows(table.rows)


if __name__ == "__main__":
    main()
