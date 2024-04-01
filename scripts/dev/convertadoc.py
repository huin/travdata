#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Converts asciidoc files listed as parameters into Markdown and reStructuredText.
"""

import argparse
import pathlib
import subprocess
import tempfile
from typing import cast


def _to_docbook(input_asciidoc: pathlib.Path, output: pathlib.Path) -> None:
    subprocess.check_call(
        [
            "asciidoc",
            "--backend",
            "docbook",
            f"--out-file={output}",
            str(input_asciidoc),
        ]
    )


def _to_format(input_docbook: pathlib.Path, fmt: str, output: pathlib.Path) -> None:
    subprocess.check_call(
        [
            "pandoc",
            "--from=docbook",
            f"--to={fmt}",
            f"--output={output}",
            str(input_docbook),
        ]
    )


def _convert(input_asciidoc: pathlib.Path) -> None:
    with tempfile.NamedTemporaryFile() as docbook:
        docbook_path = pathlib.Path(docbook.name)
        _to_docbook(input_asciidoc=input_asciidoc, output=docbook_path)
        _to_format(
            input_docbook=docbook_path,
            fmt="markdown_strict",
            output=input_asciidoc.with_suffix(".md"),
        )
        _to_format(
            input_docbook=docbook_path,
            fmt="rst",
            output=input_asciidoc.with_suffix(".rst"),
        )


def main() -> None:
    """Entry point for program."""
    argparser = argparse.ArgumentParser(__doc__)
    argparser.add_argument(
        "input_asciidocs",
        type=pathlib.Path,
        nargs="+",
        help="Input asciidoc files with .adoc extension.",
    )
    args = argparser.parse_args()

    input_asciidocs = cast(list[pathlib.Path], args.input_asciidocs)

    if not all(p.suffix == ".adoc" for p in input_asciidocs):
        argparser.exit(status=1, message="All input filenames must have `.adoc` suffix.")

    for input_asciidoc in input_asciidocs:
        _convert(input_asciidoc)


if __name__ == "__main__":
    main()
