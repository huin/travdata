const travdata = (function () {
  exports = {};

  /**
   * Registers a function for export. 
   * @param {Function} fn Function to export.
   */
  function regExport(exports, fn) {
    exports[fn.name] = fn;
  }
  regExport(exports, regExport);

  /**
   * @typedef TableData
   * @type {string[][]}
   */

  /**
   * @typedef ExtractedTable
   * @type {object}
   * @property {TableData} data
   */

  /**
   * Concatenates the given tables.
   * @param {TableData[]} tables Tables to concatenate.
   * @returns {TableData} Result of concatenation.
   */
  function concatTableData(tables) {
    const result = [];
    for (const table of tables) {
      result.splice(result.length, 0, ...table);
    }
    return result;
  };
  regExport(exports, concatTableData);

  /**
   * Returns an array of the `data` property of the given extracted tables.
   * @param {ExtractedTable[]} extTables
   * @returns {TableData[]}
   */
  function tableData(extTables) {
    const tables = [];
    for (const extTable of extTables) {
      tables.push(extTable.data);
    }
    return tables;
  };
  regExport(exports, tableData);

  return exports;
})();
