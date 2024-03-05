# -*- coding: utf-8 -*-
from travdata import tableconverters

CONVERTERS = tableconverters.Registry()


def load_all_converters() -> None:
    """Import converters in all subpackages.

    This has the side effect of registering them in ``CONVERTERS``.
    """
    from travdata.tableconverters.core import (
        trade,
        worldcreation,
    )

    del trade, worldcreation
