mod cfgselect;
mod components;
mod extract;
mod extractionlist;
mod gobjects;
mod inputpdf;
pub mod mainwin;
mod outputselect;
mod pageview;
mod treelist;
mod util;
mod workers;
pub use workers::extractor::MainThreadWorker;

/// Installs the GUI's CSS stylesheet.
///
/// Note: Must be called _after_ [relm4::RelmApp::new].
pub fn install_stylesheet() {
    relm4::set_global_css(include_str!("styles.css"));
}
