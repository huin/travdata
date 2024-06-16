#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import dataclasses
import functools
import json
import os
import pathlib
import re
import sys
from typing import Any, TypedDict, cast

from travdata import config
from travdata.config import cfgextract, yamlreg

_TABLES = [
    "components",
    "crew",
    "hull",
    "running-costs",
    "power-requirements",
]


class Template(TypedDict):
    page: int
    extraction_method: str
    x1: float
    x2: float
    y1: float
    y2: float
    width: float
    height: float


def _ensure_template(t: Template) -> Any:
    t["page"] = int(t["page"])
    t["extraction_method"] = "stream"


def _template_sort_cmp(a: Template, b: Template) -> int:
    if a["page"] != b["page"]:
        raise ValueError(f"should be on same page {a=} {b=}")
    # The one single table ("components") on the LHS is first.
    rhs_a = a["x1"] >= 200
    rhs_b = b["x1"] >= 200
    if rhs_a < rhs_b:
        return -1
    elif rhs_b < rhs_a:
        return 1
    # Other tables are in order of their y-position.
    return int(1000 * (a["y1"] - b["y1"]))


def _sort_templates(d: list[Template]) -> None:
    d.sort(key=functools.cmp_to_key(_template_sort_cmp))


@dataclasses.dataclass
class InputTmpl:
    name: str
    category_tag: str
    tmpls: list[Template]

    @classmethod
    def load(cls, p: pathlib.Path) -> "InputTmpl":
        with p.open("rt") as f:
            d: list[Template] = json.load(f)
            for t in d:
                _ensure_template(t)
        p_stem, _ = p.name.split(".", maxsplit=1)
        if m := re.match(r"(.+)-small-craft", p_stem):
            name = m.group(1)
            cat = "category/small-craft"
        elif m := re.match(r"(.+)-type-(.+)$", p_stem):
            name = m.group(1)
            cat = "spacecraft-type/" + m.group(2)
        elif m := re.match(r"(.+)-class-(.+)$", p_stem):
            name = m.group(1)
            cat = "spacecraft-class/" + m.group(2)
        else:
            name = p_stem
            cat = ""
        _sort_templates(d)
        return InputTmpl(
            name=name,
            category_tag=cat,
            tmpls=d,
        )

    def sort_key(self) -> str:
        return self.name


def add_table(
    table_name: str,
    grp_dir: pathlib.Path,
    tmpl: Template,
) -> None:
    tmpl_filename = (grp_dir / table_name).with_suffix(".tabula-template.json")
    with tmpl_filename.open("wt") as f:
        json.dump([tmpl], f, indent=2)


def main() -> None:
    argparser = argparse.ArgumentParser()
    argparser.add_argument("output_dir", type=pathlib.Path)
    argparser.add_argument("inputs", type=pathlib.Path, nargs="+")
    args = argparser.parse_args()

    output_dir = cast(pathlib.Path, args.output_dir)
    os.makedirs(output_dir, exist_ok=True)

    input_tmpls: list[InputTmpl] = [InputTmpl.load(p) for p in args.inputs]
    input_tmpls.sort(key=InputTmpl.sort_key)

    template_group = config._YamlGroup()
    for table_name in _TABLES:
        template_group.tables[table_name] = config._YamlTable(
            tags={
                f"type/spacecraft-{table_name}",
            },
            extraction=cfgextract.TableExtraction(),
        )

    print("## Template tables:")
    yamlreg.YAML.dump(template_group, sys.stdout)

    print()

    top_grp = config._YamlGroup()
    print("## Groups:")
    for input_tmpl in input_tmpls:
        if len(_TABLES) != len(input_tmpl.tmpls):
            print(
                (
                    f"{input_tmpl.name} has {len(input_tmpl.tmpls)} tables, "
                    f"but expected {len(_TABLES)}"
                ),
                file=sys.stderr,
            )
            continue

        grp = config._YamlGroup(
            tags={
                "spacecraft",
                f"spacecraft/{input_tmpl.name}",
            },
        )
        if input_tmpl.category_tag:
            grp.tags.add(input_tmpl.category_tag)
        grp_dir = args.output_dir / input_tmpl.name
        grp_dir.mkdir(parents=True, exist_ok=True)
        top_grp.groups[input_tmpl.name] = grp

        for table_name, t in zip(_TABLES, input_tmpl.tmpls):
            add_table(
                table_name=table_name,
                grp_dir=grp_dir,
                tmpl=t,
            )

    yamlreg.YAML.dump(top_grp, sys.stdout)


if __name__ == "__main__":
    main()
