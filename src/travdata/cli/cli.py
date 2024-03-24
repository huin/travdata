#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""travdata commandline interface.

This is a suite of tools that perform either or both of the following:

* extract data from Mongoose Traveller PDF files,
* produced data derived from said data.

The extracted and produced data is *not* for redistribution, as it is almost
certainly subject to copyright. This utility (and its output) is intended as an
aid to those who legally own a copy of the Mongoose Traveller materials, and
wish to make use of the data for their own purposes.

It is the sole responsibility of the user of this program to use the extracted
data in a manner that respects the publisher's IP rights.
"""


import argparse
from typing import Optional

import travdata
from travdata.cli.cmds import csvtoyaml, extractcsvtables, licenses, listbooks, tradetable


def main() -> Optional[int]:
    """Entrypoint for the program."""

    argparser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawTextHelpFormatter,
    )
    argparser.set_defaults(run=None)
    argparser.add_argument(
        "--version",
        "-V",
        help="Print the version of the program.",
        action="version",
        version=f"%(prog)s {travdata.__executable_version__}",
    )

    subparsers = argparser.add_subparsers(required=True)
    csvtoyaml.add_subparser(subparsers)
    extractcsvtables.add_subparser(subparsers)
    licenses.add_subparser(subparsers)
    listbooks.add_subparser(subparsers)
    tradetable.add_subparser(subparsers)

    args = argparser.parse_args()
    return args.run(args)


if __name__ == "__main__":
    main()
