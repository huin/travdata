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
        "--omit-unavailable",
        type=bool,
        action=argparse.BooleanOptionalAction,
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
    w.writerow(
        ["D66", "Goods", "Tons", "Base Price (cr)"] + [w.hex_location for w in worlds]
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
        row.append(tprops.base_price)
        illegality = tgood_illegality.get(tgood.d66, None)
        for world, world_trades in zip(
            worlds,
            per_world_trades,
        ):
            is_available = (
                "All" in tprops.availability
                or not tprops.availability.isdisjoint(world_trades)
            )
            if not is_available and args.omit_unavailable:
                row.append("")
                continue

            purchase_dm = max(tprops.purchase_dm.get(wt, 0) for wt in world_trades)
            if illegality is not None and world.uwp.law_level >= illegality:
                sale_dm = world.uwp.law_level - illegality
            else:
                sale_dm = max(tprops.sale_dm.get(wt, 0) for wt in world_trades)

            dm = purchase_dm - sale_dm

            row.append(f"{dm:+}")

        w.writerow(row)


if __name__ == "__main__":
    main()
