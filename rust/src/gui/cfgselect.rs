use std::sync::Arc;

use gtk::prelude::{BoxExt, FrameExt, GridExt, OrientableExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
};
use relm4_components::{
    open_button::{OpenButton, OpenButtonSettings},
    open_dialog::OpenDialogSettings,
};

use crate::{
    config::root::Config,
    gui::util::{self, SelectedFileIo},
};

/// Input messages for [ConfigSelector].
#[derive(Debug)]
pub enum Input {
    /// Specifies the currently selected extraction configuration.
    ConfigIo(SelectedFileIo),
}

/// Initialisation parameters for [ConfigSelector].
pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
}

/// Relm4 component to select Travdata extraction configuration.
#[allow(dead_code)]
pub struct ConfigSelector {
    cfg_dir: Controller<OpenButton>,
    cfg_zip: Controller<OpenButton>,
    cfg_io: Option<util::SelectedFileIo>,
    cfg: Option<Config>,
    cfg_error: Option<String>,
    cfg_version: Option<String>,
}

#[relm4::component(pub)]
impl SimpleComponent for ConfigSelector {
    type Init = Init;

    type Input = Input;
    type Output = ();

    view! {
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
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::ConfigIo(io) => self.cfg_io = Some(io),
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let recent_cfg_dirs = util::xdg_cfg_static_str(&init.xdg_dirs, "recent_cfg_dirs.txt");
        let recent_cfg_zips = util::xdg_cfg_static_str(&init.xdg_dirs, "recent_cfg_zips.txt");

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
                        accept_label: "Open".to_string(),
                        create_folders: false,
                        is_modal: true,
                        filters: vec![],
                    },
                    text: "Select directory",
                    recently_opened_files: recent_cfg_dirs,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), |path| {
                    Input::ConfigIo(SelectedFileIo::for_dir(path))
                }),
            cfg_zip: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings {
                        folder_mode: false,
                        cancel_label: "Cancel".to_string(),
                        accept_label: "Open".to_string(),
                        create_folders: false,
                        is_modal: true,
                        filters: vec![zip_filter.clone()],
                    },
                    text: "Select ZIP",
                    recently_opened_files: recent_cfg_zips,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), |path| {
                    Input::ConfigIo(SelectedFileIo::for_zip(path))
                }),
            cfg_io: None,
            cfg: None,
            cfg_error: None,
            cfg_version: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
