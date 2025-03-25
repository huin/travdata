use std::{path::PathBuf, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;

use crate::{
    commontext,
    config::root,
    extraction::pdf::pdfiumthread::PdfiumClient,
    filesio::FileIoPath,
    gui::{cfgselect, extract, inputpdf, outputselect, pageview},
    template,
};

use super::{
    components::{errordialog, tmplimport},
    extractionlist, mainmenu, treelist,
    workers::{self, extractor},
};

/// Input messages for [MainWindow].
#[derive(Debug)]
pub enum Input {
    // Internal:
    Noop,
    ShowError(String),
    ImportConfig(Option<(FileIoPath, Arc<root::Config>)>),
    ImportTemplate(template::Book),
    #[allow(clippy::enum_variant_names)]
    ExtractorInput(extract::Input),
    MainMenuAction(mainmenu::Action),
}

/// Initialisation parameters for [MainWindow].
pub struct Init {
    pub xdg_dirs: Arc<xdg::BaseDirectories>,
    pub default_config: Option<PathBuf>,
    pub pdfium_client: PdfiumClient,
    pub worker_channel: workers::extractor::WorkChannel,
}

/// Relm4 window component that acts as the main window for the GUI interface to Travdata.
pub struct MainWindow {
    // main_menu: Controller<mainmenu::MainMenu>,
    error_msg_dialog: Controller<errordialog::ErrorDialog>,
    cfg_selector: Controller<cfgselect::ConfigSelector>,
    tmpl_importer: Controller<tmplimport::TemplateImporter>,
    input_pdf_selector: Controller<inputpdf::InputPdfSelector>,
    output_selector: Controller<outputselect::OutputSelector>,
    extractor: Controller<extract::Extractor>,

    tab_label_extract: gtk::Label,
    tab_label_tree_list: gtk::Label,
    tab_label_list: gtk::Label,
    tab_label_edit_config: gtk::Label,
    tree_list: Controller<treelist::TreeList>,
    extraction_list: Controller<extractionlist::ExtractionList>,
    page_view: Controller<pageview::PageView>,
}

#[relm4::component(pub)]
impl SimpleComponent for MainWindow {
    type Init = Init;

    type Input = Input;
    type Output = ();

    view! {
        window = gtk::ApplicationWindow {
            set_title: Some("Travdata"),
            set_default_width: 300,
            set_default_height: 600,
            set_show_menubar: true,

            gtk::Notebook {
                append_page[Some(&model.tab_label_extract)] = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                    set_margin_all: 5,

                    model.tmpl_importer.widget(),

                    gtk::Label {
                        set_label: commontext::DATA_USAGE,
                        set_halign: gtk::Align::Start,
                        set_hexpand: true,
                    },

                    model.cfg_selector.widget(),
                    model.input_pdf_selector.widget(),
                    model.output_selector.widget(),

                    model.extractor.widget(),
                },

                append_page[Some(&model.tab_label_tree_list)] = &gtk::Box {
                    gtk::ScrolledWindow {
                        container_add: model.tree_list.widget(),
                    },
                },

                append_page[Some(&model.tab_label_list)] = &gtk::Box {
                    gtk::ScrolledWindow {
                        container_add: model.extraction_list.widget(),
                    },
                },

                // TODO: Implement appropriate editing GUI.
                append_page[Some(&model.tab_label_edit_config)] = &gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    model.page_view.widget(),
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::Noop => {}
            Input::ShowError(message) => {
                self.error_msg_dialog
                    .emit(errordialog::Input::ShowErrorMessage(message));
            }
            Input::ExtractorInput(extractor_input) => {
                // Update page view if the event relates to PDF selection.
                if let extract::Input::InputPdf(path) = &extractor_input {
                    let path = path.clone();
                    self.page_view.emit(pageview::Input::SelectPdf { path });
                }

                self.extractor.emit(extractor_input);
            }
            Input::ImportTemplate(tmpl) => {
                // TODO
                log::info!("TODO use the `tmpl`: {:?}", tmpl);
            }
            Input::ImportConfig(config_opt) => match config_opt {
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
            Input::MainMenuAction(action) => {
                self.handle_menu_action(action);
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            error_msg_dialog: errordialog::ErrorDialog::builder()
                .transient_for(&root)
                .launch(())
                .forward(sender.input_sender(), |_| Input::Noop),
            cfg_selector: cfgselect::ConfigSelector::builder()
                .launch(cfgselect::Init {
                    xdg_dirs: init.xdg_dirs.clone(),
                    default_config: init.default_config,
                })
                .forward(sender.input_sender(), |msg| match msg {
                    cfgselect::Output::SelectedConfig(config_opt) => {
                        Input::ImportConfig(config_opt)
                    }
                }),
            tmpl_importer: tmplimport::TemplateImporter::builder().launch(()).forward(
                sender.input_sender(),
                |msg| {
                    use tmplimport::Output::*;
                    match msg {
                        TemplateImported(tmpl) => Input::ImportTemplate(tmpl),
                        Error(message) => Input::ShowError(message),
                    }
                },
            ),
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

            tab_label_extract: gtk::Label::new(Some("Extract")),
            tab_label_tree_list: gtk::Label::new(Some("Tree list")),
            tab_label_list: gtk::Label::new(Some("List")),
            tab_label_edit_config: gtk::Label::new(Some("Edit Configuration")),

            tree_list: treelist::TreeList::builder().launch(()).detach(),
            extraction_list: extractionlist::ExtractionList::builder()
                .launch(())
                .detach(),
            page_view: pageview::PageView::builder()
                .launch(init.pdfium_client)
                .detach(),
        };

        let widgets = view_output!();

        {
            let sender = sender.clone();
            mainmenu::init_for_widget(&widgets.window, move |action| {
                sender.input(Input::MainMenuAction(action));
            });
        }

        ComponentParts { model, widgets }
    }
}

impl MainWindow {
    fn handle_menu_action(&mut self, action: mainmenu::Action) {
        use mainmenu::Action::*;
        match action {
            FileQuit => {
                relm4::main_application().quit();
            }
            TemplateImportDir => {
                self.tmpl_importer
                    .emit(tmplimport::Input::RequestImportFromDir);
            }
            TemplateImportZip => {
                self.tmpl_importer
                    .emit(tmplimport::Input::RequestImportFromZip);
            }
            action => {
                // TODO: Handle the other actions.
                log::warn!("Unimplemented menu action: {:?}.", action);
            }
        }
    }
}
