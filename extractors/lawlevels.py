# -*- coding: utf-8 -*-
import dataclasses
from typing import Iterable, Iterator, Optional, TypedDict, cast

import jsonenc
import parseutil
import tabulautil
from extractors import params


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class LawLevel(jsonenc.Decodable, jsonenc.Encodable):
    min_level: int
    max_level: Optional[int]
    description: Optional[str]
    weapons_banned: Optional[str]
    armour_banned: Optional[str]

    @classmethod
    def json_type(cls) -> str:
        return "LawLevel"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "LawLevel":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


_RawRow = TypedDict(
    "_RawRow",
    {
        "Law Level": str,
        "Weapons Banned": str,
        "Armour": str,
    },
)


def _preprocess_rows(
    rows: Iterable[tabulautil.TabularRow],
) -> Iterator[list[str]]:
    text_rows = tabulautil.table_rows_text(rows)
    text_rows = parseutil.amalgamate_streamed_rows(
        rows=text_rows,
        # Header is split over two lines.
        # Thereafter, the first column is always empty on subsequent
        # continuation rows.
        continuation=lambda i, row: i == 1 or (i > 1 and row[0] == ""),
    )
    return parseutil.clean_rows(text_rows)


def extract_from_pdf(
    param: params.CoreParams,
) -> list[LawLevel]:
    rows_list = tabulautil.table_rows_concat(
        tabulautil.read_pdf_with_template(
            pdf_path=param.core_rulebook,
            template_path=param.templates_dir / "law-levels.tabula-template.json",
        ),
    )

    rows = _preprocess_rows(rows_list)
    header, rows = parseutil.headers_and_iter_rows(rows)
    labeled_rows = parseutil.label_rows(rows, header)

    results: list[LawLevel] = []
    for row in cast(Iterator[_RawRow], labeled_rows):
        level = row["Law Level"]
        if level.endswith("+"):
            min_level = int(level.removesuffix("+"))
            max_level = None
        else:
            min_level = max_level = int(level)
        if "Armour" not in row:
            results.append(
                LawLevel(
                    min_level=min_level,
                    max_level=max_level,
                    description=row["Weapons Banned"],
                    weapons_banned=None,
                    armour_banned=None,
                )
            )
        else:
            results.append(
                LawLevel(
                    min_level=min_level,
                    max_level=max_level,
                    description=None,
                    weapons_banned=row["Weapons Banned"] or results[-1].weapons_banned,
                    armour_banned=row["Armour"] or results[-1].armour_banned,
                )
            )

    return results
