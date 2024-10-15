use std::{path::PathBuf, sync::Arc};

use gtk::prelude::{FrameExt, GridExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
};
use relm4_components::{
    open_button::{OpenButton, OpenButtonSettings},
    open_dialog::OpenDialogSettings,
};

use crate::gui::util;

/// Input messages for [InputPdfSelector].
#[derive(Debug)]
pub enum Input {
    /// Sepecifies the currently selected input PDF file path.
    #[allow(clippy::enum_variant_names)]
    InputPdf(PathBuf),
}

/// Initialisation parameters for [InputPdfSelector].
pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
}

/// Relm4 component to select an input PDF file for Travdata.
#[allow(dead_code)]
pub struct InputPdfSelector {
    input_pdf_open: Controller<OpenButton>,
    input_pdf: Option<PathBuf>,
    book_id: Option<String>,
}

#[relm4::component(pub)]
impl SimpleComponent for InputPdfSelector {
    type Init = Init;

    type Input = Input;
    type Output = ();

    view! {
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
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::InputPdf(path) => {
                self.input_pdf = Some(path.clone());
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let recent_input_pdfs = util::xdg_cfg_static_str(&init.xdg_dirs, "recent_input_pdfs.txt");

        let pdf_filter = gtk::FileFilter::new();
        pdf_filter.set_name(Some("PDF file"));
        pdf_filter.add_pattern("*.pdf");
        pdf_filter.add_mime_type("application/pdf");

        let model = Self {
            input_pdf_open: OpenButton::builder()
                .launch(OpenButtonSettings {
                    dialog_settings: OpenDialogSettings {
                        folder_mode: false,
                        cancel_label: "Cancel".to_string(),
                        accept_label: "Open".to_string(),
                        create_folders: false,
                        is_modal: true,
                        filters: vec![pdf_filter.clone()],
                    },
                    text: "Select Input PDF",
                    recently_opened_files: recent_input_pdfs,
                    max_recent_files: 10,
                })
                .forward(sender.input_sender(), Input::InputPdf),
            input_pdf: None,
            book_id: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
