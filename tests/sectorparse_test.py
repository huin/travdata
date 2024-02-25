# -*- coding: utf-8 -*-
import io
import unittest
from typing import Callable, TypeVar

import testfixtures
from travdata.travellermap import sectorparse
from travdata.travellermap.world import (UWP, BaseCode, StellarCode,
                                         SubsectorCode, SubSectorLoc,
                                         TradeCode, TravelCode, World,
                                         WorldExtensions, WorldLocation,
                                         WorldSocial, WorldSystem)

C = TypeVar("C")
T = TypeVar("T")
U = TypeVar("U")


# https://travellermap.com/data/Spinward%20Marches/A/tab
T5_TSV_EXAMPLE = """\
Sector\tSS\tHex\tName\tUWP\tBases\tRemarks\tZone\tPBG\tAllegiance\tStars\t{Ix}\t(Ex)\t[Cx]\tNobility\tW\tRU
Spin\tA\t0101\tZeycude\tC430698-9\t\tDe Na Ni Po\t\t613\tZhIN\tK9 V\t{ -1 }\t(C53-1)\t[6559]\t\t8\t-180
Spin\tA\t0102\tReno\tC4207B9-A\t\tDe He Na Po Pi Pz\tA\t603\tZhIN\tG8 V M1 V\t{ 1 }\t(C6A+2)\t[886B]\t\t12\t1440
Spin\tA\t0103\tErrere\tB563664-B\tKM\tNi Ri O:0304\t\t910\tZhIN\tM1 V M4 V\t{ 3 }\t(957+1)\t[4939]\t\t9\t315
"""


def typed_frozenset(
    fn: Callable[[T], U],
    *values: T,
) -> frozenset[U]:
    return frozenset(fn(v) for v in values)


def typed_tuple(
    fn: Callable[[T], U],
    *values: T,
) -> tuple[U, ...]:
    return tuple(fn(v) for v in values)


T5_TSV_EXAMPLE_CONTENT: list[World] = [
    World(
        name="Zeycude",
        ext=WorldExtensions(
            importance=-1,
            economic="C53-1",
            cultural="6559",
            resource_units=-180,
        ),
        location=WorldLocation(
            sector_abbv="Spin",
            subsector=SubsectorCode("A"),
            subsector_hex=SubSectorLoc(1, 1),
        ),
        social=WorldSocial(
            allegiance="ZhIN",
            bases=frozenset(),
            nobility="",
            pop_multiplier=6,
            trade_codes=typed_frozenset(TradeCode, "De", "Na", "Ni", "Po"),
        ),
        system=WorldSystem(
            num_belts=1,
            num_gas_giants=3,
            num_worlds=8,
            stellar=typed_tuple(StellarCode, "K9", "V"),
        ),
        travel_code=TravelCode.NONE,
        uwp=UWP.parse("C430698-9"),
    ),
    World(
        name="Reno",
        ext=WorldExtensions(
            importance=1,
            economic="C6A+2",
            cultural="886B",
            resource_units=1440,
        ),
        location=WorldLocation(
            sector_abbv="Spin",
            subsector=SubsectorCode("A"),
            subsector_hex=SubSectorLoc(1, 2),
        ),
        social=WorldSocial(
            allegiance="ZhIN",
            bases=frozenset(),
            nobility="",
            pop_multiplier=6,
            trade_codes=typed_frozenset(TradeCode, "De", "He", "Na", "Po", "Pi", "Pz"),
        ),
        system=WorldSystem(
            num_belts=0,
            num_gas_giants=3,
            num_worlds=12,
            stellar=typed_tuple(StellarCode, "G8", "V", "M1", "V"),
        ),
        travel_code=TravelCode.AMBER,
        uwp=UWP.parse("C4207B9-A"),
    ),
    World(
        name="Errere",
        ext=WorldExtensions(
            importance=3,
            economic="957+1",
            cultural="4939",
            resource_units=315,
        ),
        location=WorldLocation(
            sector_abbv="Spin", subsector=SubsectorCode("A"), subsector_hex=SubSectorLoc(1, 3)
        ),
        social=WorldSocial(
            allegiance="ZhIN",
            bases=typed_frozenset(BaseCode, "K", "M"),
            nobility="",
            pop_multiplier=9,
            trade_codes=typed_frozenset(TradeCode, "Ni", "Ri", "O:0304"),
        ),
        system=WorldSystem(
            num_belts=1,
            num_gas_giants=0,
            num_worlds=9,
            stellar=typed_tuple(StellarCode, "M1", "V", "M4", "V"),
        ),
        travel_code=TravelCode.NONE,
        uwp=UWP.parse("B563664-B"),
    ),
]

# https://travellermap.com/data/Spinward%20Marches/A
T5_COL_EXAMPLE = """\
Hex  Name                 UWP       Remarks              {Ix}   (Ex)    [Cx]   N B  Z PBG W  A    Stellar  
---- -------------------- --------- -------------------- ------ ------- ------ - -- - --- -- ---- ---------
0101 Zeycude              C430698-9 De Na Ni Po          { -1 } (C53-1) [6559] - -  - 613 8  ZhIN K9 V     
0102 Reno                 C4207B9-A De He Na Po Pi Pz    { 1 }  (C6A+2) [886B] - -  A 603 12 ZhIN G8 V M1 V
0103 Errere               B563664-B Ni Ri O:0304         { 3 }  (957+1) [4939] - KM - 910 9  ZhIN M1 V M4 V
"""


class T5TSVTest(unittest.TestCase):

    def test_parse_example(self) -> None:
        fp = io.StringIO(T5_TSV_EXAMPLE)
        actual = list(sectorparse.t5_tsv(fp))
        testfixtures.compare(actual=actual, expected=T5_TSV_EXAMPLE_CONTENT)


if __name__ == "__main__":
    unittest.main()
