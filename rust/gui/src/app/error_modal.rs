use egui::Id;

pub fn display_error(ui: &egui::Ui, displayed_error: &str) -> bool {
    egui::Modal::new(Id::new("displayed_error"))
        .show(ui.ctx(), |ui| {
            ui.heading("Error");

            ui.label(displayed_error);
            if ui.button("Close").clicked() {
                ui.close();
            }
        })
        .should_close()
}
