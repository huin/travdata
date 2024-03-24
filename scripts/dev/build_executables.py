#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Builds standalone executables for the project using shiv."""

import argparse
import ast
import contextlib
import os
import pathlib
import re
import sys
import tempfile
from typing import Iterator

import shiv.cli  # type: ignore[import-untyped]


def main() -> None:
    """Builds standalone executables."""
    argparser = argparse.ArgumentParser(description=__doc__)
    argparser.add_argument("version")
    args = argparser.parse_args()

    package_arg: str
    if args.version == "localdev":
        site_packages = _get_site_packages_path()
        package_arg = f"--site-packages={site_packages}"
    elif re.fullmatch(r"v\d+[.]\d+[.]\d+", args.version):
        package_arg = f"travdata=={args.version}"
    else:
        argparser.exit(
            status=1,
            message="version number must be 'localdev' or in the form vX.Y.Z\n",
        )

    build_dir = pathlib.Path("build")
    build_dir.mkdir(parents=True, exist_ok=True)

    with _preamble_script(args.version) as preamble:
        ctx = shiv.cli.main.make_context(
            "build travdata_cli",
            [
                "--entry-point=travdata.cli.cli:main",
                f"--output-file={build_dir / 'travdata_cli.pyz'}",
                f"--preamble={preamble}",
                package_arg,
            ],
        )
        ctx.forward(shiv.cli.main)


def _get_site_packages_path() -> pathlib.Path:
    env_dir = pathlib.Path(os.environ["VIRTUAL_ENV"])
    for path_str in sys.path:
        path = pathlib.Path(path_str)
        if env_dir in path.parents:
            return env_dir
    raise RuntimeError("could not identify site-packages path")


@contextlib.contextmanager
def _preamble_script(version: str) -> Iterator[pathlib.Path]:
    with tempfile.TemporaryDirectory() as tmpdir_str:
        tmpdir = pathlib.Path(tmpdir_str)

        with open(tmpdir / "preamble.py", mode="wt", encoding="utf-8") as script:
            version_literal = ast.unparse(ast.Constant(version))
            script.write(
                f"""\
# -*- coding: utf-8 -*-
import travdata
from travdata import config

travdata.__executable_version__ = {version_literal}
config.__executable_environment__ = "release"
"""
            )
            script.flush()
            yield pathlib.Path(script.name)


if __name__ == "__main__":
    main()
