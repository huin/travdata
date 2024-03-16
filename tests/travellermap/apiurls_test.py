# -*- coding: utf-8 -*-

from typing import Any
from urllib import parse as urlparse

import pytest
from travdata.travellermap import apiurls

EXPECTED_DEFAULTS = dict(
    type=["TabDelimited"],
    sscoords=["1"],
)


@pytest.mark.parametrize(
    "expected_query,kwargs",
    [
        # Sector selection:
        (
            EXPECTED_DEFAULTS
            | dict(
                sector=["spin"],
            ),
            dict(
                sector=apiurls.SectorId("spin"),
            ),
        ),
        (
            EXPECTED_DEFAULTS
            | dict(
                sx=["5"],
                sy=["-3"],
            ),
            dict(
                sector=apiurls.SectorCoords(5, -3),
            ),
        ),
        # Subsector selection:
        (
            EXPECTED_DEFAULTS
            | dict(
                sector=["spin"],
                subsector=["C"],
            ),
            dict(
                sector=apiurls.SectorId("spin"),
                subsector=apiurls.SubSectorCode.C,
            ),
        ),
        (
            EXPECTED_DEFAULTS
            | dict(
                sector=["spin"],
                subsector=["C"],
            ),
            dict(
                sector=apiurls.SectorId("spin"),
                subsector=apiurls.SubSectorCode.C,
            ),
        ),
        # Format:
        (
            EXPECTED_DEFAULTS
            | dict(
                sector=["spin"],
                type=["Legacy"],
            ),
            dict(
                sector=apiurls.SectorId("spin"),
                format=apiurls.Format.LEGACY,
            ),
        ),
        (
            EXPECTED_DEFAULTS
            | dict(
                sector=["spin"],
                type=["SecondSurvey"],
            ),
            dict(
                sector=apiurls.SectorId("spin"),
                format=apiurls.Format.SECOND_SURVEY,
            ),
        ),
        (
            EXPECTED_DEFAULTS
            | dict(
                sector=["spin"],
                type=["TabDelimited"],
            ),
            dict(
                sector=apiurls.SectorId("spin"),
                format=apiurls.Format.TAB_DELIMITED,
            ),
        ),
        # Coords style:
        (
            EXPECTED_DEFAULTS
            | dict(
                sector=["spin"],
                sscoords=["1"],
            ),
            dict(
                sector=apiurls.SectorId("spin"),
                coords_style=apiurls.CoordsStyle.SECTOR,
            ),
        ),
        (
            EXPECTED_DEFAULTS
            | dict(
                sector=["spin"],
                sscoords=["0"],
            ),
            dict(
                sector=apiurls.SectorId("spin"),
                coords_style=apiurls.CoordsStyle.SUBSECTOR,
            ),
        ),
    ]
)
def test_uwp_data(expected_query: dict[str, list[str]], kwargs: dict[str, Any]) -> None:
    actual_str = apiurls.uwp_data(**kwargs)
    actual = urlparse.urlparse(actual_str)

    assert "https" == actual.scheme
    assert "travellermap.com" == actual.netloc
    assert "/api/sec" == actual.path
    assert not actual.params
    assert not actual.username
    assert not actual.password
    assert not actual.fragment
    actual_query = urlparse.parse_qs(actual.query)
    assert expected_query == actual_query
