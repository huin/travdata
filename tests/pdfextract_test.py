# -*- coding: utf-8 -*-
import pathlib
import textwrap
import unittest

import testfixtures  # type: ignore[import-untyped]
from travdata import pdfextract


class LoadConfigTest(unittest.TestCase):

    def test_load_tables(self) -> None:
        actual = pdfextract.load_config_from_str(
            textwrap.dedent(
                """
        !Group
        groups:
            grp-a: !Group
                tables:
                    foo: !Table
                        type: Foo
                        num_header_lines: 2
                        continuation_empty_column: 0
                    bar: !Table
                        type: Bar
                        num_header_lines: 1
                        continuation_empty_column: 3
                    defaults: !Table
                        type: Defaults
        """
            )
        )

        testfixtures.compare(
            actual=actual,
            expected=pdfextract.Group(
                directory=pathlib.Path("."),
                groups={
                    "grp-a": pdfextract.Group(
                        directory=pathlib.Path("./grp-a"),
                        tables={
                            "foo": pdfextract.Table(
                                file_stem=pathlib.Path("./grp-a/foo"),
                                type="Foo",
                                num_header_lines=2,
                                continuation_empty_column=0,
                            ),
                            "bar": pdfextract.Table(
                                file_stem=pathlib.Path("./grp-a/bar"),
                                type="Bar",
                                num_header_lines=1,
                                continuation_empty_column=3,
                            ),
                            "defaults": pdfextract.Table(
                                file_stem=pathlib.Path("./grp-a/defaults"),
                                type="Defaults",
                            ),
                        },
                    ),
                },
            ),
        )


if __name__ == "__main__":
    unittest.main()
