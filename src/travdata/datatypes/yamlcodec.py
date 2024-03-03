# -*- coding: utf-8 -*-

from typing import TypeVar

from ruamel import yaml

T = TypeVar("T")

# Provides a YAML instance that will safely read and write datatypes.
DATATYPES_YAML = yaml.YAML(typ="pure")
DATATYPES_YAML.indent(mapping=2, sequence=4, offset=2)


def register_type(t: type[T]) -> type[T]:
    return yaml.yaml_object(DATATYPES_YAML)(t)
