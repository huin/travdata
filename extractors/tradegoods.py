# -*- coding: utf-8 -*-
import dataclasses
import re
from typing import Iterable, Iterator, Optional, TypedDict, cast

import jsonenc
import parseutil
import tabulautil
from extractors import params


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class TradeGoodProperties(jsonenc.Decodable, jsonenc.Encodable):
    availability: set[str]
    tons: str
    base_price: str
    purchase_dm: dict[str, int]
    sale_dm: dict[str, int]
    examples: str

    @classmethod
    def json_type(cls) -> str:
        return "TradeGoodProperties"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "TradeGoodProperties":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class TradeGood(jsonenc.Decodable, jsonenc.Encodable):
    d66: str
    name: str
    description: Optional[str]
    properties: Optional[TradeGoodProperties]

    @classmethod
    def json_type(cls) -> str:
        return "TradeGood"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "TradeGood":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


_DM_ITEM_RX = re.compile(r"(.+) ([-+]\d+)")


def _parse_trade_dm(s: str) -> dict[str, int]:
    s = parseutil.clean_text(s)
    result: dict[str, int] = {}
    for item in s.split(","):
        match = _DM_ITEM_RX.fullmatch(item)
        if not match:
            raise ValueError(item)
        name, dm = match.group(1, 2)
        result[name.strip()] = int(dm)
    return result


_RawRow = TypedDict(
    "_RawRow",
    {
        "D66": str,
        "Type": str,
        "Availability": str,
        "Tons": str,
        "Base Price": str,
        "Purchase DM": str,
        "Sale DM": str,
        "Examples": str,
    },
    total=True,
)


def _preprocess_rows(
    rows: Iterable[tabulautil.TabularRow],
) -> Iterator[list[str]]:
    text_rows = tabulautil.table_rows_text(rows)
    text_rows = parseutil.amalgamate_streamed_rows(
        rows=text_rows,
        # The first column is always empty on subsequent continuation rows.
        continuation=lambda _, row: row[0] == "",
    )
    return parseutil.clean_rows(text_rows)


def _extract_rows(
    param: params.CoreParams,
) -> Iterator[TradeGood]:
    rows_list: list[parseutil.TabularRow] = tabulautil.table_rows_concat(
        tabulautil.read_pdf_with_template(
            pdf_path=param.core_rulebook,
            template_path=param.templates_dir / "trade-goods.json",
        )
    )

    rows = _preprocess_rows(rows_list)
    header, rows = parseutil.headers_and_iter_rows(rows)
    labeled_rows = parseutil.label_rows(rows, header)

    for row in cast(Iterator[_RawRow], labeled_rows):
        yield TradeGood(
            d66=row["D66"],
            name=row["Type"],
            description=None,
            properties=TradeGoodProperties(
                availability=parseutil.parse_set(row["Availability"]),
                tons=row["Tons"],
                base_price=row["Base Price"],
                purchase_dm=_parse_trade_dm(row["Purchase DM"]),
                sale_dm=_parse_trade_dm(row["Sale DM"]),
                examples=row["Examples"],
            ),
        )


_SpecialRawRow = TypedDict(
    "_SpecialRawRow",
    {
        "D66": str,
        "Type": str,
        "Description": str,
    },
    total=True,
)


def _extract_special_rows(
    param: params.CoreParams,
) -> Iterator[TradeGood]:
    rows_list = tabulautil.table_rows_concat(
        tabulautil.read_pdf_with_template(
            pdf_path=param.core_rulebook,
            template_path=param.templates_dir / "trade-goods-special.json",
        )
    )

    header = ["D66", "Type", "Description"]
    rows = _preprocess_rows(rows_list)
    labeled_rows = parseutil.label_rows(rows, header)

    for special_row in cast(Iterator[_SpecialRawRow], labeled_rows):
        yield TradeGood(
            d66=parseutil.clean_text(special_row["D66"]),
            name=parseutil.clean_text(special_row["Type"]),
            description=parseutil.clean_text(special_row["Description"]),
            properties=None,
        )


def extract_from_pdf(
    param: params.CoreParams,
) -> Iterator[TradeGood]:
    yield from _extract_rows(param)
    yield from _extract_special_rows(param)
