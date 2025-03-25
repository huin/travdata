//! Defines (de)serialisation of extracton templates from various format versions.

mod v0_6;

use std::{fmt::Display, io::Read, path::Path};

use anyhow::{anyhow, bail, Context, Result};

use crate::{filesio, template};

const VERSION_PATH_STR: &str = "version.txt";

/// Set of template format versions supported. Acceptable as an argument for [preload]'s
/// `assume_version` parameter.
pub const VERSIONS_SUPPORTED: &[&str] = &["0.6"];

/// Concrete error type returned by [preload] for some specific error cases that may be
/// recoverable.
#[derive(Debug)]
pub enum PreloadError {
    UnknownVersion,
}

impl PreloadError {
    pub fn is_unknown_version(&self) -> bool {
        matches!(self, Self::UnknownVersion)
    }
}

impl Display for PreloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PreloadError::*;
        match self {
            UnknownVersion => write!(f, "unknown template format version"),
        }
    }
}

/// Data known from preloading the extraction template, which provides information for supplying
/// specifying [LoadArgs].
#[derive(Debug)]
pub struct PreloadData {
    /// Set of book identifiers found in the preloaded data, if the format supports multiple books.
    pub book_ids: Option<Vec<BookIdName>>,
}

/// Summary descriptor for a book extraction template that can be loaded.
#[derive(Debug)]
pub struct BookIdName {
    pub id: String,
    pub name: String,
}

impl Display for BookIdName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

/// Parameters to completing the file load.
#[derive(Debug, Default)]
pub struct LoadArg {
    /// [PreloadData::book_ids] is a [Option::Some], then this must be a [Option::Some] containing
    /// one of the values from it. Otherwise it must be [Option::None].
    pub book_id: Option<String>,
}

/// Preloaded extraction template data that may or may not need further data prior to calling
/// [PreloadedTemplate::load].
pub trait PreloadedTemplate: std::fmt::Debug + Send {
    /// Returns an acceptable parameter for [VersionLoader::load] if there is a single possible
    /// option. Otherwise returns `None` to indicate that specific argument must be provided.
    fn default_load_arg(&self) -> Option<LoadArg>;

    /// Returns data known before completing the loading.
    fn preload_data(&self) -> PreloadData;

    /// Completes the loading of an extraction template.
    fn load(&self, file_io: &dyn filesio::Reader, arg: LoadArg) -> Result<template::Book>;
}

/// Attempts to preload the extraction template from the given `file_io`.
///
/// If `assume_version` is given, then it is used in precedence to any version found in `file_io`.
pub fn preload(
    file_io: &dyn filesio::Reader,
    assume_version: Option<&str>,
) -> Result<Box<dyn PreloadedTemplate>> {
    let found_version = load_version(file_io)?;
    let version = if let Some(version) = assume_version {
        version
    } else if let Some(version) = &found_version {
        version
    } else {
        return Err(anyhow!(PreloadError::UnknownVersion));
    };

    if v0_6::Loader::matches_version(version) {
        let loader = v0_6::Loader::preload(file_io)?;
        Ok(Box::new(loader))
    } else {
        bail!("unsupported version to load: {}", version);
    }
}

/// Loads the template version from `file_io`.
fn load_version(file_io: &dyn filesio::Reader) -> Result<Option<String>> {
    let mut rdr = match file_io.open_read(Path::new(VERSION_PATH_STR)) {
        Ok(rdr) => rdr,
        Err(error) if filesio::FilesIoError::NotFound.eq_anyhow(&error) => {
            return Ok(None);
        }
        Err(error) => {
            return Err(error).with_context(|| "opening configuration version file");
        }
    };
    let mut file_content = String::new();
    rdr.read_to_string(&mut file_content)
        .with_context(|| "reading configuration version file")?;
    Ok(Some(file_content.trim().to_string()))
}
