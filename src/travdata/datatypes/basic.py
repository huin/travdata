# -*- coding: utf-8 -*-
import dataclasses
from typing import Any, Optional, TypeVar

from travdata import jsonenc

T = TypeVar("T")


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class IntRange:
    """Inclusive integer range [self.min, self.max]."""

    min: Optional[int]
    max: Optional[int]

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
        range_hyphen = "\u2013"
        if not v:
            return cls(None, None)
        elif v.endswith("+"):
            min_value = int(v.removesuffix("+"))
            return cls(min=min_value, max=None)
        elif v.endswith(range_hyphen):
            max_value = int(v.removesuffix(range_hyphen))
            return cls(min=None, max=max_value)
        elif v.endswith("-"):
            max_value = int(v.removesuffix("-"))
            return cls(min=None, max=max_value)
        elif range_hyphen in v:
            min_s, _, max_s = v.partition(range_hyphen)
            return cls(min=int(min_s), max=int(max_s))
        elif "-" in v:
            min_s, _, max_s = v.partition("-")
            return cls(min=int(min_s), max=int(max_s))
        else:
            v_int = int(v)
            return cls(min=v_int, max=v_int)

    def __str__(self) -> str:
        match self:
            case IntRange(min=None, max=None):
                return ""
            case IntRange(min=min, max=None):
                return f"{min}+"
            case IntRange(min=None, max=max):
                return f"{max}-"
            case IntRange(min=min, max=max) if min == max:
                return str(min)
            case IntRange(min=min, max=max):
                return f"{min}-{max}"
            case _:
                raise TypeError(self)

    def __contains__(self, v: Any) -> bool:
        if not isinstance(v, (int, float)):
            return False
        if self.min is not None and v < self.min:
            return False
        if self.max is not None and v > self.max:
            return False
        return True

    @classmethod
    def json_type(cls) -> str:
        return "IntRange"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "IntRange":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)


@dataclasses.dataclass
@jsonenc.DEFAULT_CODEC.register_json_decodable
class IntRangeSet:
    """A set of inclusive integer ranges."""
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
    def json_type(cls) -> str:
        return "IntRangeSet"

    @classmethod
    def from_json(cls, o: jsonenc.Object) -> "IntRangeSet":
        return cls(**o)

    def to_json(self) -> jsonenc.Object:
        return jsonenc.dataclass_to_dict(self)
