# -*- coding: utf-8 -*-
"""Data types relating to world creation.

Notionally these are types derived from the World and Universe Creation chapter
in the core rulebook.
"""

import dataclasses
import enum
from typing import ClassVar, Optional

from travdata import jsonenc, parseutil
from travdata.datatypes import basic, yamlcodec


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
@yamlcodec.register_type
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
@yamlcodec.register_type
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
@yamlcodec.register_type
class TradeCode(jsonenc.Decodable, jsonenc.Encodable):
    classification: str
    code: str
    planet_sizes: basic.IntRangeSet
    atmospheres: basic.IntRangeSet
    hydro: basic.IntRangeSet
    population: basic.IntRangeSet
    government: basic.IntRangeSet
    law_level: basic.IntRangeSet
    tech_level: basic.IntRangeSet

    @classmethod
    def json_type(cls) -> str:
        return "TradeCode"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "TradeCode":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


@enum.unique
@yamlcodec.register_type
class StarportType(enum.StrEnum):
    EXCELLENT = "A"
    GOOD = "B"
    ROUTINE = "C"
    POOR = "D"
    FRONTIER = "E"
    NONE = "X"


@dataclasses.dataclass(frozen=True)
@yamlcodec.register_type
class UWP:
    yaml_tag: ClassVar = "!UWP"
    starport: StarportType
    size: int
    atmosphere: int
    hydrographic: int
    population: int
    government: int
    law_level: int
    tech_level: int

    @classmethod
    def parse(cls, uwp: str) -> "UWP":
        codes = uwp.replace("-", "")
        if len(codes) != 8:
            raise ValueError(uwp)
        int_codes = [parseutil.parse_ehex_char(v) for v in codes[1:]]
        return UWP(StarportType(codes[0]), *int_codes)

    def __str__(self) -> str:
        return "".join(
            [
                str(self.starport),
                parseutil.fmt_ehex_char(self.size),
                parseutil.fmt_ehex_char(self.atmosphere),
                parseutil.fmt_ehex_char(self.hydrographic),
                parseutil.fmt_ehex_char(self.population),
                parseutil.fmt_ehex_char(self.government),
                parseutil.fmt_ehex_char(self.law_level),
                parseutil.fmt_ehex_char(self.tech_level),
            ]
        )

    @classmethod
    def to_yaml(cls, representer, node):
        return representer.represent_scalar(cls.yaml_tag, str(node))

    @classmethod
    def from_yaml(cls, constructor, node):
        return cls.parse(node.value)