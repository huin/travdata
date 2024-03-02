# -*- coding: utf-8 -*-
import textwrap
import unittest

import testfixtures
from travdata import pdfextract


class LoadConfigTest(unittest.TestCase):

    def test_load_tables(self) -> None:
        actual = pdfextract.load_config_from_str(
            textwrap.dedent(
                """
        !Config
        tables:
        -   !Table
            name: foo
            num_header_lines: 2
            continuation_empty_column: 0
        -   !Table
            name: bar
            num_header_lines: 1
            continuation_empty_column: 3
        -   !Table
            name: defaults
        """
            )
        )

        testfixtures.compare(
            actual=actual,
            expected=pdfextract.Config(
                tables=[
                    pdfextract.Table(
                        name="foo",
                        num_header_lines=2,
                        continuation_empty_column=0,
                    ),
                    pdfextract.Table(
                        name="bar",
                        num_header_lines=1,
                        continuation_empty_column=3,
                    ),
                    pdfextract.Table(
                        name="defaults",
                    ),
                ],
            ),
        )


if __name__ == "__main__":
    unittest.main()
