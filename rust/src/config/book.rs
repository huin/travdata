use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::extraction::tableextract;

#[derive(Deserialize, Debug)]
pub struct YamlTable {
    #[serde(default = "Default::default")]
    pub tags: HashSet<String>,
    #[serde(default = "default_true")]
    pub extraction_enabled: bool,
    #[serde(default = "Default::default")]
    pub extraction: tableextract::TableExtraction,
}

fn default_true() -> bool {
    true
}

// def prepare(
//     self,
//     name: str,
//     rel_group_dir: pathlib.PurePath,
//     parent_tags: set[str],
// ) -> Table:
//     """Creates a ``Table`` from self.

//     :param name: Name of the table within its ``Group.groups``.
//     :param rel_group_dir: Path to the directory of the table's parent
//     group's directory, relative to the top-level config directory.
//     :param parent_tags: Tags to inherit from parent ``Group``.
//     :return: Prepared ``Table``.
//     """
//     tags = self.tags | parent_tags
//     return Table(
//         file_stem=rel_group_dir / name,
//         tags=tags,
//         extraction=self.extraction,
//     )

#[derive(Deserialize, Debug)]
pub struct YamlGroup {
    #[serde(default = "Default::default")]
    pub tags: HashSet<String>,
    #[serde(default = "Default::default")]
    pub groups: HashMap<String, YamlGroup>,
    #[serde(default = "Default::default")]
    pub tables: HashMap<String, YamlTable>,
}

// def prepare(
//     self,
//     rel_group_dir: pathlib.PurePath,
//     parent_tags: set[str],
// ) -> Group:
//     """Creates a ``Group`` from self.

//     :param rel_group_dir: Path to the directory of this group's directory,
//     relative to the top-level config directory.
//     :param parent_tags: Tags to inherit from parent ``Group``.
//     :return: Prepared ``Group``.
//     """
//     tags = self.tags | parent_tags
//     return Group(
//         rel_dir=rel_group_dir,
//         tags=tags,
//         tables={
//             name: table.prepare(name, rel_group_dir, parent_tags=tags)
//             for name, table in self.tables.items()
//         },
//         groups={
//             name: group.prepare(rel_group_dir / name, parent_tags=tags)
//             for name, group in self.groups.items()
//         },
//         # templates not included, as it is only for use in anchoring and
//         # aliasing by the cfgyaml.YAML.file author at the time of YAML parsing.
//     )
