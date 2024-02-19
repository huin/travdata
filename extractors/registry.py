# -*- coding: utf-8 -*-
import dataclasses
from typing import Callable, Iterable

import jsonenc
from extractors import governments, lawlevels, params, tradecodes, tradegoods


@dataclasses.dataclass
class Extractor:
    name: str
    fn: Callable[[params.CoreParams], Iterable[jsonenc.Encodable]]


EXTRACTORS: list[Extractor] = [
    Extractor("governments", governments.extract_from_pdf),
    Extractor("law_levels", lawlevels.extract_from_pdf),
    Extractor("trade_codes", tradecodes.extract_from_pdf),
    Extractor("trade_goods", tradegoods.extract_from_pdf),
]
