# -*- coding: utf-8 -*-
"""General datatypes, non-specific to any specific area."""

import dataclasses
import re
from typing import Any, ClassVar, Optional, TypeVar

from travdata.datatypes import yamlcodec

T = TypeVar("T")


_FULL_RANGE_RX = re.compile("(\\d+)[-\u2013](\\d+)")
_MAX_RANGE_RX = re.compile("(\\d+)[-\u2013]")
_MIN_RANGE_RX = re.compile("(\\d+)[+]")


@dataclasses.dataclass
@yamlcodec.register_type
class IntRange:
    """Inclusive integer range [self.min_value, self.max_value]."""

    min_value: Optional[int]
    max_value: Optional[int]

    @classmethod
    def parse(cls, v: str) -> "IntRange":
        """Parses an IntRange.

        The string can be any of the forms:

        - ``""`` Matches any integer.
        - ``"a+"`` Matches any integer >= a.
        - ``"a-"`` Matches any integer <= a.
        - ``"a-b"`` Matches any integer between a and b inclusive.

        :param v: The string to parse, as per the above.
        :return: Parsed IntRange.
        """
        if not v:
            return cls(None, None)
        if match := _MIN_RANGE_RX.fullmatch(v):
            min_value = int(match.group(1))
            return cls(min_value=min_value, max_value=None)
        if match := _MAX_RANGE_RX.fullmatch(v):
            max_value = int(match.group(1))
            return cls(min_value=None, max_value=max_value)
        if match := _FULL_RANGE_RX.fullmatch(v):
            min_s, max_s = match.group(1, 2)
            return cls(min_value=int(min_s), max_value=int(max_s))
        v_int = int(v)
        return cls(min_value=v_int, max_value=v_int)

    def __str__(self) -> str:
        match self:
            case IntRange(min_value=None, max_value=None):
                return ""
            case IntRange(min_value=min_value, max_value=None):
                return f"{min_value}+"
            case IntRange(min_value=None, max_value=max_value):
                return f"{max_value}-"
            case IntRange(min_value=min_value, max_value=max_value) if min_value == max_value:
                return str(min_value)
            case IntRange(min_value=min_value, max_value=max_value):
                return f"{min_value}-{max_value}"
            case _:
                raise TypeError(self)

    def __contains__(self, v: Any) -> bool:
        if not isinstance(v, (int, float)):
            return False
        if self.min_value is not None and v < self.min_value:
            return False
        if self.max_value is not None and v > self.max_value:
            return False
        return True


@dataclasses.dataclass
@yamlcodec.register_type
class IntRangeSet:
    """A set of inclusive integer ranges."""

    yaml_tag: ClassVar = "!IntRangeSet"
    ranges: list[IntRange] = dataclasses.field(default_factory=list)

    @classmethod
    def parse(cls, s: str) -> "IntRangeSet":
        """Parses the range set.

        Note that the empty string deliberately evaluates to a single item that
        matches any integer. If this is undesired, then the caller may wish to
        handle the empty string specially.

        :param s: The string to parse. Must be a comma-delimited list of ranges
        (see ``IntRange``). Whitespace around each item is accepted and
        discarded.
        :return: Parsed IntRangeSet.
        """
        return IntRangeSet([IntRange.parse(rs.strip()) for rs in s.split(",")])

    def __str__(self) -> str:
        return ", ".join(str(r) for r in self.ranges)

    def __contains__(self, v: Any) -> bool:
        return any(v in r for r in self.ranges)

    @classmethod
    def to_yaml(cls, representer, node):
        """Implements ruamel.yaml serialisation."""
        return representer.represent_sequence(cls.yaml_tag, node.ranges)

    @classmethod
    def from_yaml(cls, constructor, node):
        """Implements ruamel.yaml serialisation."""
        del constructor  # unused
        return cls(node.value)
