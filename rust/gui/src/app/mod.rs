mod ddo;
mod error_modal;
mod shortcuts;
mod workers;

use std::sync::Arc;

use shortcuts::Shortcuts;

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
    /// NOTE: to mutate the PipelineNodes, use Arc::make_mut. This will clone only if necessary
    /// (e.g. making a modification if the pipeline is currently being saved).
    pipeline: Loadable<Arc<ddo::PipelineNodes>, ddo::PathSelection>,
}

#[derive(Default)]
struct TransientState {
    inbox: egui_inbox::UiInbox<InboxMessage>,
    displayed_error: Option<String>,
    disable_file_pickers: bool,
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
            let reload: Option<ddo::PathSelection> = match &self.state.pipeline {
                Loadable::Unloaded => {
                    ui.label("No pipeline loaded.");
                    None
                }
                Loadable::Loading { source } => {
                    ui.label(&source.as_string);
                    ui.label("loading...");
                    ui.spinner();
                    None
                }
                Loadable::LoadOk { source: _, loaded } => {
                    ui.label("Node count: ");
                    // TODO: don't format every frame
                    ui.label(format!("{}", loaded.len()));

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for node in loaded.iter() {
                            ui.label(&node.meta.id.0);
                        }
                    });

                    None
                }
                Loadable::LoadErr { source, error } => {
                    ui.label(&source.as_string);
                    ui.label("Loading error:");
                    ui.label(error);
                    if ui.button("Reload").clicked() {
                        Some(source.clone())
                    } else {
                        None
                    }
                }
            };
            if let Some(source) = reload {
                self.start_loading_pipeline(source);
            }

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
                    self.state.pipeline = match result {
                        Ok(loaded) => Loadable::LoadOk {
                            source,
                            loaded: Arc::new(loaded),
                        },
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

                let opt_load_ok = self.state.pipeline.as_load_ok();
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
                        !self.state.pipeline.is_unloaded(),
                        egui::Button::new("Close pipeline"),
                    )
                    .clicked()
                {
                    self.state.pipeline = Loadable::Unloaded;
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
        // TODO: if we serialize in this state, it won't load without more help on startup to
        // restart the loading thread.
        self.state.pipeline = Loadable::Loading {
            source: source.clone(),
        };
        workers::pipeline_loader::start_load(
            source,
            self.transient.inbox.sender(),
            InboxMessage::LoadedPipeline,
        );
    }

    fn start_saving_pipeline(&mut self) {
        if let Some((path_selection, pipeline)) = self.state.pipeline.as_load_ok() {
            workers::pipeline_loader::start_save(
                path_selection.path.to_path_buf(),
                pipeline,
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
