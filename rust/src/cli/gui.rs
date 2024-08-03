use std::thread;

use anyhow::Result;

pub fn run() -> Result<()> {
    thread::scope(|s| {
        // Run the gui in a non-main thread, as the JVM will likely want to be
        // on the main thread.
        s.spawn(crate::gui::mainwin::run_gui);
    });

    Ok(())
}
