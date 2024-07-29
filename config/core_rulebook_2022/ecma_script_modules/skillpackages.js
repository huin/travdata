const skillpackages = (function () {
    exports = {};

    const PKG_HEADER_SUFFIX = " SKILLS PACKAGE";
    const SKILLS_HDR = "Skills:";

    /**
     * Returns name of the package if the line is a package name heading,
     * otherwise null.
     * @param {string} line
     * @returns {string}
     */
    function parsePackageHeading(line) {
        if (!line.endsWith(PKG_HEADER_SUFFIX)) {
            return null;
        }
        return line.slice(0, line.length - PKG_HEADER_SUFFIX.length);
    }

    function toCsv(packages) {
        const table = [["Package", "Skill", "Level", "Description"]];
        for (const pkg of packages) {
            table.push([pkg.name, "", "", pkg.description]);
            for (const skill of pkg.skills) {
                table.push(["", skill.name, "" + skill.level, ""]);
            }
        }
        return table;
    }
    travdata.regExport(exports, toCsv);

    /**
     * @typedef StateFn
     * @type {Function}
     */

    function parse(tables) {
        /**
         * @type {Object[]}
         */
        let packages = [];

        let packageName = "";
        let descLines = [];
        let skillLines = [];
        function flushPackage() {
            if (packageName == "") {
                return;
            }

            const skills = [
                ...skillLines.join(" ").matchAll(/([^,\d]+)(\d+)/g),
            ].map((match) => {
                return { name: match[1].trim(), level: parseInt(match[2]) };
            });
            packages.push({
                name: packageName,
                description: descLines.join(" "),
                skills: skills,
            });

            packageName = "";
            descLines = [];
            skillLines = [];
        }

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function stateFindPkgHeading(line) {
            if (parsePackageHeading(line)) {
                return statePkgHeading(line);
            }
            return stateFindPkgHeading;
        }

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function statePkgHeading(line) {
            packageName = parsePackageHeading(line);
            return statePkgDescription;
        }

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function statePkgDescription(line) {
            if (line == SKILLS_HDR) {
                return stateSkills;
            }
            descLines.push(line);
            return statePkgDescription;
        }

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function stateSkills(line) {
            if (parsePackageHeading(line)) {
                flushPackage();
                return statePkgHeading(line);
            }
            skillLines.push(line);
            return stateSkills;
        }

        let table = travdata.concatTableData(tables);
        let lines = table.map((row) => row.join(" "));

        let state = stateFindPkgHeading;
        for (const line of lines) {
            state = state(line);
        }
        flushPackage();

        return packages;
    }
    travdata.regExport(exports, parse);

    return exports;
})();
