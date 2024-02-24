#!/usr/bin/env python3
# -*- coding: utf-8 -*-


import argparse
import csv
import dataclasses
import enum
import io
import sys
from typing import Iterator, Optional, TypeAlias, TypeVar, cast

import jsonenc
from extractors import tradecodes, tradegoods

T = TypeVar("T")
# Maps from TradeGood.d66 to the lowest law level at which that good is illegal.
TradeGoodIllegality: TypeAlias = dict[str, int]


def _load_json(t: type[T], fp: io.TextIOBase) -> T:
    return cast(T, jsonenc.DEFAULT_CODEC.load(fp))


class StarportType(enum.StrEnum):
    EXCELLENT = "A"
    GOOD = "B"
    ROUTINE = "C"
    POOR = "D"
    FRONTIER = "E"
    NONE = "X"


class TravelCode(enum.StrEnum):
    AMBER = "Amber"
    RED = "Red"


@dataclasses.dataclass
class UWP:
    starport: StarportType
    size: int
    atmosphere: int
    hydrographic: int
    population: int
    government: int
    law_level: int
    tech_level: int

    @classmethod
    def from_string(cls, uwp: str) -> "UWP":
        codes = uwp.replace("-", "")
        if len(codes) != 8:
            raise ValueError(uwp)
        int_codes = [int(v, 16) for v in codes[1:]]
        return UWP(StarportType(codes[0]), *int_codes)


@dataclasses.dataclass
class _WorldData:
    hex_location: str
    name: str
    uwp: UWP
    bases: set[str]
    trade_codes: set[str]
    travel_code: Optional[TravelCode]


def _load_world_data(fp: io.TextIOBase) -> Iterator[_WorldData]:
    # TODO: Use a more generic way to load and represent world data.
    r = csv.DictReader(fp)
    for row in r:
        c = row["Travel Code"]
        yield _WorldData(
            hex_location=row["Location"],
            name=row["Name"],
            uwp=UWP.from_string(row["UWP"]),
            bases=set(row["Bases"].split(":")),
            trade_codes=set(row["Trade Codes"].split(":")),
            travel_code=None if not c else TravelCode(c),
        )


def _trade_dm(dms: dict[str, int], world_trades: set[str]) -> int:
    return max(
        (dms[wt] for wt in world_trades if wt in dms),
        default=0,
    )


def main() -> None:
    argparser = argparse.ArgumentParser(
        description="""Produce a purchase DM table based on:
        https://sirpoley.tumblr.com/post/643218580118323200/on-creating-a-frictionless-traveller-part-i
        """,
    )
    argparser.add_argument(
        "--trade-codes",
        type=argparse.FileType("rt"),
        metavar="trade-codes.json",
        required=True,
    )
    argparser.add_argument(
        "--trade-goods",
        type=argparse.FileType("rt"),
        metavar="trade-goods.json",
        required=True,
    )
    argparser.add_argument(
        "--trade-good-illegality",
        type=argparse.FileType("rt"),
        metavar="trade-good-illegality.json",
    )
    argparser.add_argument(
        "--world-data",
        type=argparse.FileType("rt"),
        metavar="world-data.csv",
        required=True,
    )
    argparser.add_argument(
        "--format-common",
        help="Python str.format string for DMs of commonly available goods." " world.",
        default="<b>{}</b>",
    )
    argparser.add_argument(
        "--format-unavailable",
        help="Python str.format string for items that are unavailable for purchase.",
        default="{}!",
    )
    argparser.add_argument(
        "--format-legal",
        help="Python str.format string for DMs of goods that are legal on a" " world.",
        default="{}",
    )
    argparser.add_argument(
        "--format-illegal",
        help="Python str.format string for DMs of goods that are illegal on a"
        " world.",
        default="<ul>{}</ul>",
    )
    argparser.add_argument(
        "--include-headers",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    argparser.add_argument(
        "--include-key",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    argparser.add_argument(
        "--include-explanation",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )

    args = argparser.parse_args()

    jsonenc.DEFAULT_CODEC.self_register_builtins()

    tcodes = _load_json(list[tradecodes.TradeCode], args.trade_codes)
    tgoods = _load_json(list[tradegoods.TradeGood], args.trade_goods)
    tgood_illegality: TradeGoodIllegality
    if args.trade_good_illegality:
        tgood_illegality = _load_json(TradeGoodIllegality, args.trade_good_illegality)
    else:
        tgood_illegality = {}

    worlds = list(_load_world_data(args.world_data))
    tcodes_by_code = {tc.code: tc for tc in tcodes}
    # Parallel list of the trade classifications that the world has.
    per_world_trades: list[set[str]] = [
        {tcodes_by_code[tc].classification for tc in world.trade_codes}
        for world in worlds
    ]

    w = csv.writer(sys.stdout)
    if args.include_headers:
        w.writerow(
            ["D66", "Goods", "Tons", "Base Price (cr)"]
            + [w.hex_location for w in worlds]
        )
    for tgood in tgoods:
        row = [
            tgood.d66,
            tgood.name,
        ]
        if not tgood.properties:
            w.writerow(row)
            continue
        tprops = tgood.properties
        row.append(tprops.tons)
        row.append(str(tprops.base_price))
        illegality = tgood_illegality.get(tgood.d66, None)
        for world, world_trades in zip(
            worlds,
            per_world_trades,
        ):
            fmts = []
            is_common = "All" in tprops.availability
            if is_common:
                fmts.append(args.format_common)

            is_available = is_common or not tprops.availability.isdisjoint(world_trades)
            if not is_available:
                fmts.append(args.format_unavailable)

            purchase_dm = _trade_dm(tprops.purchase_dm, world_trades)
            illegal = illegality is not None and world.uwp.law_level >= illegality
            if illegal:
                fmts.append(args.format_illegal)
                sale_dm = world.uwp.law_level - cast(int, illegality)
            else:
                fmts.append(args.format_legal)
                sale_dm = _trade_dm(tprops.sale_dm, world_trades)

            dm = purchase_dm - sale_dm
            dm_str = f"{dm:+}"

            for fmt in fmts:
                dm_str = fmt.format(dm_str)

            row.append(dm_str)

        w.writerow(row)

    if args.include_key:
        w.writerow(["Key:"])
        entries = [
            (args.format_common, "Commonly available goods."),
            (args.format_unavailable, "Good unavailable for purchase."),
            (args.format_legal, "Legal by planetary law."),
            (args.format_illegal, "Illegal by planetary law."),
        ]
        for fmt, explanation in entries:
            w.writerow([fmt.format("Good name"), explanation])

    if args.include_explanation:
        if args.include_key:
            w.writerow([])
        w.writerow(
            ["Number is added when buying goods, and subtracted when selling goods."]
        )
        w.writerow(
            ["High numbers indicate excess of supply, low numbers indicate demand."]
        )


if __name__ == "__main__":
    main()
