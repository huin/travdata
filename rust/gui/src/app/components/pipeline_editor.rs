use generic_pipeline::systems::TypedNode as _;
use strum::VariantMetadata as _;

use crate::app::{components::node_editor, data};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct PipelineEditor {
    selected_node_ref: Option<data::NodeRef>,
}

impl PipelineEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui, pipeline: &mut data::EditablePipeline) {
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
                    body.rows(text_height, pipeline.len(), |mut row| {
                        let row_index = row.index();
                        let (node_ref, node) = match pipeline.get_node_by_index(row_index) {
                            Some(node) => node,
                            None => return,
                        };
                        row.set_selected(self.selected_node_ref == Some(node_ref));

                        let mut do_select = false;
                        row.col(|ui| {
                            do_select |= ui.label(&node.meta.id.0).clicked();
                        });
                        row.col(|ui| {
                            do_select |= ui.label(node.spec.node_type().variant_name()).clicked();
                        });
                        do_select |= row.response().clicked();

                        if do_select {
                            self.selected_node_ref = Some(node_ref);
                        }
                    });
                });

            ui.separator();

            match self
                .selected_node_ref
                .and_then(|node_ref| pipeline.get_node_ctx_by_ref_mut(node_ref))
            {
                Some(mut node_ctx) => {
                    ui.push_id(node_ctx.node_ref, |ui| {
                        node_editor::node_editor_ui(ui, &mut node_ctx);
                    });
                }
                None => {
                    ui.push_id("no-selection", |ui| {
                        ui.label("No node selected.");
                    });
                }
            };
        });
    }
}
