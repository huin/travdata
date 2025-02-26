//! Defines (de)serialisation of extracton templates from various format versions.

mod v0_6;

use anyhow::Result;

use crate::{filesio, template};

pub enum LoaderType {
    V0_6(v0_6::Loader),
}

pub trait VersionLoader: Sized {
    type PreloadData;
    type LoadArg;

    /// Starts loading the extraction template.
    fn preload(file_io: &dyn filesio::Reader) -> Result<Self>;

    /// Returns data known before completing the loading.
    fn preload_data(&self) -> Self::PreloadData;

    /// Completes the loading of an extraction template.
    fn load(self, file_io: &dyn filesio::Reader, arg: Self::LoadArg) -> Result<template::Book>;
}
