#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Builds a combined license file from the main dependencies."""

from importlib import metadata
import operator
import pathlib
import shutil
import subprocess
from typing import cast

import piplicenses_lib


def main() -> None:
    """Builds standalone executables."""

    dep_names = get_dep_names()

    print("= Third-party licenses\n")

    for dep_name in sorted(dep_names):
        dist = metadata.distribution(dep_name)
        info = piplicenses_lib.get_package_info(dist, include_files=True)

        print(f"== {info['namever']}\n")

        print_metadata(dist)

        lfiles = cast(list[str], info["licensefile"])
        ltexts = cast(list[str], info["licensetext"])
        files: list[tuple[str, str]] = [
            (pathlib.Path(fname).name, text) for fname, text in zip(lfiles, ltexts)
        ]
        files.sort(key=operator.itemgetter(0))

        for fname, text in files:
            p = pathlib.Path(fname)
            print(f"=== {p.name}\n")
            print("[listing]\n----")
            print(text)
            print("----\n")


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


if __name__ == "__main__":
    main()
