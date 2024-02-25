# -*- coding: utf-8 -*-
import dataclasses
from typing import Iterator, TypedDict, cast

from travellerutil import jsonenc, parseutil, tabulautil
from travellerutil.extractors import params


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class Government(jsonenc.Decodable, jsonenc.Encodable):
    code: str
    name: str
    description: str
    examples: str
    example_contaband: set[str]

    @classmethod
    def json_type(cls) -> str:
        return "Government"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "Government":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


_RawRow = TypedDict(
    "_RawRow",
    {
        "Government": str,
        "Government Type": str,
        "Description": str,
        "Examples": str,
        "Example Contraband": str,
    },
    total=True,
)


def _continuation(
    i: int,
    row: list[str],
) -> bool:
    # Header is split over two lines.
    # Thereafter, the first column is always empty on subsequent continuation
    # rows.
    return i == 1 or (i > 1 and row[0] == "")


def extract_from_pdf(
    param: params.CoreParams,
) -> Iterator[Government]:
    rows_list = tabulautil.table_rows_concat(
        tabulautil.read_pdf_with_template(
            pdf_path=param.core_rulebook,
            template_path=param.templates_dir / "governments.json",
        ),
    )

    rows = parseutil.amalgamate_streamed_rows(
        rows=tabulautil.table_rows_text(rows_list),
        continuation=_continuation,
    )
    rows = parseutil.clean_rows(rows)
    header, rows = parseutil.headers_and_iter_rows(rows)
    labeled_rows = parseutil.label_rows(rows, header)

    for row in cast(Iterator[_RawRow], labeled_rows):
        yield Government(
            code=row["Government"],
            name=row["Government Type"],
            description=row["Description"],
            examples=row["Examples"],
            example_contaband=parseutil.parse_set(row["Example Contraband"]),
        )