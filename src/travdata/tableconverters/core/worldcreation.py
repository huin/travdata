# -*- coding: utf-8 -*-
from typing import Iterable, Iterator, Optional, TypedDict, cast

from travdata import parseutil
from travdata.datatypes import basic
from travdata.datatypes.core import worldcreation
from travdata.tableconverters import core

_register_conv = core.CONVERTERS.make_group_decorator(worldcreation.GROUP)


@_register_conv("governments")
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


@_register_conv("law-levels")
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


@_register_conv("trade-codes")
def trade_codes(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[worldcreation.TradeCode]:
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
            planet_sizes=basic.IntRangeSet.parse(row["Planet Size"]),
            atmospheres=basic.IntRangeSet.parse(row["Atmosphere"]),
            hydro=basic.IntRangeSet.parse(row["Hydro"]),
            population=basic.IntRangeSet.parse(row["Population"]),
            government=basic.IntRangeSet.parse(row["Government"]),
            law_level=basic.IntRangeSet.parse(row["Law Level"]),
            tech_level=basic.IntRangeSet.parse(row["Tech Level"]),
        )
