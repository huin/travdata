#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import dataclasses
import functools
import json
import pathlib
import sys
from typing import Any, TypedDict, cast

from travdata import config
from travdata.config import cfgextract, yamlreg

_TABLES = [
    "equipment",
    "spec",
    "armour",
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


def _approx_eq(a: float, b: float) -> bool:
    return abs(a - b) < 50.0


def _template_sort_cmp(a: Template, b: Template) -> int:
    if a["page"] != b["page"]:
        return a["page"] - b["page"]
    elif not _approx_eq(a["y1"], b["y1"]):
        return int(1000 * (a["y1"] - b["y1"]))
    else:
        return int(1000 * (a["x1"] - b["x1"]))


def _sort_templates(d: list[Template]) -> None:
    d.sort(key=functools.cmp_to_key(_template_sort_cmp))


@dataclasses.dataclass
class InputTmpl:
    vehicle_name: str
    tmpls: list[Template]

    @classmethod
    def load(cls, p: pathlib.Path) -> "InputTmpl":
        with p.open("rt") as f:
            d: list[Template] = json.load(f)
            for t in d:
                _ensure_template(t)
        p_stem, _ = p.name.split(".", maxsplit=1)
        _sort_templates(d)
        return InputTmpl(
            vehicle_name=p_stem,
            tmpls=d,
        )

    def sort_key(self) -> str:
        return self.vehicle_name


def add_table(
    grp: config._YamlGroup,
    table_name: str,
    grp_dir: pathlib.Path,
    tmpl: Template,
    tags: set[str],
) -> None:
    grp.tables[table_name] = config._YamlTable(
        tags=tags,
        transform=cfgextract.LegacyTransformSeq(),
    )
    tmpl_filename = (grp_dir / table_name).with_suffix(".tabula-template.json")
    with tmpl_filename.open("wt") as f:
        json.dump([tmpl], f, indent=2)


def main() -> None:
    argparser = argparse.ArgumentParser()
    argparser.add_argument("output_dir", type=pathlib.Path)
    argparser.add_argument("inputs", type=pathlib.Path, nargs="+")
    args = argparser.parse_args()

    output_dir = cast(pathlib.Path, args.output_dir)
    if not output_dir.is_dir():
        argparser.exit(1, f"{output_dir} is not a directory")

    input_tmpls: list[InputTmpl] = [InputTmpl.load(p) for p in args.inputs]
    input_tmpls.sort(key=InputTmpl.sort_key)

    top_grp = config._YamlGroup()

    for input_tmpl in input_tmpls:
        if 3 != len(input_tmpl.tmpls):
            print(
                (
                    f"{input_tmpl.vehicle_name} has {len(input_tmpl.tmpls)} tables, "
                    f"but expected 3"
                ),
                file=sys.stderr,
            )
            continue

        grp = config._YamlGroup(
            tags={
                "vehicle",
                f"vehicle/{input_tmpl.vehicle_name}",
            },
        )
        grp_dir = args.output_dir / input_tmpl.vehicle_name
        grp_dir.mkdir(parents=True, exist_ok=True)
        top_grp.groups[input_tmpl.vehicle_name] = grp

        for table_name, t in zip(_TABLES, input_tmpl.tmpls):
            add_table(
                grp=grp,
                table_name=table_name,
                grp_dir=grp_dir,
                tmpl=t,
                tags={
                    f"type/vehicle-{table_name}",
                },
            )

    yamlreg.YAML.dump(top_grp, sys.stdout)


if __name__ == "__main__":
    main()
