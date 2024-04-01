# -*- coding: utf-8 -*-
"""Utility code for handling dataclasses."""

from __future__ import annotations

import dataclasses
from typing import Any, TYPE_CHECKING

if TYPE_CHECKING:
    from _typeshed import DataclassInstance


def shallow_asdict(dc: DataclassInstance) -> dict[str, Any]:
    """Shallow version of ``dataclasses.asdict``.

    ``dataclasses.asdict`` will recurse into child dataclasses of the subject.
    This version stops at the dataclass given, which is particularly useful when
    deserialising objects.

    :param dc: Dataclass instance to convert to a dictionary.
    :return: Dictionary, containing keys and values mirroring the attributes of
    ``dc``.
    """
    result: dict[str, Any] = {}
    for field in dataclasses.fields(dc):
        result[field.name] = getattr(dc, field.name)
    return result


def has_default(field: dataclasses.Field) -> bool:
    """Returns True if the given field has a default value."""
    return field.default != dataclasses.MISSING or field.default_factory != dataclasses.MISSING
