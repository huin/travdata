# -*- coding: utf-8 -*-
"""Registry of converters of core types."""

from travdata.tableconverters import registry

CONVERTERS = registry.Registry()


def load_all_converters() -> None:
    """Import converters in all subpackages.

    This has the side effect of registering them in ``CONVERTERS``.
    """
    # pylint: disable=cyclic-import,import-outside-toplevel
    from travdata.tableconverters.core import (
        trade,
        worldcreation,
    )

    # pylint: enable=cyclic-import,import-outside-toplevel

    del trade, worldcreation
