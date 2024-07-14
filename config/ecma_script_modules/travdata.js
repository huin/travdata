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

  return exports;
})();
