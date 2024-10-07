use std::thread;

use anyhow::Result;

pub fn run(xdg_dirs: xdg::BaseDirectories) -> Result<()> {
    thread::scope(|s| {
        // Run the gui in a non-main thread, as the JVM will likely want to be
        // on the main thread.
        s.spawn(move || crate::gui::mainwin::run_gui(crate::gui::mainwin::Init { xdg_dirs }));
    });

    Ok(())
}
