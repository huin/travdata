# -*- coding: utf-8 -*-

import unittest
from typing import Any
from urllib import parse as urlparse

from travdata.travellermap import apiurls


class UWPDataTest(unittest.TestCase):

    def test(self) -> None:
        expected_defaults = dict(
            type=["TabDelimited"],
            sscoords=["1"],
        )
        cases: list[tuple[dict[str, list[str]], dict[str, Any]]] = [
            # Sector selection:
            (
                expected_defaults
                | dict(
                    sector=["spin"],
                ),
                dict(
                    sector=apiurls.SectorId("spin"),
                ),
            ),
            (
                expected_defaults
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
                expected_defaults
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
                expected_defaults
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
                expected_defaults
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
                expected_defaults
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
                expected_defaults
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
                expected_defaults
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
                expected_defaults
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
        for expected_query, kwargs in cases:
            with self.subTest(kwargs):
                actual_str = apiurls.uwp_data(**kwargs)
                actual = urlparse.urlparse(actual_str)

                self.assertEqual("https", actual.scheme)
                self.assertEqual("travellermap.com", actual.netloc)
                self.assertEqual("/api/sec", actual.path)
                self.assertFalse(actual.params)
                actual_query = urlparse.parse_qs(actual.query)
                self.assertEqual(expected_query, actual_query)


if __name__ == "__main__":
    unittest.main()
