use crate::app::data::GuiNodeId;

/// Placeholder for a more complete node reference selection component.
pub fn node_ref_editor_ui(ui: &mut egui::Ui, node_id_ref: &mut GuiNodeId) {
    // TODO: Some form of ID selection component.
    // TODO: Button or something to bubble up viewing/editing the referenced node?
    match node_id_ref {
        GuiNodeId::Unresolved(node_id) => {
            ui.label(&*node_id);
        }
        GuiNodeId::Resolved(_node_ref) => {
            ui.label("<node ref>");
        }
    }
}
