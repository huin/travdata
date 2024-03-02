# -*- coding: utf-8 -*-
import dataclasses
import io
from typing import Callable, Iterable

from travdata import jsonenc


@dataclasses.dataclass
class Converter:
    name: str
    description: str
    fn: Callable[[io.TextIOBase], Iterable[jsonenc.Encodable]]
