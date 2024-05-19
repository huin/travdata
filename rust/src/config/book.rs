use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

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

impl YamlTable {
    /// Creates a `Table` from `self`.
    ///
    /// * `name` name of the table within the parent `YamlGroup.tables`.
    /// * `rel_group_dir` path to the directory of the table's parent
    /// `YamlGroup`.
    /// * `parent_tags` tags to inherit from parent `YamlGroup`.
    pub fn prepare(self, name: &str, rel_group_dir: &Path, parent_tags: &HashSet<String>) -> Table {
        let tags = self.tags.union(parent_tags).cloned().collect();
        Table {
            file_stem: rel_group_dir.join(name),
            tags,
            extraction_enabled: self.extraction_enabled,
            extraction: self.extraction,
        }
    }
}

pub struct Table {
    pub file_stem: PathBuf,
    pub tags: HashSet<String>,
    pub extraction_enabled: bool,
    pub extraction: tableextract::TableExtraction,
}

#[derive(Deserialize, Debug)]
pub struct YamlGroup {
    #[serde(default = "Default::default")]
    pub tags: HashSet<String>,
    #[serde(default = "Default::default")]
    pub groups: HashMap<String, YamlGroup>,
    #[serde(default = "Default::default")]
    pub tables: HashMap<String, YamlTable>,
}

impl YamlGroup {
    /// Creates a `Group` from `self`.
    ///
    /// * `rel_dir` Path to this group's directory.
    /// * `parent_tags` tags to inherit from parent `YamlGroup`.
    pub fn prepare(self, rel_dir: PathBuf, parent_tags: &HashSet<String>) -> Group {
        let tags: HashSet<String> = self.tags.union(parent_tags).cloned().collect();
        let tables = self
            .tables
            .into_iter()
            .map(|(name, yaml_table)| {
                let table = yaml_table.prepare(&name, &rel_dir, &tags);
                (name, table)
            })
            .collect();
        let groups = self
            .groups
            .into_iter()
            .map(|(name, yaml_group)| {
                let child_rel_dir = rel_dir.join(&name);
                (name, yaml_group.prepare(child_rel_dir, &tags))
            })
            .collect();
        Group {
            rel_dir,
            tags,
            tables,
            groups,
        }
    }
}

pub struct Group {
    pub rel_dir: PathBuf,
    pub tags: HashSet<String>,
    pub groups: HashMap<String, Group>,
    pub tables: HashMap<String, Table>,
}
