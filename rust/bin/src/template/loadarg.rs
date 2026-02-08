use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use crate::{filesio, template};

/// CLI arguments relating to [template::Book].
#[derive(Args, Clone, Debug)]
pub struct TemplateArgs {
    /// Path to the extraction template. This must be either a directory or ZIP file containing the
    /// template's extraction data. This directory or ZIP file will typically contain a
    /// `version.txt` file that specifies the format version. If it does not, then
    /// `--template-version` must be specified.
    #[arg(long)]
    template: PathBuf,

    /// ID of the book in the extraction template to extract. Should be specified for older
    /// extraction template formats that contain multiple book templates.
    #[arg(long)]
    book_id: Option<String>,

    /// Override/specify the version of extraction template format.
    #[arg(long)]
    template_version: Option<String>,
}

impl TemplateArgs {
    /// Loads the extraction template.
    pub fn load_template(&self) -> Result<template::Book> {
        let file_io = self.file_io()?;
        let preload =
            template::serialised::preload(file_io.as_ref(), self.template_version.as_deref())?;

        let mut load_arg = preload.default_load_arg().unwrap_or_default();
        if self.book_id.is_some() {
            load_arg.book_id = self.book_id.clone();
        }

        preload.load(file_io.as_ref(), load_arg)
    }

    fn file_io(&self) -> Result<Box<dyn filesio::Reader<'_>>> {
        let cfg_type = filesio::IoType::resolve_auto(None, &self.template);

        cfg_type
            .new_reader(&self.template)
            .with_context(|| format!("opening config path {:?} as {:?}", self.template, cfg_type))
    }
}
