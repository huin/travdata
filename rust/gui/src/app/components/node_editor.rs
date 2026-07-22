use pipeline::specs::Spec;

use crate::app::components::{node_ref_editor_ui, todo_ui};

// TODO: Consider if we need component state, passing in NodeRef so that a change to which node is
// being edited can be detected.

pub fn node_editor_ui(ui: &mut egui::Ui, node: &mut pipeline::Node) {
    form_grid(ui, "node_editor_ui", |ui| {
        node_meta_editor(ui, node);

        node_spec_editor(ui, node);
    });
}

fn node_meta_editor(ui: &mut egui::Ui, node: &mut pipeline::Node) {
    ui.label("ID:");
    ui.text_edit_singleline(&mut node.meta.id.0);
    ui.end_row();
}

fn node_spec_editor(ui: &mut egui::Ui, node: &mut pipeline::Node) {
    match &mut node.spec {
        Spec::InputPdfFile(spec) => {
            ui.label("PDF description:");
            ui.text_edit_multiline(&mut spec.description);
            ui.end_row();
        }
        Spec::JsContext(_spec) => {
            // No fields yet.
            ui.label("No settings for JsContext yet.");
            ui.end_row();
        }
        Spec::JsTransform(spec) => {
            ui.label("Context:");
            node_ref_editor_ui(ui, &mut spec.context);
            ui.end_row();

            // TODO: Input data.

            ui.label("Code:");
            ui.text_edit_multiline(&mut spec.code);
            ui.end_row();
        }
        Spec::OutputDirectory(spec) => {
            ui.label("Directory description:");
            ui.text_edit_multiline(&mut spec.description);
            ui.end_row();
        }
        Spec::OutputFileCsv(spec) => {
            ui.label("Input data:");
            node_ref_editor_ui(ui, &mut spec.input_data);
            ui.end_row();

            ui.label("Directory:");
            node_ref_editor_ui(ui, &mut spec.directory);
            ui.end_row();

            ui.label("Filename:");
            todo_ui(ui, "Output file path editor.");
            ui.end_row();
        }
        Spec::OutputFileJson(spec) => {
            ui.label("Input data:");
            node_ref_editor_ui(ui, &mut spec.input_data);
            ui.end_row();

            ui.label("Directory:");
            node_ref_editor_ui(ui, &mut spec.directory);
            ui.end_row();

            ui.label("Filename:");
            todo_ui(ui, "Output file path editor.");
            ui.end_row();
        }
        Spec::PdfExtractTable(spec) => {
            ui.label("PDF:");
            node_ref_editor_ui(ui, &mut spec.pdf);
            ui.end_row();

            ui.label("Page:");
            todo_ui(ui, "Numerical selection component.");
            ui.end_row();

            // TODO: Other fields.
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
