use std::{path::PathBuf, sync::Arc};

use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use crate::{config::root, gui::util};

/// Input messages for [Extractor].
#[derive(Debug)]
pub enum Input {
    Config(Option<Arc<root::Config>>),
    #[allow(clippy::enum_variant_names)]
    InputPdf(Option<PathBuf>),
    BookId(Option<String>),
    OutputIo(Option<util::SelectedFileIo>),
}

pub struct Extractor {
    config: Option<Arc<root::Config>>,
    input_pdf: Option<PathBuf>,
    book_id: Option<String>,
    output_io: Option<util::SelectedFileIo>,
}

impl Extractor {
    fn is_extraction_ready(&self) -> bool {
        self.config.is_some()
            && self.input_pdf.is_some()
            && self.book_id.is_some()
            && self.output_io.is_some()
    }
}

#[relm4::component(pub)]
impl SimpleComponent for Extractor {
    type Init = ();

    type Input = Input;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,

            gtk::Button::with_label("Extract") {
                #[watch]
                set_sensitive: model.is_extraction_ready(),
            },
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::Config(config) => {
                self.config = config;
            }
            Input::InputPdf(input_pdf) => {
                self.input_pdf = input_pdf;
            }
            Input::BookId(book_id) => {
                self.book_id = book_id;
            }
            Input::OutputIo(output_io) => {
                self.output_io = output_io;
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            config: None,
            input_pdf: None,
            book_id: None,
            output_io: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
