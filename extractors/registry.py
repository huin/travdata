# -*- coding: utf-8 -*-
import dataclasses
from typing import Callable, Iterable

import jsonenc
from extractors import governments, lawlevels, params, tradecodes, tradegoods


@dataclasses.dataclass
class Extractor:
    name: str
    description: str
    fn: Callable[[params.CoreParams], Iterable[jsonenc.Encodable]]


EXTRACTORS: list[Extractor] = [
    Extractor(
        name="governments",
        description="government types",
        fn=governments.extract_from_pdf,
    ),
    Extractor(
        name="law_levels",
        description="law levels",
        fn=lawlevels.extract_from_pdf,
    ),
    Extractor(
        name="trade_codes",
        description="trade types",
        fn= tradecodes.extract_from_pdf,
    ),
    Extractor(
        name="trade_goods",
        description="trade good types",
        fn= tradegoods.extract_from_pdf,
    ),
]
