//! Relm4 GUI worker for preloading and loading extraction templates.

use anyhow::{Context, Error, Result};
use relm4::Worker;

use crate::{
    filesio::FileIoPath,
    gui::util,
    template::{self, serialised},
};

/// Input messages for [TemplateLoader].
#[derive(Debug)]
pub enum Input {
    /// Requests to load a template. This may respond with any of the [Output] variants. It skips
    /// [Output::PreloadComplete] if there was no need for additional information.
    RequestPreloadTemplate(PreloadRequest),
    /// Requests completion of a template load that previously resulted in
    /// [Output::PreloadComplete].
    RequestLoadTemplate(LoadRequest),
}

/// Request to perform a preload.
#[derive(Debug)]
pub struct PreloadRequest {
    pub file_io_path: FileIoPath,
    pub assume_version: Option<&'static str>,
}

/// Request to complete a previously completed preload.
#[derive(Debug)]
pub struct LoadRequest {
    pub file_io_path: FileIoPath,
    pub load_arg: serialised::LoadArg,
    pub preload: Box<dyn serialised::PreloadedTemplate>,
}

/// Output messages for [TemplateLoader].
#[derive(Debug)]
pub enum Output {
    /// Preload of template is complete, but more input is required, to be provided by
    /// [Input::RequestLoadTemplate].
    PreloadComplete(Preload),
    /// Template load is complete.
    LoadComplete(Load),
    /// Error during the preload or load process.
    LoadError(Error),
}

/// Preloaded template state.
#[derive(Debug)]
pub struct Preload {
    pub file_io_path: FileIoPath,
    pub preload: Box<dyn serialised::PreloadedTemplate>,
}

/// Loaded template.
#[derive(Debug)]
pub struct Load {
    pub tmpl: template::Book,
}

/// Worker component for loading extraction templates.
pub struct TemplateLoader;

impl Worker for TemplateLoader {
    type Init = ();
    type Input = Input;
    type Output = Output;

    fn init(_init: Self::Init, _sender: relm4::ComponentSender<Self>) -> Self {
        Self
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        let output_result: Result<Output> = match message {
            Input::RequestPreloadTemplate(preload_request) => preload(preload_request),
            Input::RequestLoadTemplate(load_request) => load(load_request),
        };

        let output: Output = match output_result {
            Ok(output) => output,
            Err(err) => Output::LoadError(err.into()),
        };

        util::send_output_or_log(output, "template load output", &sender);
    }
}

fn preload(preload_request: PreloadRequest) -> Result<Output> {
    let PreloadRequest {
        file_io_path,
        assume_version,
    } = preload_request;

    let reader = file_io_path.new_reader().context("while creating reader")?;
    let preload = serialised::preload(reader.as_ref(), assume_version)
        .context("while preloading template")?;

    match preload.default_load_arg() {
        Some(load_arg) => load(LoadRequest {
            file_io_path,
            load_arg,
            preload,
        }),
        None => Ok(Output::PreloadComplete(Preload {
            file_io_path,
            preload,
        })),
    }
}

fn load(load_request: LoadRequest) -> Result<Output> {
    let LoadRequest {
        file_io_path,
        load_arg,
        preload,
    } = load_request;

    let reader = file_io_path.new_reader().context("while creating reader")?;

    let tmpl = preload
        .load(reader.as_ref(), load_arg)
        .context("while loading template")?;

    Ok(Output::LoadComplete(Load { tmpl }))
}
