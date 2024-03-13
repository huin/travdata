# -*- coding: utf-8 -*-
import pathlib
import textwrap
import unittest

import testfixtures  # type: ignore[import-untyped]
from travdata import config


class LoadConfigTest(unittest.TestCase):

    def test_load_tables(self) -> None:
        actual = config.load_config_from_str(
            textwrap.dedent(
                """
        !Group
        groups:
            grp-a: !Group
                tables:
                    foo: !Table
                        type: Foo
                        extraction: !TableExtraction
                            num_header_lines: 2
                            continuation_empty_column: 0
                    bar: !Table
                        type: Bar
                        extraction: !TableExtraction
                            num_header_lines: 1
                            continuation_empty_column: 3
                    defaults: !Table
                        type: Defaults
        """
            )
        )

        testfixtures.compare(
            actual=actual,
            expected=config.Group(
                directory=pathlib.Path("."),
                groups={
                    "grp-a": config.Group(
                        directory=pathlib.Path("./grp-a"),
                        tables={
                            "foo": config.Table(
                                file_stem=pathlib.Path("./grp-a/foo"),
                                type="Foo",
                                extraction=config.TableExtraction(
                                    num_header_lines=2,
                                    continuation_empty_column=0,
                                ),
                            ),
                            "bar": config.Table(
                                file_stem=pathlib.Path("./grp-a/bar"),
                                type="Bar",
                                extraction=config.TableExtraction(
                                    num_header_lines=1,
                                    continuation_empty_column=3,
                                ),
                            ),
                            "defaults": config.Table(
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
