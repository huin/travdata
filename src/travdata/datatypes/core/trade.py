# -*- coding: utf-8 -*-
"""Data types relating to trade."""

import dataclasses
from typing import Optional

GROUP = "trade"


@dataclasses.dataclass
class TradeGoodProperties:
    """Specific about a single trade good that are not present for all trade goods."""

    availability: set[str]
    tons: str
    base_price: int
    purchase_dm: dict[str, int]
    sale_dm: dict[str, int]
    examples: str


@dataclasses.dataclass
class TradeGood:
    """Data about a single trade good."""

    d66: str
    name: str
    description: Optional[str]
    properties: Optional[TradeGoodProperties]
