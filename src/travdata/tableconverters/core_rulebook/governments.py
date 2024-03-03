# -*- coding: utf-8 -*-
from typing import Iterable, Iterator, Optional, TypedDict, cast

from travdata import parseutil
from travdata.datatypes.core import worldcreation

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


def convert_from_rows(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[worldcreation.Government]:
    for row in cast(Iterable[_RawRow], rows):
        yield worldcreation.Government(
            code=row["Government"],
            name=row["Government Type"],
            description=row["Description"],
            examples=row["Examples"],
            example_contaband=parseutil.parse_set(row["Example Contraband"]),
        )
