#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import pathlib

from travdata import jsonenc
from travdata.extractors import params, registry


def _add_extractor_args(argparser: argparse._ArgumentGroup) -> None:
    for ext in registry.EXTRACTORS:
        flag = ext.name.replace("_", "-")
        argparser.add_argument(
            f"--{flag}",
            help=f"Path to file to extract {ext.description} data to.",
            type=argparse.FileType("wt", encoding="utf-8"),
            metavar="JSON_FILE",
        )


def _handle_extractor_args(args: argparse.Namespace, param: params.CoreParams) -> None:
    for ext in registry.EXTRACTORS:
        if out := getattr(args, ext.name):
            jsonenc.DEFAULT_CODEC.dump(
                fp=out,
                obj=list(ext.fn(param)),
            )


def main() -> None:
    argparser = argparse.ArgumentParser(
        description="""
        Extracts data tables from the Mongoose Traveller 2022 core rules PDF.

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

    inputs_grp = argparser.add_argument_group(
        title="Inputs",
        description="""
        Arguments that provide input for the extraction process.
        """,
    )
    inputs_grp.add_argument(
        "core_rulebook",
        help="Path to the core rules 2022 update PDF.",
        type=pathlib.Path,
        metavar="CORE_RULES_PDF",
    )
    inputs_grp.add_argument(
        "--templates-dir",
        help="""
        Path to the directory containing the Tabula templates supplied with this
        program.
        """,
        type=pathlib.Path,
        metavar="DIR",
        default=pathlib.Path("./tabula-templates"),
        required=True,
    )

    extract_grp = argparser.add_argument_group("Table extractors")
    _add_extractor_args(extract_grp)

    args = argparser.parse_args()

    _handle_extractor_args(
        args,
        param=params.CoreParams(
            core_rulebook=args.core_rulebook,
            templates_dir=args.templates_dir,
        ),
    )


if __name__ == "__main__":
    main()
