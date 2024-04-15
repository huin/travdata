#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Builds release assets for the project with PyInstaller."""

import argparse
import ast
import contextlib
import os
import pathlib
import re
import sys
from typing import Iterator
import zipfile

import tabula.backend
import PyInstaller.__main__


def main() -> None:
    """Builds standalone executables."""
    argparser = argparse.ArgumentParser(description=__doc__)
    argparser.add_argument("version")
    argparser.add_argument("output_zip", type=pathlib.Path)
    args = argparser.parse_args()

    if args.version == "localdev":
        pass
    elif re.fullmatch(r"v\d+[.]\d+[.]\d+", args.version):
        pass
    else:
        argparser.exit(
            status=1,
            message="version number must be 'localdev' or in the form vX.Y.Z\n",
        )

    src_dir = pathlib.Path(".")
    build_dir = src_dir / "dist"
    build_dir.mkdir(parents=True, exist_ok=True)

    with _write_hook_script(args.version, src_dir):
        PyInstaller.__main__.run(
            [
                "pyinstaller.spec",
                # Do not ask for interactive confirmation to overwrite output
                # directory.
                "--noconfirm",
                "--",
                tabula.backend.jar_path(),
            ],
        )

    _build_zip(src_dir=src_dir, build_dir=build_dir, output_zip=args.output_zip)


@contextlib.contextmanager
def _write_hook_script(version: str, src_dir: pathlib.Path) -> Iterator[None]:
    """Temporarily overwrites hook.py."""
    hook_script = src_dir / "scripts/pyinstaller/hook.py"
    original = hook_script.read_bytes()

    try:
        with open(hook_script, mode="wt", encoding="utf-8") as script:
            exec_literal = ast.unparse(ast.Constant("pyinstaller"))
            version_literal = ast.unparse(ast.Constant(version))
            script.write(
                f"""\
# -*- coding: utf-8 -*-
import travdata
from travdata import travdatarelease

travdatarelease.EXECUTABLE_ENVIRONMENT = {exec_literal}
travdatarelease.EXECUTABLE_VERSION = {version_literal}
"""
            )

        yield
    finally:
        hook_script.write_bytes(original)


def _build_zip(
    build_dir: pathlib.Path,
    src_dir: pathlib.Path,
    output_zip: pathlib.Path,
) -> None:
    """Builds zipfile for release."""
    if sys.platform == "win32":
        system_exec_suffix = ".exe"
    else:
        system_exec_suffix = ""

    with zipfile.ZipFile(
        output_zip,
        mode="w",
        compression=zipfile.ZIP_DEFLATED,
    ) as zf:
        zf.write(
            build_dir / "travdata" / f"travdata_cli{system_exec_suffix}",
            arcname=f"travdata_cli{system_exec_suffix}",
        )
        zf.write(
            build_dir / "travdata" / f"travdata_gui{system_exec_suffix}",
            arcname=f"travdata_gui{system_exec_suffix}",
        )
        zf.write(src_dir / "LICENSE", "LICENSE")
        zf.write(src_dir / "README.adoc", "README.adoc")
        _copy_internal(zf, build_dir / "travdata/_internal", pathlib.PurePath("_internal"))


def _copy_internal(zf: zipfile.ZipFile, from_dir: pathlib.Path, to_dir: pathlib.PurePath) -> None:
    for root_str, dir_strs, file_strs in os.walk(from_dir):
        dir_strs.sort()
        root = pathlib.PurePath(root_str)
        rel_root = root.relative_to(from_dir)
        for f_str in sorted(file_strs):
            zf.write(root / f_str, arcname=to_dir / rel_root / f_str)


if __name__ == "__main__":
    main()
