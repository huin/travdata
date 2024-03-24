#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Builds a zipfile for release."""

import argparse
import os
import pathlib
import zipfile


def main() -> None:
    """Builds zipfile for release."""
    argparser = argparse.ArgumentParser(description=__doc__)
    argparser.add_argument("version")
    args = argparser.parse_args()
    src_dir = pathlib.Path(".")
    build_dir = src_dir / "build"

    with zipfile.ZipFile(
        build_dir / f"travdata-{args.version}.zip",
        mode="w",
        compression=zipfile.ZIP_DEFLATED,
    ) as zf:
        zf.write(
            build_dir / "travdata_cli",
            arcname="travdata_cli",
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
