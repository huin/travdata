# -*- coding: utf-8 -*-
"""Defines the configuration around data extraction and other metadata.

Values of these types are read from ``config.yaml`` files, relating to a single
input PDF. See development.adoc for more information in how this is used.
"""

import abc
import dataclasses
import pathlib
from typing import Any, ClassVar, Iterator, Optional

from ruamel import yaml
from travdata import dataclassutil

_YAML = yaml.YAML(typ="safe")


class RowFolder(abc.ABC):
    """Abstract base marker for configuring row grouping."""


@dataclasses.dataclass
@_YAML.register_class
class StaticRowCounts(RowFolder):
    """Specifies explicit input row counts for output grouped rows."""

    yaml_tag: ClassVar = "!StaticRowCounts"
    row_counts: list[int]


@dataclasses.dataclass
@_YAML.register_class
class EmptyColumn(RowFolder):
    """Specifies to group rows by when a given column is empty."""

    yaml_tag: ClassVar = "!EmptyColumn"
    column_index: int


@dataclasses.dataclass
@_YAML.register_class
class TableExtraction:
    """Configures the specifics of extracting the CSV from the PDF."""

    yaml_tag: ClassVar = "!TableExtraction"
    add_header_row: Optional[list[str]] = None
    row_folding: list[RowFolder] = dataclasses.field(default_factory=list)

    def __setstate__(self, state):
        try:
            self.__init__(**state)
        except Exception as e:
            e.add_note(f"processing !TableExtraction with {state=}")
            raise


@dataclasses.dataclass
@_YAML.register_class
class _YamlGroup:
    yaml_tag: ClassVar = "!Group"
    groups: dict[str, "_YamlGroup"] = dataclasses.field(default_factory=dict)
    tables: dict[str, "_YamlTable"] = dataclasses.field(default_factory=dict)
    extraction_templates: Optional[list[TableExtraction]] = None

    def __setstate__(self, state):
        try:
            self.__init__(**state)
        except Exception as e:
            e.add_note(f"processing !Group with {state=}")
            raise

    def prepare(self, directory: pathlib.Path) -> "Group":
        """Creates a ``Group`` from self.

        :param directory: Path to the directory of the parent ``Group``.
        :return: Prepared ``Group``.
        """
        return Group(
            directory=directory,
            tables={name: table.prepare(name, directory) for name, table in self.tables.items()},
            groups={name: group.prepare(directory / name) for name, group in self.groups.items()},
            # extraction_templates not included, as it is only for use in
            # anchoring and aliasing by the YAML file author at the time of YAML
            # parsing.
        )


@dataclasses.dataclass
@_YAML.register_class
class _YamlTable:
    yaml_tag: ClassVar = "!Table"
    type: Optional[str] = None
    extraction: Optional["TableExtraction"] = None

    def __setstate__(self, state):
        try:
            self.__init__(**state)
        except Exception as e:
            e.add_note(f"processing !Table with {state=}")
            raise

    def prepare(self, name: str, directory: pathlib.Path) -> "Table":
        """Creates a ``Table`` from self.

        :param name: Name of the table within its ``Group.groups``.
        :param directory: Path to the directory of the parent ``Group``.
        :return: Prepared ``Table``.
        """
        kw = dataclassutil.shallow_asdict(self)
        return Table(file_stem=directory / name, **kw)


@dataclasses.dataclass
class Group:
    """Group of items to extract from the PDF.

    A top-level group within a book is often aligned with a book chapter.

    The table items have Tabula templates in ``.directory``.
    """

    directory: pathlib.Path
    tables: dict[str, "Table"] = dataclasses.field(default_factory=dict)
    groups: dict[str, "Group"] = dataclasses.field(default_factory=dict)

    def all_tables(self) -> Iterator["Table"]:
        """Iterates over all tables in this group and its child groups.

        :yield: Descendent tables.
        """
        yield from self.tables.values()
        for group in self.groups.values():
            yield from group.all_tables()


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


def _prepare_config(cfg: Any, cfg_dir: pathlib.Path) -> Group:
    if not isinstance(cfg, _YamlGroup):
        raise TypeError(cfg)
    return cfg.prepare(cfg_dir)


def load_config_from_str(yaml_str: str) -> Group:
    """Loads the configuration from the given string containing YAML."""
    cfg = _YAML.load(yaml_str)
    return _prepare_config(cfg, pathlib.Path("."))


def load_config(cfg_dir: pathlib.Path) -> Group:
    """Loads the configuration from the directory."""
    cfg = _YAML.load(cfg_dir / "config.yaml")
    return _prepare_config(cfg, pathlib.Path("."))
