# -*- coding: utf-8 -*-
"""Converts trade CSV data into Python data types."""

import re
from typing import Iterable, Iterator, Optional, TypedDict, cast

from travdata.datatypes.core import trade
from travdata.extraction import parseutil
from travdata.tableconverters.core import registry

_register_conv = registry.CONVERTERS.make_group_decorator(trade.GROUP)

_DM_ITEM_RX = re.compile(r"(.+) ([-+]\d+)")


def _parse_trade_dm(s: str) -> dict[str, int]:
    result: dict[str, int] = {}
    for item in s.split(","):
        match = _DM_ITEM_RX.fullmatch(item)
        if not match:
            raise ValueError(item)
        name, dm = match.group(1, 2)
        result[name.strip()] = int(dm)
    return result


@_register_conv("trade-goods")
def trade_goods(rows: Iterable[dict[str, Optional[str]]]) -> Iterator[trade.TradeGood]:
    """Parses trade goods CSV data into TradeGood structures.

    :param rows: Input rows from CSV file.
    :yield: TradeGood objects.
    """
    raw_row = TypedDict(
        "raw_row",
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
    for row in cast(Iterable[raw_row], rows):
        if row["Base Price"] is None:
            properties = None
            description = row["Availability"]
        else:
            properties = trade.TradeGoodProperties(
                availability=parseutil.parse_set(row["Availability"]),
                tons=row["Tons"],
                base_price=parseutil.parse_credits(row["Base Price"]),
                purchase_dm=_parse_trade_dm(row["Purchase DM"]),
                sale_dm=_parse_trade_dm(row["Sale DM"]),
                examples=row["Examples"],
            )
            description = None
        yield trade.TradeGood(
            d66=row["D66"],
            name=row["Type"],
            description=description,
            properties=properties,
        )
