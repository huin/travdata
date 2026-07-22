/// Placeholder for a more complete node reference selection component.
pub fn node_ref_editor_ui(ui: &mut egui::Ui, node_id_ref: &mut pipeline::NodeId) {
    // TODO: Some form of ID selection component.
    // TODO: Button or something to bubble up viewing/editing the referenced node?
    ui.text_edit_singleline(&mut node_id_ref.0);
}
