# -*- coding: utf-8 -*-
"""Jsonnet-based extracted table transformations."""

import json

import _jsonnet as jsonnetlib

from travdata.config import cfgerror, cfgextract
from travdata.extraction.pdf import tablereader
from travdata import tabledata
from travdata.tabledata import TableData


def perform_transforms(
    cfg: cfgextract.JsonnetExtraction,
    extracted_tables: list[tablereader.ExtractedTable],
) -> TableData:
    """Transforms the rows based on the configured Jsonnet."""
    result_json = jsonnetlib.evaluate_snippet(
        "JsonnetExtraction",
        cfg.code,
        tla_codes={
            "tables": json.dumps(extracted_tables),
        },
    )
    result = json.loads(result_json)

    # Type check the result.
    try:
        tabledata.check_table_type(result)
    except TypeError as e:
        raise cfgerror.ConfigurationError(str(e)) from e

    return result
