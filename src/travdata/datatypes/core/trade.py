# -*- coding: utf-8 -*-
import dataclasses
from typing import Optional

from travdata import jsonenc
from travdata.datatypes import yamlcodec


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
@yamlcodec.register_type
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
@yamlcodec.register_type
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
