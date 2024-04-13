# -*- coding: utf-8 -*-
"""Defines the configuration around data extraction and other metadata.

Values of these types are read from two types of file:

* ``config.yaml`` top-level configuration for multiple books.
* ``book.yaml`` relating to a single input PDF.

See development.adoc for more information in how this is used.
"""

from __future__ import annotations

import abc
import argparse
import dataclasses
import pathlib
import sys
import textwrap
from typing import Any, ClassVar, Iterator, Optional, Self

from ruamel import yaml
from travdata import yamlutil

_YAML = yaml.YAML()
# Retain the original ordering in mappings.
_YAML.representer.sort_base_mapping_type_on_output = False

__executable_environment__ = "development"


_INT_METADATA = {yamlutil.TO_YAML: int, yamlutil.FROM_YAML: int}
_SET_METADATA = {yamlutil.TO_YAML: sorted, yamlutil.FROM_YAML: set}

TABULA_TEMPLATE_SUFFIX = ".tabula-template.json"


class TableTransform(abc.ABC):
    """Marker base class for configuration of table transformations."""


@dataclasses.dataclass
@_YAML.register_class
class ExpandColumnOnRegex(TableTransform, yamlutil.YamlMappingMixin):
    """Splits a column by the matches of a regex."""

    yaml_tag: ClassVar = "!ExpandColumnOnRegex"
    column: int
    pattern: str
    # When `pattern` matches (using `Pattern.fullmatch`), `on_match` produces
    # the resulting cells, using groups from the match to the pattern. Each
    # string is expanded using `Match.expand`, see
    # https://docs.python.org/3/library/re.html#match-objects.
    on_match: list[str]
    # When `pattern` does not match, default produces cells as if matching on a
    # regex ".*" when `pattern` does not match. Similarly, each string is
    # expanded using `Match.expand` (using \g<0> makes sense here to extract the
    # entire original text into a cell).
    default: list[str]

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(
            column=0,
            pattern="",
            on_match=[],
            default=[],
        )


@dataclasses.dataclass
@_YAML.register_class
class PrependRow(TableTransform, yamlutil.YamlSequenceMixin):
    """Appends given literal row values to the start of a table."""

    yaml_tag: ClassVar = "!PrependRow"
    row: list[str]

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(row=[])


class RowGrouper(abc.ABC):
    """Abstract base marker for configuring row grouping."""


@dataclasses.dataclass
@_YAML.register_class
class StaticRowCounts(RowGrouper, yamlutil.YamlSequenceMixin):
    """Specifies explicit input row counts for output grouped rows."""

    yaml_tag: ClassVar = "!StaticRowCounts"
    row_counts: list[int]

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(row_counts=[])


@dataclasses.dataclass
@_YAML.register_class
class EmptyColumn(RowGrouper, yamlutil.YamlScalarMixin):
    """Specifies to group rows by when a given column is empty."""

    yaml_tag: ClassVar = "!EmptyColumn"
    column_index: int = dataclasses.field(metadata={"from_yaml": int})

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(column_index=0)


@dataclasses.dataclass
@_YAML.register_class
class FoldRows(TableTransform, yamlutil.YamlSequenceMixin):
    """Folds rows, according to the given sequence of groupings."""

    yaml_tag: ClassVar = "!FoldRows"
    group_by: list[RowGrouper] = dataclasses.field(default_factory=list)


@dataclasses.dataclass
@_YAML.register_class
class JoinColumns(TableTransform, yamlutil.YamlMappingMixin):
    """Joins a range of columns."""

    yaml_tag: ClassVar = "!JoinColumns"
    from_: Optional[int] = dataclasses.field(default=None, metadata={yamlutil.YAML_NAME: "from"})
    to: Optional[int] = None
    delim: str = ""


@dataclasses.dataclass
@_YAML.register_class
class WrapRowEveryN(TableTransform, yamlutil.YamlScalarMixin):
    """Wraps a row every N columns."""

    yaml_tag: ClassVar = "!WrapRowEveryN"
    columns: int = dataclasses.field(metadata=_INT_METADATA)

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(columns=0)


@dataclasses.dataclass
@_YAML.register_class
class TableExtraction(yamlutil.YamlSequenceMixin):
    """Configures the specifics of extracting the CSV from the PDF."""

    yaml_tag: ClassVar = "!TableExtraction"
    transforms: list[TableTransform] = dataclasses.field(default_factory=list)


@dataclasses.dataclass
class Table:
    """Defines metadata and extraction configuration relating to a single table.

    The "path" of group names and the table name form the path for both the
    ``.tabula-template.json`` file within the configuration directory and the
    output ``.csv`` file in the output directory.
    """

    cfg_dir: pathlib.Path
    file_stem: pathlib.Path
    tags: set[str] = dataclasses.field(default_factory=set)
    extraction: Optional[TableExtraction] = dataclasses.field(default_factory=TableExtraction)

    @property
    def tabula_template_path(self) -> pathlib.Path:
        """Path to the Tabula template, if it exists."""
        return self.cfg_dir / self.file_stem.with_suffix(TABULA_TEMPLATE_SUFFIX)


@dataclasses.dataclass
class Group:
    """Group of items to extract from the PDF.

    A top-level group within a book is often aligned with a book chapter.

    The table items have Tabula templates in ``.directory``.
    """

    cfg_dir: pathlib.Path
    rel_dir: pathlib.Path
    tags: set[str] = dataclasses.field(default_factory=set)
    tables: dict[str, Table] = dataclasses.field(default_factory=dict)
    groups: dict[str, "Group"] = dataclasses.field(default_factory=dict)

    def all_tables(self) -> Iterator[Table]:
        """Iterates over all tables in this group and its child groups.

        :yield: Descendent tables.
        """
        yield from self.tables.values()
        for group in self.groups.values():
            yield from group.all_tables()


@dataclasses.dataclass
class Book:
    """Top level information about a book."""

    cfg_dir: pathlib.Path
    id_: str
    name: str
    default_filename: str
    tags: set[str] = dataclasses.field(default_factory=set)
    _group: Optional[Group] = None

    def load_group(self) -> Group:
        """Loads and returns the top-level group in the `Book`."""
        if self._group is None:
            rel_book_dir = pathlib.Path(self.id_)
            yaml_group = _YAML.load(self.cfg_dir / rel_book_dir / "book.yaml")
            self._group = _prepare_group(
                yaml_group=yaml_group,
                cfg_dir=self.cfg_dir,
                rel_book_dir=rel_book_dir,
                parent_tags=self.tags,
            )
        return self._group


@dataclasses.dataclass
class Config:
    """Top-level configuration."""

    cfg_dir: pathlib.Path
    books: dict[str, Book] = dataclasses.field(default_factory=dict)


@dataclasses.dataclass
@_YAML.register_class
class _YamlTable(yamlutil.YamlMappingMixin):
    yaml_tag: ClassVar = "!Table"
    tags: set[str] = dataclasses.field(default_factory=set, metadata=_SET_METADATA)
    extraction: Optional[TableExtraction] = None

    def prepare(
        self,
        cfg_dir: pathlib.Path,
        name: str,
        directory: pathlib.Path,
        parent_tags: set[str],
    ) -> Table:
        """Creates a ``Table`` from self.

        :param cfg_dir: Path to the directory of the top-level ``Config``.
        :param name: Name of the table within its ``Group.groups``.
        :param directory: Path to the directory of the parent ``Group``,
        relative to the top-level config directory.
        :param parent_tags: Tags to inherit from parent ``Group``.
        :return: Prepared ``Table``.
        """
        tags = self.tags | parent_tags
        return Table(
            cfg_dir=cfg_dir,
            file_stem=directory / name,
            tags=tags,
            extraction=self.extraction,
        )


@dataclasses.dataclass
@_YAML.register_class
class _YamlGroup(yamlutil.YamlMappingMixin):
    yaml_tag: ClassVar = "!Group"
    tags: set[str] = dataclasses.field(default_factory=set, metadata=_SET_METADATA)
    templates: Optional[list[TableExtraction]] = None
    groups: dict[str, "_YamlGroup"] = dataclasses.field(default_factory=dict)
    tables: dict[str, _YamlTable] = dataclasses.field(default_factory=dict)

    def prepare(
        self,
        cfg_dir: pathlib.Path,
        rel_group_dir: pathlib.Path,
        parent_tags: set[str],
    ) -> Group:
        """Creates a ``Group`` from self.

        :param cfg_dir: Path to the directory of the top-level ``Config``.
        :param rel_group_dir: Path to the directory of this group's directory,
        relative to the top-level config directory.
        :param parent_tags: Tags to inherit from parent ``Group``.
        :return: Prepared ``Group``.
        """
        tags = self.tags | parent_tags
        return Group(
            cfg_dir=cfg_dir,
            rel_dir=rel_group_dir,
            tags=tags,
            tables={
                name: table.prepare(cfg_dir, name, rel_group_dir, parent_tags=tags)
                for name, table in self.tables.items()
            },
            groups={
                name: group.prepare(cfg_dir, rel_group_dir / name, parent_tags=tags)
                for name, group in self.groups.items()
            },
            # templates not included, as it is only for use in anchoring and
            # aliasing by the YAML file author at the time of YAML parsing.
        )


@dataclasses.dataclass
@_YAML.register_class
class _YamlBook(yamlutil.YamlMappingMixin):
    yaml_tag: ClassVar = "!Book"
    name: str
    default_filename: str
    tags: set[str] = dataclasses.field(default_factory=set, metadata=_SET_METADATA)

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(name="", default_filename="")

    def prepare(
        self,
        cfg_dir: pathlib.Path,
        book_id: str,
    ) -> Book:
        """Creates a ``Book`` from self.

        :param cfg_dir: Path to the directory of the top-level ``Config``.
        :param book_id: ID of the book within the parent _YamlConfig.
        :return: Prepared ``Book``.
        """
        tags = self.tags | {f"book/{self.name}"}
        return Book(
            cfg_dir=cfg_dir,
            id_=book_id,
            name=self.name,
            default_filename=self.default_filename,
            tags=tags,
        )


@dataclasses.dataclass
@_YAML.register_class
class _YamlConfig(yamlutil.YamlMappingMixin):
    yaml_tag: ClassVar = "!Config"
    books: dict[str, _YamlBook]

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(books={})

    def prepare(self, cfg_dir: pathlib.Path) -> Config:
        """Creates a ``Group`` from self.

        :param cfg_dir: Path to the directory of the top-level ``Config``.
        :return: Prepared ``Config``.
        """
        books: dict[str, Book] = {}
        for book_id, yaml_book in self.books.items():
            books[book_id] = yaml_book.prepare(
                cfg_dir=cfg_dir,
                book_id=book_id,
            )
        return Config(
            cfg_dir=cfg_dir,
            books=books,
        )


def _prepare_group(
    yaml_group: Any | _YamlGroup,
    cfg_dir: pathlib.Path,
    rel_book_dir: pathlib.Path,
    parent_tags: set[str],
) -> Group:
    if not isinstance(yaml_group, _YamlGroup):
        raise TypeError(yaml_group)
    return yaml_group.prepare(
        cfg_dir=cfg_dir,
        rel_group_dir=rel_book_dir,
        parent_tags=parent_tags,
    )


def load_group_from_str(yaml_str: str, parent_tags: set[str]) -> Group:
    """Loads the configuration from the given string containing YAML."""
    cfg = _YAML.load(yaml_str)
    return _prepare_group(
        yaml_group=cfg,
        cfg_dir=pathlib.Path("."),
        rel_book_dir=pathlib.Path("."),
        parent_tags=parent_tags,
    )


def parse_yaml_for_testing(yaml_str: str) -> Any:
    """Parses the given YAML, without preparing it.

    This is only exposed for testing purposes.

    :param yaml_str: YAML to parse.
    :return: Parsed objects.
    """
    return _YAML.load(yaml_str)


def _prepare_config(
    cfg: Any | _YamlConfig,
    cfg_dir: pathlib.Path,
) -> Config:
    if not isinstance(cfg, _YamlConfig):
        raise TypeError(cfg)
    return cfg.prepare(cfg_dir)


def load_config(cfg_dir: pathlib.Path) -> Config:
    """Loads the configuration from the directory."""
    cfg = _YAML.load(cfg_dir / "config.yaml")
    return _prepare_config(
        cfg=cfg,
        cfg_dir=cfg_dir,
    )


def add_config_flag(argparser: argparse.ArgumentParser) -> None:
    """Adds the flag required to call ``load_config_from_flag`` on the parsed args."""

    default_config_dir = get_default_config_path()

    argparser.add_argument(
        "--config-dir",
        "-c",
        help=textwrap.dedent(
            """
            Path to the configuration directory. This must contain a config.yaml
            file, and its required Tabula templates. Some configurations for
            this should be included with this program's distribution.
            """
        ),
        type=pathlib.Path,
        metavar="CONFIG_DIR",
        required=default_config_dir is None,
        default=default_config_dir,
    )


def get_default_config_path() -> Optional[pathlib.Path]:
    """Returns the default path to the config directory.

    :raises RuntimeError: If the environment is not recognised.
    :return: Default path to the config directory, if known.
    """
    match __executable_environment__:
        case "development":
            install_dir = _data_dir_for_development()
        case "pyinstaller":
            install_dir = _data_dir_for_pyinstaller()
        case unknown_env:
            raise RuntimeError(f"unknown executable environment {unknown_env!r}")

    config_dir = install_dir / "config"
    config_file = config_dir / "config.yaml"
    if not config_file.is_file():
        return None
    return config_dir


def _data_dir_for_development() -> pathlib.Path:
    return pathlib.Path.cwd()


def _data_dir_for_pyinstaller() -> pathlib.Path:
    return pathlib.Path(getattr(sys, "_MEIPASS"))


def load_config_from_flag(args: argparse.Namespace) -> Config:
    """Returns a ``Config`` specified by the parsed arguments.

    :param args: Parsed arguments. This must have been generated from a parser
    that included the argument added by ``add_config_flag``.
    :return: Loaded configuration.
    """
    return load_config(args.config_dir)
