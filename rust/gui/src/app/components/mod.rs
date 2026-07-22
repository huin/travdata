mod node_editor;
mod node_ref_editor;
mod pipeline_editor;

pub use node_ref_editor::node_ref_editor_ui;
pub use pipeline_editor::PipelineEditor;

/// Placeholder UI element.
pub fn todo_ui(ui: &mut egui::Ui, desc: &str) {
    // TODO: Replace all uses of this function.
    ui.group(|ui| {
        ui.label("TODO");
        ui.label(desc);
    });
}
