#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import pathlib

import jsonenc
from extractors import governments, lawlevels, tradegoods


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
    extract_grp.add_argument(
        "--governments",
        type=argparse.FileType("wt", encoding="utf-8"),
        metavar="JSON_FILE",
    )
    extract_grp.add_argument(
        "--law-levels",
        type=argparse.FileType("wt", encoding="utf-8"),
        metavar="JSON_FILE",
    )
    extract_grp.add_argument(
        "--trade-goods",
        type=argparse.FileType("wt", encoding="utf-8"),
        metavar="JSON_FILE",
    )

    args = argparser.parse_args()

    if out := args.governments:
        jsonenc.DEFAULT_CODEC.dump(
            fp=out,
            obj=governments.extract_from_pdf(
                core_rulebook=args.core_rulebook,
                templates_dir=args.templates_dir,
            ),
        )
    if out := args.law_levels:
        jsonenc.DEFAULT_CODEC.dump(
            fp=out,
            obj=lawlevels.extract_from_pdf(
                core_rulebook=args.core_rulebook,
                templates_dir=args.templates_dir,
            ),
        )
    if out := args.trade_goods:
        jsonenc.DEFAULT_CODEC.dump(
            fp=out,
            obj=tradegoods.extract_from_pdf(
                core_rulebook=args.core_rulebook,
                templates_dir=args.templates_dir,
            ),
        )


if __name__ == "__main__":
    main()
