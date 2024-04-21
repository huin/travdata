# -*- coding: utf-8 -*-
"""Utility code for helping with ruamel.yaml."""

from __future__ import annotations

import dataclasses
from typing import Any, ClassVar, Iterator, Self, TypeVar, cast, TYPE_CHECKING

from ruamel import yaml
from travdata import dataclassutil
from travdata.config import cfgerror

_T = TypeVar("_T")


# Keys in dataclasses.field metadata used by this module:
YAML_NAME = "yaml"  # Override the name of the field in a YAML mapping.
TO_YAML = "to_yaml"  # Callable to convert to YAML value.
FROM_YAML = "from_yaml"  # Callable to convert from YAML value.


INT_METADATA = {TO_YAML: int, FROM_YAML: int}
SET_METADATA = {TO_YAML: sorted, FROM_YAML: set}


if TYPE_CHECKING:
    from _typeshed import DataclassInstance


def _check_node_type(
    yaml_tag: str,
    node: Any,
    want_type: type[_T],
) -> _T:
    if not isinstance(node, want_type):
        raise cfgerror.ConfigurationError(
            f"{node.start_mark.name}:{node.start_mark.line+1}:{node.start_mark.column+1}: "
            f"{yaml_tag} should be a {want_type.__name__}, but is {type(node).__name__}"
        )
    return node


def _yaml_field(cls: type) -> dataclasses.Field:
    fields = dataclasses.fields(cast(type["DataclassInstance"], cls))
    if len(fields) != 1:
        raise TypeError(f"expected exactly one field in {cls.__name__}, got {len(fields)}")
    return fields[0]


class YamlMappingMixin:
    """Mixin for classes instantiated by parsing YAML mappings."""

    yaml_tag: ClassVar

    @classmethod
    def yaml_create_empty(cls) -> Self:
        """Returns an "empty" instance of the class for YAML loading.

        Must be implemented by subclasses that have required fields. The
        returned value must have default values set on fields that have
        defaults.
        """
        return cls()

    @classmethod
    def to_yaml(cls, representer, node):
        """Implements serialising the node as basic YAML types."""
        mapping = {}
        for field in dataclasses.fields(cast(type["DataclassInstance"], cls)):
            value = getattr(node, field.name)
            if not value and dataclassutil.has_default(field):
                continue
            if fn := field.metadata.get(TO_YAML):
                value = fn(value)
            key = field.metadata.get(YAML_NAME, field.name)
            mapping[key] = value
        return representer.represent_mapping(cls.yaml_tag, mapping)

    @classmethod
    def from_yaml(cls, constructor, node) -> Iterator[Self]:
        """Implements deserialising the node from basic YAML types."""
        node = _check_node_type(cls.yaml_tag, node, yaml.MappingNode)

        obj = cls.yaml_create_empty()
        yield obj
        data = yaml.CommentedMap()
        constructor.construct_mapping(node, maptyp=data, deep=True)
        if not isinstance(data, dict):
            raise TypeError(data)
        for field in dataclasses.fields(cast(type["DataclassInstance"], cls)):
            key = field.metadata.get("yaml", field.name)
            try:
                value = data.pop(key)
            except KeyError as exc:
                if dataclassutil.has_default(field):
                    continue
                raise TypeError(
                    f"required field {field.name} not specified in {cls.yaml_tag}",
                ) from exc
            if fn := field.metadata.get(FROM_YAML):
                value = fn(value)
            setattr(obj, field.name, value)
        if data:
            names = ", ".join(sorted(data))
            raise TypeError(f"unexpected fields {names} in {cls.yaml_tag}")


class YamlScalarMixin:
    """Mixin for classes instantiated by parsing YAML scalars.

    Does not support non-empty defaults in the sense that there is no indication
    if the YAML value is an intended "falsy" scalar or intended to be a default.
    """

    yaml_tag: ClassVar

    @classmethod
    def yaml_create_empty(cls) -> Self:
        """Returns an "empty" instance of the class for YAML loading.

        Must be implemented by subclasses that have required fields. The
        returned value must have default values set on fields that have
        defaults.
        """
        return cls()

    @classmethod
    def to_yaml(cls, representer, node):
        """Implements serialising the node as basic YAML types."""

        field = _yaml_field(cls)

        scalar = getattr(node, field.name)
        if fn := field.metadata.get("to_yaml"):
            scalar = fn(scalar)

        return representer.represent_scalar(cls.yaml_tag, scalar)

    @classmethod
    def from_yaml(cls, constructor, node) -> Self:
        """Implements deserialising the node from basic YAML types."""
        del constructor  # unused
        node = _check_node_type(cls.yaml_tag, node, yaml.ScalarNode)
        value = node.value
        field = _yaml_field(cls)
        if fn := field.metadata.get("from_yaml"):
            value = fn(value)
        obj = cls.yaml_create_empty()
        setattr(obj, field.name, value)
        return obj


class YamlSequenceMixin:
    """Mixin for classes instantiated by parsing YAML sequences.

    Does not support non-empty defaults in the sense that there is no indication
    if the YAML value is an intended empty sequence or intended to be a default.
    """

    yaml_tag: ClassVar

    @classmethod
    def yaml_create_empty(cls) -> Self:
        """Returns an "empty" instance of the class for YAML loading.

        Must be implemented by subclasses that have required fields. The
        returned value must have default values set on fields that have
        defaults.
        """
        return cls()

    @classmethod
    def to_yaml(cls, representer, node):
        """Implements serialising the node as basic YAML types."""

        field = _yaml_field(cls)

        sequence = getattr(node, field.name)
        if fn := field.metadata.get("to_yaml"):
            sequence = fn(sequence)

        return representer.represent_sequence(cls.yaml_tag, sequence)

    @classmethod
    def from_yaml(cls, constructor, node) -> Iterator[Self]:
        """Implements deserialising the node from basic YAML types."""
        node = _check_node_type(cls.yaml_tag, node, yaml.SequenceNode)
        obj = cls.yaml_create_empty()
        yield obj
        sequence = constructor.construct_rt_sequence(
            node,
            seqtyp=yaml.CommentedSeq(),
            deep=True,
        )
        if not isinstance(sequence, list):
            raise TypeError(sequence)

        field = _yaml_field(cls)
        if fn := field.metadata.get("from_yaml"):
            sequence = fn(sequence)
        setattr(obj, field.name, sequence)
