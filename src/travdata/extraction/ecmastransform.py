# -*- coding: utf-8 -*-
"""ECMAScript-based extracted table transformations."""


import json
import pathlib
from typing import Self

import STPyV8  # type: ignore[import-untyped]

from travdata import filesio
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
        self._ctxt = STPyV8.JSContext()

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
        expression: str,
    ) -> TableData:
        """Transforms extracted tables.

        :param ext_tables: Extracted tables to transform.
        :param expression: ECMAScript expression that performs the
        transformations.
        :return: Transformed tables.
        """
        ext_tables_json = json.dumps(ext_tables)
        fn = self._ctxt.eval(
            f"""\
(extTablesJson) => {{
    const extTables = JSON.parse(extTablesJson);
    const result = function (extTables) {{
        return (
{expression}
);
    }}(extTables);
    return JSON.stringify(result);
}}
""",
        )
        result_json = fn(ext_tables_json)
        return json.loads(result_json)
