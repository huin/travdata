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


_FIRST_TABLES = [
    "career-progress",
    "mustering-out",
    "skills-and-training-general",
    "skills-and-training-assignment",
]
_RB_TABLES = "ranks"
_LAST_TABLES = [
    "mishap",
    "event",
]

_CAREER_RBS: dict[str, list[str | None]] = {
    "agent": [
        "law-enforcement",
        "intel-corp",
    ],
    "army": [
        "enlisted",
        "officer",
    ],
    "citizen": [
        "corporate",
        "worker",
        "colonist",
    ],
    "drifter": [
        "barbarian",
        "wanderer",
        "scavenger",
    ],
    "entertainer": [
        "artist",
        "journalist",
        "performer",
    ],
    "marine": [
        "enlisted",
        "officer",
    ],
    "merchant": [
        "merchant-marine",
        "free-trader",
        "broker",
    ],
    "navy": [
        "enlisted",
        "officer",
    ],
    "noble": [
        "adminstrator",
        "diplomat",
        "dilettante",
    ],
    "rogue": [
        "thief",
        "enforcer",
        "pirate",
    ],
    "scholar": [
        "field-researcher",
        "scientist",
        "physician",
    ],
    "scout": [
        None,
    ],
    "prisoner": [
        None,
    ],
    "psion": [
        "wild-talent",
        "adept",
        "psi-warrior",
    ],
}


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
    career_code: str
    career_name: str
    tmpls: list[Template]

    @classmethod
    def load(cls, p: pathlib.Path) -> "InputTmpl":
        with p.open("rt") as f:
            d: list[Template] = json.load(f)
            for t in d:
                _ensure_template(t)
        p_stem, _ = p.name.split(".", maxsplit=1)
        career_code, _, career_name = p_stem.partition("-")
        _sort_templates(d)
        return InputTmpl(
            career_code=career_code,
            career_name=career_name,
            tmpls=d,
        )

    def sort_key(self) -> str:
        try:
            code = int(self.career_code)
        except ValueError:
            return self.career_code
        return f"{code:02d}"


def add_table(
    grp: config._YamlGroup,
    table_name: str,
    career_dir: pathlib.Path,
    tmpl: Template,
    tags: set[str],
) -> None:
    grp.tables[table_name] = config._YamlTable(
        tags=tags,
        extraction=cfgextract.TableExtraction(),
    )
    tmpl_filename = (career_dir / table_name).with_suffix(".tabula-template.json")
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

    career_grp = config._YamlGroup()

    for input_tmpl in input_tmpls:
        rank_table_names = _CAREER_RBS[input_tmpl.career_name]
        num_expected_tables = len(_FIRST_TABLES) + len(rank_table_names) + len(_LAST_TABLES)
        if num_expected_tables != len(input_tmpl.tmpls):
            print(
                (
                    f"{input_tmpl.career_name} has {len(input_tmpl.tmpls)} tables, "
                    f"but expected {num_expected_tables}"
                ),
                file=sys.stderr,
            )
            continue

        first_tables = input_tmpl.tmpls[: len(_FIRST_TABLES)]
        rank_tables = input_tmpl.tmpls[len(_FIRST_TABLES) : -len(_LAST_TABLES)]
        last_tables = input_tmpl.tmpls[-len(_LAST_TABLES) :]
        if len(first_tables) != len(_FIRST_TABLES):
            raise RuntimeError()
        if len(rank_tables) != len(rank_table_names):
            raise RuntimeError()
        if len(last_tables) != len(_LAST_TABLES):
            raise RuntimeError()

        grp = config._YamlGroup(
            tags={
                "career",
                f"career-code/{input_tmpl.career_code}",
                f"career/{input_tmpl.career_name}",
            },
        )
        career_grp_name = f"{input_tmpl.sort_key()}-{input_tmpl.career_name}"
        career_dir = args.output_dir / career_grp_name
        career_dir.mkdir(parents=True, exist_ok=True)
        career_grp.groups[career_grp_name] = grp

        for table_name, t in zip(_FIRST_TABLES, first_tables):
            add_table(
                grp=grp,
                table_name=table_name,
                career_dir=career_dir,
                tmpl=t,
                tags={
                    f"type/career-{table_name}",
                },
            )

        for rank_table_name, t in zip(rank_table_names, rank_tables):
            if rank_table_name is None:
                table_name = _RB_TABLES
            else:
                table_name = f"{_RB_TABLES}-{rank_table_name}"
            add_table(
                grp=grp,
                table_name=table_name,
                career_dir=career_dir,
                tmpl=t,
                tags={
                    "type/career-rank",
                    f"rank/{rank_table_name}",
                },
            )

        for table_name, t in zip(_LAST_TABLES, last_tables):
            add_table(
                grp=grp,
                table_name=table_name,
                career_dir=career_dir,
                tmpl=t,
                tags={
                    f"type/career-{table_name}",
                },
            )

    yamlreg.YAML.dump(career_grp, sys.stdout)


if __name__ == "__main__":
    main()
