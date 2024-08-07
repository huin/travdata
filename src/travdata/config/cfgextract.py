# -*- coding: utf-8 -*-
"""Defines dataclasses that configure table extraction."""

import abc
import dataclasses
from typing import ClassVar, Optional, Self

from travdata import yamlutil
from travdata.config import yamlreg


class LegacyTransform(abc.ABC):
    """Marker base class for configuration of a singular table transformations."""


@dataclasses.dataclass
@yamlreg.YAML.register_class
class ExpandColumnOnRegex(LegacyTransform, yamlutil.YamlMappingMixin):
    """Splits a column by the matches of a regex."""

    yaml_tag: ClassVar = "!ExpandColumnOnRegex"
    column: int
    pattern: str
    # When `pattern` matches (using `Pattern.fullmatch`), `on_match` produces
    # the resulting cells, using groups from the match to the pattern. Each
    # string is expanded using `Match.expand`, see
    # https://docs.python.org/3/library/re.html#match-objects.
    on_match: list[str]
    # When `pattern` does not match, default produces cells as if matching on a
    # regex ".*" when `pattern` does not match. Similarly, each string is
    # expanded using `Match.expand` (using \g<0> makes sense here to extract the
    # entire original text into a cell).
    default: list[str]

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(
            column=0,
            pattern="",
            on_match=[],
            default=[],
        )


@dataclasses.dataclass
@yamlreg.YAML.register_class
class PrependRow(LegacyTransform, yamlutil.YamlSequenceMixin):
    """Appends given literal row values to the start of a table."""

    yaml_tag: ClassVar = "!PrependRow"
    row: list[str]

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(row=[])


@dataclasses.dataclass
@yamlreg.YAML.register_class
class Transpose(LegacyTransform, yamlutil.YamlMappingMixin):
    """Transposes the table (rows become columns and vice versa)."""

    yaml_tag: ClassVar = "!Transpose"


class RowGrouper(abc.ABC):
    """Abstract base marker for configuring row grouping."""


@dataclasses.dataclass
@yamlreg.YAML.register_class
class AllRows(RowGrouper, yamlutil.YamlMappingMixin):
    """Specifies to group all remaining rows."""

    yaml_tag: ClassVar = "!AllRows"


@dataclasses.dataclass
@yamlreg.YAML.register_class
class StaticRowCounts(RowGrouper, yamlutil.YamlSequenceMixin):
    """Specifies explicit input row counts for output grouped rows."""

    yaml_tag: ClassVar = "!StaticRowCounts"
    row_counts: list[int]

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(row_counts=[])


@dataclasses.dataclass
@yamlreg.YAML.register_class
class EmptyColumn(RowGrouper, yamlutil.YamlScalarMixin):
    """Specifies to group rows by when a given column is empty."""

    yaml_tag: ClassVar = "!EmptyColumn"
    column_index: int = dataclasses.field(metadata=yamlutil.INT_METADATA)

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(column_index=0)


@dataclasses.dataclass
@yamlreg.YAML.register_class
class FoldRows(LegacyTransform, yamlutil.YamlSequenceMixin):
    """Folds rows, according to the given sequence of groupings."""

    yaml_tag: ClassVar = "!FoldRows"
    group_by: list[RowGrouper] = dataclasses.field(default_factory=list)


@dataclasses.dataclass
@yamlreg.YAML.register_class
class JoinColumns(LegacyTransform, yamlutil.YamlMappingMixin):
    """Joins a range of columns."""

    yaml_tag: ClassVar = "!JoinColumns"
    from_: Optional[int] = dataclasses.field(default=None, metadata={yamlutil.YAML_NAME: "from"})
    to: Optional[int] = None
    delim: str = ""


@dataclasses.dataclass
@yamlreg.YAML.register_class
class SplitColumn(LegacyTransform, yamlutil.YamlMappingMixin):
    """Splits a column on a pattern."""

    yaml_tag: ClassVar = "!SplitColumn"
    column: int
    pattern: str

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(column=0, pattern="")


@dataclasses.dataclass
@yamlreg.YAML.register_class
class WrapRowEveryN(LegacyTransform, yamlutil.YamlScalarMixin):
    """Wraps a row every N columns."""

    yaml_tag: ClassVar = "!WrapRowEveryN"
    columns: int = dataclasses.field(metadata=yamlutil.INT_METADATA)

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(columns=0)


class TableTransform(abc.ABC):
    """Marker base class for configuration of table transforms."""


@dataclasses.dataclass
@yamlreg.YAML.register_class
class LegacyTransformSeq(TableTransform, yamlutil.YamlSequenceMixin):
    """Legacy table transformation sequence."""

    yaml_tag: ClassVar = "!LegacyTransformSeq"
    transforms: list[LegacyTransform] = dataclasses.field(default_factory=list)


@dataclasses.dataclass
@yamlreg.YAML.register_class
class ESTransform(TableTransform, yamlutil.YamlMappingMixin):
    """ECMAScript based table transformation."""

    yaml_tag: ClassVar = "!ESTransform"
    src: str

    @classmethod
    def yaml_create_empty(cls) -> Self:
        return cls(src="")
