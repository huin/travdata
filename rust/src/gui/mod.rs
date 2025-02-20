mod cfgselect;
mod components;
mod extract;
mod extractionlist;
mod gobjects;
mod inputpdf;
pub mod mainmenu;
pub mod mainwin;
mod outputselect;
mod pageview;
mod treelist;
mod util;
mod workers;
pub use workers::extractor::MainThreadWorker;

use gtk::prelude::*;
use gtk::Application;

/// Installs the GUI's CSS stylesheet on [Application] startup.
pub fn install_css_on_startup(app: &Application) {
    app.connect_startup(|_| {
        relm4::set_global_css(include_str!("styles.css"));
    });
}
