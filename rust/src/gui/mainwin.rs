use std::{path::PathBuf, sync::Arc};

use gtk::prelude::{BoxExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
};

use crate::{
    commontext,
    config::root,
    gui::{cfgselect, inputpdf, outputselect, util},
};

/// Input messages for [MainWindow].
#[derive(Debug)]
enum Input {
    /// No-op message.
    SelectedConfig(Option<Arc<root::Config>>),
    SelectedInputPdf(Option<PathBuf>),
    SelectedBookId(Option<String>),
    SelectedOutputIo(Option<util::SelectedFileIo>),
}

/// Initialisation parameters for [MainWindow].
pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
}

/// Relm4 window component that acts as the main window for the GUI interface to Travdata.
struct MainWindow {
    cfg_selector: Controller<cfgselect::ConfigSelector>,
    input_pdf_selector: Controller<inputpdf::InputPdfSelector>,
    output_selector: Controller<outputselect::OutputSelector>,

    config: Option<Arc<root::Config>>,
    input_pdf: Option<PathBuf>,
    book_id: Option<String>,
    output_io: Option<util::SelectedFileIo>,
}

impl MainWindow {
    fn is_extraction_ready(&self) -> bool {
        self.config.is_some()
            && self.input_pdf.is_some()
            && self.book_id.is_some()
            && self.output_io.is_some()
    }
}

#[relm4::component]
impl SimpleComponent for MainWindow {
    type Init = Init;

    type Input = Input;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Travdata"),
            set_default_width: 300,
            set_default_height: 600,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Label {
                    set_label: commontext::DATA_USAGE,
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                model.cfg_selector.widget(),
                model.input_pdf_selector.widget(),
                model.output_selector.widget(),

                gtk::Box {
                    // Spacer.
                    set_vexpand: true,
                },

                gtk::Button::with_label("Extract") {
                    #[watch]
                    set_sensitive: model.is_extraction_ready(),
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::SelectedConfig(config) => {
                self.input_pdf_selector
                    .emit(inputpdf::Input::SelectedConfig(config.clone()));
                self.config = config;
            }
            Input::SelectedInputPdf(input_pdf) => {
                self.input_pdf = input_pdf;
            }
            Input::SelectedBookId(book_id) => {
                self.book_id = book_id;
            }
            Input::SelectedOutputIo(output_io) => {
                self.output_io = output_io;
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            cfg_selector: cfgselect::ConfigSelector::builder()
                .launch(cfgselect::Init {
                    xdg_dirs: init.xdg_dirs.clone(),
                })
                .forward(sender.input_sender(), |msg| match msg {
                    cfgselect::Output::SelectedConfig(config) => Input::SelectedConfig(config),
                }),
            input_pdf_selector: inputpdf::InputPdfSelector::builder()
                .launch(inputpdf::Init {
                    xdg_dirs: init.xdg_dirs.clone(),
                })
                .forward(sender.input_sender(), |msg| match msg {
                    inputpdf::Output::SelectedInputPdf(input_pdf) => {
                        Input::SelectedInputPdf(input_pdf)
                    }
                    inputpdf::Output::SelectedBookId(book_id) => Input::SelectedBookId(book_id),
                }),
            output_selector: outputselect::OutputSelector::builder()
                .launch(outputselect::Init {
                    xdg_dirs: init.xdg_dirs.clone(),
                })
                .forward(sender.input_sender(), |msg| match msg {
                    outputselect::Output::SelectedOutputIo(output_io) => {
                        Input::SelectedOutputIo(output_io)
                    }
                }),

            config: None,
            input_pdf: None,
            book_id: None,
            output_io: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

/// Runs the GUI thread for the lifetime of the GUI itself.
pub fn run_gui(init: Init) {
    let app = RelmApp::new("travdata.gui").with_args(Vec::new());
    app.run::<MainWindow>(init);
}
