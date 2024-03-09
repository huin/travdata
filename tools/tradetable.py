#!/usr/bin/env python3
# -*- coding: utf-8 -*-


import argparse
import codecs
import csv
import dataclasses
import enum
import io
import pathlib
import sys
import urllib.request
from typing import (Callable, Iterable, Iterator, Optional, TypeAlias, TypeVar,
                    cast)

from travdata import parseutil
from travdata.datatypes import yamlcodec
from travdata.datatypes.core import trade, worldcreation
from travdata.travellermap import apiurls, sectorparse, world

T = TypeVar("T")
# Maps from TradeGood.d66 to the lowest law level at which that good is illegal.
TradeGoodIllegality: TypeAlias = dict[str, int]


class UserError(Exception):
    pass


class _IgnoreUnknown(enum.StrEnum):
    TRADE_CODES = "trade-codes"


# Trade overrides for a trade good on a world.
@dataclasses.dataclass
@yamlcodec.register_type
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


def _load_yaml(t: type[T], stream: pathlib.Path | io.TextIOBase) -> T:
    data = yamlcodec.DATATYPES_YAML.load(stream)
    if not isinstance(data, t):
        raise TypeError(type(data))
    return data


def _load_world_csv_data(path: str) -> Iterator[world.World]:
    """Parses a CSV file containing specific fields.

    These fields must be declared in the header row, and must include:

    - "Location" - the subsector hex location.
    - "Name" - the name of the world.
    - "UWP" - the UWP code.
    - "Trade Codes" - a colon (":") delimited list of two-letter trade codes.

    :param fp: File to read CSV data from.
    :yield: World data.
    """
    with open(path, "rt") as fp:
        r = csv.DictReader(fp)
        for row in r:
            c = row["Travel Code"]
            yield world.World(
                name=row["Name"],
                location=world.WorldLocation(
                    subsector_hex=world.SubSectorLoc.parse(row["Location"]),
                ),
                uwp=worldcreation.UWP.parse(row["UWP"]),
                social=world.WorldSocial(
                    trade_codes=frozenset(
                        world.TradeCode(tc) for tc in row["Trade Codes"].split(":")
                    ),
                ),
            )


def _load_travellermap_tsv_file(path: str) -> list[world.World]:
    with open(path, "rt") as fp:
        return list(sectorparse.t5_tsv(fp))


def _load_travellermap_tsv_url(url: str) -> list[world.World]:
    utf8 = codecs.lookup("utf-8")
    with urllib.request.urlopen(url) as response:
        decoder = cast(io.TextIOBase, utf8.streamreader(response))
        return list(sectorparse.t5_tsv(decoder))


def _load_travellermap_subsector(spec: str) -> list[world.World]:
    sector, slash, subsector_str = spec.partition("/")
    if not any([sector, slash, subsector_str]):
        raise UserError(
            "Invalid format for travellermap_subsector - expected sector/subsectorletter, e.g. spin/C"
        )
    url = apiurls.uwp_data(
        sector=apiurls.SectorId(sector), # type: ignore[arg-type]
        subsector=apiurls.SubSectorCode[subsector_str], # type: ignore[arg-type]
    )
    return _load_travellermap_tsv_url(url)


_WORLD_DATA_TYPES: dict[str, Callable[[str], Iterable[world.World]]] = {
    "csv": _load_world_csv_data,
    "travellermap_tsv_file": _load_travellermap_tsv_file,
    "travellermap_tsv_url": _load_travellermap_tsv_url,
    "travellermap_subsector": _load_travellermap_subsector,
}


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
        "data_dir",
        help="Path to the directory to read the Traveller YAML files from.",
        type=pathlib.Path,
        metavar="IN_DIR",
    )
    data_inputs_grp.add_argument(
        "--trade-good-illegality",
        help=(
            "Goods illegality data. Simple YAML mapping from trade good d66"
            " string to the numeric minimum law level at which it considered"
            " illegal."
        ),
        type=argparse.FileType("rt"),
        metavar="trade-good-illegality.yaml",
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
        "world_data",
        help="""World data within a single subsector. The meaning and
        interpretation of this parameter is determined by
        --world-data-source. An example of the type of URL required by
        travellermap_tsv_url is:
        https://travellermap.com/api/sec?sector=spin&subsector=A&type=TabDelimited""",
        metavar="WORLD_DATA",
    )
    data_inputs_grp.add_argument(
        "--world-data-source",
        help="""Determines the meaning of the world_data argument.""",
        choices=_WORLD_DATA_TYPES.keys(),
        default="csv",
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

    argparser.add_argument(
        "--ignore-unknowns",
        help="""Ignore unknown trade codes.""",
        choices=sorted(_IgnoreUnknown),
        action="append",
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


def _not_none(v: Optional[T], desc: str) -> T:
    if v is None:
        raise ValueError(f"{desc} was missing a value")
    return v


def _must_world_trade_codes(w: world.World) -> frozenset[world.TradeCode]:
    return _not_none(_not_none(w.social, "world social data").trade_codes, "world trade codes")


def _must_world_subsector_loc(w: world.World) -> str:
    return str(
        _not_none(
            _not_none(w.location, "world location data").subsector_hex,
            "world subsector location",
        )
    )


def _must_world_law_level(w: world.World) -> int:
    return _not_none(w.uwp, "world UWP data").law_level


def process(args: argparse.Namespace) -> None:
    tcodes = cast(
        list[worldcreation.TradeCode],
        _load_yaml(list, args.data_dir / worldcreation.GROUP / "trade-codes.yaml"),
    )
    tgoods = cast(
        list[trade.TradeGood], _load_yaml(list, args.data_dir / trade.GROUP / "trade-goods.yaml")
    )
    tgood_illegality: TradeGoodIllegality
    if args.trade_good_illegality:
        tgood_illegality = cast(TradeGoodIllegality, _load_yaml(dict, args.trade_good_illegality))
    else:
        tgood_illegality = {}
    if args.world_trade_overrides:
        wt_overrides = _load_world_trade_overrides(args.world_trade_overrides)
    else:
        wt_overrides = {}

    world_reader = _WORLD_DATA_TYPES[args.world_data_source]
    worlds = list(world_reader(args.world_data))

    tcodes_by_code = {tc.code: tc for tc in tcodes}
    # Construct parallel list of the trade classifications that the world has.
    per_world_trades: list[set[str]] = []
    for w in worlds:
        trades: set[str] = set()
        per_world_trades.append(trades)
        for tc in _must_world_trade_codes(w):
            try:
                trade_code = tcodes_by_code[tc]
            except KeyError as e:
                if _IgnoreUnknown.TRADE_CODES not in args.ignore_unknowns:
                    raise UserError(
                        f"unknown trade code {e}, use --ignore-unknowns=trade-codes to ignore"
                    ) from e
            else:
                trades.add(trade_code.classification)

    csv_writer = csv.writer(sys.stdout)
    if args.include_headers:
        csv_writer.writerow(
            ["D66", "Goods", "Tons", "Base Price (cr)"]
            + [_must_world_subsector_loc(w) for w in worlds]
        )
    for tgood in tgoods:
        row = [
            tgood.d66,
            tgood.name,
        ]
        if not tgood.properties:
            csv_writer.writerow(row)
            continue
        tprops = tgood.properties
        row.append(tprops.tons)
        row.append(str(tprops.base_price))
        illegality = tgood_illegality.get(tgood.d66, None)
        for w, world_trades in zip(
            worlds,
            per_world_trades,
        ):
            overrides = wt_overrides.get(
                (_must_world_subsector_loc(w), tgood.d66), _EMPTY_OVERRIDES
            )

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

            law_level = _must_world_law_level(w)

            illegal = illegality is not None and law_level >= illegality
            illegal = _eval_override(illegal, overrides.illegal)
            if illegal:
                fmts.append(args.format_illegal)
            else:
                fmts.append(args.format_legal)
            if illegal and illegality is not None:
                sale_dm = law_level - illegality
            else:
                sale_dm = _trade_dm(tprops.sale_dm, world_trades)
            sale_dm = _eval_override(sale_dm, overrides.sale_dm)

            dm = purchase_dm - sale_dm
            dm_str = f"{dm:+}"

            for fmt in fmts:
                dm_str = fmt.format(dm_str)

            row.append(dm_str)

        csv_writer.writerow(row)

    if args.include_key:
        csv_writer.writerow(["Key:"])
        entries = [
            (args.format_common, "Commonly available goods."),
            (args.format_unavailable, "Good unavailable for purchase."),
            (args.format_legal, "Legal by planetary law."),
            (args.format_illegal, "Illegal by planetary law."),
        ]
        for fmt, explanation in entries:
            csv_writer.writerow([fmt.format("Good name"), explanation])

    if args.include_explanation:
        if args.include_key:
            csv_writer.writerow([])
        csv_writer.writerow(
            ["Number is added when buying goods, and subtracted when selling goods."]
        )
        csv_writer.writerow(
            ["High numbers indicate excess of supply, low numbers indicate demand."]
        )


def main() -> None:
    args = parse_args()
    try:
        process(args)
    except UserError as e:
        print(f"Error: {e}", file=sys.stderr)


if __name__ == "__main__":
    main()
