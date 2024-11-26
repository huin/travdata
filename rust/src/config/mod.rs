use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Args;

use crate::{distpaths, filesio};

pub mod book;
pub mod root;

/// CLI arguments relating to [root::Config].
#[derive(Args, Clone, Debug)]
pub struct ConfigArgs {
    /// Path to the configuration. This must be either a directory or ZIP file,
    /// directly containing a config.yaml file, book.yaml files in directories,
    /// and its required Tabula templates. Some configurations for this should
    /// be included with this program's distribution.
    #[arg(long)]
    config: Option<PathBuf>,
}

impl ConfigArgs {
    /// Creates a new [filesio::Reader] for the configuration.
    pub fn new_cfg_reader(&self) -> Result<Box<dyn filesio::Reader>> {
        let cfg_path = self
            .config
            .as_ref()
            .map(|p| p.to_owned())
            .or_else(distpaths::config_zip)
            .ok_or_else(|| {
                anyhow!("--config must be specified, as config.zip could not be located")
            })?;
        let cfg_type = filesio::IoType::resolve_auto(None, &cfg_path);
        cfg_type
            .new_reader(&cfg_path)
            .with_context(|| format!("opening config path {:?} as {:?}", cfg_path, cfg_type))
    }
}
