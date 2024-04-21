#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""travdata commandline interface.

This is a suite of tools that perform either or both of the following:

* extract data from Mongoose Traveller PDF files,
* produced data derived from said data.
"""


import argparse
import os
import sys

from travdata import commontext
from travdata import travdatarelease
from travdata.cli import cliutil
from travdata.cli.cmds import (
    csvtoyaml,
    extractcsvtables,
    licenses,
    listbooks,
    tradetable,
)
from travdata.config import cfgerror


def main() -> None:
    """Entrypoint for the program."""

    argparser = argparse.ArgumentParser(
        description=f"{__doc__}\n{commontext.DATA_USAGE}",
        formatter_class=argparse.RawTextHelpFormatter,
    )
    argparser.set_defaults(run=None)
    argparser.add_argument(
        "--version",
        "-V",
        help="Print the version of the program.",
        action="version",
        version=f"%(prog)s {travdatarelease.EXECUTABLE_VERSION}",
    )

    subparsers = argparser.add_subparsers(required=True)
    csvtoyaml.add_subparser(subparsers)
    extractcsvtables.add_subparser(subparsers)
    licenses.add_subparser(subparsers)
    listbooks.add_subparser(subparsers)
    tradetable.add_subparser(subparsers)

    args = argparser.parse_args()
    try:
        sys.exit(args.run(args))
    except cfgerror.ConfigurationError as exc:
        print(exc, file=sys.stderr)
        sys.exit(os.EX_CONFIG)
    except cliutil.UsageError as exc:
        argparser.print_usage(sys.stderr)
        print(exc, file=sys.stderr)
        sys.exit(exc.exit_code)
    except cliutil.CLIError as exc:
        print(exc, file=sys.stderr)
        sys.exit(exc.exit_code)


if __name__ == "__main__":
    main()
