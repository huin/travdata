#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import pathlib

import jsonenc
from extractors import params, registry


def _add_extractor_args(argparser: argparse._ArgumentGroup) -> None:
    for ext in registry.EXTRACTORS:
        flag = ext.name.replace("_", "-")
        argparser.add_argument(
            f"--{flag}",
            type=argparse.FileType("wt", encoding="utf-8"),
            metavar="JSON_FILE",
        )


def _handle_extractor_args(args: argparse.Namespace, param: params.CoreParams) -> None:
    for ext in registry.EXTRACTORS:
        if out := getattr(args, ext.name):
            jsonenc.DEFAULT_CODEC.dump(
                fp=out,
                obj=list(ext.fn(param)),
            )


def main() -> None:
    argparser = argparse.ArgumentParser()
    argparser.add_argument(
        "core_rulebook",
        type=pathlib.Path,
        metavar="PDF",
    )
    argparser.add_argument(
        "--templates-dir",
        type=pathlib.Path,
        metavar="DIR",
        default=pathlib.Path("./tabula-templates"),
        required=True,
    )

    extract_grp = argparser.add_argument_group("Table extractors")
    _add_extractor_args(extract_grp)

    args = argparser.parse_args()

    _handle_extractor_args(
        args,
        param=params.CoreParams(
            core_rulebook=args.core_rulebook,
            templates_dir=args.templates_dir,
        ),
    )


if __name__ == "__main__":
    main()
