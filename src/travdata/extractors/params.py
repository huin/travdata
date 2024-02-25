# -*- coding: utf-8 -*-
import dataclasses
import pathlib


@dataclasses.dataclass
class CoreParams:
    core_rulebook: pathlib.Path
    templates_dir: pathlib.Path
