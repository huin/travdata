# -*- coding: utf-8 -*-
"""YAML registry for the travdata.config package."""

from ruamel import yaml

YAML = yaml.YAML()
# Retain the original ordering in mappings.
YAML.representer.sort_base_mapping_type_on_output = False
