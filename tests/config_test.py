# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring

import pathlib
import textwrap
from typing import Any

import testfixtures  # type: ignore[import-untyped]
import pytest
from travdata import config, filesio
from travdata.config import cfgerror, cfgextract


def test_load_group_from_str() -> None:
    book_name = "book-name"
    book_yaml_path = pathlib.PurePath(book_name) / "book.yaml"
    book_yaml = """
!Group
tags: [top]
groups:
    grp-a: !Group
        tables:
            foo: !Table
                tags: [type/foo]
                extraction: !TableExtraction
                    - !WrapRowEveryN 2
            bar: !Table
                tags: [type/bar]
                extraction: !TableExtraction
                    - !FoldRows
                        - !StaticRowCounts [1]
                        - !EmptyColumn 3
            defaults: !Table {}
"""
    files = {book_yaml_path: book_yaml}
    with filesio.MemReadWriter.new_reader(files) as cfg_reader:
        actual = config.load_book(cfg_reader, book_name, {"outer"})

    testfixtures.compare(
        actual=actual,
        expected=config.Group(
            rel_dir=pathlib.Path(book_name),
            tags={"outer", "top"},
            groups={
                "grp-a": config.Group(
                    rel_dir=pathlib.Path(f"{book_name}/grp-a"),
                    tags={"outer", "top"},
                    tables={
                        "foo": config.Table(
                            file_stem=pathlib.Path(f"{book_name}/grp-a/foo"),
                            tags={"outer", "top", "type/foo"},
                            extraction=cfgextract.TableExtraction(
                                transforms=[
                                    cfgextract.WrapRowEveryN(2),
                                ],
                            ),
                        ),
                        "bar": config.Table(
                            file_stem=pathlib.Path(f"{book_name}/grp-a/bar"),
                            tags={"outer", "top", "type/bar"},
                            extraction=cfgextract.TableExtraction(
                                transforms=[
                                    cfgextract.FoldRows(
                                        [
                                            cfgextract.StaticRowCounts([1]),
                                            cfgextract.EmptyColumn(3),
                                        ],
                                    ),
                                ],
                            ),
                        ),
                        "defaults": config.Table(
                            file_stem=pathlib.Path(f"{book_name}/grp-a/defaults"),
                            tags={"outer", "top"},
                            extraction=None,
                        ),
                    },
                ),
            },
        ),
    )


@pytest.mark.parametrize(
    "name,yaml",
    [
        (
            "ExpectMappingGotString",
            """
            !Table "not a mapping"
            """,
        ),
        (
            "ExpectSequenceGotString",
            """
            !TableExtraction "not a sequence"
            """,
        ),
        (
            "ExpectScalarGotMapping",
            """
            !EmptyColumn {}
            """,
        ),
    ],
)
def test_config_parse_errors(
    name: str,
    yaml: str,
) -> None:
    print(name)
    with pytest.raises(cfgerror.ConfigurationError):
        config.parse_yaml_for_testing(textwrap.dedent(yaml))


@pytest.mark.parametrize(
    "name,yaml,expected",
    [
        (
            "JoinColumns",
            """
            !JoinColumns
              from: 1
              to: 2
              delim: " "
            """,
            cfgextract.JoinColumns(
                from_=1,
                to=2,
                delim=" ",
            ),
        ),
        (
            "ExpandColumnOnRegex",
            """
            !ExpandColumnOnRegex
              column: 1    
              pattern: '([^:]+): (.+)'
              on_match: ['\\1', '\\2']
              default: ['', '\\g<0>']
            """,
            cfgextract.ExpandColumnOnRegex(
                column=1,
                pattern=r"([^:]+): (.+)",
                on_match=[r"\1", r"\2"],
                default=[r"", r"\g<0>"],
            ),
        ),
        (
            "WrapRowEveryN",
            """
            !WrapRowEveryN 2
            """,
            cfgextract.WrapRowEveryN(2),
        ),
        (
            "FoldRows",
            """
            !FoldRows
              - !StaticRowCounts [2]
              - !EmptyColumn 0
              - !AllRows {}
            """,
            cfgextract.FoldRows(
                [
                    cfgextract.StaticRowCounts([2]),
                    cfgextract.EmptyColumn(0),
                    cfgextract.AllRows(),
                ]
            ),
        ),
        (
            "PrependRow",
            """
            !PrependRow [foo, bar]
            """,
            cfgextract.PrependRow(["foo", "bar"]),
        ),
        (
            "Transpose",
            """
            !Transpose {}
            """,
            cfgextract.Transpose(),
        ),
    ],
)
def test_parse(name: str, yaml: str, expected: Any) -> None:
    print(name)
    actual = config.parse_yaml_for_testing(yaml)
    testfixtures.compare(expected=expected, actual=actual)
