# -*- coding: utf-8 -*-
"""Defines the configuration around data extraction and other metadata.

Values of these types are read from two types of file:

* ``config.yaml`` top-level configuration for multiple books.
* ``book.yaml`` relating to a single input PDF.

See development.adoc for more information in how this is used.
"""

import abc
import argparse
import dataclasses
import pathlib
import shutil
import sys
import textwrap
from typing import Any, ClassVar, Iterator, Optional

from ruamel import yaml
from travdata import dataclassutil

_YAML = yaml.YAML(typ="safe")

__executable_environment__ = "development"


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
    extraction: Optional[TableExtraction] = dataclasses.field(default_factory=TableExtraction)


@dataclasses.dataclass
class Group:
    """Group of items to extract from the PDF.

    A top-level group within a book is often aligned with a book chapter.

    The table items have Tabula templates in ``.directory``.
    """

    directory: pathlib.Path
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
class Config:
    """Top-level configuration."""

    directory: pathlib.Path
    books: dict[str, Group] = dataclasses.field(default_factory=dict)
    book_names: list[str] = dataclasses.field(default_factory=list)


@dataclasses.dataclass
@_YAML.register_class
class _YamlTable(YamlDataclassMixin):
    yaml_tag: ClassVar = "!Table"
    type: Optional[str] = None
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
class _YamlConfig(YamlDataclassMixin):
    yaml_tag: ClassVar = "!Config"
    books: list[str] = dataclasses.field(default_factory=list)

    def prepare(self, cfg_dir: pathlib.Path, limit_books: list[str]) -> Config:
        """Creates a ``Group`` from self.

        :param cfg_dir: Path to the directory of the ``Config``.
        :param limit_books: Allowlist of book names to load configuration for.
        :return: Prepared ``Config``.
        """
        books: dict[str, Group] = {}
        for book_name in limit_books:
            if book_name not in self.books:
                raise UserError(f"book {book_name!r} does not exist")
            book_dir = pathlib.Path(book_name)
            books[book_name] = _load_book_config(cfg_dir, book_dir)
        return Config(
            directory=cfg_dir,
            books=books,
            book_names=self.books,
        )


def _prepare_book_config(cfg: Any, rel_book_dir: pathlib.Path) -> Group:
    if not isinstance(cfg, _YamlGroup):
        raise TypeError(cfg)
    return cfg.prepare(rel_book_dir)


def load_book_config_from_str(yaml_str: str) -> Group:
    """Loads the configuration from the given string containing YAML."""
    cfg = _YAML.load(yaml_str)
    return _prepare_book_config(cfg, pathlib.Path("."))


def _load_book_config(cfg_dir: pathlib.Path, rel_book_dir: pathlib.Path) -> Group:
    """Loads the configuration from the directory."""
    cfg = _YAML.load(cfg_dir / rel_book_dir / "book.yaml")
    return _prepare_book_config(cfg, rel_book_dir)


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

    default_config_dir = _get_default_config_path()

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


def _get_default_config_path() -> Optional[pathlib.Path]:
    install_dir: Optional[pathlib.Path]
    if __executable_environment__ == "development":
        install_dir = _installation_dir_for_development()
    else:
        install_dir = _installation_dir_for_release()

    if install_dir is None:
        return None

    config_dir = install_dir / "config"
    config_file = config_dir / "config.yaml"
    if not config_file.is_file():
        return None
    return config_dir


def _installation_dir_for_development() -> pathlib.Path:
    return pathlib.Path.cwd()


def _installation_dir_for_release() -> Optional[pathlib.Path]:
    script_str = shutil.which(sys.argv[0])
    if script_str is None:
        return None
    script_path = pathlib.Path(script_str)
    return script_path.parent


def load_config_from_flag(args: argparse.Namespace, limit_books: list[str]) -> Config:
    """Returns a ``Config`` specified by the parsed arguments.

    :param args: Parsed arguments. This must have been generated from a parser
    that included the argument added by ``add_config_flag``.
    :param limit_books: Name identifiers for the books to load configuration for.
    :return: Loaded configuration.
    """
    return load_config(args.config_dir, limit_books)
