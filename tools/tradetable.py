#!/usr/bin/env python3
# -*- coding: utf-8 -*-


import argparse
import csv
import dataclasses
import enum
import io
import sys
from typing import Iterator, Optional, TypeAlias, TypeVar, cast

from travellerutil import jsonenc, parseutil
from travellerutil.extractors import tradecodes, tradegoods

T = TypeVar("T")
# Maps from TradeGood.d66 to the lowest law level at which that good is illegal.
TradeGoodIllegality: TypeAlias = dict[str, int]


# Trade overrides for a trade good on a world.
@dataclasses.dataclass
class WorldTradeOverrides:
    available: Optional[bool] = None
    purchase_dm: Optional[int] = None
    sale_dm: Optional[int] = None
    illegal: Optional[bool] = None


def _eval_override(v: T, override: Optional[T]) -> T:
    if override is not None:
        return override
    return v


_EMPTY_OVERRIDES = WorldTradeOverrides()


# Maps: [world location hex,trade good d66] -> overrides
WorldTradeOverridesMap: TypeAlias = dict[tuple[str, str], WorldTradeOverrides]


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


def _pbool(s: str) -> bool:
    v = s.lower()
    if v == "true":
        return True
    elif v == "false":
        return False
    raise ValueError(v)


def _load_world_trade_overrides(fp: io.TextIOBase) -> WorldTradeOverridesMap:
    r = csv.DictReader(fp)
    result: WorldTradeOverridesMap = {}
    for row in r:
        key = row["Location"], row["D66"]
        result[key] = WorldTradeOverrides(
            available=parseutil.map_opt_dict_key(_pbool, row, "Available"),
            purchase_dm=parseutil.map_opt_dict_key(int, row, "Purchase DM"),
            sale_dm=parseutil.map_opt_dict_key(int, row, "Sale DM"),
            illegal=parseutil.map_opt_dict_key(_pbool, row, "Illegal"),
        )
    return result


def _trade_dm(dms: dict[str, int], world_trades: set[str]) -> int:
    return max(
        (dms[wt] for wt in world_trades if wt in dms),
        default=0,
    )


def parse_args(args: Optional[list[str]] = None) -> argparse.Namespace:
    argparser = argparse.ArgumentParser(
        description="""Produce a purchase DM table based on:
        https://sirpoley.tumblr.com/post/643218580118323200/on-creating-a-frictionless-traveller-part-i
        """,
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    data_inputs_grp = argparser.add_argument_group("Data inputs")
    data_inputs_grp.add_argument(
        "--trade-codes",
        help=(
            "Traveller data for trade codes. This can be the output from"
            " extract_tables.py --trade-codes."
        ),
        type=argparse.FileType("rt"),
        metavar="trade-codes.json",
        required=True,
    )
    data_inputs_grp.add_argument(
        "--trade-goods",
        help=(
            "Traveller data for trade goods. This can be the output from"
            " extract_tables.py --trade-goods."
        ),
        type=argparse.FileType("rt"),
        metavar="trade-goods.json",
        required=True,
    )
    data_inputs_grp.add_argument(
        "--trade-good-illegality",
        help=(
            "Goods illegality data. Simple JSON mapping from trade good d66"
            " string to the numeric minimum law level at which it considered"
            " illegal."
        ),
        type=argparse.FileType("rt"),
        metavar="trade-good-illegality.json",
    )
    data_inputs_grp.add_argument(
        "--world-trade-overrides",
        help="""File containing trade overrides for the worlds. CSV file with
        columns: Location,D66,Available,Purchase DM,Sale DM,Illegal
        """,
        type=argparse.FileType("rt"),
        metavar="world-trade-overrides.csv",
    )
    data_inputs_grp.add_argument(
        "--world-data",
        help="World data within a single subsector.",
        type=argparse.FileType("rt"),
        metavar="world-data.csv",
        required=True,
    )

    fmt_grp = argparser.add_argument_group(
        title="DM formatting",
        description=(
            "Extra formatting for DM cells, based on the goods status on a"
            " world. Each takes a single unamed argument `{}`, and is"
            " formatted according to Python str.format - see"
            " https://docs.python.org/3/library/stdtypes.html#str.format."
        ),
    )
    fmt_grp.add_argument(
        "--format-common",
        help="When the goods are commonly available.",
        default="<b>{}</b>",
        metavar="FORMAT_STRING",
    )
    fmt_grp.add_argument(
        "--format-unavailable",
        help="When the goods is unavailable for purchase.",
        default="{}!",
        metavar="FORMAT_STRING",
    )
    fmt_grp.add_argument(
        "--format-legal",
        help="When the goods are legal.",
        default="{}",
        metavar="FORMAT_STRING",
    )
    fmt_grp.add_argument(
        "--format-illegal",
        help="When the goods are illegal.",
        default="<ul>{}</ul>",
        metavar="FORMAT_STRING",
    )

    inc_grp = argparser.add_argument_group("Extra information to include")
    inc_grp.add_argument(
        "--include-headers",
        help="Include the table headers.",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    inc_grp.add_argument(
        "--include-key",
        help="Include a key to the DM formatting.",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    inc_grp.add_argument(
        "--include-explanation",
        help="Include explanation for how to use the table.",
        type=bool,
        action=argparse.BooleanOptionalAction,
        default=True,
    )

    return argparser.parse_args()


def process(args: argparse.Namespace) -> None:
    jsonenc.DEFAULT_CODEC.self_register_builtins()

    tcodes = _load_json(list[tradecodes.TradeCode], args.trade_codes)
    tgoods = _load_json(list[tradegoods.TradeGood], args.trade_goods)
    tgood_illegality: TradeGoodIllegality
    if args.trade_good_illegality:
        tgood_illegality = _load_json(TradeGoodIllegality, args.trade_good_illegality)
    else:
        tgood_illegality = {}
    if args.world_trade_overrides:
        wt_overrides = _load_world_trade_overrides(args.world_trade_overrides)
    else:
        wt_overrides = {}

    worlds = list(_load_world_data(args.world_data))
    tcodes_by_code = {tc.code: tc for tc in tcodes}
    # Parallel list of the trade classifications that the world has.
    per_world_trades: list[set[str]] = [
        {tcodes_by_code[tc].classification for tc in world.trade_codes} for world in worlds
    ]

    w = csv.writer(sys.stdout)
    if args.include_headers:
        w.writerow(["D66", "Goods", "Tons", "Base Price (cr)"] + [w.hex_location for w in worlds])
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
            overrides = wt_overrides.get((world.hex_location, tgood.d66), _EMPTY_OVERRIDES)

            fmts = []
            is_common = "All" in tprops.availability
            if is_common:
                fmts.append(args.format_common)

            is_available = is_common or not tprops.availability.isdisjoint(world_trades)
            is_available = _eval_override(is_available, overrides.available)
            if not is_available:
                fmts.append(args.format_unavailable)

            purchase_dm = _trade_dm(tprops.purchase_dm, world_trades)
            purchase_dm = _eval_override(purchase_dm, overrides.purchase_dm)

            illegal = illegality is not None and world.uwp.law_level >= illegality
            illegal = _eval_override(illegal, overrides.illegal)
            if illegal:
                fmts.append(args.format_illegal)
            else:
                fmts.append(args.format_legal)
            if illegal and illegality is not None:
                sale_dm = world.uwp.law_level - illegality
            else:
                sale_dm = _trade_dm(tprops.sale_dm, world_trades)
            sale_dm = _eval_override(sale_dm, overrides.sale_dm)

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
        w.writerow(["Number is added when buying goods, and subtracted when selling goods."])
        w.writerow(["High numbers indicate excess of supply, low numbers indicate demand."])


def main() -> None:
    args = parse_args()
    process(args)


if __name__ == "__main__":
    main()
