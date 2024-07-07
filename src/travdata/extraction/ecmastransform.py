# -*- coding: utf-8 -*-
"""ECMAScript-based extracted table transformations."""


import json
import pathlib
import sys
from typing import Any, Self

import STPyV8  # type: ignore[import-untyped]

from travdata import filesio
from travdata import tabledata
from travdata.config import cfgerror
from travdata.extraction.pdf import tablereader
from travdata.tabledata import TableData


class EcmaScriptTransformer:
    """ECMAScript based transformer for tables."""

    _cfg_reader: filesio.Reader
    _ctxt: STPyV8.JSContext

    def __init__(self, cfg_reader: filesio.Reader) -> None:
        """Initialise the transformer.

        :param cfg_reader: Reader to read modules from.
        """
        self._cfg_reader = cfg_reader
        self._ctxt = STPyV8.JSContext(_Globals())

    def __enter__(self) -> Self:
        self._ctxt.__enter__()
        return self

    def __exit__(self, *args) -> None:
        self._ctxt.__exit__(*args)

    def load_module(
        self,
        path: pathlib.PurePath,
    ) -> None:
        """Loads a module.

        :param path: Path to the module souce code within the ``cfg_reader``.
        """
        with self._cfg_reader.open_read(path) as r:
            source = r.read()
        self._ctxt.eval(source=source, name=str(path), line=1, col=1)

    def transform(
        self,
        ext_tables: list[tablereader.ExtractedTable],
        source: str,
    ) -> TableData:
        """Transforms extracted tables.

        :param ext_tables: Extracted tables to transform.
        :param expression: ECMAScript expression that performs the
        transformations.
        :return: Transformed tables.
        """
        fn = self._ctxt.eval(
            """\
(extTablesJson, source) => {
    const fn = Function("extTables", `"use strict"; ${source}`);

    const extTables = JSON.parse(extTablesJson);
    const result = fn(extTables);
    return JSON.stringify(result);
}
""",
        )

        ext_tables_json = json.dumps(ext_tables)
        result_json = fn(ext_tables_json, source)
        if not isinstance(result_json, str):
            raise cfgerror.ConfigurationError(
                f"EcmaScriptTransform returned non-string type {type(result_json).__name__}",
            )
        result = json.loads(result_json)
        tabledata.check_table_type(result)
        return result


class _Globals(STPyV8.JSClass):
    """Provides global ``this`` values for a ``JSContext``."""

    def log(self, *args: Any) -> None:
        """Logs the given values to stderr."""
        msg = " ".join(str(arg) for arg in args)
        print(f"{msg}\n", file=sys.stderr)

    def isPrototypeOf(self, obj):
        # Required implementation by base class.
        del obj  # unused
        return False
