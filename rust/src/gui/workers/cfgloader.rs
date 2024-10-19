use anyhow::{Context, Result};
use relm4::Worker;

use crate::{config::root, gui::util::SelectedFileIo};

/// Input messages for [ConfigLoader].
#[derive(Debug)]
pub enum Input {
    RequestLoadConfig(SelectedFileIo),
}

#[allow(unused)]
#[derive(Debug)]
pub struct LoadComplete {
    pub io: SelectedFileIo,
    pub config: root::Config,
    pub version: Option<String>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct LoadError {
    pub io: SelectedFileIo,
    pub message: String,
}

/// Output messages for [ConfigLoader].
#[derive(Debug)]
pub enum Output {
    LoadComplete(LoadComplete),
    LoadError(LoadError),
}

/// Worker component for loading root configuration.
pub struct ConfigLoader;

impl Worker for ConfigLoader {
    type Init = ();
    type Input = Input;
    type Output = Output;

    fn init(_init: Self::Init, _sender: relm4::ComponentSender<Self>) -> Self {
        Self
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        let Input::RequestLoadConfig(io) = message;

        let output = match Self::load_config(&io) {
            Ok((config, version)) => Output::LoadComplete(LoadComplete {
                io,
                config,
                version,
            }),
            Err(error) => Output::LoadError(LoadError {
                io,
                message: format!("Error loading configuration: {:?}", error),
            }),
        };

        if let Err(error) = sender.output(output) {
            log::error!("Could not send configuration load output: {:?}", error);
        }
    }
}

impl ConfigLoader {
    fn load_config(file_io: &SelectedFileIo) -> Result<(root::Config, Option<String>)> {
        let reader = file_io
            .io_type
            .new_reader(&file_io.path)
            .with_context(|| format!("while creating reader for {}", file_io))?;
        let config = root::load_config(reader.as_ref())
            .with_context(|| "while reading root configuration")?;
        let version = root::load_config_version(reader.as_ref())
            .with_context(|| "while reading configuration verstion")?;
        Ok((config, version))
    }
}
