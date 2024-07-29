const skills = (function () {
    exports = {};

    const SPECIALITIES_HDR = "Specialities";

    // The line contains 1-2 words in Title Case.
    function isSkillHeading(line) {
        const words = line.split(" ");
        if (words.length < 1 || words.length > 2) {
            return false;
        }

        if (words.length == 1 && words[0] == SPECIALITIES_HDR) {
            return false;
        }

        return words.every((word) => word.match(/[A-Z][a-z]+/));
    }

    function isNewSpeciality(line) {
        return !!line.match(/^• /);
    }

    function isNewExample(line) {
        return !!line.match(/:/);
    }

    function toCsv(skills) {
        const table = [["Skill", "Speciality", "Description"]];
        for (const skill of skills) {
            table.push([skill.name, "", skill.desc]);
            for (const spec of skill.specs) {
                table.push(["", spec.name, spec.desc]);
            }
        }
        return table;
    }
    travdata.regExport(exports, toCsv);

    function parse(tables) {
        let table = travdata.concatTableData(tables);

        // Normalise some parts that extract bullet markers into two columns,
        // such that we get a single column.
        const lines = table.map((row) => {
            switch (row.length) {
                case 0:
                    return "";
                case 1:
                    return row[0];
                default:
                    return row.filter((cell) => cell.length > 0).join(" ");
            }
        });

        // Accumulate specialisation.
        let specs = [];
        let specLine = [];
        function flushSpec() {
            if (specLine.length == 0) {
                return;
            }

            const line = specLine.join(" ");
            const match = line.match(/• ([^:]+): (.+)/);

            specs.push({
                name: match[1],
                desc: match[2],
            });

            specLine = [];
        }

        // Accumulate skill.
        const skills = [];
        let skillName = "";
        let skillDesc = [];
        function flushSkill() {
            if (skillName == "") {
                return;
            }

            flushSpec();
            skills.push({
                name: skillName,
                desc: skillDesc.join(" "),
                specs: specs,
            });

            skillName = "";
            skillDesc = [];
            specs = [];
        }

        /**
         * @typedef StateFn
         * @type {Function}
         */

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function stateFindSkill(line) {
            return stateFindSkill;
        }

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function stateSkillDesc(line) {
            if (line == SPECIALITIES_HDR) {
                return stateFindSpec;
            } else if (isNewExample(line)) {
                return stateFindSkill;
            }
            skillDesc.push(line);
            return stateSkillDesc;
        }

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function stateFindSpec(line) {
            if (isNewSpeciality(line)) {
                specLine.push(line);
                return stateSpeciality;
            }
            // Still looking for first speciality.
            return stateFindSpec;
        }

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function stateSpeciality(line) {
            if (isNewSpeciality(line)) {
                flushSpec();
                specLine.push(line);
            } else if (isNewExample(line)) {
                flushSpec();
                return stateExample;
            } else {
                specLine.push(line);
            }
            return stateSpeciality;
        }

        /**
         * @param {string} line
         * @returns {StateFn}
         */
        function stateExample(line) {
            if (isNewSpeciality(line)) {
                specLine.push(line);
                return stateSpeciality;
            }
            return stateExample;
        }

        let state = stateFindSkill;
        for (const line of lines) {
            if (isSkillHeading(line)) {
                // New skill.
                flushSkill();
                skillName = line;
                state = stateSkillDesc;
                continue;
            }
            state = state(line);
        }
        flushSkill();

        return skills;
    }
    travdata.regExport(exports, parse);

    return exports;
})();
