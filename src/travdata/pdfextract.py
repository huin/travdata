# -*- coding: utf-8 -*-
import dataclasses
import pathlib
from typing import Any, Callable, ClassVar, Iterable, Iterator

from ruamel import yaml
from travdata import parseutil, tabulautil

_YAML = yaml.YAML(typ="safe")


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
    type: str
    num_header_lines: int = 1
    continuation_empty_column: int = 0

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

    def num_tables(self) -> int:
        """Calculates the number of tables in this group and child groups.

        :return: Number of tables.
        """
        return len(self.tables) + sum(group.num_tables() for group in self.groups.values())


@dataclasses.dataclass
class Table:
    file_stem: pathlib.Path
    type: str
    num_header_lines: int = 1
    continuation_empty_column: int = 0


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


def _extract_table(
    config_dir: pathlib.Path,
    pdf_path: pathlib.Path,
    table: Table,
) -> Iterator[list[str]]:
    tabula_rows: Iterator[tabulautil.TabulaRow] = tabulautil.table_rows_concat(
        tabulautil.read_pdf_with_template(
            pdf_path=pdf_path,
            template_path=config_dir / table.file_stem.with_suffix(".tabula-template.json"),
        )
    )

    def continuation(i: int, row: list[str]) -> bool:
        if i == 0:
            return False
        elif i < table.num_header_lines:
            return True
        else:
            return row[table.continuation_empty_column] == ""

    text_rows = tabulautil.table_rows_text(tabula_rows)
    text_rows = _amalgamate_streamed_rows(
        rows=text_rows,
        continuation=continuation,
    )
    return _clean_rows(text_rows)


@dataclasses.dataclass
class ExtractedTable:
    table_cfg: Table
    rows: Iterator[list[str]]


def extract_tables(
    group: Group,
    config_dir: pathlib.Path,
    pdf_path: pathlib.Path,
) -> Iterator[ExtractedTable]:
    """Extracts table data from the PDF.

    :param cfg: Configuration of tables to extact. `cfg.tabula_tmpl_dir` must be
    set to a valid path.
    :param pdf_path: Path to the PDF file to read from.
    """
    for table in group.tables.values():
        yield ExtractedTable(
            table_cfg=table,
            rows=_extract_table(
                config_dir=config_dir,
                table=table,
                pdf_path=pdf_path,
            ),
        )
    for sub_group in group.groups.values():
        yield from extract_tables(group=sub_group, config_dir=config_dir, pdf_path=pdf_path)


def _amalgamate_streamed_rows(
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
