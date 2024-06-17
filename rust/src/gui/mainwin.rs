use std::path::PathBuf;

use gtk::prelude::{BoxExt, FrameExt, GridExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
};
use relm4_components::{
    open_button::{OpenButton, OpenButtonSettings},
    open_dialog::OpenDialogSettings,
};

use crate::{commontext, config::root::Config};

#[derive(Debug)]
enum MainInputMsg {
    SelectInputPdf(PathBuf),
}

#[allow(dead_code)]
struct MainModel {
    cfg: Option<Config>,
    cfg_error: Option<String>,
    cfg_version: Option<String>,

    input_pdf_open: Controller<OpenButton>,
    input_pdf: Option<PathBuf>,
    book_id: Option<String>,

    output_path: Option<PathBuf>,
}

#[relm4::component]
impl SimpleComponent for MainModel {
    type Init = ();

    type Input = MainInputMsg;
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

                gtk::Frame {
                    set_label: Some("Extraction configuration"),

                    gtk::Grid {
                        set_margin_start: 5,
                        set_margin_end: 5,
                        set_margin_top: 5,
                        set_margin_bottom: 5,
                        set_column_spacing: 5,
                        set_row_spacing: 5,

                        attach[0, 0, 1, 1] = &gtk::Label {
                            set_label: "Select config:",
                            set_halign: gtk::Align::Start,
                        },

                        attach[1, 0, 1, 1] = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_homogeneous: true,
                            set_spacing: 5,

                            gtk::Button::with_label("Select directory") {
                            },
                            gtk::Button::with_label("Select ZIP") {
                            },
                            gtk::Button::with_label("Default") {
                            },
                        },

                        attach[0, 1, 1, 1] = &gtk::Label {
                            set_label: "Config path:",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 1, 1, 1] = &gtk::Label {
                            set_label: "<not selected>",
                            set_halign: gtk::Align::Start,
                        },

                        attach[0, 2, 1, 1] = &gtk::Label {
                            set_label: "Config version:",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 2, 1, 1] = &gtk::Label {
                            set_label: "<not selected>",
                            set_halign: gtk::Align::Start,
                        },

                        attach[0, 3, 2, 1] = &gtk::Label {
                            // Error box.
                            set_halign: gtk::Align::Start,
                        },
                    },
                },

                gtk::Frame {
                    set_label: Some("Input PDF"),

                    gtk::Grid {
                        set_margin_start: 5,
                        set_margin_end: 5,
                        set_margin_top: 5,
                        set_margin_bottom: 5,
                        set_column_spacing: 5,
                        set_row_spacing: 5,

                        attach[0, 0, 1, 1] = &gtk::Label {
                            set_label: "Select PDF:",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 0, 1, 1] = model.input_pdf_open.widget(),

                        attach[0, 1, 1, 1] = &gtk::Label {
                            set_label: "Input PDF",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 1, 1, 1] = &gtk::Label {
                            #[watch]
                            set_label: match &model.input_pdf {
                                    None => "<not selected>",
                                    Some(path) => match path.to_str() {
                                        None => "<selected - cannot be displayed>",
                                        Some(path_str) => path_str,
                                    },
                                }
                            ,
                            set_halign: gtk::Align::Start,
                        },

                        attach[0, 2, 2, 1] = &gtk::Label {
                            // Error box.
                            set_halign: gtk::Align::Start,
                        },

                        attach[0, 3, 1, 1] = &gtk::Label {
                            set_label: "Select book:",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 3, 1, 1] = &gtk::DropDown {
                            set_hexpand: true,
                        },
                    },
                },

                gtk::Frame {
                    set_label: Some("Output"),

                    gtk::Grid {
                        set_margin_start: 5,
                        set_margin_end: 5,
                        set_margin_top: 5,
                        set_margin_bottom: 5,
                        set_column_spacing: 5,
                        set_row_spacing: 5,

                        attach[0, 0, 1, 1] = &gtk::Label {
                            set_label: "Output",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 0, 1, 1] = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_homogeneous: true,
                            set_spacing: 5,

                            gtk::Button::with_label("Select directory") {
                            },
                            gtk::Button::with_label("Select ZIP") {
                            },
                        },

                        attach[0, 1, 1, 1] = &gtk::Label {
                            set_label: "Output path:",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 1, 1, 1] = &gtk::Label {
                            set_label: "<not selected>",
                            set_halign: gtk::Align::Start,
                        },
                    },
                },

                gtk::Box {
                    // Spacer.
                    set_vexpand: true,
                },

                gtk::Button::with_label("Extract") {
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            MainInputMsg::SelectInputPdf(path) => self.input_pdf = Some(path),
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            cfg: None,
            cfg_error: None,
            cfg_version: None,

            input_pdf_open: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings::default(),
                    text: "Select PDF",
                    recently_opened_files: Some(".input_pdfs"),
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), MainInputMsg::SelectInputPdf),
            input_pdf: None,
            book_id: None,

            output_path: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

/// Runs the GUI thread for the lifetime of the GUI itself.
pub fn run_gui() {
    let app = RelmApp::new("travdata.gui").with_args(Vec::new());
    app.run::<MainModel>(());
}
