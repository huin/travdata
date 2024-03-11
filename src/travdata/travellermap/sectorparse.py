# -*- coding: utf-8 -*-
"""Parses various sector data formats.

Several are documented at https://travellermap.com/doc/fileformats
"""

import csv
import io
from typing import Iterator

from travdata.datatypes.core import worldcreation
from travdata.extraction import parseutil
from travdata.travellermap import world

_T5_TRAVEL_CODES: dict[str, world.TravelCode] = {
    "A": world.TravelCode.AMBER,
    "R": world.TravelCode.RED,
    "": world.TravelCode.NONE,
}


def _remove_brackets(v: str, brackets: str) -> str:
    return v.removeprefix(brackets[0]).removesuffix(brackets[1]).strip()


def t5_tsv(fp: io.TextIOBase) -> Iterator[world.World]:
    """Parses T5 Tab Delimited Format.

    Specified by https://travellermap.com/doc/fileformats#t5col

    :param fp: File to read TSV data from.
    :yield: World records.
    """
    r = csv.DictReader(fp, delimiter="\t", quoting=csv.QUOTE_NONE, strict=True)
    for row in r:
        if not row:
            # Skip blank lines.
            continue
        pbg = row["PBG"]
        yield world.World(
            ext=world.WorldExtensions(
                cultural=_remove_brackets(row["[Cx]"], "[]"),
                economic=_remove_brackets(row["(Ex)"], "()"),
                importance=int(_remove_brackets(row["{Ix}"], "{}")),
                resource_units=int(row["RU"]) if "RU" in row else None,
            ),
            location=world.WorldLocation(
                sector_abbv=row.get("Sector"),
                subsector=parseutil.map_opt_dict_key(world.SubsectorCode, row, "SS"),
                subsector_hex=world.SubSectorLoc.parse(row["Hex"]),
            ),
            name=row["Name"],
            social=world.WorldSocial(
                allegiance=row["Allegiance"],
                bases=frozenset(world.BaseCode(b) for b in row["Bases"]),
                nobility=row["Nobility"],
                pop_multiplier=int(pbg[0]),
                trade_codes=frozenset(world.TradeCode(tc) for tc in row["Remarks"].split()),
            ),
            system=world.WorldSystem(
                num_belts=int(pbg[1]),
                num_gas_giants=int(pbg[2]),
                num_worlds=int(row["W"]),
                stellar=tuple(world.StellarCode(sc) for sc in row["Stars"].split()),
            ),
            travel_code=_T5_TRAVEL_CODES[row["Zone"]],
            uwp=worldcreation.UWP.parse(row["UWP"]),
        )
