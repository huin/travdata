# -*- coding: utf-8 -*-
"""Defines the configuration around data extraction and other metadata.

Values of these types are read from two types of file:

* ``config.yaml`` top-level configuration for multiple books.
* ``book.yaml`` relating to a single input PDF.

See development.adoc for more information in how this is used.
"""

from __future__ import annotations

import argparse
import contextlib
import dataclasses
import pathlib
import sys
import textwrap
from typing import Any, ClassVar, Iterator, Optional, Self

from travdata import filesio, travdatarelease, yamlutil
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

    file_stem: pathlib.PurePath
    tags: set[str] = dataclasses.field(default_factory=set)
    extraction: Optional[cfgextract.TableExtraction] = dataclasses.field(
        default_factory=cfgextract.TableExtraction
    )

    @property
    def tabula_template_path(self) -> pathlib.PurePath:
        """Path to the Tabula template, assuming that it exists."""
        return self.file_stem.with_suffix(TABULA_TEMPLATE_SUFFIX)


@dataclasses.dataclass
class Group:
    """Group of items to extract from the PDF.

    A top-level group within a book is often aligned with a book chapter.

    The table items have Tabula templates in ``.directory``.
    """

    rel_dir: pathlib.PurePath
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

    id_: str
    name: str
    default_filename: str
    tags: set[str] = dataclasses.field(default_factory=set)
    _group: Optional[Group] = None

    def load_group(self, cfg_reader: filesio.Reader) -> Group:
        """Loads and returns the top-level group in the `Book`."""
        if self._group is None:
            self._group = load_book(cfg_reader, self.id_, self.tags)
        return self._group


@dataclasses.dataclass
class Config:
    """Top-level configuration."""

    books: dict[str, Book] = dataclasses.field(default_factory=dict)


@dataclasses.dataclass
@yamlreg.YAML.register_class
class _YamlTable(yamlutil.YamlMappingMixin):
    yaml_tag: ClassVar = "!Table"
    tags: set[str] = dataclasses.field(default_factory=set, metadata=yamlutil.SET_METADATA)
    extraction: Optional[cfgextract.TableExtraction] = None

    def prepare(
        self,
        name: str,
        rel_group_dir: pathlib.PurePath,
        parent_tags: set[str],
    ) -> Table:
        """Creates a ``Table`` from self.

        :param name: Name of the table within its ``Group.groups``.
        :param rel_group_dir: Path to the directory of the table's parent
        group's directory, relative to the top-level config directory.
        :param parent_tags: Tags to inherit from parent ``Group``.
        :return: Prepared ``Table``.
        """
        tags = self.tags | parent_tags
        return Table(
            file_stem=rel_group_dir / name,
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
        rel_group_dir: pathlib.PurePath,
        parent_tags: set[str],
    ) -> Group:
        """Creates a ``Group`` from self.

        :param rel_group_dir: Path to the directory of this group's directory,
        relative to the top-level config directory.
        :param parent_tags: Tags to inherit from parent ``Group``.
        :return: Prepared ``Group``.
        """
        tags = self.tags | parent_tags
        return Group(
            rel_dir=rel_group_dir,
            tags=tags,
            tables={
                name: table.prepare(name, rel_group_dir, parent_tags=tags)
                for name, table in self.tables.items()
            },
            groups={
                name: group.prepare(rel_group_dir / name, parent_tags=tags)
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
        book_id: str,
    ) -> Book:
        """Creates a ``Book`` from self.

        :param cfg_reader: Reader for the configuration files.
        :param book_id: ID of the book within the parent _YamlConfig.
        :return: Prepared ``Book``.
        """
        tags = self.tags | {f"book/{self.name}"}
        return Book(
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

    def prepare(self) -> Config:
        """Creates a ``Group`` from self.

        :param cfg_dir: Path to the directory of the top-level ``Config``.
        :return: Prepared ``Config``.
        """
        books: dict[str, Book] = {}
        for book_id, yaml_book in self.books.items():
            books[book_id] = yaml_book.prepare(book_id=book_id)
        return Config(books=books)


def _prepare_group(
    yaml_group: Any | _YamlGroup,
    rel_book_dir: pathlib.PurePath,
    parent_tags: set[str],
) -> Group:
    if not isinstance(yaml_group, _YamlGroup):
        raise TypeError(yaml_group)
    return yaml_group.prepare(
        rel_group_dir=rel_book_dir,
        parent_tags=parent_tags,
    )


def load_book(
    cfg_reader: filesio.Reader,
    book_id: str,
    parent_tags: set[str],
) -> Group:
    """Loads the book configuration from the given reader."""
    rel_book_dir = pathlib.PurePath(book_id)
    config_path = rel_book_dir / "book.yaml"
    with cfg_reader.open_read(config_path) as f:
        cfg = yamlreg.YAML.load(f)
    return _prepare_group(
        yaml_group=cfg,
        rel_book_dir=rel_book_dir,
        parent_tags=parent_tags,
    )


def parse_yaml_for_testing(yaml_str: str) -> Any:
    """Parses the given cfgyaml.YAML. without preparing it.

    This is only exposed for testing purposes.

    :param yaml_str: cfgyaml.YAML.to parse.
    :return: Parsed objects.
    """
    return yamlreg.YAML.load(yaml_str)


def _prepare_config(cfg: Any | _YamlConfig) -> Config:
    if not isinstance(cfg, _YamlConfig):
        raise TypeError(cfg)
    return cfg.prepare()


def load_config(cfg_reader: filesio.Reader) -> Config:
    """Loads the configuration from the directory."""
    with cfg_reader.open_read(pathlib.PurePath("config.yaml")) as f:
        cfg = yamlreg.YAML.load(f)
    return _prepare_config(cfg=cfg)


def add_config_flag(argparser: argparse.ArgumentParser) -> None:
    """Adds the flag required to call ``load_config_from_flag`` on the parsed args."""

    default_config_dir = get_default_config_path()

    argparser.add_argument(
        "--config",
        "-c",
        help=textwrap.dedent(
            """
            Path to the configuration. This must be either a directory or ZIP
            file, directly containing a config.yaml file, book.yaml files in
            directories, and its required Tabula templates. Some configurations
            for this should be included with this program's distribution.
            """
        ),
        type=pathlib.Path,
        metavar="CONFIG_PATH",
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


def config_reader(
    args: argparse.Namespace,
) -> contextlib.AbstractContextManager[filesio.Reader]:
    """Returns a Reader for the configuration.

    :param args: Parsed arguments. This must have been generated from a parser
    that included the argument added by ``add_config_flag``.
    :return: Context manager for a configuration reader.
    """
    path = args.config
    if path.is_dir():
        return filesio.DirReader.open(path)
    if path.is_file():
        return filesio.ZipReader.open(path)
    raise ValueError(f"config path {path} is neither file nor directory")
