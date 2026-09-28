use std::sync::Arc;

use itertools::intersperse;

use crate::app::{
    components::{node_ref_editor::NodeIdRefEditor, todo_ui},
    data::{self, guinode},
};

pub struct NodeEditor<'node_ctx, 'node_ref> {
    node_ctx: &'node_ctx mut data::NodeContextMut<'node_ref>,
}

impl<'node_ref, 'node> egui::Widget for NodeEditor<'node_ref, 'node> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.show(ui);
        ui.response()
    }
}

impl<'node_ctx, 'node_ref> NodeEditor<'node_ctx, 'node_ref>
where
    'node_ref: 'node_ctx,
{
    pub fn new(node_ctx: &'node_ctx mut data::NodeContextMut<'node_ref>) -> Self {
        Self { node_ctx }
    }

    pub fn show(mut self, ui: &mut egui::Ui) {
        ui.push_id(("NodeEditor", self.node_ctx.node.meta.self_ref), |ui| {
            form_grid(ui, "node_editor_ui", |ui| {
                self.node_meta_editor(ui);
                self.node_spec_editor(ui);
            });
        });
    }

    fn node_meta_editor(&mut self, ui: &mut egui::Ui) {
        ui.label("ID:");
        if ui
            .text_edit_singleline(&mut self.node_ctx.node.meta.id)
            .changed()
        {
            self.node_ctx.mark_node_changed();
        }
        ui.end_row();
    }

    fn node_spec_editor(&mut self, ui: &mut egui::Ui) {
        use guinode::Spec;

        match &mut self.node_ctx.node.spec {
            Spec::InputPdfFile(spec) => {
                ui.label("PDF description:");
                if ui.text_edit_multiline(&mut spec.description).changed() {
                    self.node_ctx.mark_node_changed();
                }
                ui.end_row();
            }
            Spec::JsContext(_spec) => {
                // No fields yet.
                ui.label("No settings for JsContext yet.");
                ui.end_row();
            }
            Spec::JsTransform(spec) => {
                ui.label("Context:");
                ui.add(NodeIdRefEditor::new(
                    &mut spec.context,
                    self.node_ctx.node_index,
                ));
                ui.end_row();

                ui.label("Inputs:");
                ui.vertical(|ui| {
                    let mut remove_indices = Vec::with_capacity(0);
                    for (index, param) in spec.input_data.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut param.name);
                            ui.add(NodeIdRefEditor::new(
                                &mut param.input,
                                self.node_ctx.node_index,
                            ));
                            if ui.button("Remove input").clicked() {
                                remove_indices.push(index);
                            }
                        });
                    }

                    remove_indices.reverse();
                    for index in remove_indices.into_iter() {
                        spec.input_data.remove(index);
                    }

                    if ui.button("New input").clicked() {
                        spec.input_data.push(guinode::JsTransformParam {
                            name: "new_input".into(),
                            input: guinode::NodeIdRef::Unresolved("".into()),
                        });
                    }
                });
                ui.end_row();

                ui.label("Code:");
                let response = ui.vertical(|ui| {
                    let func_sig = FunctionSigRenderer::render(ui, &spec.input_data);
                    ui.code(func_sig.as_ref());
                    let response = ui.code_editor(&mut spec.code);
                    ui.code("}");
                    response
                });
                if response.inner.changed() {
                    self.node_ctx.mark_node_changed();
                }
                ui.end_row();
            }
            Spec::OutputDirectory(spec) => {
                ui.label("Directory description:");
                if ui.text_edit_multiline(&mut spec.description).changed() {
                    self.node_ctx.mark_node_changed();
                }
                ui.end_row();
            }
            Spec::OutputFileCsv(spec) => {
                ui.label("Input data:");
                ui.add(NodeIdRefEditor::new(
                    &mut spec.input_data,
                    self.node_ctx.node_index,
                ));
                ui.end_row();

                ui.label("Directory:");
                ui.add(NodeIdRefEditor::new(
                    &mut spec.directory,
                    self.node_ctx.node_index,
                ));
                ui.end_row();

                ui.label("Filename:");
                todo_ui(ui, "Output file path editor.");
                ui.end_row();
            }
            Spec::OutputFileJson(spec) => {
                ui.label("Input data:");
                ui.add(NodeIdRefEditor::new(
                    &mut spec.input_data,
                    self.node_ctx.node_index,
                ));
                ui.end_row();

                ui.label("Directory:");
                ui.add(NodeIdRefEditor::new(
                    &mut spec.directory,
                    self.node_ctx.node_index,
                ));
                ui.end_row();

                ui.label("Filename:");
                todo_ui(ui, "Output file path editor.");
                ui.end_row();
            }
            Spec::PdfExtractTable(spec) => {
                ui.label("PDF:");
                ui.add(NodeIdRefEditor::new(
                    &mut spec.pdf,
                    self.node_ctx.node_index,
                ));
                ui.end_row();

                ui.label("Page:");
                todo_ui(ui, "Numerical selection component.");
                ui.end_row();

                // TODO: Other fields.
            }
        }
    }
}

fn form_grid<F: FnOnce(&mut egui::Ui) -> R, R>(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    show: F,
) -> egui::InnerResponse<R> {
    egui::Grid::new(id_salt).num_columns(2).show(ui, show)
}

#[derive(Default)]
struct FunctionSigRenderer;

impl FunctionSigRenderer {
    fn render(ui: &mut egui::Ui, inputs: &[guinode::JsTransformParam]) -> Arc<str> {
        ui.memory_mut(|mem| mem.caches.cache::<FunctionSigCache>().get(inputs).clone())
    }
}

impl egui::cache::ComputerMut<&[guinode::JsTransformParam], Arc<str>> for FunctionSigRenderer {
    fn compute(&mut self, key: &[guinode::JsTransformParam]) -> Arc<str> {
        ["function("]
            .into_iter()
            .chain(intersperse(
                key.iter().map(|param| param.name.as_str()),
                ", ",
            ))
            .chain([") {"])
            .collect::<String>()
            .into()
    }
}

type FunctionSigCache = egui::cache::FrameCache<Arc<str>, FunctionSigRenderer>;
