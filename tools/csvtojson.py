#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import csv
import pathlib

from travdata import jsonenc
from travdata.tableconverters import core_rulebook


def main() -> None:
    argparser = argparse.ArgumentParser(
        description="""
        Converts CSV data tables from the Mongoose Traveller 2022 core rules PDF
        into JSON files.

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
        "input_dir",
        help="Path to the directory to read the CSV files from.",
        type=pathlib.Path,
        metavar="IN_DIR",
    )

    argparser.add_argument(
        "output_dir",
        help="Path to the directory to output the JSON files into.",
        type=pathlib.Path,
        metavar="OUT_DIR",
    )

    args = argparser.parse_args()

    for ext in core_rulebook.CONVERTERS:
        with (
            open(args.input_dir / f"{ext.name}.csv", "rt") as csv_file_in,
            open(args.output_dir / f"{ext.name}.json", "wt") as json_file_out,
        ):
            r = csv.DictReader(csv_file_in)
            data = ext.fn(iter(r))
            jsonenc.DEFAULT_CODEC.dump(
                fp=json_file_out,
                obj=list(data),
            )


if __name__ == "__main__":
    main()
