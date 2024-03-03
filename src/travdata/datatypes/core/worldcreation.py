# -*- coding: utf-8 -*-
"""Data types relating to world creation.

Notionally these are types derived from the World and Universe Creation chapter
in the core rulebook.
"""

import dataclasses
from typing import Optional

from travdata import jsonenc


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class Government(jsonenc.Decodable, jsonenc.Encodable):
    code: str
    name: str
    description: str
    examples: str
    example_contaband: set[str]

    @classmethod
    def json_type(cls) -> str:
        return "Government"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "Government":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


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
