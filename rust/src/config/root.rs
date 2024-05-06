use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Loads the configuration from the given `path`.
pub fn load_config(path: &Path) -> Result<Config> {
    let cfg_path = path.join("config.yaml");
    let rdr = std::fs::File::open(cfg_path)
        .with_context(|| format!("opening configuration file {:?}", path))?;
    let config: YamlConfig = serde_yaml_ng::from_reader(rdr)
        .with_context(|| format!("parsing configuration file {:?}", path))?;

    Ok(config.prepare())
}

/// Top level configuration, read and prepared from a `config.yaml`.
pub struct Config {
    pub books: HashMap<String, Book>,
}

/// Top level information about a book.
pub struct Book {
    pub id: String,
    pub name: String,
    pub default_filename: String,
    pub tags: HashSet<String>,
    pub page_offset: i32,
}

/// Top level configuration, read from a `config.yaml`.
#[derive(Deserialize, Debug)]
struct YamlConfig {
    books: HashMap<String, YamlBook>,
}

impl YamlConfig {
    /// Creates a `Config` from self.
    fn prepare(self) -> Config {
        Config {
            books: self
                .books
                .into_iter()
                .map(|(id, book)| (id.clone(), book.prepare(id)))
                .collect(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct YamlBook {
    name: String,
    default_filename: String,
    #[serde(default)]
    tags: HashSet<String>,
    #[serde(default = "default_i32_one")]
    page_offset: i32,
}

fn default_i32_one() -> i32 {
    1
}

impl YamlBook {
    /// Creates a `Book` from self. `id` is the ID of the book within the parent
    /// `YamlConfig`.
    fn prepare(self, id: String) -> Book {
        let mut tags = self.tags;
        tags.insert(format!("book/{}", id));
        Book {
            id,
            name: self.name,
            default_filename: self.default_filename,
            tags,
            page_offset: self.page_offset,
        }
    }
}
