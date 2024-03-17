# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

from typing import Any
from urllib import parse as urlparse

import pytest
from travdata.travellermap import apiurls

EXPECTED_DEFAULTS = {
    "type": ["TabDelimited"],
    "sscoords": ["1"],
}


@pytest.mark.parametrize(
    "expected_query,kwargs",
    [
        # Sector selection:
        (
            EXPECTED_DEFAULTS | {"sector": ["spin"]},
            {"sector": apiurls.SectorId("spin")},
        ),
        (
            EXPECTED_DEFAULTS | {"sx": ["5"], "sy": ["-3"]},
            {"sector": apiurls.SectorCoords(5, -3)},
        ),
        # Subsector selection:
        (
            EXPECTED_DEFAULTS | {"sector": ["spin"], "subsector": ["C"]},
            {"sector": apiurls.SectorId("spin"), "subsector": apiurls.SubSectorCode.C},
        ),
        (
            EXPECTED_DEFAULTS | {"sector": ["spin"], "subsector": ["C"]},
            {"sector": apiurls.SectorId("spin"), "subsector": apiurls.SubSectorCode.C},
        ),
        # Format:
        (
            EXPECTED_DEFAULTS | {"sector": ["spin"], "type": ["Legacy"]},
            {"sector": apiurls.SectorId("spin"), "response_type": apiurls.Type.LEGACY},
        ),
        (
            EXPECTED_DEFAULTS | {"sector": ["spin"], "type": ["SecondSurvey"]},
            {"sector": apiurls.SectorId("spin"), "response_type": apiurls.Type.SECOND_SURVEY},
        ),
        (
            EXPECTED_DEFAULTS | {"sector": ["spin"], "type": ["TabDelimited"]},
            {"sector": apiurls.SectorId("spin"), "response_type": apiurls.Type.TAB_DELIMITED},
        ),
        # Coords style:
        (
            EXPECTED_DEFAULTS | {"sector": ["spin"], "sscoords": ["1"]},
            {"sector": apiurls.SectorId("spin"), "coords_style": apiurls.CoordsStyle.SECTOR},
        ),
        (
            EXPECTED_DEFAULTS | {"sector": ["spin"], "sscoords": ["0"]},
            {"sector": apiurls.SectorId("spin"), "coords_style": apiurls.CoordsStyle.SUBSECTOR},
        ),
    ],
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
