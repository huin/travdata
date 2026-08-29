use crate::app::{
    components::{node_ref_editor::NodeIdRefEditor, todo_ui},
    data,
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
        ui.push_id(("NodeEditor", self.node_ctx.node_ref), |ui| {
            form_grid(ui, "node_editor_ui", |ui| {
                self.node_meta_editor(ui);
                self.node_spec_editor(ui);
            });
        });
    }

    fn node_meta_editor(&mut self, ui: &mut egui::Ui) {
        ui.label("ID:");
        if ui
            .text_edit_singleline(&mut self.node_ctx.node.node_id)
            .changed()
        {
            self.node_ctx.mark_node_changed();
        }
        ui.end_row();
    }

    fn node_spec_editor(&mut self, ui: &mut egui::Ui) {
        use data::GuiSpec;

        match &mut self.node_ctx.node.node.spec {
            GuiSpec::InputPdfFile(spec) => {
                ui.label("PDF description:");
                if ui.text_edit_multiline(&mut spec.description).changed() {
                    self.node_ctx.mark_node_changed();
                }
                ui.end_row();
            }
            GuiSpec::JsContext(_spec) => {
                // No fields yet.
                ui.label("No settings for JsContext yet.");
                ui.end_row();
            }
            GuiSpec::JsTransform(spec) => {
                ui.label("Context:");
                ui.add(NodeIdRefEditor::new(
                    &mut spec.context,
                    self.node_ctx.node_index,
                ));
                ui.end_row();

                for (var_name, node_ref) in spec.input_data.iter_mut() {
                    ui.label(var_name);
                    ui.add(NodeIdRefEditor::new(node_ref, self.node_ctx.node_index));
                    ui.end_row();
                }

                ui.label("Code:");
                if ui.text_edit_multiline(&mut spec.code).changed() {
                    self.node_ctx.mark_node_changed();
                }
                ui.end_row();
            }
            GuiSpec::OutputDirectory(spec) => {
                ui.label("Directory description:");
                if ui.text_edit_multiline(&mut spec.description).changed() {
                    self.node_ctx.mark_node_changed();
                }
                ui.end_row();
            }
            GuiSpec::OutputFileCsv(spec) => {
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
            GuiSpec::OutputFileJson(spec) => {
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
            GuiSpec::PdfExtractTable(spec) => {
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
