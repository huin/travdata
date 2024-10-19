use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::filesio::{FilesIoError, Reader};

use super::book::{load_book, Group};

const ROOT_PATH_STR: &str = "config.yaml";
const VERSION_PATH_STR: &str = "version.txt";

/// Loads the configuration from `cfg_reader`.
pub fn load_config(cfg_reader: &dyn Reader) -> Result<Config> {
    let rdr = cfg_reader
        .open_read(Path::new(ROOT_PATH_STR))
        .with_context(|| "opening root configuration file")?;
    let config: YamlConfig =
        serde_yaml_ng::from_reader(rdr).with_context(|| "parsing root configuration file")?;

    Ok(config.prepare())
}

/// Loads the configuration version `cfg_reader`.
pub fn load_config_version(cfg_reader: &dyn Reader) -> Result<Option<String>> {
    let mut rdr = match cfg_reader.open_read(Path::new(VERSION_PATH_STR)) {
        Ok(rdr) => rdr,
        Err(error) if FilesIoError::NotFound.eq_anyhow(&error) => {
            return Ok(None);
        }
        Err(error) => {
            return Err(error).with_context(|| "opening configuration version file");
        }
    };
    let mut version_string = String::new();
    rdr.read_to_string(&mut version_string)
        .with_context(|| "reading configuration version file")?;
    Ok(Some(version_string))
}

/// Top level configuration, read and prepared from a `config.yaml`.
pub struct Config {
    pub ecma_script_modules: Vec<PathBuf>,
    pub books: HashMap<String, Book>,
}

impl Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Config(with {} books)", self.books.len())
    }
}

#[allow(dead_code)] // The GUI will use previously unused fields.
/// Top level information about a book.
pub struct Book {
    pub id: String,
    pub name: String,
    pub default_filename: String,
    pub ecma_script_modules: Vec<PathBuf>,
    pub tags: HashSet<String>,
    pub page_offset: i32,
}

impl Book {
    /// Loads and returns the top-level group in the `Book`.
    pub fn load_group(&self, cfg_reader: &dyn Reader) -> Result<Group> {
        load_book(cfg_reader, &self.id, &self.tags)
    }
}

/// Top level configuration, read from a `config.yaml`.
#[derive(Deserialize, Debug)]
struct YamlConfig {
    ecma_script_modules: Vec<PathBuf>,
    books: HashMap<String, YamlBook>,
}

impl YamlConfig {
    /// Creates a `Config` from self.
    fn prepare(self) -> Config {
        Config {
            ecma_script_modules: self.ecma_script_modules,
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
    ecma_script_modules: Vec<PathBuf>,
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
            ecma_script_modules: self.ecma_script_modules,
            tags,
            page_offset: self.page_offset,
        }
    }
}
