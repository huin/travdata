# -*- coding: utf-8 -*-
"""Defines the configuration around data extraction and other metadata.

Values of these types are read from two types of file:

* ``config.yaml`` top-level configuration for multiple books.
* ``book.yaml`` relating to a single input PDF.

See development.adoc for more information in how this is used.
"""

from __future__ import annotations

import argparse
import dataclasses
import pathlib
import sys
import textwrap
from typing import Any, ClassVar, Iterator, Optional, Self

from travdata import travdatarelease, yamlutil
from travdata.config import yamlreg
from travdata.config import cfgextract


TABULA_TEMPLATE_SUFFIX = ".tabula-template.json"


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
    extraction: Optional[cfgextract.TableExtraction] = dataclasses.field(
        default_factory=cfgextract.TableExtraction
    )

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
            yaml_group = yamlreg.YAML.load(self.cfg_dir / rel_book_dir / "book.yaml")
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
@yamlreg.YAML.register_class
class _YamlTable(yamlutil.YamlMappingMixin):
    yaml_tag: ClassVar = "!Table"
    tags: set[str] = dataclasses.field(default_factory=set, metadata=yamlutil.SET_METADATA)
    extraction: Optional[cfgextract.TableExtraction] = None

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
@yamlreg.YAML.register_class
class _YamlGroup(yamlutil.YamlMappingMixin):
    yaml_tag: ClassVar = "!Group"
    tags: set[str] = dataclasses.field(default_factory=set, metadata=yamlutil.SET_METADATA)
    templates: Optional[list[cfgextract.TableExtraction]] = None
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
            # aliasing by the cfgyaml.YAML.file author at the time of YAML parsing.
        )


@dataclasses.dataclass
@yamlreg.YAML.register_class
class _YamlBook(yamlutil.YamlMappingMixin):
    yaml_tag: ClassVar = "!Book"
    name: str
    default_filename: str
    tags: set[str] = dataclasses.field(default_factory=set, metadata=yamlutil.SET_METADATA)

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
@yamlreg.YAML.register_class
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
    """Loads the configuration from the given string containing cfgyaml.YAML."""
    cfg = yamlreg.YAML.load(yaml_str)
    return _prepare_group(
        yaml_group=cfg,
        cfg_dir=pathlib.Path("."),
        rel_book_dir=pathlib.Path("."),
        parent_tags=parent_tags,
    )


def parse_yaml_for_testing(yaml_str: str) -> Any:
    """Parses the given cfgyaml.YAML. without preparing it.

    This is only exposed for testing purposes.

    :param yaml_str: cfgyaml.YAML.to parse.
    :return: Parsed objects.
    """
    return yamlreg.YAML.load(yaml_str)


def _prepare_config(
    cfg: Any | _YamlConfig,
    cfg_dir: pathlib.Path,
) -> Config:
    if not isinstance(cfg, _YamlConfig):
        raise TypeError(cfg)
    return cfg.prepare(cfg_dir)


def load_config(cfg_dir: pathlib.Path) -> Config:
    """Loads the configuration from the directory."""
    cfg = yamlreg.YAML.load(cfg_dir / "config.yaml")
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
    match travdatarelease.EXECUTABLE_ENVIRONMENT:
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
