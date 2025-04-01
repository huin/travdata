mod components;
mod extract;
mod extractionlist;
// TODO: Enable when need editor.
#[allow(dead_code)]
mod gobjects;
mod inputpdf;
mod main;
mod mainmenu;
mod mainwin;
mod outputselect;
mod pageview;
mod tmplmodel;
mod treelist;
mod util;
mod workers;

pub use main::run;
