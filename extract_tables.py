#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import itertools
import pathlib
from pprint import pprint

import jsonenc
import tradegoods


def main() -> None:
    argparser = argparse.ArgumentParser()
    argparser.add_argument("core_rulebook", type=pathlib.Path, metavar="PDF")
    argparser.add_argument(
        "--templates-dir",
        type=pathlib.Path,
        metavar="DIR",
        default=pathlib.Path("./tabula-templates"),
        required=True,
    )
    argparser.add_argument(
        "--trade-goods",
        type=argparse.FileType("wt", encoding="utf-8"),
        metavar="JSON_FILE",
    )

    args = argparser.parse_args()

    if out := args.trade_goods:
        goods = tradegoods.extract_from_pdf(
            core_rulebook=args.core_rulebook,
            templates_dir=args.templates_dir,
        )
        jsonenc.DEFAULT_CODEC.dump(obj=goods, fp=out)


if __name__ == "__main__":
    main()
