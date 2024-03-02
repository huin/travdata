# -*- coding: utf-8 -*-
import dataclasses
import io
from typing import Callable, Iterable, Optional

from travdata import jsonenc


@dataclasses.dataclass
class Converter:
    name: str
    description: str
    fn: Callable[[Iterable[dict[str, Optional[str]]]], Iterable[jsonenc.Encodable]]
