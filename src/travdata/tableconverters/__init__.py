# -*- coding: utf-8 -*-
import dataclasses
from typing import Any, Callable, Iterable, Optional


@dataclasses.dataclass
class Converter:
    name: str
    description: str
    fn: Callable[[Iterable[dict[str, Optional[str]]]], Iterable[Any]]
