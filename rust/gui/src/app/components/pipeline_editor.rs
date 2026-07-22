use generic_pipeline::systems::TypedNode as _;
use strum::VariantMetadata as _;

use crate::app::{components::node_editor, data, ddo};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct PipelineEditor {
    // TODO: Consider factoring out `nodes`, and only keep the component state in here. Passing in
    // OrderedNodeSet to `::ui()`.
    nodes: data::OrderedNodeSet,
    selected_node_ref: Option<data::NodeRef>,
}

impl PipelineEditor {
    pub fn new(pipeline: ddo::PipelineNodes) -> Result<Self, String> {
        Ok(Self {
            nodes: data::OrderedNodeSet::new(pipeline)?,
            selected_node_ref: None,
        })
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
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
                    body.rows(text_height, self.nodes.len(), |mut row| {
                        let row_index = row.index();
                        let (node_ref, node) = match self.nodes.get_by_index_mut(row_index) {
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

            match self.selected_node_ref.and_then(|node_ref| {
                self.nodes
                    .get_by_node_ref_mut(node_ref)
                    .map(|node| (node_ref, node))
            }) {
                Some((node_ref, node)) => {
                    ui.push_id(node_ref, |ui| {
                        node_editor::node_editor_ui(ui, node);
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

    pub fn pipeline_for_serialisation(&self) -> Result<ddo::PipelineNodes, String> {
        self.nodes
            .nodes_in_order()
            .map(|node_opt| {
                node_opt
                    .cloned()
                    .ok_or("could not resolve NodeRef")
                    .map_err(String::from)
            })
            .collect::<Result<Vec<_>, String>>()
    }
}
