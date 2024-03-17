# -*- coding: utf-8 -*-
"""Registry of datatype converters."""

import dataclasses
from typing import Any, Callable, Iterable, Optional, TypeAlias

Converter: TypeAlias = Callable[[Iterable[dict[str, Optional[str]]]], Iterable[Any]]
ConverterDecorator: TypeAlias = Callable[[Converter], Converter]


@dataclasses.dataclass(frozen=True)
class Key:
    """Identifies a table by group and name."""

    group_name: str
    table_name: str


@dataclasses.dataclass
class Registry:
    """Registry of datatype converters."""

    converters: dict[Key, Converter] = dataclasses.field(default_factory=dict)

    def make_group_decorator(self, group_name: str) -> Callable[[str], ConverterDecorator]:
        """Create a decorator factory for the given configuration group."""

        def make_decorator(table_name: str) -> ConverterDecorator:
            def decorator(fn: Converter) -> Converter:
                key = Key(group_name=group_name, table_name=table_name)
                self.converters[key] = fn
                return fn

            return decorator

        return make_decorator
