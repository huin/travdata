use std::{
    collections::{hash_map, HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{extraction::tableextract, filesio::Reader};

pub fn load_book(
    cfg_reader: &dyn Reader,
    book_name: &str,
    parent_tags: &HashSet<String>,
) -> Result<Group> {
    let rel_book_dir: PathBuf = book_name.into();
    let config_path = rel_book_dir.join("book.yaml");

    let file = cfg_reader
        .open_read(&config_path)
        .with_context(|| format!("opening book configuration {:?}", &book_name))?;
    let yaml_group: YamlGroup =
        serde_yaml_ng::from_reader(file).with_context(|| "parsing book configuration")?;

    Ok(yaml_group.prepare(rel_book_dir, parent_tags))
}

#[derive(Deserialize, Debug)]
struct YamlTable {
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
    fn prepare(self, name: &str, rel_group_dir: &Path, parent_tags: &HashSet<String>) -> Table {
        let tags = self.tags.union(parent_tags).cloned().collect();
        Table {
            file_stem: rel_group_dir.join(name),
            tags,
            extraction_enabled: self.extraction_enabled,
            extraction: self.extraction,
        }
    }
}

/// Configuration for extraction of a table from a PDF.
pub struct Table {
    pub file_stem: PathBuf,
    pub tags: HashSet<String>,
    pub extraction_enabled: bool,
    pub extraction: tableextract::TableExtraction,
}

impl Table {
    /// Path to the Tabula template, assuming that it exists.
    pub fn tabula_template_path(&self) -> PathBuf {
        self.file_stem.with_extension("tabula-template.json")
    }
}

#[derive(Deserialize, Debug)]
struct YamlGroup {
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
    fn prepare(self, rel_dir: PathBuf, parent_tags: &HashSet<String>) -> Group {
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

/// Configuration for extraction of many tables from a PDF.
/// 
/// Each [Group] corresponds to a directory within the configuration directory.
/// The hierarchy of [Group]s is not significant, it is merely a convenience for
/// organising Tabula template files. However, it does provide inheritance of
/// tags during the import of [Group]s from YAML.
pub struct Group {
    pub rel_dir: PathBuf,
    pub tags: HashSet<String>,
    pub groups: HashMap<String, Group>,
    pub tables: HashMap<String, Table>,
}

impl Group {
    /// Iterate over all tables in the group and its descendant groups, in no
    /// particular order (that is: the order is unstable, and might change in
    /// future, and between runs of the program).
    pub fn iter_tables(&self) -> GroupTablesIter {
        GroupTablesIter::new(self)
    }
}

pub struct GroupTablesIter<'g> {
    grp_stack: Vec<hash_map::Iter<'g, String, Group>>,
    table_iter: hash_map::Iter<'g, String, Table>,
}

impl<'g> GroupTablesIter<'g> {
    fn new(top: &'g Group) -> Self {
        Self {
            grp_stack: vec![top.groups.iter()],
            table_iter: top.tables.iter(),
        }
    }
}

impl<'g> Iterator for GroupTablesIter<'g> {
    type Item = &'g Table;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.table_iter.next() {
                Some((_, table)) => return Some(table),
                None => {
                    // Exhausted tables on current group.
                    match self.grp_stack.last_mut() {
                        None => {
                            // No more groups, iteration complete.
                            return None;
                        }
                        Some(group_iter) => {
                            // Try for next child group.
                            match group_iter.next() {
                                Some((_, group)) => {
                                    // Start processing tables and groups in the
                                    // next child group.
                                    self.table_iter = group.tables.iter();
                                    self.grp_stack.push(group.groups.iter());
                                }
                                None => {
                                    // No more child groups. Go up a level to
                                    // continue next sibling group.
                                    self.grp_stack.pop();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
