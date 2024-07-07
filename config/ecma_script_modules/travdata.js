const travdata = (function () {
  exports = {};

  /**
   * @typedef TableData
   * @type {string[][]}
   */

  /**
   * @typedef ExtractedTable
   * @type {object}
   * @property {TableData} data
   */

  /***
   * Concatenates the given tables.
   * @param {TableData[]} tables Tables to concatenate.
   * @returns {TableData} Result of concatenation.
   */
  exports.concatTableData = function (tables) {
    const result = [];
    for (const table of tables) {
      result.splice(result.length, 0, ...table);
    }
    return result;
  };

  /**
   * Returns an array of the `data` property of the given extracted tables.
   * @param {ExtractedTable[]} extTables
   * @returns {TableData[]}
   */
  exports.tableData = function (extTables) {
    const tables = [];
    for (const extTable of extTables) {
      tables.push(extTable.data);
    }
    return tables;
  };

  return exports;
})();
