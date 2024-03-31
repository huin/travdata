# -*- coding: utf-8 -*-
"""
Lists the names of books in the configuration.
"""

import argparse

from travdata import config


def add_subparser(subparsers) -> None:
    """Adds a subcommand parser to ``subparsers``."""
    argparser: argparse.ArgumentParser = subparsers.add_parser(
        "listbooks",
        description=__doc__,
        formatter_class=argparse.RawTextHelpFormatter,
    )
    config.add_config_flag(argparser)
    argparser.set_defaults(run=run)


def run(args: argparse.Namespace) -> None:
    """CLI entry point."""

    cfg = config.load_config_from_flag(args, [])
    for name in sorted(cfg.books):
        print(name)
