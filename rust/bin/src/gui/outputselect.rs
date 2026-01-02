use std::sync::Arc;

use gtk::prelude::{BoxExt, ButtonExt, FrameExt, GridExt, OrientableExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
    gtk,
};
use relm4_components::{
    open_button::{OpenButton, OpenButtonSettings},
    open_dialog::OpenDialogSettings,
    save_dialog::{SaveDialog, SaveDialogResponse, SaveDialogSettings},
};

use crate::{
    filesio::{FileIoPath, IoType},
    gui::util,
};

/// Input messages for [OutputSelector].
#[derive(Debug)]
pub enum Input {
    /// Specifies the currently selected output destinatiom.
    OutputIo(FileIoPath),
    /// (Internal) Triggers opening the ZIP file selection.
    OutputZipRequest,
    /// No-op message.
    Ignore,
}

/// Output messages for [OutputSelector].
#[derive(Debug)]
pub enum Output {
    SelectedOutputIo(Option<FileIoPath>),
}

/// Initialisation parameters for [OutputSelector].
pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
}

/// Relm4 component to select Travdata extraction output destination file/folder.
#[allow(dead_code)]
pub struct OutputSelector {
    output_io: Option<FileIoPath>,
    output_dir: Controller<OpenButton>,
    output_zip_dialog: Controller<SaveDialog>,
}

#[relm4::component(pub)]
impl SimpleComponent for OutputSelector {
    type Init = Init;

    type Input = Input;
    type Output = Output;

    view! {
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
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Input::OutputIo(io) => {
                self.output_io = Some(io);
                sender
                    .output_sender()
                    .emit(Output::SelectedOutputIo(self.output_io.clone()));
            }
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

        let zip_filter = gtk::FileFilter::new();
        zip_filter.set_name(Some("ZIP archive"));
        zip_filter.add_pattern("*.zip");
        zip_filter.add_mime_type("application/zip");

        let model = Self {
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
                    icon: None,
                    recently_opened_files: recent_output_dirs,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), |path| {
                    Input::OutputIo(FileIoPath::for_dir(path))
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
                    SaveDialogResponse::Accept(path) => Input::OutputIo(FileIoPath::for_zip(path)),
                    SaveDialogResponse::Cancel => Input::Ignore,
                }),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
