# -*- coding: utf-8 -*-
"""Data types relating to Travellermap worlds."""

import dataclasses
import enum
from dataclasses import field
from typing import TYPE_CHECKING, Any, NewType, Optional, TypeVar

from travdata.datatypes.core import worldcreation

if TYPE_CHECKING:
    from _typeshed import DataclassInstance
else:
    DataclassInstance = object

# Many types inherently have a lot of attributes, this reflects the data from
# the book.
# pylint: disable=too-many-instance-attributes

# https://wiki.travellerrpg.com/Base_Code
BaseCode = NewType("BaseCode", str)
StellarCode = NewType("StellarCode", str)
SubsectorCode = NewType("SubsectorCode", str)
TradeCode = NewType("TradeCode", str)


@dataclasses.dataclass(frozen=True)
class SubSectorLoc:
    """Numeric two dimensional coordinates of a world within a sector."""

    x: int
    y: int

    @classmethod
    def parse(cls, s: str) -> "SubSectorLoc":
        """Parses from a four-decimal digit value."""
        if len(s) != 4 or not s.isdigit():
            raise ValueError(s)
        return SubSectorLoc(
            x=int(s[0:2]),
            y=int(s[2:4]),
        )

    def __str__(self) -> str:
        return f"{self.x:02d}{self.y:02d}"


@enum.unique
class TravelCode(enum.StrEnum):
    """Advisory code about travel to a world."""

    NONE = ""
    AMBER = "Amber"
    RED = "Red"


_DC = TypeVar("_DC", bound=DataclassInstance)


_RECURSE_KEY = "recurse"
_MD_RECURSE_MERGE = {_RECURSE_KEY: True}


def _merged(
    t: type[_DC],
    a: _DC,
    b: _DC,
) -> _DC:
    kw: dict[str, Any] = {}
    for f in dataclasses.fields(t):
        av = getattr(a, f.name)
        bv = getattr(b, f.name)
        match av, bv:
            case None, None:
                continue
            case _, None:
                kw[f.name] = av
                continue
            case None, _:
                kw[f.name] = bv
                continue
            case _:
                # Both set; fall though to following logic.
                pass
        if f.metadata.get(_RECURSE_KEY):
            kw[f.name] = av.merged(bv)
            continue
        if av == bv:
            kw[f.name] = av
        raise ValueError(f"conflicting values for field {f.name}: {av} versus {bv}")
    return t(**kw)


@dataclasses.dataclass(kw_only=True, frozen=True)
class World:
    """Data about a world and its solar system."""

    comments: Optional[str] = None
    ext: Optional["WorldExtensions"] = field(default=None, metadata=_MD_RECURSE_MERGE)
    location: Optional["WorldLocation"] = field(default=None, metadata=_MD_RECURSE_MERGE)
    name: Optional[str] = None
    travel_code: Optional[TravelCode] = None
    social: Optional["WorldSocial"] = field(default=None, metadata=_MD_RECURSE_MERGE)
    system: Optional["WorldSystem"] = field(default=None, metadata=_MD_RECURSE_MERGE)
    uwp: Optional[worldcreation.UWP] = None

    def merged(self, b: "World") -> "World":
        """Creates a merged copy of self and b."""
        return _merged(World, self, b)


@dataclasses.dataclass(kw_only=True, frozen=True)
class WorldLocation:
    """World location within the universe."""

    sector: Optional[str] = None
    sector_abbv: Optional[str] = None
    subsector: Optional[SubsectorCode] = None
    subsector_hex: Optional[SubSectorLoc] = None

    def merge(self, b: "WorldLocation") -> "WorldLocation":
        """Creates a merged copy of self and b."""
        return _merged(WorldLocation, self, b)


@dataclasses.dataclass(kw_only=True, frozen=True)
class WorldExtensions:
    """Various world extensions from Travellermap."""

    cultural: Optional[str] = None
    economic: Optional[str] = None
    importance: Optional[int] = None
    resource_units: Optional[int] = None

    def merge(self, b: "WorldExtensions") -> "WorldExtensions":
        """Creates a merged copy of self and b."""
        return _merged(WorldExtensions, self, b)


@dataclasses.dataclass(kw_only=True, frozen=True)
class WorldSocial:
    """Data about the socioeconomic aspects of a world."""

    allegiance: Optional[str] = None
    bases: Optional[frozenset[BaseCode]] = None
    nobility: Optional[str] = None
    pop_multiplier: Optional[int] = None
    trade_codes: Optional[frozenset[TradeCode]] = None

    def merge(self, b: "WorldSocial") -> "WorldSocial":
        """Creates a merged copy of self and b."""
        return _merged(WorldSocial, self, b)


@dataclasses.dataclass(kw_only=True, frozen=True)
class WorldSystem:
    """Data about the solar system."""

    num_belts: Optional[int] = None
    num_gas_giants: Optional[int] = None
    num_stars: Optional[int] = None
    num_worlds: Optional[int] = None
    stellar: Optional[tuple[StellarCode, ...]] = None

    def merge(self, b: "WorldSystem") -> "WorldSystem":
        """Creates a merged copy of self and b."""
        return _merged(WorldSystem, self, b)
