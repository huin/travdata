# -*- coding: utf-8 -*-
"""
Converts CSV data tables from the Mongoose Traveller 2022 core rules PDF into
YAML files.
"""

import argparse
import csv
import pathlib

from travdata import csvutil
from travdata.datatypes import yamlcodec
from travdata.tableconverters.core import registry


def add_subparser(subparsers) -> None:
    """Adds a subcommand parser to ``subparsers``."""
    argparser: argparse.ArgumentParser = subparsers.add_parser(
        "csvtoyaml",
        description=__doc__,
        formatter_class=argparse.RawTextHelpFormatter,
    )
    argparser.set_defaults(run=run)

    argparser.add_argument(
        "input_dir",
        help="Path to the directory to read the CSV files from.",
        type=pathlib.Path,
        metavar="IN_DIR",
    )

    argparser.add_argument(
        "output_dir",
        help="Path to the directory to output the YAML files into.",
        type=pathlib.Path,
        metavar="OUT_DIR",
    )


def run(args: argparse.Namespace) -> None:
    """CLI entry point."""
    registry.load_all_converters()

    created_directories: set[pathlib.Path] = set()
    for conv_key, conv_fn in registry.CONVERTERS.converters.items():
        in_group_dir = args.input_dir / conv_key.group_name
        out_group_dir = args.output_dir / conv_key.group_name
        if out_group_dir not in created_directories:
            out_group_dir.mkdir(parents=True, exist_ok=True)
        with (
            csvutil.open_read(
                in_group_dir / f"{conv_key.table_name}.csv",
            ) as csv_file_in,
            open(
                out_group_dir / f"{conv_key.table_name}.yaml",
                "wt",
                encoding="utf-8",
            ) as yaml_file_out,
        ):
            r = csv.DictReader(csv_file_in)
            data = conv_fn(iter(r))
            yamlcodec.DATATYPES_YAML.dump(
                data=list(data),
                stream=yaml_file_out,
            )
