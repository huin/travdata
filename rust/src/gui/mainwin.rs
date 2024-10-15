use std::sync::Arc;

use gtk::prelude::{BoxExt, ButtonExt, FrameExt, GridExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
};
use relm4_components::{
    open_button::{OpenButton, OpenButtonSettings},
    open_dialog::OpenDialogSettings,
    save_dialog::{SaveDialog, SaveDialogResponse, SaveDialogSettings},
};

use crate::{
    commontext,
    filesio::IoType,
    gui::{
        cfgselect, inputpdf,
        util::{self, SelectedFileIo},
    },
};

#[derive(Debug)]
enum Input {
    OutputIo(SelectedFileIo),
    OutputZipRequest,
    Ignore,
}

pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
}

#[allow(dead_code)]
struct Model {
    cfg_selector: Controller<cfgselect::ConfigSelector>,

    input_pdf_selector: Controller<inputpdf::InputPdfSelector>,

    output_io: Option<util::SelectedFileIo>,
    output_dir: Controller<OpenButton>,
    output_zip_dialog: Controller<SaveDialog>,
}

#[relm4::component]
impl SimpleComponent for Model {
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

                            model.output_dir.widget(),
                            gtk::Button::with_label("Output into ZIP") {
                                connect_clicked => Input::OutputZipRequest,
                            },
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
            Input::OutputIo(io) => self.output_io = Some(io),
            Input::OutputZipRequest => self
                .output_zip_dialog
                .emit(util::save_dialog_msg(&self.output_io, IoType::Zip)),
            Input::Ignore => {}
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let recent_output_dirs = util::xdg_cfg_static_str(&init.xdg_dirs, "recent_output_dirs.txt");

        let pdf_filter = gtk::FileFilter::new();
        pdf_filter.set_name(Some("PDF file"));
        pdf_filter.add_pattern("*.pdf");
        pdf_filter.add_mime_type("application/pdf");
        let zip_filter = gtk::FileFilter::new();
        zip_filter.set_name(Some("ZIP archive"));
        zip_filter.add_pattern("*.zip");
        zip_filter.add_mime_type("application/zip");

        let model = Self {
            cfg_selector: cfgselect::ConfigSelector::builder()
                .launch(cfgselect::Init {
                    xdg_dirs: init.xdg_dirs.clone(),
                })
                .forward(sender.input_sender(), |_msg| Input::Ignore),
            input_pdf_selector: inputpdf::InputPdfSelector::builder()
                .launch(inputpdf::Init {
                    xdg_dirs: init.xdg_dirs.clone(),
                })
                .forward(sender.input_sender(), |_msg| Input::Ignore),

            output_io: None,
            output_dir: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings {
                        folder_mode: true,
                        cancel_label: "Cancel".to_string(),
                        accept_label: "Output Folder".to_string(),
                        create_folders: false,
                        is_modal: true,
                        filters: vec![],
                    },
                    text: "Output Folder",
                    recently_opened_files: recent_output_dirs,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), |path| {
                    Input::OutputIo(SelectedFileIo::for_dir(path))
                }),
            output_zip_dialog: SaveDialog::builder()
                .transient_for_native(&root)
                .launch(SaveDialogSettings {
                    cancel_label: "Cancel".to_string(),
                    accept_label: "Output ZIP".to_string(),
                    create_folders: true,
                    is_modal: true,
                    filters: vec![zip_filter],
                })
                .forward(sender.input_sender(), |response| match response {
                    SaveDialogResponse::Accept(path) => {
                        Input::OutputIo(SelectedFileIo::for_zip(path))
                    }
                    SaveDialogResponse::Cancel => Input::Ignore,
                }),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

/// Runs the GUI thread for the lifetime of the GUI itself.
pub fn run_gui(init: Init) {
    let app = RelmApp::new("travdata.gui").with_args(Vec::new());
    app.run::<Model>(init);
}
