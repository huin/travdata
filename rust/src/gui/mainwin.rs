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

use crate::{commontext, config::root::Config, gui::util};

use super::util::SelectedFileIo;

#[derive(Debug)]
enum MainInputMsg {
    ConfigIo(SelectedFileIo),
    InputPdf(PathBuf),
    OutputIo(SelectedFileIo),
}

pub struct Init {
    pub xdg_dirs: xdg::BaseDirectories,
}

impl Init {
    /// Get a `&'static str` reference to the filename within the XDG configuration directory.
    ///
    /// NOTE: Leaks the [String] that backs the return value, because the Relm4 field that uses it
    /// ([OpenButtonSettings::recently_opened_files]) requires a `&'static str`, but the value must
    /// be dynamically generated based on the XDG configuration path.
    fn xdg_cfg_static_str(&self, filename: &str) -> Option<&'static str> {
        self.xdg_dirs
            .place_config_file(filename)
            .map_err(|e| {
                log::warn!("Could not create {:?} file: {:?}", filename, e);
                e
            })
            .ok()
            .and_then(|p: PathBuf| {
                p.to_str().map(|s: &str| {
                    let static_str: &'static str = s.to_owned().leak();
                    static_str
                })
            })
    }
}

#[allow(dead_code)]
struct MainModel {
    cfg_dir: Controller<OpenButton>,
    cfg_zip: Controller<OpenButton>,
    cfg_io: Option<util::SelectedFileIo>,
    cfg: Option<Config>,
    cfg_error: Option<String>,
    cfg_version: Option<String>,

    input_pdf_open: Controller<OpenButton>,
    input_pdf: Option<PathBuf>,
    book_id: Option<String>,

    output_path_dir: Controller<OpenButton>,
    output_path_zip: Controller<OpenButton>,
    output_io: Option<util::SelectedFileIo>,
}

#[relm4::component]
impl SimpleComponent for MainModel {
    type Init = Init;

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

                            model.cfg_dir.widget(),
                            model.cfg_zip.widget(),
                            gtk::Button::with_label("Default") {
                            },
                        },

                        attach[0, 1, 1, 1] = &gtk::Label {
                            set_label: "Config path:",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 1, 1, 1] = &gtk::Label {
                            #[watch]
                            set_label: &util::format_opt_selected_file_io(&model.cfg_io),
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
                            set_label: util::format_opt_path(&model.input_pdf),
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

                            model.output_path_dir.widget(),
                            model.output_path_zip.widget(),
                        },

                        attach[0, 1, 1, 1] = &gtk::Label {
                            set_label: "Output path:",
                            set_halign: gtk::Align::Start,
                        },
                        attach[1, 1, 1, 1] = &gtk::Label {
                            #[watch]
                            set_label: &util::format_opt_selected_file_io(&model.output_io),
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
            MainInputMsg::ConfigIo(io) => self.cfg_io = Some(io),
            MainInputMsg::InputPdf(path) => self.input_pdf = Some(path),
            MainInputMsg::OutputIo(io) => self.output_io = Some(io),
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let recent_cfg_dirs = init.xdg_cfg_static_str("recent_cfg_dirs.txt");
        let recent_cfg_zips = init.xdg_cfg_static_str("recent_cfg_zips.txt");
        let recent_input_pdfs = init.xdg_cfg_static_str("recent_input_pdfs.txt");
        let recent_output_dirs = init.xdg_cfg_static_str("recent_output_dirs.txt");
        let recent_output_zips = init.xdg_cfg_static_str("recent_output_zips.txt");

        let pdf_filter = gtk::FileFilter::new();
        pdf_filter.set_name(Some("PDF file"));
        pdf_filter.add_pattern("*.pdf");
        pdf_filter.add_mime_type("application/pdf");
        let zip_filter = gtk::FileFilter::new();
        zip_filter.set_name(Some("ZIP archive"));
        zip_filter.add_pattern("*.zip");
        zip_filter.add_mime_type("application/zip");

        let model = Self {
            cfg_dir: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings {
                        folder_mode: true,
                        cancel_label: "Cancel".to_string(),
                        accept_label: "Read config from directory".to_string(),
                        create_folders: false,
                        is_modal: true,
                        filters: vec![],
                    },
                    text: "Select directory",
                    recently_opened_files: recent_cfg_dirs,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), |path| {
                    MainInputMsg::ConfigIo(SelectedFileIo::for_dir(path))
                }),
            cfg_zip: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings {
                        folder_mode: false,
                        cancel_label: "Cancel".to_string(),
                        accept_label: "Read config from ZIP".to_string(),
                        create_folders: false,
                        is_modal: true,
                        filters: vec![zip_filter.clone()],
                    },
                    text: "Select ZIP",
                    recently_opened_files: recent_cfg_zips,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), |path| {
                    MainInputMsg::ConfigIo(SelectedFileIo::for_zip(path))
                }),
            cfg_io: None,
            cfg: None,
            cfg_error: None,
            cfg_version: None,

            input_pdf_open: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings {
                        folder_mode: false,
                        cancel_label: "Cancel".to_string(),
                        accept_label: "Choose Input PDF".to_string(),
                        create_folders: false,
                        is_modal: true,
                        filters: vec![pdf_filter.clone()],
                    },
                    text: "Select PDF",
                    recently_opened_files: recent_input_pdfs,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), MainInputMsg::InputPdf),
            input_pdf: None,
            book_id: None,

            output_io: None,
            output_path_dir: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings {
                        folder_mode: true,
                        cancel_label: "Cancel".to_string(),
                        accept_label: "Choose Output Directory".to_string(),
                        create_folders: true,
                        is_modal: true,
                        filters: vec![],
                    },
                    text: "Select directory",
                    recently_opened_files: recent_output_dirs,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), |path| {
                    MainInputMsg::OutputIo(SelectedFileIo::for_dir(path))
                }),
            output_path_zip: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings {
                        folder_mode: false,
                        cancel_label: "Cancel".to_string(),
                        accept_label: "Choose Output ZIP".to_string(),
                        create_folders: true,
                        is_modal: true,
                        filters: vec![zip_filter],
                    },
                    text: "Select ZIP",
                    recently_opened_files: recent_output_zips,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), |path| {
                    MainInputMsg::OutputIo(SelectedFileIo::for_zip(path))
                }),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

/// Runs the GUI thread for the lifetime of the GUI itself.
pub fn run_gui(init: Init) {
    let app = RelmApp::new("travdata.gui").with_args(Vec::new());
    app.run::<MainModel>(init);
}
