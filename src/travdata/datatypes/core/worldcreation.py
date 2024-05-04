# -*- coding: utf-8 -*-
"""Data types relating to world creation.

Notionally these are types derived from the World and Universe Creation chapter
in the core rulebook.
"""

# Many types inherently have a lot of attributes, this reflects the data from
# the book.
# pylint: disable=too-many-instance-attributes

import dataclasses
import enum
from typing import Optional

from travdata.datatypes import basic
from travdata.extraction import parseutil

GROUP = "worldcreation"


@dataclasses.dataclass
class Government:
    """Government type identifying code and information."""

    code: str
    name: str
    description: str
    examples: str
    example_contaband: set[str]


@dataclasses.dataclass
class LawLevel:
    """Descriptors for law levels, and weapons and armour banned in them."""

    min_level: int
    max_level: Optional[int]
    description: Optional[str]
    weapons_banned: Optional[str]
    armour_banned: Optional[str]


@dataclasses.dataclass
class TradeCode:
    """Criteria for a world trade classification/code."""

    classification: str
    code: str
    planet_sizes: basic.IntRangeSet
    atmospheres: basic.IntRangeSet
    hydro: basic.IntRangeSet
    population: basic.IntRangeSet
    government: basic.IntRangeSet
    law_level: basic.IntRangeSet
    tech_level: basic.IntRangeSet


@enum.unique
class StarportType(enum.StrEnum):
    """Enumeration for core starport type codes."""

    EXCELLENT = "A"
    GOOD = "B"
    ROUTINE = "C"
    POOR = "D"
    FRONTIER = "E"
    NONE = "X"


@dataclasses.dataclass(frozen=True)
class UWP:
    """Universal World Profile."""

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
        """Parse a UWP string."""
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
