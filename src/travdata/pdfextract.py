# -*- coding: utf-8 -*-
import dataclasses
import pathlib
from typing import Iterator, Optional

from ruamel import yaml
from travdata import parseutil, tabulautil

_YAML = yaml.YAML(typ="safe")


@dataclasses.dataclass
@yaml.yaml_object(_YAML)
class Config:
    tables: list["Table"]
    tabula_tmpl_dir: Optional[pathlib.Path] = None


@dataclasses.dataclass
@yaml.yaml_object(_YAML)
class Table:
    name: str
    num_header_lines: int = 1
    continuation_empty_column: int = 0


def load_config_from_str(yaml: str) -> Config:
    cfg = _YAML.load(yaml)
    if not isinstance(cfg, Config):
        raise TypeError(cfg)
    return cfg


def load_config(cfg_dir: pathlib.Path) -> Config:
    cfg = _YAML.load(cfg_dir / "config.yaml")
    if not isinstance(cfg, Config):
        raise TypeError(cfg)
    if cfg.tabula_tmpl_dir is None:
        cfg.tabula_tmpl_dir = cfg_dir
    return cfg


def _extract_table(
    table: Table,
    pdf_path: pathlib.Path,
    tabula_tmpl_dir: pathlib.Path,
) -> Iterator[list[str]]:
    tabula_rows: Iterator[tabulautil.TabularRow] = tabulautil.table_rows_concat(
        tabulautil.read_pdf_with_template(
            pdf_path=pdf_path,
            template_path=tabula_tmpl_dir / f"{table.name}.tabula-template.json",
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
    text_rows = parseutil.amalgamate_streamed_rows(
        rows=text_rows,
        continuation=continuation,
    )
    return parseutil.clean_rows(text_rows)


@dataclasses.dataclass
class ExtractedTable:
    name: str
    rows: Iterator[list[str]]


def extract_tables(
    cfg: Config,
    pdf_path: pathlib.Path,
) -> Iterator[ExtractedTable]:
    """Extracts table data from the PDF.

    :param cfg: Configuration of tables to extact. `cfg.tabula_tmpl_dir` must be
    set to a valid path.
    :param pdf_path: Path to the PDF file to read from.
    """
    if cfg.tabula_tmpl_dir is None:
        raise ValueError("cfg.tabula_tmpl_dir must be set")
    for table in cfg.tables:
        yield ExtractedTable(
            name=table.name,
            rows=_extract_table(
                table=table,
                pdf_path=pdf_path,
                tabula_tmpl_dir=cfg.tabula_tmpl_dir,
            ),
        )
