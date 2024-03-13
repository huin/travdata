# -*- coding: utf-8 -*-
"""Utility code for handling dataclasses."""
import dataclasses
from typing import Any


def shallow_asdict(dc: Any) -> dict[str, Any]:
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
