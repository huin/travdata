#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Builds release assets for the project with shiv."""

import argparse
import ast
import contextlib
import os
import pathlib
import re
import sys
import tempfile
from typing import Iterator
import zipfile

import shiv.cli  # type: ignore[import-untyped]


def main() -> None:
    """Builds standalone executables."""
    argparser = argparse.ArgumentParser(description=__doc__)
    argparser.add_argument("version")
    argparser.add_argument("output_zip", type=pathlib.Path)
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

    src_dir = pathlib.Path(".")
    build_dir = src_dir / "dist"
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

    _build_zip(src_dir=src_dir, build_dir=build_dir, output_zip=args.output_zip)


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
config.__executable_environment__ = "pyz"
"""
            )
            script.flush()
            yield pathlib.Path(script.name)


def _build_zip(
    build_dir: pathlib.Path,
    src_dir: pathlib.Path,
    output_zip: pathlib.Path,
) -> None:
    """Builds zipfile for release."""

    with zipfile.ZipFile(
        output_zip,
        mode="w",
        compression=zipfile.ZIP_DEFLATED,
    ) as zf:
        zf.write(
            build_dir / "travdata_cli.pyz",
            arcname="travdata_cli.pyz",
            # The binary is itself a zipfile, so compression won't be
            # effective.
            compress_type=zipfile.ZIP_STORED,
        )
        zf.write(src_dir / "LICENSE", "LICENSE")
        zf.write(src_dir / "README.adoc", "README.adoc")
        _copy_config_files(zf, src_dir)


def _copy_config_files(zf: zipfile.ZipFile, src_dir: pathlib.Path) -> None:
    config_dir = pathlib.PurePath(src_dir) / "config"

    for root_str, dir_strs, file_strs in os.walk(config_dir):
        dir_strs.sort()
        root = pathlib.PurePath(root_str)
        for f_str in sorted(file_strs):
            if not f_str.endswith((".json", ".yaml")):
                continue
            zf.write(root / f_str, arcname=root / f_str)


if __name__ == "__main__":
    main()
