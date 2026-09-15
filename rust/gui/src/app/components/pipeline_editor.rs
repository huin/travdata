use slotmap::Key;

use crate::app::{components::node_editor, data};

#[derive(Copy, Clone, Default, serde::Deserialize, serde::Serialize)]
struct PipelineEditorState {
    selected_node_ref: data::NodeRef,
}

impl PipelineEditorState {
    fn get_or_default(ui: &mut egui::Ui) -> Self {
        ui.data_mut(|data| data.get_persisted(ui.id()).unwrap_or_default())
    }

    fn store(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| data.insert_persisted(ui.id(), self));
    }
}

pub struct PipelineEditor<'pl> {
    pipeline: &'pl mut data::EditablePipeline,
}

impl<'pl> egui::Widget for PipelineEditor<'pl> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.show(ui);
        ui.response()
    }
}

impl<'pl> PipelineEditor<'pl> {
    pub fn new(pipeline: &'pl mut data::EditablePipeline) -> Self {
        Self { pipeline }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        ui.push_id("PipelineEditor", |ui| {
            let mut state = PipelineEditorState::get_or_default(ui);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let text_height = egui::TextStyle::Body
                    .resolve(ui.style())
                    .size
                    .max(ui.spacing().interact_size.y);

                let available_height = ui.available_height();
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .column(egui_extras::Column::auto())
                    .column(egui_extras::Column::remainder())
                    .min_scrolled_height(10.0)
                    .max_scroll_height(available_height)
                    .sense(egui::Sense::click())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("ID");
                        });
                        header.col(|ui| {
                            ui.strong("Type");
                        });
                    })
                    .body(|body| {
                        body.rows(text_height, self.pipeline.len(), |mut row| {
                            let row_index = row.index();
                            let (node_ref, gui_node) =
                                match self.pipeline.get_node_by_index(row_index) {
                                    Some(node) => node,
                                    None => return,
                                };
                            row.set_selected(state.selected_node_ref == node_ref);

                            let mut do_select = false;
                            row.col(|ui| {
                                ui.dnd_drag_source(ui.id().with("node_id"), node_ref, |ui| {
                                    do_select |= ui.label(&gui_node.node_id).clicked();
                                });
                            });
                            row.col(|ui| {
                                do_select |= ui.label(gui_node.node.spec.variant_name()).clicked();
                            });
                            do_select |= row.response().clicked();

                            if do_select {
                                state.selected_node_ref = node_ref;
                            }
                        });
                    });
            });

            ui.separator();

            if ui
                .add_enabled(
                    !state.selected_node_ref.is_null(),
                    egui::Button::new("Delete node"),
                )
                .clicked()
            {
                self.pipeline.remove_node(state.selected_node_ref);
                state.selected_node_ref = data::NodeRef::null();
            }

            self.pipeline
                .with_node_ctx_by_ref_mut(state.selected_node_ref, |node_ctx| {
                    match node_ctx {
                        Some(mut node_ctx) => {
                            ui.add(node_editor::NodeEditor::new(&mut node_ctx));
                        }
                        None => {
                            ui.push_id("no-selection", |ui| {
                                ui.label("No node selected.");
                            });
                        }
                    };
                });

            state.store(ui);
        });
    }
}
