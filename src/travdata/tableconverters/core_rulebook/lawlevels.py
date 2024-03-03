# -*- coding: utf-8 -*-
from typing import Iterable, Optional, TypedDict, cast

from travdata.datatypes.core import worldcreation

_RawRow = TypedDict(
    "_RawRow",
    {
        "Law Level": str,
        "Weapons Banned": str,
        "Armour": str,
    },
)


def convert_from_rows(rows: Iterable[dict[str, Optional[str]]]) -> list[worldcreation.LawLevel]:
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
