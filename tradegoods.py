# -*- coding: utf-8 -*-
import dataclasses
import itertools
import pathlib
import re
from typing import Iterator, Optional

import jsonenc
import parseutil


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class TradeGoodProperties(jsonenc.Decodable, jsonenc.Encodable):
    availability: set[str]
    tons: str
    base_price: int
    purchase_dm: dict[str, int]
    sale_dm: dict[str, int]

    @classmethod
    def json_type(cls) -> str:
        return "TradeGoodProperties"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "TradeGoodProperties":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return dataclasses.asdict(self)


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class TradeGood(jsonenc.Decodable, jsonenc.Encodable):
    d66: str
    name: str
    properties: Optional[TradeGoodProperties]

    @classmethod
    def json_type(cls) -> str:
        return "TradeGood"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "TradeGood":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return dataclasses.asdict(self)


_DM_ITEM_RX = re.compile(r"(.+) ([-+]\d+)")


def _parse_trade_dm(s: str) -> dict[str, int]:
    s = parseutil.clean_text(s)
    result: dict[str, int] = {}
    for item in s.split(","):
        match = _DM_ITEM_RX.fullmatch(item)
        if not match:
            raise ValueError(item)
        name, dm = match.group(1, 2)
        result[name.strip()] = int(dm)
    return result


def extract_from_pdf(core_rulebook: pathlib.Path) -> list[TradeGood]:
    tables = parseutil.read_pdf(pdf_path=core_rulebook, pages=[245, 246])

    goods: list[TradeGood] = []
    rows = itertools.chain(*[t["data"] for t in tables])

    for d66, row in zip(parseutil.d66_enum(), rows):
        (name, availability, tons, base_price, purchase_dm, sale_dm) = [
            v["text"] for v in row
        ]
        goods.append(
            TradeGood(
                d66=str(d66),
                name=parseutil.clean_text(name),
                properties=TradeGoodProperties(
                    availability=parseutil.parse_set(availability),
                    tons=tons,
                    base_price=parseutil.parse_credits(base_price),
                    purchase_dm=_parse_trade_dm(purchase_dm),
                    sale_dm=_parse_trade_dm(sale_dm),
                ),
            )
        )
    goods.append(
        TradeGood(
            d66="66",
            name="Exotics",
            properties=None,
        )
    )

    return goods
