# -*- coding: utf-8 -*-
"""ECMAScript-based extracted table transformations."""

import contextlib
import json
import pathlib
import sys
from typing import Any, Iterator, Protocol

import STPyV8  # type: ignore[import-untyped]

from travdata import filesio
from travdata import tabledata
from travdata.config import cfgerror
from travdata.tabledata import TableData


class ESTransformer(Protocol):
    """Protocol provided for ECMAScript transformations."""

    def load_module(
        self,
        path: pathlib.PurePath,
    ) -> None:
        """Loads a module.

        :param path: Path to the module souce code within the ``cfg_reader``.
        """
        ...

    def transform(
        self,
        tables: list[TableData],
        source: str,
    ) -> TableData:
        """Transforms extracted tables.

        :param tables: Tables to transform.
        :param expression: ECMAScript expression that performs the
        transformations.
        :return: Transformed tables.
        """
        ...


class _ESTransformer(ESTransformer):
    _cfg_reader: filesio.Reader
    _ctxt: STPyV8.JSContext
    _transform_entry: STPyV8.JSFunction

    def __init__(
        self,
        cfg_reader: filesio.Reader,
        ctxt: STPyV8.JSContext,
    ) -> None:
        """Initialise the transformer.

        :param cfg_reader: Reader to read modules from.
        :param ctxt: An active ``JSContext``.
        """
        self._cfg_reader = cfg_reader
        self._ctxt = ctxt
        self._transform_entry = self._ctxt.eval(
            """\
(tablesJson, source) => {
    const fn = Function("tables", `"use strict"; ${source}`);

    const tables = JSON.parse(tablesJson);
    const result = fn(tables);
    return JSON.stringify(result);
}
""",
        )

    def load_module(
        self,
        path: pathlib.PurePath,
    ) -> None:
        """Loads a module.

        :param path: Path to the module souce code within the ``cfg_reader``.
        """
        with self._cfg_reader.open_read(path) as r:
            source = r.read()
        self._ctxt.eval(source=source, name=str(path), line=0, col=0)

    def transform(
        self,
        tables: list[TableData],
        source: str,
    ) -> TableData:
        """Transforms extracted tables.

        :param tables: Tables to transform.
        :param source: ECMAScript code for a function body that performs the
        transformations and returns the result.
        :return: Transformed tables.
        """
        tables_json = json.dumps(tables)
        try:
            result_json = self._transform_entry(tables_json, source)
        except (ReferenceError, STPyV8.JSError, SyntaxError, TypeError) as e:
            raise cfgerror.ConfigurationError(str(e)) from e
        if not isinstance(result_json, str):
            raise cfgerror.ConfigurationError(
                f"ESTransform returned non-string type {type(result_json).__name__}",
            )
        result = json.loads(result_json)

        try:
            tabledata.check_table_type(result)
        except TypeError as e:
            raise cfgerror.ConfigurationError(str(e)) from e
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


@contextlib.contextmanager
def transformer(
    cfg_reader: filesio.Reader,
) -> Iterator[ESTransformer]:
    """Context manager for an ECMAScript based transformer.

    :param cfg_reader: Used to read ECMAScript modules.
    :yield: A ``Transformer`` ready for use.
    """
    with STPyV8.JSContext(_Globals()) as ctxt:
        yield _ESTransformer(cfg_reader, ctxt)
