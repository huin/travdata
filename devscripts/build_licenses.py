#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Builds a combined license file from the main dependencies."""

from importlib import metadata
import operator
import os.path
import pathlib
import shutil
import subprocess
import textwrap
from typing import cast

import piplicenses_lib


def main() -> None:
    """Builds standalone executables."""

    dep_names = get_dep_names()

    print("= Third-party licenses\n")

    for dep_name in sorted(dep_names):
        dist = metadata.distribution(dep_name)
        print(f"== {dist.metadata['Name']} {dist.metadata['Version']}\n")
        print_metadata(dist)
        print_licenses(dist)


def get_dep_names() -> list[str]:
    """Returns the list of dependency names."""
    poetry_binary = shutil.which("poetry") or "poetry"
    deps_output = subprocess.check_output(
        [
            poetry_binary,
            "show",
            "--no-ansi",
            "--no-interaction",
            "--only=main",
        ],
        text=True,
    )
    dep_names: list[str] = []
    for line in deps_output.splitlines():
        dep_name, _ = line.split(maxsplit=1)
        dep_names.append(dep_name)
    return dep_names


_METADATAS: list[tuple[str, str]] = [
    ("Home-page", "Homepage"),
    ("Download-URL", "Download"),
    ("Author", "Author"),
    ("Maintainer", "Maintainer"),
]


def print_metadata(dist: metadata.Distribution) -> None:
    """Prints attribution metadata."""
    displayed_metadata = False
    for key, name in _METADATAS:
        if key not in dist.metadata:
            continue
        print(f"{name}:: {dist.metadata[key]}")
        displayed_metadata = True
    if displayed_metadata:
        print()


def print_licenses(dist: metadata.Distribution) -> None:
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


if __name__ == "__main__":
    main()
