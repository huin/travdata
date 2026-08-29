mod node_editor;
mod node_ref_editor;
mod pipeline_editor;

pub use pipeline_editor::PipelineEditor;

/// Placeholder UI element.
pub fn todo_ui(ui: &mut egui::Ui, desc: &str) {
    // TODO: Replace all uses of this function.
    ui.group(|ui| {
        ui.label("TODO");
        ui.label(desc);
    });
}

trait WidgetState {
    fn get_or_default(ui: &mut egui::Ui) -> Self;

    fn store(self, ui: &mut egui::Ui);
}

trait WidgetStateTemporary: WidgetState + Clone + Default + Send + Sync + 'static {}

impl<T> WidgetState for T
where
    T: WidgetStateTemporary,
{
    fn get_or_default(ui: &mut egui::Ui) -> Self {
        ui.data_mut(|data| data.get_temp(ui.id()).unwrap_or_default())
    }

    fn store(self, ui: &mut egui::Ui) {
        ui.data_mut(|data| data.insert_temp(ui.id(), self));
    }
}
