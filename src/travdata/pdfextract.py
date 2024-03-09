# -*- coding: utf-8 -*-
import dataclasses
import itertools
import pathlib
from typing import Any, Callable, ClassVar, Iterable, Iterator, Optional

from ruamel import yaml
from travdata import parseutil, tabulautil

_YAML = yaml.YAML(typ="safe")


class ConfigurationError(Exception):
    pass


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
    num_header_lines: int = 1
    add_header_row: Optional[list[str]] = None
    continuation_empty_column: int = 0
    row_num_lines: Optional[list[int]] = None

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
    num_header_lines: int = 1
    add_header_row: Optional[list[str]] = None
    continuation_empty_column: Optional[int] = 0
    row_num_lines: Optional[list[int]] = None


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


def _iter_num_rows_continuations(row_num_lines: list[int]) -> Iterator[bool]:
    for num_lines in row_num_lines:
        yield False
        for _ in range(num_lines - 1):
            yield True


def extract_table(
    config_dir: pathlib.Path,
    pdf_path: pathlib.Path,
    table: Table,
    tabula_cfg: tabulautil.TabulaConfig,
) -> Iterator[list[str]]:
    """Extracts a table from the PDF.

    :param config_dir: Config directory containing the config.yaml file.
    :param pdf_path: Path to the PDF to extract from.
    :param table: Table configuration to extract.
    :param tabula_cfg: Configuration for Tabula extractor.
    :returns: Iterator over rows from the table.
    """
    tabula_rows: Iterator[tabulautil.TabulaRow] = tabulautil.table_rows_concat(
        tabulautil.read_pdf_with_template(
            pdf_path=pdf_path,
            template_path=config_dir / table.file_stem.with_suffix(".tabula-template.json"),
            config=tabula_cfg,
        )
    )

    if table.row_num_lines is not None:
        iter_num_rows_continuations = _iter_num_rows_continuations(table.row_num_lines)
    else:
        iter_num_rows_continuations = None

    def continuation(i: int, row: list[str]) -> bool:
        if table.add_header_row is None:
            if i == 0:
                return False
            elif i < table.num_header_lines:
                return True

        if table.continuation_empty_column is not None:
            return row[table.continuation_empty_column] == ""
        elif iter_num_rows_continuations is not None:
            try:
                return next(iter_num_rows_continuations)
            except StopIteration:
                raise ConfigurationError("Not enough total lines specified in row_num_lines.")
        else:
            return False

    text_rows = tabulautil.table_rows_text(tabula_rows)
    text_rows = _fold_rows(
        rows=text_rows,
        continuation=continuation,
    )
    text_rows = _clean_rows(text_rows)
    if table.add_header_row is not None:
        text_rows = itertools.chain([table.add_header_row], text_rows)
    return text_rows


def _fold_rows(
    rows: Iterable[list[str]],
    continuation: Callable[[int, list[str]], bool],
    join: str = "\n",
) -> Iterator[list[str]]:
    row_accum: list[list[str]] = []

    def form_row():
        return [join.join(cell) for cell in row_accum]

    for i, row in enumerate(rows):
        try:
            if not continuation(i, row) and row_accum:
                yield form_row()
                row_accum = []
            missing_count = len(row) - len(row_accum)
            if missing_count > 0:
                for _ in range(missing_count):
                    row_accum.append([])
            for acc, text in zip(row_accum, row):
                if text:
                    acc.append(text)
        except Exception as e:
            e.add_note(f"for {row=}")
            raise

    if row_accum:
        yield form_row()


def _clean_rows(rows: Iterable[list[str]]) -> Iterator[list[str]]:
    for row in rows:
        yield [parseutil.clean_text(text) for text in row]
