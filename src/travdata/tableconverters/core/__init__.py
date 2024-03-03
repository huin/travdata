# -*- coding: utf-8 -*-
from travdata.tableconverters import Converter
from travdata.tableconverters.core import (governments, lawlevels, tradecodes,
                                           tradegoods)

CONVERTERS: list[Converter] = [
    Converter(
        name="governments",
        description="government types",
        fn=governments.convert_from_rows,
    ),
    Converter(
        name="law-levels",
        description="law levels",
        fn=lawlevels.convert_from_rows,
    ),
    Converter(
        name="trade-codes",
        description="trade types",
        fn=tradecodes.convert_from_rows,
    ),
    Converter(
        name="trade-goods",
        description="trade good types",
        fn=tradegoods.convert_from_rows,
    ),
]
