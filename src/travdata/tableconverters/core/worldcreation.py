# -*- coding: utf-8 -*-
import re
from typing import Iterable, Iterator, Optional, TypedDict, cast

from travdata import parseutil
from travdata.datatypes.core import trade, worldcreation


def governments(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[worldcreation.Government]:
    _RawRow = TypedDict(
        "_RawRow",
        {
            "Government": str,
            "Government Type": str,
            "Description": str,
            "Examples": str,
            "Example Contraband": str,
        },
        total=True,
    )
    for row in cast(Iterable[_RawRow], rows):
        yield worldcreation.Government(
            code=row["Government"],
            name=row["Government Type"],
            description=row["Description"],
            examples=row["Examples"],
            example_contaband=parseutil.parse_set(row["Example Contraband"]),
        )


def law_levels(rows: Iterable[dict[str, Optional[str]]]) -> list[worldcreation.LawLevel]:
    _RawRow = TypedDict(
        "_RawRow",
        {
            "Law Level": str,
            "Weapons Banned": str,
            "Armour": str,
        },
    )
    results: list[worldcreation.LawLevel] = []
    for row in cast(Iterable[_RawRow], rows):
        level = row["Law Level"]
        if level.endswith("+"):
            min_level = int(level.removesuffix("+"))
            max_level = None
        else:
            min_level = max_level = int(level)
        if row["Armour"] is None:
            results.append(
                worldcreation.LawLevel(
                    min_level=min_level,
                    max_level=max_level,
                    description=row["Weapons Banned"],
                    weapons_banned=None,
                    armour_banned=None,
                )
            )
        else:
            results.append(
                worldcreation.LawLevel(
                    min_level=min_level,
                    max_level=max_level,
                    description=None,
                    weapons_banned=row["Weapons Banned"] or results[-1].weapons_banned,
                    armour_banned=row["Armour"] or results[-1].armour_banned,
                )
            )

    return results


def _parse_range(v: str, max_value: Optional[int]) -> Iterable[int]:
    _range_hyphen = "–"
    if not v:
        return ()
    elif v.endswith("+"):
        min_value = int(v.removesuffix("+"))
        if max_value is None or max_value < min_value:
            raise ValueError(f"{v=} {min_value=} {max_value=}")
        return range(min_value, max_value + 1)
    elif v.endswith(_range_hyphen):
        max_value = int(v.removesuffix(_range_hyphen))
        return range(0, max_value + 1)
    elif _range_hyphen in v:
        min_s, _, max_s = v.partition(_range_hyphen)
        return range(int(min_s), int(max_s) + 1)
    else:
        return (int(v),)


def _parse_set(v: str, max_value: Optional[int] = None) -> set[int]:
    ranges = v.split(",")
    result: set[int] = set()
    for r in ranges:
        result.update(_parse_range(r, max_value))
    return result


def trade_codes(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[worldcreation.TradeCode]:
    max_size = 10
    max_atmosphere = 15
    max_hydro = 10
    max_population = 12
    max_tech = 15
    _RawRow = TypedDict(
        "_RawRow",
        {
            "Classification": str,
            "Code": str,
            "Planet Size": str,
            "Atmosphere": str,
            "Hydro": str,
            "Population": str,
            "Government": str,
            "Law Level": str,
            "Tech Level": str,
        },
    )
    for row in cast(Iterable[_RawRow], rows):
        yield worldcreation.TradeCode(
            classification=row["Classification"],
            code=row["Code"],
            planet_sizes=_parse_set(row["Planet Size"], max_size),
            atmospheres=_parse_set(row["Atmosphere"], max_atmosphere),
            hydro=_parse_set(row["Hydro"], max_hydro),
            population=_parse_set(row["Population"], max_population),
            government=_parse_set(row["Government"]),
            law_level=_parse_set(row["Law Level"]),
            tech_level=_parse_set(row["Tech Level"], max_tech),
        )


_DM_ITEM_RX = re.compile(r"(.+) ([-+]\d+)")


def _parse_trade_dm(s: str) -> dict[str, int]:
    result: dict[str, int] = {}
    for item in s.split(","):
        match = _DM_ITEM_RX.fullmatch(item)
        if not match:
            raise ValueError(item)
        name, dm = match.group(1, 2)
        result[name.strip()] = int(dm)
    return result


def trade_goods(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[trade.TradeGood]:
    _RawRow = TypedDict(
        "_RawRow",
        {
            "D66": str,
            "Type": str,
            "Availability": str,
            "Tons": str,
            "Base Price": str,
            "Purchase DM": str,
            "Sale DM": str,
            "Examples": str,
        },
        total=True,
    )
    for row in cast(Iterable[_RawRow], rows):
        if row["Base Price"] is None:
            properties = None
            description = row["Availability"]
        else:
            properties = trade.TradeGoodProperties(
                availability=parseutil.parse_set(row["Availability"]),
                tons=row["Tons"],
                base_price=parseutil.parse_credits(row["Base Price"]),
                purchase_dm=_parse_trade_dm(row["Purchase DM"]),
                sale_dm=_parse_trade_dm(row["Sale DM"]),
                examples=row["Examples"],
            )
            description = None
        yield trade.TradeGood(
            d66=row["D66"],
            name=row["Type"],
            description=description,
            properties=properties,
        )
