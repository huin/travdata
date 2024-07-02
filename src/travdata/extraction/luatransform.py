# -*- coding: utf-8 -*-
"""Lua transform support."""

from typing import Any

import lupa  # type: ignore[import-untyped]


def _attribute_filter(obj: Any, attr_name: Any, is_setting: bool) -> None:
    raise AttributeError()


def _attr_getter(obj: Any, attr_name: Any) -> None:
    raise AttributeError()


def _attr_setter(obj: Any, attr_name: Any, value: Any) -> None:
    raise AttributeError()


def make_lua_runtime() -> lupa.LuaRuntime:
    """Create a Lua runtime."""
    return lupa.LuaRuntime(
        encoding="utf-8",
        source_encoding="utf-8",
        # attribute_filter=_attribute_filter,
        attribute_handlers=(_attr_getter, _attr_setter),
        register_eval=False,
        register_builtins=False,
    )
