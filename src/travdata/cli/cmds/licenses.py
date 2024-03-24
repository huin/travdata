# -*- coding: utf-8 -*-
"""
Prints the licenses from the source code distributed as part of this program.
"""

import argparse
from importlib import metadata
import operator
import os.path
import pathlib
import textwrap
from typing import cast

import piplicenses_lib


def add_subparser(subparsers) -> None:
    """Adds a subcommand parser to ``subparsers``."""
    argparser: argparse.ArgumentParser = subparsers.add_parser(
        "licenses",
        description=__doc__,
        formatter_class=argparse.RawTextHelpFormatter,
    )
    argparser.set_defaults(run=run)


def run(args: argparse.Namespace) -> None:
    """CLI entry point."""
    del args  # unused

    print("= Third-party licenses\n")

    dists = list(metadata.MetadataPathFinder.find_distributions())
    dists.sort(key=operator.attrgetter("name"))

    for dist in dists:
        print(f"== {dist.name} {dist.version}\n")
        _print_metadata(dist)
        _print_licenses(dist)


_METADATAS: list[tuple[str, str]] = [
    ("Home-page", "Homepage"),
    ("Project-URL", "Project"),
    ("Author", "Author"),
    ("Author-email", "Author email"),
    ("Maintainer", "Maintainer"),
    ("Maintainer-email", "Maintainer email"),
    ("License", "License"),
    ("Download-URL", "Download"),
]


def _print_metadata(dist: metadata.Distribution) -> None:
    """Prints attribution metadata."""
    displayed_metadata = False
    for key, name in _METADATAS:
        if key not in dist.metadata:
            continue
        print(f"{name}:: {dist.metadata[key]}")
        displayed_metadata = True
    if displayed_metadata:
        print()


def _print_licenses(dist: metadata.Distribution) -> None:
    """Prints each discovered license file and content."""
    info = piplicenses_lib.get_package_info(dist, include_files=True)

    lfiles = cast(list[str], info["licensefile"])
    lpaths: list[pathlib.PurePath]
    if len(lfiles) > 1:
        comm_path = pathlib.PurePath(os.path.commonpath(lfiles))
        lpaths = [pathlib.PurePath(p).relative_to(comm_path) for p in lfiles]
    else:
        lpaths = [pathlib.PurePath(p) for p in lfiles]

    ltexts = cast(list[str], info["licensetext"])
    files: list[tuple[pathlib.PurePath, str]] = list(zip(lpaths, ltexts))
    files.sort(key=operator.itemgetter(0))

    for lpath, text in files:
        if lpath.is_absolute():
            fname = lpath.name
        else:
            fname = str(lpath)
        print(f"=== {fname}\n")
        print("[listing]\n----")
        text = textwrap.dedent(text)
        text = textwrap.indent(text, "  ")
        print(text)
        print("----\n")
