mod app;
mod error;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    simplelog::SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default())
        .map_err(|err| eframe::Error::AppCreation(Box::new(err)))?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
