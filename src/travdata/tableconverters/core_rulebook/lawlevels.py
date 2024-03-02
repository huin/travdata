# -*- coding: utf-8 -*-
import dataclasses
from typing import Iterable, Iterator, Optional, TypedDict, cast

from travdata import jsonenc


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class LawLevel(jsonenc.Decodable, jsonenc.Encodable):
    min_level: int
    max_level: Optional[int]
    description: Optional[str]
    weapons_banned: Optional[str]
    armour_banned: Optional[str]

    @classmethod
    def json_type(cls) -> str:
        return "LawLevel"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "LawLevel":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


_RawRow = TypedDict(
    "_RawRow",
    {
        "Law Level": str,
        "Weapons Banned": str,
        "Armour": str,
    },
)


def convert_from_rows(rows: Iterable[dict[str, Optional[str]]]) -> list[LawLevel]:
    results: list[LawLevel] = []
    for row in cast(Iterable[_RawRow], rows):
        level = row["Law Level"]
        if level.endswith("+"):
            min_level = int(level.removesuffix("+"))
            max_level = None
        else:
            min_level = max_level = int(level)
        if row["Armour"] is None:
            results.append(
                LawLevel(
                    min_level=min_level,
                    max_level=max_level,
                    description=row["Weapons Banned"],
                    weapons_banned=None,
                    armour_banned=None,
                )
            )
        else:
            results.append(
                LawLevel(
                    min_level=min_level,
                    max_level=max_level,
                    description=None,
                    weapons_banned=row["Weapons Banned"] or results[-1].weapons_banned,
                    armour_banned=row["Armour"] or results[-1].armour_banned,
                )
            )

    return results
