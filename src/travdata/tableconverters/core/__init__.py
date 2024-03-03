# -*- coding: utf-8 -*-
from travdata.tableconverters import Converter
from travdata.tableconverters.core import trade, worldcreation

CONVERTERS: list[Converter] = [
    Converter(
        name="governments",
        description="government types",
        fn=worldcreation.governments,
    ),
    Converter(
        name="law-levels",
        description="law levels",
        fn=worldcreation.law_levels,
    ),
    Converter(
        name="trade-codes",
        description="trade types",
        fn=worldcreation.trade_codes,
    ),
    Converter(
        name="trade-goods",
        description="trade good types",
        fn=trade.trade_goods,
    ),
]
