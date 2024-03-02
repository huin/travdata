# -*- coding: utf-8 -*-
import dataclasses
from typing import Iterable, Iterator, Optional, TypedDict, cast

from travdata import jsonenc, parseutil


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


def convert_from_rows(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[Government]:
    for row in cast(Iterable[_RawRow], rows):
        yield Government(
            code=row["Government"],
            name=row["Government Type"],
            description=row["Description"],
            examples=row["Examples"],
            example_contaband=parseutil.parse_set(row["Example Contraband"]),
        )
