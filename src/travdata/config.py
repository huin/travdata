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
from typing import Any, ClassVar, Iterator, Optional, cast, TYPE_CHECKING

from ruamel import yaml
from travdata import dataclassutil

if TYPE_CHECKING:
    from _typeshed import DataclassInstance

_YAML = yaml.YAML(typ="safe")
# Retain the original ordering in mappings.
_YAML.representer.sort_base_mapping_type_on_output = False

__executable_environment__ = "development"


_SET_METADATA = {"to_yaml": sorted, "from_yaml": set}


class UserError(Exception):
    """Exception raised for user errors."""


class YamlDataclassMixin:
    """Mixin for dataclasses created by YAML parsing.

    It is necessary to implement __setstate__ in any dataclass instances that
    have been parsed from YAML, because the YAML parser bypasses __init__. This
    is needed because:

    * Fields with defaults need to be set if not specified.
    * Errors need to be raised for any fields missing a value that have no
      default.
    """

    yaml_tag: ClassVar

    def __setstate__(self, state):
        try:
            self.__init__(**state)
        except Exception as e:
            e.add_note(f"processing {self.yaml_tag} with {state=}")
            raise

    @classmethod
    def to_yaml(cls, representer, node):
        """Implements serialising the node as basic YAML types."""
        mapping = {}
        for field in dataclasses.fields(cast(type["DataclassInstance"], cls)):
            value = getattr(node, field.name)
            if not value and dataclassutil.has_default(field):
                continue
            if fn := field.metadata.get("to_yaml"):
                value = fn(value)
            mapping[field.name] = value
        return representer.represent_mapping(cls.yaml_tag, mapping)

    @classmethod
    def from_yaml(cls, constructor, node):
        """Implements deserialising the node from basic YAML types."""
        data = constructor.construct_mapping(node)
        if not isinstance(data, dict):
            raise TypeError(data)
        kwargs = {}
        for field in dataclasses.fields(cast(type["DataclassInstance"], cls)):
            try:
                value = data.pop(field.name)
            except KeyError:
                if dataclassutil.has_default(field):
                    continue
                raise
            if fn := field.metadata.get("from_yaml"):
                value = fn(value)
            kwargs[field.name] = value
        if data:
            names = ", ".join(sorted(data))
            raise TypeError(f"unexpected fields {names} in {cls.yaml_tag}")
        return cls(**kwargs)


class RowFolder(abc.ABC):
    """Abstract base marker for configuring row grouping."""


@dataclasses.dataclass
@_YAML.register_class
class StaticRowCounts(RowFolder, YamlDataclassMixin):
    """Specifies explicit input row counts for output grouped rows."""

    yaml_tag: ClassVar = "!StaticRowCounts"
    row_counts: list[int]


@dataclasses.dataclass
@_YAML.register_class
class EmptyColumn(RowFolder, YamlDataclassMixin):
    """Specifies to group rows by when a given column is empty."""

    yaml_tag: ClassVar = "!EmptyColumn"
    column_index: int


@dataclasses.dataclass
@_YAML.register_class
class TableExtraction(YamlDataclassMixin):
    """Configures the specifics of extracting the CSV from the PDF."""

    yaml_tag: ClassVar = "!TableExtraction"
    add_header_row: Optional[list[str]] = None
    row_folding: list[RowFolder] = dataclasses.field(default_factory=list)


@dataclasses.dataclass
class Table:
    """Defines metadata and extraction configuration relating to a single table.

    The "path" of group names and the table name form the path for both the
    ``.tabula-template.json`` file within the configuration directory and the
    output ``.csv`` file in the output directory.
    """

    file_stem: pathlib.Path
    type: str
    tags: set[str] = dataclasses.field(default_factory=set)
    extraction: Optional[TableExtraction] = dataclasses.field(default_factory=TableExtraction)


@dataclasses.dataclass
class Group:
    """Group of items to extract from the PDF.

    A top-level group within a book is often aligned with a book chapter.

    The table items have Tabula templates in ``.directory``.
    """

    directory: pathlib.Path
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
class Book(YamlDataclassMixin):
    """Top level information about a book."""

    id_: str
    name: str
    default_filename: str
    tags: set[str] = dataclasses.field(default_factory=set)
    group: Optional[Group] = None


@dataclasses.dataclass
class Config:
    """Top-level configuration."""

    directory: pathlib.Path
    books: dict[str, Book] = dataclasses.field(default_factory=dict)


@dataclasses.dataclass
@_YAML.register_class
class _YamlTable(YamlDataclassMixin):
    yaml_tag: ClassVar = "!Table"
    type: Optional[str] = None
    tags: set[str] = dataclasses.field(default_factory=set, metadata=_SET_METADATA)
    extraction: Optional[TableExtraction] = None

    def prepare(self, name: str, directory: pathlib.Path) -> Table:
        """Creates a ``Table`` from self.

        :param name: Name of the table within its ``Group.groups``.
        :param directory: Path to the directory of the parent ``Group``,
        relative to the top-level config directory.
        :return: Prepared ``Table``.
        """
        kw = dataclassutil.shallow_asdict(self)
        return Table(file_stem=directory / name, **kw)


@dataclasses.dataclass
@_YAML.register_class
class _YamlGroup(YamlDataclassMixin):
    yaml_tag: ClassVar = "!Group"
    tags: set[str] = dataclasses.field(default_factory=set, metadata=_SET_METADATA)
    groups: dict[str, "_YamlGroup"] = dataclasses.field(default_factory=dict)
    tables: dict[str, _YamlTable] = dataclasses.field(default_factory=dict)
    extraction_templates: Optional[list[TableExtraction]] = None

    def prepare(self, rel_group_dir: pathlib.Path) -> Group:
        """Creates a ``Group`` from self.

        :param rel_group_dir: Path to the directory of this group's directory,
        relative to the top-level config directory.
        :return: Prepared ``Group``.
        """
        return Group(
            directory=rel_group_dir,
            tags=self.tags,
            tables={
                name: table.prepare(name, rel_group_dir) for name, table in self.tables.items()
            },
            groups={
                name: group.prepare(rel_group_dir / name) for name, group in self.groups.items()
            },
            # extraction_templates not included, as it is only for use in
            # anchoring and aliasing by the YAML file author at the time of YAML
            # parsing.
        )


@dataclasses.dataclass
@_YAML.register_class
class _YamlBook(YamlDataclassMixin):
    yaml_tag: ClassVar = "!Book"
    name: str
    default_filename: str
    tags: set[str] = dataclasses.field(default_factory=set, metadata=_SET_METADATA)

    def prepare(
        self,
        cfg_dir: pathlib.Path,
        book_id: str,
        limit_books: list[str],
    ) -> Book:
        """Creates a ``Book`` from self.

        :param cfg_dir: Path to the directory of the ``Config``.
        :param book_id: ID of the book within the parent _YamlConfig.
        :param limit_books: Allowlist of book names to load configuration for.
        :return: Prepared ``Book``.
        """
        book = Book(
            id_=book_id,
            name=self.name,
            default_filename=self.default_filename,
            tags=self.tags,
        )
        if book_id in limit_books:
            rel_book_dir = pathlib.Path(book_id)
            cfg = _YAML.load(cfg_dir / rel_book_dir / "book.yaml")
            book.group = _prepare_group(cfg, rel_book_dir)
        return book


@dataclasses.dataclass
@_YAML.register_class
class _YamlConfig(YamlDataclassMixin):
    yaml_tag: ClassVar = "!Config"
    books: dict[str, _YamlBook]

    def prepare(self, cfg_dir: pathlib.Path, limit_books: list[str]) -> Config:
        """Creates a ``Group`` from self.

        :param cfg_dir: Path to the directory of the ``Config``.
        :param limit_books: Allowlist of book names to load configuration for.
        :return: Prepared ``Config``.
        """
        books: dict[str, Book] = {}
        for book_id, yaml_book in self.books.items():
            books[book_id] = yaml_book.prepare(
                cfg_dir=cfg_dir,
                book_id=book_id,
                limit_books=limit_books,
            )
        return Config(
            directory=cfg_dir,
            books=books,
        )


def _prepare_group(cfg: Any, rel_book_dir: pathlib.Path) -> Group:
    if not isinstance(cfg, _YamlGroup):
        raise TypeError(cfg)
    return cfg.prepare(rel_book_dir)


def load_group_from_str(yaml_str: str) -> Group:
    """Loads the configuration from the given string containing YAML."""
    cfg = _YAML.load(yaml_str)
    return _prepare_group(cfg, pathlib.Path("."))


def _prepare_config(cfg: Any, cfg_dir: pathlib.Path, limit_books: list[str]) -> Config:
    if not isinstance(cfg, _YamlConfig):
        raise TypeError(cfg)
    return cfg.prepare(cfg_dir, limit_books)


def load_config(cfg_dir: pathlib.Path, limit_books: list[str]) -> Config:
    """Loads the configuration from the directory."""
    cfg = _YAML.load(cfg_dir / "config.yaml")
    return _prepare_config(cfg=cfg, cfg_dir=cfg_dir, limit_books=limit_books)


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


def load_config_from_flag(args: argparse.Namespace, limit_books: list[str]) -> Config:
    """Returns a ``Config`` specified by the parsed arguments.

    :param args: Parsed arguments. This must have been generated from a parser
    that included the argument added by ``add_config_flag``.
    :param limit_books: Name identifiers for the books to load configuration for.
    :return: Loaded configuration.
    """
    return load_config(args.config_dir, limit_books)
