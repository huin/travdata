# -*- coding: utf-8 -*-
import dataclasses
import pathlib
from typing import Any, ClassVar, Iterator, Optional

from ruamel import yaml

_YAML = yaml.YAML(typ="safe")


@dataclasses.dataclass
@_YAML.register_class
class TableExtraction:
    num_header_lines: int = 1
    add_header_row: Optional[list[str]] = None
    continuation_empty_column: int = 0
    row_num_lines: Optional[list[int]] = None


@dataclasses.dataclass
@_YAML.register_class
class _YamlGroup:
    yaml_tag: ClassVar = "!Group"
    groups: dict[str, "_YamlGroup"] = dataclasses.field(default_factory=dict)
    tables: dict[str, "_YamlTable"] = dataclasses.field(default_factory=dict)

    def __setstate__(self, state):
        self.__init__(**state)

    def prepare(self, directory: pathlib.Path) -> "Group":
        return Group(
            directory=directory,
            tables={name: table.prepare(name, directory) for name, table in self.tables.items()},
            groups={name: group.prepare(directory / name) for name, group in self.groups.items()},
        )


@dataclasses.dataclass
@_YAML.register_class
class _YamlTable:
    yaml_tag: ClassVar = "!Table"
    type: Optional[str] = None
    extraction: "TableExtraction" = dataclasses.field(default_factory=TableExtraction)

    def __setstate__(self, state):
        self.__init__(**state)

    def prepare(self, name: str, directory: pathlib.Path) -> "Table":
        kw = dataclasses.asdict(self)
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
    file_stem: pathlib.Path
    type: str
    extraction: "TableExtraction" = dataclasses.field(default_factory=TableExtraction)


def _prepare_config(cfg: Any, cfg_dir: pathlib.Path) -> Group:
    if not isinstance(cfg, _YamlGroup):
        raise TypeError(cfg)
    return cfg.prepare(cfg_dir)


def load_config_from_str(yaml: str) -> Group:
    cfg = _YAML.load(yaml)
    return _prepare_config(cfg, pathlib.Path("."))


def load_config(cfg_dir: pathlib.Path) -> Group:
    cfg = _YAML.load(cfg_dir / "config.yaml")
    return _prepare_config(cfg, pathlib.Path("."))
