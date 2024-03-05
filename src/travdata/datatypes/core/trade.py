# -*- coding: utf-8 -*-
import dataclasses
from typing import Optional

from travdata.datatypes import yamlcodec

GROUP = "trade"


@dataclasses.dataclass
@yamlcodec.register_type
class TradeGoodProperties:
    availability: set[str]
    tons: str
    base_price: int
    purchase_dm: dict[str, int]
    sale_dm: dict[str, int]
    examples: str


@dataclasses.dataclass
@yamlcodec.register_type
class TradeGood:
    d66: str
    name: str
    description: Optional[str]
    properties: Optional[TradeGoodProperties]
