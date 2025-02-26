//! Defines serialisation structures specific to the v0.6.x extraction template format.

use std::{
    collections::{HashMap, HashSet},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::{extraction::tableextract, filesio, template};

const ROOT_PATH_STR: &str = "config.yaml";

/// Top level configuration, read from a `config.yaml`.
#[derive(Deserialize, Debug)]
struct Root {
    ecma_script_modules: Vec<PathBuf>,
    books: HashMap<String, Book>,
}

#[derive(Deserialize, Debug)]
struct Book {
    name: String,
    default_filename: String,
    #[serde(default)]
    ecma_script_modules: Vec<PathBuf>,
    #[serde(default)]
    tags: HashSet<String>,
    #[serde(default = "default_i32_one")]
    page_offset: i32,
}

#[derive(Deserialize, Debug)]
struct Table {
    #[serde(default = "Default::default")]
    pub tags: HashSet<String>,
    #[serde(default = "default_false")]
    pub disable_extraction: bool,
    #[serde(default = "Default::default")]
    pub transform: Option<tableextract::TableTransform>,
}

impl Table {
    fn load(self, path: PathBuf, file_io: &dyn filesio::Reader) -> Result<template::Table> {
        let template: TabulaTemplate = serde_json::from_reader(
            file_io
                .open_read(&path)
                .with_context(|| format!("opening template file {:?}", path))?,
        )?;

        let portions = template
            .0
            .into_iter()
            .map(|entry| template::TablePortion {
                key: None,
                extraction_method: entry.extraction_method.load(),
                page: entry.page,
                left: entry.x1,
                top: entry.y1,
                right: entry.x2,
                bottom: entry.y2,
            })
            .collect();

        Ok(template::Table {
            tags: self.tags,
            portions,
            transform: self.transform,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
struct TabulaTemplate(pub Vec<TabulaTemplateEntry>);

#[derive(Clone, Copy, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum TabulaExtractionMethod {
    Guess,
    Lattice,
    Stream,
}

impl TabulaExtractionMethod {
    fn load(self) -> template::TabulaExtractionMethod {
        match self {
            Self::Guess => template::TabulaExtractionMethod::Guess,
            Self::Lattice => template::TabulaExtractionMethod::Lattice,
            Self::Stream => template::TabulaExtractionMethod::Stream,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct TabulaTemplateEntry {
    pub page: i32,
    pub extraction_method: TabulaExtractionMethod,
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
    pub width: f32,
    pub height: f32,
}

/// Top and intermediate level of hierarchy, defined in `book.yaml`, in a directory` adjacent to
/// `config.yaml` that is named for the [Book]'s `id`.
#[derive(Deserialize, Debug)]
struct Group {
    #[serde(default = "Default::default")]
    pub tags: HashSet<String>,
    #[serde(default = "Default::default")]
    pub groups: HashMap<String, Group>,
    #[serde(default = "Default::default")]
    pub tables: HashMap<String, Table>,
}

impl Group {
    fn load(self, path: PathBuf, file_io: &dyn filesio::Reader) -> Result<template::Group> {
        let groups: HashMap<String, template::Group> = self
            .groups
            .into_iter()
            .map(|(k, v)| {
                let group_path = path.join(&k);
                v.load(group_path, file_io)
                    .with_context(|| format!("in group {:?}", k))
                    .map(|group| (k, group))
            })
            .collect::<Result<HashMap<String, template::Group>>>()?;

        let tables: HashMap<String, template::Table> = self
            .tables
            .into_iter()
            .map(|(k, v)| {
                let table_path = path.join(format!("{}.json", k));
                v.load(table_path, file_io)
                    .with_context(|| format!("in table {:?}", k))
                    .map(|table| (k, table))
            })
            .collect::<Result<_>>()?;

        Ok(template::Group {
            tags: self.tags,
            groups,
            tables,
        })
    }
}

fn default_i32_one() -> i32 {
    1
}

fn default_false() -> bool {
    false
}

pub struct Loader {
    root: Root,
}

pub struct PreloadData {
    pub book_ids: Vec<String>,
}

pub struct LoadArg {
    pub book_id: String,
}

impl super::VersionLoader for Loader {
    type PreloadData = PreloadData;
    type LoadArg = LoadArg;

    fn preload(file_io: &dyn filesio::Reader) -> Result<Self> {
        let rdr = file_io
            .open_read(Path::new(ROOT_PATH_STR))
            .with_context(|| "opening root configuration file")?;
        let root: Root =
            serde_yaml_ng::from_reader(rdr).with_context(|| "parsing root configuration file")?;

        Ok(Self { root })
    }

    fn preload_data(&self) -> Self::PreloadData {
        let book_ids: Vec<String> = self.root.books.keys().cloned().collect();
        Self::PreloadData { book_ids }
    }

    fn load(self, file_io: &dyn filesio::Reader, arg: Self::LoadArg) -> Result<template::Book> {
        let raw_book = self
            .root
            .books
            .get(&arg.book_id)
            .ok_or_else(|| anyhow!("book ID {:?} not found in configuration", arg.book_id))?;

        let rel_book_dir: PathBuf = arg.book_id.as_str().into();
        let config_path = rel_book_dir.join("book.yaml");

        let mut scripts = Vec::with_capacity(
            self.root.ecma_script_modules.len() + raw_book.ecma_script_modules.len(),
        );
        load_scripts(file_io, &self.root.ecma_script_modules, &mut scripts)
            .with_context(|| "loading root scripts")?;
        load_scripts(file_io, &raw_book.ecma_script_modules, &mut scripts)
            .with_context(|| "loading book scripts")?;

        let group = {
            let group_file = file_io.open_read(&config_path).with_context(|| {
                format!(
                    "opening book configuration {:?} from file {:?}",
                    arg.book_id, &config_path,
                )
            })?;
            let raw_group: Group = serde_yaml_ng::from_reader(group_file)
                .with_context(|| "parsing book configuration")?;

            raw_group
                .load(rel_book_dir, file_io)
                .with_context(|| "in root group")?
        };

        Ok(template::Book { scripts, group })
    }
}

fn load_scripts(
    file_io: &dyn filesio::Reader,
    script_paths: &[PathBuf],
    scripts: &mut Vec<template::Script>,
) -> Result<()> {
    for script_path in script_paths {
        let script = load_script(file_io, script_path)
            .with_context(|| format!("for script: {:?}", script_path))?;
        scripts.push(script);
    }
    Ok(())
}

fn load_script(file_io: &dyn filesio::Reader, script_path: &Path) -> Result<template::Script> {
    let name: String = script_path
        .to_str()
        .ok_or_else(|| anyhow!("script path {:?} is not valid UTF-8", script_path))?
        .to_string();

    let mut code = String::new();
    file_io.open_read(script_path)?.read_to_string(&mut code)?;

    Ok(template::Script { name, code })
}
