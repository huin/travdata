# -*- coding: utf-8 -*-
# pylint: disable=missing-class-docstring,missing-function-docstring,missing-module-docstring,redefined-outer-name

import pathlib
from typing import Iterator
import hamcrest as hc
import pytest

from travdata import filesio
from travdata.extraction import ecmastransform
from .pdf import pdftestutil


@pytest.fixture
def cfg_reader() -> filesio.MemReader:
    return filesio.MemReader(
        {
            pathlib.PurePath(
                "module.js"
            ): """\
const module = function() {
  exports = {};

  exports.concatTableData = function (tables) {
    const result = [];
    for (const table of tables) {
      result.splice(result.length, 0, ...table);
    }
    return result;
  };

  exports.tableData = function (extTables) {
    const tables = [];
    for (const extTable of extTables) {
      tables.push(extTable.data);
    }
    return tables;
  };

  return exports;
}();
""",
        }
    )


@pytest.fixture
def trn(cfg_reader: filesio.MemReader) -> Iterator[ecmastransform.Transformer]:
    with ecmastransform.transformer(cfg_reader=cfg_reader) as trn:
        yield trn


def test_evaluation(trn: ecmastransform.Transformer) -> None:
    actual = trn.transform(
        ext_tables=[],
        source='return [["foo", "bar"]];',
    )
    hc.assert_that(actual, hc.equal_to([["foo", "bar"]]))


def test_transform(trn: ecmastransform.Transformer) -> None:
    trn.load_module(pathlib.PurePath("module.js"))
    t1 = pdftestutil.fake_table_data(num_rows=1)["data"]
    t2 = pdftestutil.fake_table_data(num_rows=2)["data"]
    ext_tables = [
        pdftestutil.tabula_table_from_simple(1, t1),
        pdftestutil.tabula_table_from_simple(2, t2),
    ]
    actual = trn.transform(
        ext_tables=ext_tables,
        source="return module.concatTableData(module.tableData(extTables));",
    )
    hc.assert_that(actual, hc.equal_to(t1 + t2))
