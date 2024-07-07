#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Reports unused Tabula templates."""

import argparse
import os
import pathlib
import sys

from travdata import config, filesio
from travdata.config import cfgextract, yamlreg


def main() -> None:
    """CLI entry point."""
    argparser: argparse.ArgumentParser = argparse.ArgumentParser(__doc__)
    config.add_config_flag(argparser)

    argparser.add_argument(
        "books",
        nargs="+",
        metavar="BOOK",
        help="Books to examine the configuration of.",
    )
    args = argparser.parse_args()

    with config.config_reader(args) as cfg_reader:
        if not isinstance(cfg_reader, filesio.DirReader):
            raise RuntimeError("Only supports configuration in a local directory.")
        cfg_dir = cfg_reader.dir_path

        cfg = config.load_config(cfg_reader)

        for book in args.books:
            grp = cfg.books[book].load_group(cfg_reader)
            _list_unused_templates(cfg_dir, grp, True)


def _print_error(msg: str) -> None:
    print(msg, file=sys.stderr)


def _list_unused_templates(
    cfg_dir: pathlib.Path,
    grp: config.Group,
    top_level: bool,
) -> None:
    with os.scandir(cfg_dir / grp.rel_dir) as dir_iter:
        seen_dirs: set[str] = set()
        seen_tmpl_names: set[str] = set()
        for dir_entry in dir_iter:
            dir_entry_path = pathlib.Path(dir_entry)
            if dir_entry.is_dir():
                seen_dirs.add(dir_entry.name)
                try:
                    sub_grp = grp.groups[dir_entry.name]
                except KeyError:
                    _print_error(
                        f"Directory {dir_entry_path.relative_to(cfg_dir)} is not in configuration. Missing Group?"
                    )
                    _print_basic_group_yaml(dir_entry_path)
                    continue
                _list_unused_templates(cfg_dir, sub_grp, top_level=False)

            elif dir_entry.is_file():
                if dir_entry.name.endswith(config.TABULA_TEMPLATE_SUFFIX):
                    table_name = dir_entry.name.removesuffix(config.TABULA_TEMPLATE_SUFFIX)
                    seen_tmpl_names.add(table_name)
                    try:
                        table_cfg = grp.tables[table_name]
                    except KeyError:
                        _print_error(
                            f"Tabula template {dir_entry.name} has no corresponding Table.",
                        )
                        continue

                    if table_cfg.transform is None:
                        _print_error(
                            f"In {grp.rel_dir} table {table_name} has no extraction configured.",
                        )

                elif dir_entry.name == "book.yaml" and top_level:
                    pass  # Ignore expected file.
                else:
                    _print_error(f"Unexpected file: {dir_entry_path}.")

            else:
                _print_error(f"Unexpected file type for {dir_entry}.")

    missing_dirs = set(grp.groups.keys()) - seen_dirs
    if missing_dirs:
        _print_error(f"Missing group directories in {grp.rel_dir}: {sorted(missing_dirs)}")

    missing_tmpls = set(grp.tables.keys()) - seen_tmpl_names
    if missing_tmpls:
        _print_error(f"Missing Tabula templates in {grp.rel_dir}: {sorted(missing_tmpls)}")


def _print_basic_group_yaml(grp_dir: pathlib.Path) -> None:
    top_level = {grp_dir.name: _create_basic_group_yaml(grp_dir)}
    yamlreg.YAML.dump(top_level, sys.stdout)


def _create_basic_group_yaml(grp_dir: pathlib.Path) -> config._YamlGroup:
    grp = config._YamlGroup()
    with os.scandir(grp_dir) as dir_iter:
        for dir_entry in dir_iter:
            dir_entry_path = pathlib.Path(dir_entry)
            if dir_entry.is_file():
                grp.tables[dir_entry.name.removesuffix(config.TABULA_TEMPLATE_SUFFIX)] = (
                    config._YamlTable(transform=cfgextract.LegacyTransformSeq())
                )
            elif dir_entry.is_dir():
                grp.groups[dir_entry.name] = _create_basic_group_yaml(dir_entry_path)
    return grp


if __name__ == "__main__":
    main()
