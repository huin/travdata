# -*- coding: utf-8 -*-
import dataclasses
import io
from typing import Callable, Iterable

from travdata import jsonenc
from travdata.extractors import governments, lawlevels, tradecodes, tradegoods


@dataclasses.dataclass
class Converter:
    name: str
    description: str
    fn: Callable[[io.TextIOBase], Iterable[jsonenc.Encodable]]


CONVERTERS: list[Converter] = [
    Converter(
        name="governments",
        description="government types",
        fn=governments.convert_from_csv,
    ),
    Converter(
        name="law-levels",
        description="law levels",
        fn=lawlevels.convert_from_csv,
    ),
    Converter(
        name="trade-codes",
        description="trade types",
        fn=tradecodes.convert_from_csv,
    ),
    Converter(
        name="trade-goods",
        description="trade good types",
        fn=tradegoods.convert_from_csv,
    ),
]
