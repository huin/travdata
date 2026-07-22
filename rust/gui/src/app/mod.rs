mod components;
mod data;
mod ddo;
mod error_modal;
mod shortcuts;
mod workers;

use shortcuts::Shortcuts;

use crate::app::workers::pipeline_loader;

const IS_WEB: bool = cfg!(target_arch = "wasm32");

pub struct App {
    transient: TransientState,
    shortcuts: Shortcuts,

    state: AppState,
}

/// Serializable state recovered on reloading the application.
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct AppState {
    pipeline_editor: Loadable<components::PipelineEditor, ddo::PathSelection>,
}

#[derive(Default)]
struct TransientState {
    inbox: egui_inbox::UiInbox<InboxMessage>,
    displayed_error: Option<String>,
    disable_file_pickers: bool,
    pipeline_loading: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state: AppState = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        App {
            transient: Default::default(),
            shortcuts: Shortcuts::new(&cc.egui_ctx),
            state,
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _eframe: &mut eframe::Frame) {
        self.handle_inbox(ui);
        self.handle_shortcuts(ui);

        if let Some(displayed_error) = &self.transient.displayed_error
            && error_modal::display_error(ui, displayed_error)
        {
            self.transient.displayed_error = None;
        }

        egui::Panel::top("top_panel").show(ui, |ui| {
            self.handle_main_menu(ui);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            self.handle_pipeline_panel(ui);

            ui.separator();

            egui::Panel::bottom("bottom_panel").show(ui, |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

impl App {
    fn handle_inbox(&mut self, ui: &egui::Ui) {
        for msg in self.transient.inbox.read(ui) {
            match msg {
                InboxMessage::SelectedPipelinePath(result) => {
                    self.transient.disable_file_pickers = false;
                    match result {
                        Ok(Some(path_selection)) => {
                            self.start_loading_pipeline(path_selection);
                        }
                        Ok(None) => {}
                        Err(message) => self.transient.displayed_error = Some(message),
                    }
                }
                InboxMessage::LoadedPipeline(source, result) => {
                    self.state.pipeline_editor =
                        match result.and_then(components::PipelineEditor::new) {
                            Ok(loaded) => Loadable::LoadOk { source, loaded },
                            Err(error) => Loadable::LoadErr { source, error },
                        };
                }
                InboxMessage::SaveCompleted(result) => {
                    if let Err(message) = result {
                        self.transient.displayed_error = Some(message);
                    }
                }
            }
        }
    }

    fn handle_shortcuts(&mut self, ui: &mut egui::Ui) {
        if self.shortcuts.open.consume(ui) {
            self.start_open_pipeline();
        }
        if self.shortcuts.save.consume(ui) {
            self.start_saving_pipeline();
        }
    }

    fn handle_main_menu(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui
                    .add_enabled(
                        !self.transient.disable_file_pickers,
                        egui::Button::new("Open pipeline...")
                            .shortcut_text(&self.shortcuts.open.formatted),
                    )
                    .clicked()
                {
                    self.start_open_pipeline();
                }

                let opt_load_ok = self.state.pipeline_editor.as_load_ok();
                if ui
                    .add_enabled(
                        opt_load_ok.is_some(),
                        egui::Button::new("Save pipeline")
                            .shortcut_text(&self.shortcuts.save.formatted),
                    )
                    .clicked()
                {
                    self.start_saving_pipeline();
                }

                if ui
                    .add_enabled(
                        !self.state.pipeline_editor.is_unloaded(),
                        egui::Button::new("Close pipeline"),
                    )
                    .clicked()
                {
                    self.state.pipeline_editor = Loadable::Unloaded;
                }

                if !IS_WEB {
                    // NOTE: no File->Quit on web pages!
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });

            ui.menu_button("View", |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
    }

    fn handle_pipeline_panel(&mut self, ui: &mut egui::Ui) {
        let start_loading: Option<ddo::PathSelection> = match &mut self.state.pipeline_editor {
            Loadable::Unloaded => {
                ui.push_id("unloaded", |ui| {
                    ui.label("No pipeline loaded.");
                });
                self.transient.pipeline_loading = false;
                None
            }
            Loadable::Loading { source } => {
                ui.push_id("loading", |ui| {
                    ui.label(&source.as_string);
                    ui.label("loading...");
                    ui.spinner();
                });

                if !self.transient.pipeline_loading {
                    // This should typically only happen if the application closed with the pipeline
                    // in the Loading state before.
                    Some(source.clone())
                } else {
                    None
                }
            }
            Loadable::LoadOk {
                source: _,
                loaded: pipeline_editor,
            } => {
                self.transient.pipeline_loading = false;
                ui.push_id("editor", |ui| {
                    pipeline_editor.ui(ui);
                });
                None
            }
            Loadable::LoadErr { source, error } => {
                self.transient.pipeline_loading = false;
                ui.push_id("load_err", |ui| {
                    ui.label(&source.as_string);
                    ui.label("Loading error:");
                    ui.label(&*error);
                    if ui.button("Reload").clicked() {
                        Some(source.clone())
                    } else {
                        None
                    }
                })
                .inner
            }
        };
        if let Some(source) = start_loading {
            self.start_loading_pipeline(source);
        }
    }

    fn start_open_pipeline(&mut self) {
        if self.transient.disable_file_pickers {
            return;
        }

        workers::path_selection::pick_single_file(
            rfd::FileDialog::new().add_filter("JSON", &["json"]),
            self.transient.inbox.sender(),
            InboxMessage::SelectedPipelinePath,
        );
        self.transient.disable_file_pickers = true;
    }

    fn start_loading_pipeline(&mut self, source: ddo::PathSelection) {
        self.state.pipeline_editor = Loadable::Loading {
            source: source.clone(),
        };
        workers::pipeline_loader::start_load(
            source.clone(),
            self.transient.inbox.sender(),
            InboxMessage::LoadedPipeline,
        );
        self.transient.pipeline_loading = true;
    }

    fn start_saving_pipeline(&mut self) {
        if let Some((path_selection, pipeline_editor)) = self.state.pipeline_editor.as_load_ok() {
            let pipeline = match pipeline_editor.pipeline_for_serialisation() {
                Ok(pipeline) => pipeline,
                Err(message) => {
                    self.transient.displayed_error = Some(message);
                    return;
                }
            };
            pipeline_loader::start_save(
                pipeline,
                path_selection.path.clone(),
                self.transient.inbox.sender(),
                InboxMessage::SaveCompleted,
            );
        }
    }
}

#[derive(Debug)]
enum InboxMessage {
    SelectedPipelinePath(Result<Option<ddo::PathSelection>, String>),
    LoadedPipeline(ddo::PathSelection, Result<ddo::PipelineNodes, String>),
    SaveCompleted(Result<(), String>),
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
enum Loadable<T, S> {
    #[default]
    Unloaded,
    Loading {
        source: S,
    },
    LoadOk {
        source: S,
        loaded: T,
    },
    LoadErr {
        source: S,
        error: String,
    },
}

impl<T, S> Loadable<T, S> {
    fn is_unloaded(&self) -> bool {
        matches!(self, Self::Unloaded)
    }

    fn as_load_ok(&self) -> Option<(&S, &T)> {
        match self {
            Self::LoadOk { source, loaded } => Some((source, loaded)),
            _ => None,
        }
    }
}
