# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import pathlib
import textwrap

import testfixtures  # type: ignore[import-untyped]
from travdata import config


def test_load_group_from_str() -> None:
    actual = config.load_group_from_str(
        textwrap.dedent(
            """
    !Group
    tags: [top]
    groups:
        grp-a: !Group
            tables:
                foo: !Table
                    tags: [type/foo]
                    extraction: !TableExtraction
                        row_folding:
                            - !StaticRowCounts {row_counts: [2]}
                            - !EmptyColumn {column_index: 0}
                bar: !Table
                    tags: [type/bar]
                    extraction: !TableExtraction
                        row_folding:
                            - !StaticRowCounts {row_counts: [1]}
                            - !EmptyColumn {column_index: 3}
                defaults: !Table {}
    """
        ),
        parent_tags={"outer"},
    )

    testfixtures.compare(
        actual=actual,
        expected=config.Group(
            cfg_dir=pathlib.Path("."),
            rel_dir=pathlib.Path("."),
            tags={"outer", "top"},
            groups={
                "grp-a": config.Group(
                    cfg_dir=pathlib.Path("."),
                    rel_dir=pathlib.Path("./grp-a"),
                    tags={"outer", "top"},
                    tables={
                        "foo": config.Table(
                            cfg_dir=pathlib.Path("."),
                            file_stem=pathlib.Path("./grp-a/foo"),
                            tags={"outer", "top", "type/foo"},
                            extraction=config.TableExtraction(
                                row_folding=[
                                    config.StaticRowCounts([2]),
                                    config.EmptyColumn(0),
                                ],
                            ),
                        ),
                        "bar": config.Table(
                            cfg_dir=pathlib.Path("."),
                            file_stem=pathlib.Path("./grp-a/bar"),
                            tags={"outer", "top", "type/bar"},
                            extraction=config.TableExtraction(
                                row_folding=[
                                    config.StaticRowCounts([1]),
                                    config.EmptyColumn(3),
                                ],
                            ),
                        ),
                        "defaults": config.Table(
                            cfg_dir=pathlib.Path("."),
                            file_stem=pathlib.Path("./grp-a/defaults"),
                            tags={"outer", "top"},
                            extraction=None,
                        ),
                    },
                ),
            },
        ),
    )
