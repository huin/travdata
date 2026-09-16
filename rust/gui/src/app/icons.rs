use crate::app::colours;

/// Renders an icon representing a reference to a node by its ID. `resolved` should be `true` if the
/// node ID is resolved to a node.
pub fn node_ref_dest(ui: &mut egui::Ui, resolved: bool) {
    let (painter, c, r) = node_ref_init(ui);
    painter.circle_stroke(c, r, egui::Stroke::new(1.5, colours::DRAGGABLE_NODE_ID));
    if resolved {
        node_ref_fill(&painter, c, r);
    }
}

/// Renders an icon representing a node ID.
pub fn node_ref_src(ui: &mut egui::Ui) {
    let (painter, c, r) = node_ref_init(ui);
    node_ref_fill(&painter, c, r);
}

fn node_ref_fill(painter: &egui::Painter, c: egui::Pos2, r: f32) {
    painter.circle(
        c,
        r - 2.5,
        colours::DRAGGABLE_NODE_ID,
        egui::Stroke::new(0.5, colours::DRAGGABLE_NODE_ID),
    );
}

fn node_ref_init(ui: &mut egui::Ui) -> (egui::Painter, egui::Pos2, f32) {
    let (response, painter) = ui.allocate_painter(egui::Vec2::splat(12.0), egui::Sense::drag());
    let rect = response.rect;
    let c = rect.center();
    let r = rect.width() / 2.0 - 1.5;

    (painter, c, r)
}
