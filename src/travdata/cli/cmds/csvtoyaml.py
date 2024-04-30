# -*- coding: utf-8 -*-
"""
Converts CSV data tables from the Mongoose Traveller 2022 core rules PDF into
YAML files.
"""

import argparse
import csv
import pathlib

from travdata import csvutil, filesio
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

    with (
        filesio.DirReader.new_reader(args.input_dir) as csv_reader,
        filesio.DirReadWriter.new_read_writer(args.output_dir) as yaml_writer,
    ):
        for conv_key, conv_fn in registry.CONVERTERS.converters.items():
            in_group_dir = pathlib.PurePath(conv_key.group_name)
            out_group_dir = pathlib.PurePath(conv_key.group_name)
            with (
                csvutil.open_by_reader(
                    csv_reader,
                    in_group_dir / f"{conv_key.table_name}.csv",
                ) as csv_file_in,
                yaml_writer.open_write(
                    out_group_dir / f"{conv_key.table_name}.yaml",
                ) as yaml_file_out,
            ):
                r = csv.DictReader(csv_file_in)
                data = conv_fn(iter(r))
                yamlcodec.DATATYPES_YAML.dump(
                    data=list(data),
                    stream=yaml_file_out,
                )
