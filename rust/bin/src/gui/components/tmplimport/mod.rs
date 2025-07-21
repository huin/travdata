mod importargsdialog;
mod tmplversiondialog;

use relm4::prelude::*;
use relm4_components::open_dialog;

use crate::{
    filesio,
    gui::{util, workers::tmplloader},
    template::{self, serialised},
};

/// Component to handle the interactive process of importing a template file from directory or ZIP
/// file.
pub struct TemplateImporter {
    open_dir_dialog: Controller<open_dialog::OpenDialog>,
    open_zip_dialog: Controller<open_dialog::OpenDialog>,
    version_dialog: Controller<tmplversiondialog::TemplateVersionDialog>,
    args_dialog: Controller<importargsdialog::ImportArgsDialog>,
    loader: relm4::WorkerController<tmplloader::TemplateLoader>,

    file_io_path: Option<filesio::FileIoPath>,
    preload: Option<tmplloader::Preload>,
}

#[derive(Debug)]
pub enum Input {
    RequestImportFromDir,
    RequestImportFromZip,
    // Internal:
    Noop,
    SelectedFilePathIo(filesio::FileIoPath),
    VersionResponse(tmplversiondialog::Output),
    ArgsResponse(importargsdialog::Output),
    LoadResponse(tmplloader::Output),
}

#[derive(Debug)]
pub enum Output {
    TemplateImported(template::Book),
    Error(String),
}

#[relm4::component(pub)]
impl SimpleComponent for TemplateImporter {
    type Input = Input;
    type Output = Output;
    type Init = ();

    view! {
        #[root]
        &gtk::Box {}
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let zip_filter = gtk::FileFilter::new();
        zip_filter.set_name(Some("ZIP archive"));
        zip_filter.add_pattern("*.zip");
        zip_filter.add_mime_type("application/zip");

        let model = TemplateImporter {
            open_dir_dialog: open_dialog::OpenDialog::builder()
                .transient_for_native(&root)
                .launch(open_dialog::OpenDialogSettings {
                    folder_mode: true,
                    cancel_label: "Cancel".into(),
                    accept_label: "Import".into(),
                    create_folders: false,
                    is_modal: true,
                    filters: vec![],
                })
                .forward(sender.input_sender(), |selection| {
                    use open_dialog::OpenDialogResponse::*;
                    match selection {
                        Accept(path) => {
                            Input::SelectedFilePathIo(filesio::FileIoPath::for_dir(path))
                        }
                        Cancel => Input::Noop,
                    }
                }),
            open_zip_dialog: open_dialog::OpenDialog::builder()
                .transient_for_native(&root)
                .launch(open_dialog::OpenDialogSettings {
                    folder_mode: false,
                    cancel_label: "Cancel".into(),
                    accept_label: "Import".into(),
                    create_folders: false,
                    is_modal: true,
                    filters: vec![zip_filter.clone()],
                })
                .forward(sender.input_sender(), |selection| {
                    use open_dialog::OpenDialogResponse::*;
                    match selection {
                        Accept(path) => {
                            Input::SelectedFilePathIo(filesio::FileIoPath::for_zip(path))
                        }
                        Cancel => Input::Noop,
                    }
                }),
            version_dialog: tmplversiondialog::TemplateVersionDialog::builder()
                .transient_for(&root)
                .launch(())
                .forward(sender.input_sender(), Input::VersionResponse),
            args_dialog: importargsdialog::ImportArgsDialog::builder()
                .transient_for(&root)
                .launch(())
                .forward(sender.input_sender(), Input::ArgsResponse),
            loader: tmplloader::TemplateLoader::builder()
                .detach_worker(())
                .forward(sender.input_sender(), Input::LoadResponse),

            file_io_path: None,
            preload: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Input::RequestImportFromDir => {
                self.open_dir_dialog.emit(open_dialog::OpenDialogMsg::Open);
            }
            Input::RequestImportFromZip => {
                self.open_zip_dialog.emit(open_dialog::OpenDialogMsg::Open);
            }
            Input::Noop => {}
            Input::SelectedFilePathIo(file_io_path) => {
                self.loader.emit(tmplloader::Input::RequestPreloadTemplate(
                    tmplloader::PreloadRequest {
                        file_io_path: file_io_path.clone(),
                        assume_version: None,
                    },
                ));
                self.file_io_path = Some(file_io_path);
            }
            Input::VersionResponse(version_response) => {
                use tmplversiondialog::Output::*;
                match version_response {
                    VersionResponse(version) => {
                        if let Some(file_io_path) = self.file_io_path.take() {
                            self.loader.emit(tmplloader::Input::RequestPreloadTemplate(
                                tmplloader::PreloadRequest {
                                    file_io_path,
                                    assume_version: Some(version),
                                },
                            ));
                        }
                    }
                    Cancelled => {
                        self.file_io_path = None;
                    }
                }
            }
            Input::ArgsResponse(args_response) => {
                use importargsdialog::Output::*;
                match args_response {
                    LoadArg(load_arg) => {
                        if let Some(preload) = self.preload.take() {
                            self.loader.emit(tmplloader::Input::RequestLoadTemplate(
                                tmplloader::LoadRequest {
                                    file_io_path: preload.file_io_path,
                                    load_arg,
                                    preload: preload.preload,
                                },
                            ));
                        }
                    }
                    Cancelled => {
                        self.file_io_path = None;
                        self.preload = None;
                    }
                }
            }
            Input::LoadResponse(load_response) => {
                use tmplloader::Output::*;
                match load_response {
                    PreloadComplete(preload) => {
                        let preload_data = preload.preload.preload_data();
                        self.preload = Some(preload);
                        self.args_dialog
                            .emit(importargsdialog::Input::RequestLoadArgs(preload_data));
                    }
                    LoadComplete(load) => {
                        self.file_io_path = None;
                        self.preload = None;
                        util::send_output_or_log(
                            Output::TemplateImported(load.tmpl),
                            "loaded template",
                            &sender,
                        );
                    }
                    LoadError(error) => match error.downcast_ref::<serialised::PreloadError>() {
                        Some(serialised::PreloadError::UnknownVersion) => {
                            self.version_dialog
                                .emit(tmplversiondialog::Input::RequestSelectVersion);
                        }
                        None => {
                            self.file_io_path = None;
                            util::send_output_or_log(
                                Output::Error(format!("{:?}", error)),
                                "error message",
                                &sender,
                            );
                        }
                    },
                }
            }
        }
    }
}
