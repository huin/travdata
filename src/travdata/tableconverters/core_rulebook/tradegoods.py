# -*- coding: utf-8 -*-
import dataclasses
import re
from typing import Iterable, Iterator, Optional, TypedDict, cast

from travdata import jsonenc, parseutil


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class TradeGoodProperties(jsonenc.Decodable, jsonenc.Encodable):
    availability: set[str]
    tons: str
    base_price: int
    purchase_dm: dict[str, int]
    sale_dm: dict[str, int]
    examples: str

    @classmethod
    def json_type(cls) -> str:
        return "TradeGoodProperties"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "TradeGoodProperties":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class TradeGood(jsonenc.Decodable, jsonenc.Encodable):
    d66: str
    name: str
    description: Optional[str]
    properties: Optional[TradeGoodProperties]

    @classmethod
    def json_type(cls) -> str:
        return "TradeGood"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "TradeGood":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


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


def convert_from_rows(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[TradeGood]:
    for row in cast(Iterable[_RawRow], rows):
        if row["Base Price"] is None:
            properties = None
            description = row["Availability"]
        else:
            properties = TradeGoodProperties(
                availability=parseutil.parse_set(row["Availability"]),
                tons=row["Tons"],
                base_price=parseutil.parse_credits(row["Base Price"]),
                purchase_dm=_parse_trade_dm(row["Purchase DM"]),
                sale_dm=_parse_trade_dm(row["Sale DM"]),
                examples=row["Examples"],
            )
            description = None
        yield TradeGood(
            d66=row["D66"],
            name=row["Type"],
            description=description,
            properties=properties,
        )
