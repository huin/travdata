use std::{fmt::Write, path::PathBuf};

use anyhow::{Result, anyhow};
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, TextBufferExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    gtk,
};

use crate::{extraction::bookextract, filesio::FileIoPath, template};

use super::workers::extractor;

/// Input messages for [Extractor].
#[derive(Debug)]
pub enum Input {
    // External:
    Template(Option<template::Book>),
    #[allow(clippy::enum_variant_names)]
    InputPdf(Option<PathBuf>),
    OutputIo(Option<FileIoPath>),

    // Internal:
    StartExtraction,
    CancelExtraction,
    #[allow(private_interfaces)]
    Progress(extractor::Output),
}

pub struct Extractor {
    tmpl: Option<template::Book>,
    input_pdf: Option<PathBuf>,
    out_io: Option<FileIoPath>,

    progress: Option<Progress>,
    log_buffer: gtk::TextBuffer,
    scroll: Option<gtk::ScrolledWindow>,

    worker: Controller<extractor::ExtractorWorker>,
}

impl Extractor {
    fn is_extraction_ready(&self) -> bool {
        self.tmpl.is_some() && self.input_pdf.is_some() && self.out_io.is_some()
    }

    fn form_request(&self) -> Result<extractor::Request> {
        let tmpl = self
            .tmpl
            .as_ref()
            .ok_or_else(|| anyhow!("Extraction template is not set."))?
            .clone();
        let input_pdf = self
            .input_pdf
            .as_ref()
            .ok_or_else(|| anyhow!("Input PDF is not set."))?
            .clone();
        let out_io = self
            .out_io
            .as_ref()
            .ok_or_else(|| anyhow!("Output is not set."))?
            .clone();

        Ok(extractor::Request {
            tmpl,
            input_pdf,
            out_io,
        })
    }

    fn clear_log_buffer(&mut self) {
        let (mut start, mut end) = self.log_buffer.bounds();
        self.log_buffer.delete(&mut start, &mut end);
    }

    fn scroll_to_end_of_log(&self) {
        if let Some(scroll) = &self.scroll {
            scroll.emit_scroll_child(gtk::ScrollType::End, false);
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for Extractor {
    type Init = extractor::Init;

    type Input = Input;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,

                gtk::Button::with_label("Extract") {
                    #[watch]
                    set_sensitive: model.is_extraction_ready() && !model.worker.model().is_running(),

                    connect_clicked[sender] => move |_| {
                        sender.input(Input::StartExtraction);
                    },
                },

                gtk::Button::with_label("Cancel") {
                    #[watch]
                    set_sensitive: model.worker.model().is_running(),

                    connect_clicked[sender] => move |_| {
                        sender.input(Input::CancelExtraction);
                    },
                },
            },

            gtk::ProgressBar {
                #[watch]
                set_fraction: if let Some(progress) = model.progress.as_ref() { progress.fraction} else {0.0},
                #[watch]
                set_text: if let Some(progress) = model.progress.as_ref() { Some(&progress.text) } else {None},
                #[watch]
                set_show_text: model.progress.is_some(),
            },

            gtk::Expander {
                set_label: Some("Extraction log"),

                #[name = "scroll"]
                gtk::ScrolledWindow {
                    gtk::TextView::with_buffer(&model.log_buffer) {
                        set_vexpand: true,
                    }
                }
            },
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::Template(tmpl) => {
                self.tmpl = tmpl;
            }
            Input::InputPdf(input_pdf) => {
                self.input_pdf = input_pdf;
            }
            Input::OutputIo(output_io) => {
                self.out_io = output_io;
            }
            Input::StartExtraction => match self.form_request() {
                Ok(request) => {
                    self.clear_log_buffer();
                    log_message_error(writeln!(self.log_buffer, "Starting extraction..."));
                    self.scroll_to_end_of_log();

                    self.progress = Some(Progress {
                        text: "Starting extraction...".to_string(),
                        fraction: 0.0,
                    });
                    self.worker
                        .sender()
                        .emit(extractor::Input::Start(Box::new(request)));
                }
                Err(err) => {
                    log::warn!("Could not start start extraction: {:?}", err);
                }
            },
            Input::CancelExtraction => {
                self.worker.sender().emit(extractor::Input::Cancel);
            }
            Input::Progress(extractor::Output::Event(event)) => match event {
                bookextract::ExtractEvent::Progress {
                    path,
                    completed,
                    total,
                } => {
                    log_message_error(writeln!(self.log_buffer, "Wrote {:?}", path));
                    self.progress = Some(Progress {
                        text: format!("{} / {}", completed, total),
                        fraction: (completed as f64) / (total as f64),
                    });
                    self.scroll_to_end_of_log();
                }
                bookextract::ExtractEvent::Error {
                    err,
                    terminal: false,
                } => {
                    log_message_error(writeln!(self.log_buffer, "Extraction failed: {:?}", err));
                    self.scroll_to_end_of_log();
                }
                bookextract::ExtractEvent::Error {
                    err,
                    terminal: true,
                } => {
                    log_message_error(writeln!(self.log_buffer, "Error (continuing): {:?}", err));
                    self.scroll_to_end_of_log();
                }
                bookextract::ExtractEvent::Completed => {
                    log_message_error(writeln!(self.log_buffer, "Extraction complete."));
                    self.scroll_to_end_of_log();
                }
                bookextract::ExtractEvent::Cancelled => {
                    log_message_error(writeln!(self.log_buffer, "Extraction cancelled."));
                    self.scroll_to_end_of_log();
                }
            },
            Input::Progress(extractor::Output::Failure(err)) => {
                log_message_error(writeln!(
                    self.log_buffer,
                    "Error starting extraction: {:?}",
                    err,
                ));
                self.scroll_to_end_of_log();

                self.progress = Some(Progress {
                    // Use the [Display] form of the error in the progress bar which should be more
                    // concise.
                    text: format!("Error starting extraction: {}", err),
                    fraction: 0.0,
                });
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            tmpl: None,
            input_pdf: None,
            out_io: None,

            progress: None,
            log_buffer: gtk::TextBuffer::new(None),
            scroll: None,

            worker: extractor::ExtractorWorker::builder()
                .launch(init)
                .forward(sender.input_sender(), Input::Progress),
        };

        let widgets = view_output!();

        model.scroll = Some(widgets.scroll.clone());

        ComponentParts { model, widgets }
    }
}

#[derive(Debug)]
struct Progress {
    text: String,
    fraction: f64,
}

fn log_message_error(write_result: std::fmt::Result) {
    if let Err(err) = write_result {
        log::error!(
            "Failed to log message to extraction log text view: {:?}",
            err
        );
    }
}
