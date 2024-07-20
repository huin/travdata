const equipment = (function () {
    const exports = {};

    /**
     * @param {TableData[]} tables.
     * @returns {TableData}
     */
    function armour(tables) {
        const table = travdata.concatTableData(tables);
        // Column 0 on rows 15 and 16 needs to be concatenated.
        table[15][0] = table[15][0] + " " + table[16][0];
        table[16][0] = "";
        // Treat empty column 0 as a "ditto", and copy the last value into the
        // empty cells.
        let lastCol0 = "";
        for (const row of table) {
            if (row[0] != "") {
                lastCol0 = row[0];
                continue;
            }
            row[0] = lastCol0;
        }
        return table;
    }
    travdata.regExport(exports, armour);

    return exports;
})();
