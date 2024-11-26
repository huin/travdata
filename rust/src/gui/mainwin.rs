use std::{path::PathBuf, sync::Arc};

use gtk::prelude::{BoxExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
};

use crate::{
    commontext,
    config::root,
    gui::{cfgselect, extract, inputpdf, outputselect, util},
};

use super::workers::{self, extractor};

/// Input messages for [MainWindow].
#[derive(Debug)]
enum Input {
    /// No-op message.
    Config(Option<(util::FileIoPath, Arc<root::Config>)>),
    ExtractorInput(extract::Input),
}

/// Initialisation parameters for [MainWindow].
pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
    pub default_config: Option<PathBuf>,
    pub worker_channel: workers::extractor::WorkChannel,
}

/// Relm4 window component that acts as the main window for the GUI interface to Travdata.
struct MainWindow {
    cfg_selector: Controller<cfgselect::ConfigSelector>,
    input_pdf_selector: Controller<inputpdf::InputPdfSelector>,
    output_selector: Controller<outputselect::OutputSelector>,
    extractor: Controller<extract::Extractor>,
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

                model.extractor.widget(),
            }
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::ExtractorInput(extractor_input) => {
                self.extractor.emit(extractor_input);
            }
            Input::Config(config_opt) => match config_opt {
                Some((cfg_io, config)) => {
                    self.input_pdf_selector
                        .emit(inputpdf::Input::SelectedConfig(Some(config)));
                    self.extractor.emit(extract::Input::ConfigIo(Some(cfg_io)));
                }
                None => {
                    self.input_pdf_selector
                        .emit(inputpdf::Input::SelectedConfig(None));
                    self.extractor.emit(extract::Input::ConfigIo(None));
                }
            },
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
                    default_config: init.default_config,
                })
                .forward(sender.input_sender(), |msg| match msg {
                    cfgselect::Output::SelectedConfig(config_opt) => Input::Config(config_opt),
                }),
            input_pdf_selector: inputpdf::InputPdfSelector::builder()
                .launch(inputpdf::Init {
                    xdg_dirs: init.xdg_dirs.clone(),
                })
                .forward(sender.input_sender(), |msg| match msg {
                    inputpdf::Output::SelectedInputPdf(input_pdf) => {
                        Input::ExtractorInput(extract::Input::InputPdf(input_pdf))
                    }
                    inputpdf::Output::SelectedBookId(book_id) => {
                        Input::ExtractorInput(extract::Input::BookId(book_id))
                    }
                }),
            output_selector: outputselect::OutputSelector::builder()
                .launch(outputselect::Init {
                    xdg_dirs: init.xdg_dirs.clone(),
                })
                .forward(sender.input_sender(), |msg| match msg {
                    outputselect::Output::SelectedOutputIo(output_io) => {
                        Input::ExtractorInput(extract::Input::OutputIo(output_io))
                    }
                }),
            extractor: extract::Extractor::builder()
                .launch(extractor::Init {
                    worker_channel: init.worker_channel,
                })
                .detach(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

/// Runs in the GUI thread for the lifetime of the GUI itself.
pub fn run_gui(init: Init) {
    let app = RelmApp::new("travdata.gui").with_args(Vec::new());
    app.run::<MainWindow>(init);
}
