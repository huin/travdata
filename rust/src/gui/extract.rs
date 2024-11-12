use std::path::PathBuf;

use anyhow::{anyhow, Result};
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    SimpleComponent,
};

use crate::gui::util;

use super::workers::extractor;

/// Input messages for [Extractor].
#[derive(Debug)]
pub enum Input {
    // External:
    ConfigIo(Option<util::SelectedFileIo>),
    #[allow(clippy::enum_variant_names)]
    InputPdf(Option<PathBuf>),
    BookId(Option<String>),
    OutputIo(Option<util::SelectedFileIo>),

    // Internal:
    StartExtraction,
    CancelExtraction,
    #[allow(private_interfaces)]
    Progress(Progress),
}

pub struct Extractor {
    cfg_io: Option<util::SelectedFileIo>,
    input_pdf: Option<PathBuf>,
    book_id: Option<String>,
    out_io: Option<util::SelectedFileIo>,

    progress: Option<Progress>,

    worker: Controller<extractor::ExtractorWorker>,
}

impl Extractor {
    fn is_extraction_ready(&self) -> bool {
        self.cfg_io.is_some()
            && self.input_pdf.is_some()
            && self.book_id.is_some()
            && self.out_io.is_some()
    }

    fn form_request(&self) -> Result<extractor::Request> {
        let cfg_io = self
            .cfg_io
            .as_ref()
            .ok_or_else(|| anyhow!("Config is not set."))?
            .clone();
        let input_pdf = self
            .input_pdf
            .as_ref()
            .ok_or_else(|| anyhow!("Input PDF is not set."))?
            .clone();
        let book_id = self
            .book_id
            .as_ref()
            .ok_or_else(|| anyhow!("Book ID is not set."))?
            .clone();
        let out_io = self
            .out_io
            .as_ref()
            .ok_or_else(|| anyhow!("Output is not set."))?
            .clone();

        Ok(extractor::Request {
            cfg_io,
            input_pdf,
            book_id,
            out_io,
        })
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

            gtk::ProgressBar {
                #[watch]
                set_fraction: if let Some(progress) = model.progress.as_ref() { progress.fraction} else {0.0},
                #[watch]
                set_text: if let Some(progress) = model.progress.as_ref() { Some(&progress.text) } else {None},
                #[watch]
                set_show_text: model.progress.is_some(),
            },

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
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::ConfigIo(cfg_io) => {
                self.cfg_io = cfg_io;
            }
            Input::InputPdf(input_pdf) => {
                self.input_pdf = input_pdf;
            }
            Input::BookId(book_id) => {
                self.book_id = book_id;
            }
            Input::OutputIo(output_io) => {
                self.out_io = output_io;
            }
            Input::StartExtraction => match self.form_request() {
                Ok(request) => {
                    self.worker.sender().emit(extractor::Input::Start(request));
                }
                Err(err) => {
                    log::warn!("Could not start start extraction: {:?}", err);
                }
            },
            Input::CancelExtraction => {
                self.worker.sender().emit(extractor::Input::Cancel);
            }
            Input::Progress(progress) => {
                self.progress = Some(progress);
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            cfg_io: None,
            input_pdf: None,
            book_id: None,
            out_io: None,

            progress: None,

            worker: extractor::ExtractorWorker::builder().launch(init).forward(
                sender.input_sender(),
                |msg| match msg {
                    extractor::Output::Progress {
                        path: _,
                        completed,
                        total,
                    } => Input::Progress(Progress {
                        text: format!("{} / {}", completed, total),
                        fraction: (completed as f64) / (total as f64),
                    }),
                    // TODO:Display errors.
                    extractor::Output::Error { err: _ } => todo!(),
                    extractor::Output::Completed => Input::Progress(Progress {
                        text: "Complete".to_string(),
                        fraction: 1.0,
                    }),
                },
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

#[derive(Debug)]
struct Progress {
    text: String,
    fraction: f64,
}
