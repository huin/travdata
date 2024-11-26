use std::{path::PathBuf, sync::Arc};

use gtk::prelude::{BoxExt, ButtonExt, FrameExt, GridExt, OrientableExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    WorkerController,
};
use relm4_components::{
    open_button::{OpenButton, OpenButtonSettings},
    open_dialog::OpenDialogSettings,
};

use crate::{
    config::root,
    gui::util::{self, SelectedFileIo},
};

use super::workers::cfgloader::{self, ConfigLoader};

/// Input messages for [ConfigSelector].
#[derive(Debug)]
pub enum Input {
    /// Specifies to select the default configuration.
    SelectDefault,
    /// Specifies the currently selected extraction configuration.
    ConfigIo(SelectedFileIo),
    LoadComplete(cfgloader::LoadComplete),
    LoadError(cfgloader::LoadError),
}

/// Output messages from [ConfigSelector].
#[derive(Debug)]
pub enum Output {
    SelectedConfig(Option<(SelectedFileIo, Arc<root::Config>)>),
}

/// Initialisation parameters for [ConfigSelector].
pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
    pub default_config: Option<PathBuf>,
}

/// Relm4 component to select Travdata extraction configuration.
#[allow(dead_code)]
pub struct ConfigSelector {
    default_config: Option<PathBuf>,

    cfg_dir: Controller<OpenButton>,
    cfg_zip: Controller<OpenButton>,
    cfg_io: Option<util::SelectedFileIo>,
    cfg_error: Option<String>,
    cfg_version: Option<String>,
    loader: WorkerController<ConfigLoader>,
}

#[relm4::component(pub)]
impl SimpleComponent for ConfigSelector {
    type Init = Init;

    type Input = Input;
    type Output = Output;

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
                        set_sensitive: model.default_config.is_some(),

                        connect_clicked[sender] => move |_| {
                            sender.input(Input::SelectDefault);
                        },
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
                    #[watch]
                    set_label: model
                        .cfg_version.as_ref()
                        .map(String::as_ref)
                        .unwrap_or_else(
                            || if model.cfg_io.is_some() {"<unknown>"} else {"<unselected>"}
                        ),
                    set_halign: gtk::Align::Start,
                },

                attach[0, 3, 2, 1] = &gtk::Label {
                    // Error box.
                    #[watch]
                    set_label: model.cfg_error.as_ref().map(String::as_ref).unwrap_or_default(),
                    set_halign: gtk::Align::Start,
                },
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Input::SelectDefault => match &self.default_config {
                None => {
                    log::warn!(
                        "Select default was requested, but there is no default configuration."
                    );
                }
                Some(default_config) => {
                    let io = SelectedFileIo::for_auto(default_config.clone());
                    self.loader.emit(cfgloader::Input::RequestLoadConfig(io));
                }
            },
            Input::ConfigIo(io) => {
                self.loader.emit(cfgloader::Input::RequestLoadConfig(io));
            }
            Input::LoadComplete(cfgloader::LoadComplete {
                io,
                config,
                version,
            }) => {
                self.cfg_io = Some(io.clone());
                self.cfg_version = version;
                self.cfg_error = None;
                let config = Arc::new(config);
                util::send_output_or_log(
                    Output::SelectedConfig(Some((io, config))),
                    "selected configuration",
                    sender,
                );
            }
            Input::LoadError(cfgloader::LoadError { io: _, message }) => {
                self.cfg_io = None;
                self.cfg_version = None;
                self.cfg_error = Some(message);
                util::send_output_or_log(
                    Output::SelectedConfig(None),
                    "deselected configuration",
                    sender,
                );
            }
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
            default_config: init.default_config,

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
            cfg_error: None,
            cfg_version: None,
            loader: ConfigLoader::builder().detach_worker(()).forward(
                sender.input_sender(),
                |msg| match msg {
                    cfgloader::Output::LoadComplete(v) => Input::LoadComplete(v),
                    cfgloader::Output::LoadError(v) => Input::LoadError(v),
                },
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
