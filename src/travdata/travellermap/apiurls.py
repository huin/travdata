# -*- coding: utf-8 -*-
"""Functions to generate selected URLs for the Travellermap API."""

import abc
import dataclasses
import enum
from typing import Optional
from urllib import parse as urlparse

_API_URL = "https://travellermap.com/api/"
_SEC_API_URL = urlparse.urljoin(_API_URL, "sec")


class Type(enum.StrEnum):
    """Response format types for UWP data from Travellermap."""

    LEGACY = "Legacy"
    SECOND_SURVEY = "SecondSurvey"
    TAB_DELIMITED = "TabDelimited"


class SectorSelector(abc.ABC):
    """Abstract base class for sector selection."""

    @abc.abstractmethod
    def update_query(self, query: dict[str, str]) -> None:
        """Updates query values from self."""
        raise NotImplementedError


@dataclasses.dataclass
@SectorSelector.register
class SectorId:
    """Identifies a sector by its identifier.

    :attr id: The ID of the sector, e.g. "spin".
    """

    id: str

    def update_query(self, query: dict[str, str]) -> None:
        """Updates query values from self."""
        query["sector"] = self.id


@dataclasses.dataclass
class SectorCoords:
    """Identifies a sector by its location relative to Core sector.

    :attr sx: X coordinate.
    :attr sy: Y coordinate.
    """

    sx: int
    sy: int

    def update_query(self, query: dict[str, str]) -> None:
        """Updates query values from self."""
        query["sx"] = str(self.sx)
        query["sy"] = str(self.sy)


class SubsectorSelector(abc.ABC):
    """Selects a subsector."""

    @abc.abstractmethod
    def update_query(self, query: dict[str, str]) -> None:
        """Updates query values from self."""
        raise NotImplementedError


@SubsectorSelector.register
class SubSectorCode(enum.StrEnum):
    """Selects a subset of a sector by subsector code."""

    A = "A"
    B = "B"
    C = "C"
    D = "D"
    E = "E"
    F = "F"
    G = "G"
    H = "H"
    I = "I"
    J = "J"
    K = "K"
    L = "L"
    M = "M"
    N = "N"
    O = "O"
    P = "P"

    def update_query(self, query: dict[str, str]) -> None:
        """Updates query values from self."""
        query["subsector"] = str(self)


@SubsectorSelector.register
class SectorQuadrant(enum.StrEnum):
    """Selects a subset of a sector by quadrant."""

    ALPHA = "Alpha"
    BETA = "Beta"
    GAMMA = "Gamma"
    DELTA = "Delta"

    def update_query(self, query: dict[str, str]) -> None:
        """Updates query values from self."""
        query["quadrant"] = str(self)
        query["quadrant"] = str(self)


class CoordsStyle(enum.Enum):
    """Indicate how to return world coordinates."""

    SUBSECTOR = 0  # Subsector style: 0101-0810.
    SECTOR = 1  # Sector style: 0101-3240.


def uwp_data(
    *,
    sector: SectorSelector,
    subsector: Optional[SubsectorSelector] = None,
    response_type: Type = Type.TAB_DELIMITED,
    coords_style: CoordsStyle = CoordsStyle.SECTOR,
) -> str:
    """Returns a URL to request UWP and other data from travellermap.com.

    :param sector: Sector identifier (e.g. "spin").
    :param subsector: If set, requests only that portion of the
    subsector, defaults to None (return entire subsector).
    :param format: Format of the data to return, defaults to
    Format.TAB_DELIMITED.
    :param coords_style: World location format to use, defaults to
    CoordsStyle.SECTOR.
    :return: URL to request data from travellermap.com.
    """
    query: dict[str, str] = {
        "type": str(response_type),
        "sscoords": str(coords_style.value),
    }
    sector.update_query(query)
    if subsector is not None:
        subsector.update_query(query)
    encoded_query = urlparse.urlencode(query)
    return f"{_SEC_API_URL}?{encoded_query}"
