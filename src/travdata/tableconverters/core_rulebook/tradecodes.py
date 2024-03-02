# -*- coding: utf-8 -*-
import dataclasses
from typing import Iterable, Iterator, Optional, TypedDict, cast

from travdata import jsonenc

_MAX_SIZE = 10
_MAX_ATMOSPHERE = 15
_MAX_HYDRO = 10
_MAX_POPULATION = 12
_MAX_TECH = 15


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class TradeCode(jsonenc.Decodable, jsonenc.Encodable):
    classification: str
    code: str
    planet_sizes: set[int]
    atmospheres: set[int]
    hydro: set[int]
    population: set[int]
    government: set[int]
    law_level: set[int]
    tech_level: set[int]

    @classmethod
    def json_type(cls) -> str:
        return "TradeCode"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "TradeCode":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


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


_RANGE_HYPHEN = "–"


def _parse_range(v: str, max_value: Optional[int]) -> Iterable[int]:
    if not v:
        return ()
    elif v.endswith("+"):
        min_value = int(v.removesuffix("+"))
        if max_value is None or max_value < min_value:
            raise ValueError(f"{v=} {min_value=} {max_value=}")
        return range(min_value, max_value + 1)
    elif v.endswith(_RANGE_HYPHEN):
        max_value = int(v.removesuffix(_RANGE_HYPHEN))
        return range(0, max_value + 1)
    elif _RANGE_HYPHEN in v:
        min_s, _, max_s = v.partition(_RANGE_HYPHEN)
        return range(int(min_s), int(max_s) + 1)
    else:
        return (int(v),)


def _parse_set(v: str, max_value: Optional[int] = None) -> set[int]:
    ranges = v.split(",")
    result: set[int] = set()
    for r in ranges:
        result.update(_parse_range(r, max_value))
    return result


def convert_from_rows(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[TradeCode]:
    for row in cast(Iterable[_RawRow], rows):
        yield TradeCode(
            classification=row["Classification"],
            code=row["Code"],
            planet_sizes=_parse_set(row["Planet Size"], _MAX_SIZE),
            atmospheres=_parse_set(row["Atmosphere"], _MAX_ATMOSPHERE),
            hydro=_parse_set(row["Hydro"], _MAX_HYDRO),
            population=_parse_set(row["Population"], _MAX_POPULATION),
            government=_parse_set(row["Government"]),
            law_level=_parse_set(row["Law Level"]),
            tech_level=_parse_set(row["Tech Level"], _MAX_TECH),
        )
