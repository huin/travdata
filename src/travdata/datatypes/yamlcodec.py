# -*- coding: utf-8 -*-
"""Registry for datatypes to support YAML serialising and deserialising."""

from typing import TypeVar

from ruamel import yaml

T = TypeVar("T")

# Provides a YAML instance that will safely read and write datatypes.
DATATYPES_YAML = yaml.YAML(typ="safe")
DATATYPES_YAML.indent(mapping=2, sequence=4, offset=2)


def register_type(t: type[T]) -> type[T]:
    """Registers the given type with the YAML codec."""
    return yaml.yaml_object(DATATYPES_YAML)(t)
