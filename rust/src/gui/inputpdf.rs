use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    sync::Arc,
};

use gtk::prelude::{FrameExt, GridExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
};
use relm4_components::{
    open_button::{OpenButton, OpenButtonSettings},
    open_dialog::OpenDialogSettings,
    simple_combo_box::{SimpleComboBox, SimpleComboBoxMsg},
};

use crate::{config::root, gui::util};

/// Input messages for [InputPdfSelector].
#[derive(Debug)]
pub enum Input {
    /// Sepecifies the currently selected input PDF file path.
    #[allow(clippy::enum_variant_names)]
    InputPdf(PathBuf),
    SelectedConfig(Option<Arc<root::Config>>),
    SelectedBookIndex,
}

/// Output messages for [InputPdfSelector].
#[derive(Debug)]
pub enum Output {
    SelectedInputPdf(Option<PathBuf>),
    SelectedBookId(Option<String>),
}

/// Initialisation parameters for [InputPdfSelector].
pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
}

/// Relm4 component to select an input PDF file for Travdata.
pub struct InputPdfSelector {
    input_pdf_open: Controller<OpenButton>,
    book_selector: Controller<SimpleComboBox<BookEntry>>,

    config: Option<Arc<root::Config>>,

    input_pdf: Option<PathBuf>,
    book_id: Option<String>,
}

impl InputPdfSelector {
    fn auto_select_book(&mut self) {
        let input_pdf_filename_opt: Option<&str> = self
            .input_pdf
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(std::ffi::OsStr::to_str);
        let input_pdf_filename = match input_pdf_filename_opt {
            Some(input_pdf_filename) => input_pdf_filename,
            None => return,
        };
        let index_opt = self
            .find_book_entry_index(|book_entry| book_entry.filename_matches(input_pdf_filename));
        if let Some(index) = index_opt {
            self.book_selector
                .sender()
                .emit(SimpleComboBoxMsg::SetActiveIdx(index));
            return;
        }

        let unselected_index_opt =
            self.find_book_entry_index(|book_entry| book_entry.book_cfg.is_none());
        if let Some(index) = unselected_index_opt {
            self.book_selector
                .sender()
                .emit(SimpleComboBoxMsg::SetActiveIdx(index));
        }
    }

    fn find_book_entry_index<F>(&self, pred: F) -> Option<usize>
    where
        F: Fn(&BookEntry) -> bool,
    {
        self.book_selector
            .model()
            .variants
            .iter()
            .enumerate()
            .find_map(
                |(index, book_entry)| {
                    if pred(book_entry) {
                        Some(index)
                    } else {
                        None
                    }
                },
            )
    }
}

#[relm4::component(pub)]
impl SimpleComponent for InputPdfSelector {
    type Init = Init;

    type Input = Input;
    type Output = Output;

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
                attach[1, 3, 1, 1] = model.book_selector.widget(),
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Input::InputPdf(path) => {
                self.input_pdf = Some(path);
                self.auto_select_book();
                sender
                    .output_sender()
                    .emit(Output::SelectedInputPdf(self.input_pdf.clone()));
                sender
                    .output_sender()
                    .emit(Output::SelectedBookId(self.book_id.clone()));
            }
            Input::SelectedConfig(config) => {
                self.config = config;
                let mut variants = match &self.config {
                    None => vec![BookEntry::unselected()],
                    Some(config) => Some(BookEntry::unselected())
                        .into_iter()
                        .chain(config.books.values().map(|book_cfg| BookEntry {
                            book_cfg: Some(book_cfg.clone()),
                        }))
                        .collect(),
                };
                variants.sort_by(|a, b| a.name_opt().cmp(&b.name_opt()));
                self.book_selector
                    .sender()
                    .emit(SimpleComboBoxMsg::UpdateData(SimpleComboBox {
                        variants,
                        active_index: None,
                    }));
                // TODO: Find a way to wait for `self.book_selector` to have been updated, then
                // call `self.auto_select_book()`.
            }
            Input::SelectedBookIndex => {
                self.book_id = self
                    .book_selector
                    .model()
                    .get_active_elem()
                    .and_then(|book_entry| book_entry.id_opt())
                    .map(|id| id.to_owned());
                sender
                    .output_sender()
                    .emit(Output::SelectedBookId(self.book_id.clone()));
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
            book_selector: SimpleComboBox::builder()
                .launch(SimpleComboBox {
                    variants: vec![],
                    active_index: None,
                })
                .forward(sender.input_sender(), |_| Input::SelectedBookIndex),

            config: None,

            input_pdf: None,
            book_id: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

struct BookEntry {
    book_cfg: Option<Arc<root::Book>>,
}

impl BookEntry {
    fn unselected() -> Self {
        Self { book_cfg: None }
    }

    fn id_opt(&self) -> Option<&str> {
        self.book_cfg.as_ref().map(|book_cfg| book_cfg.id.as_ref())
    }

    fn name_opt(&self) -> Option<&str> {
        self.book_cfg
            .as_ref()
            .map(|book_cfg| book_cfg.name.as_ref())
    }

    fn filename_matches(&self, filename: &str) -> bool {
        match &self.book_cfg {
            Some(book_cfg) => book_cfg.default_filename == filename,
            None => false,
        }
    }
}

impl Debug for BookEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.book_cfg {
            Some(book_cfg) => f
                .debug_struct("BookEntry")
                .field("book_cfg.id", &book_cfg.id)
                .finish(),
            None => f
                .debug_struct("BookEntry")
                .field("book_cfg", &"None")
                .finish(),
        }
    }
}

impl Display for BookEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.book_cfg {
            Some(book_cfg) => f.write_str(&book_cfg.name),
            None => f.write_str("<unselected>"),
        }
    }
}
