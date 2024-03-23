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
    argparser.set_defaults(run=run)


def run(args: argparse.Namespace) -> None:
    """CLI entry point."""

    cfg = config.load_config(args.config_dir, [])
    for name in cfg.book_names:
        print(name)
