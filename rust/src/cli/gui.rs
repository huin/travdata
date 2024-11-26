use std::{sync::Arc, thread};

use anyhow::Result;
use clap::Args;

use crate::{distpaths, extraction::pdf::TableReaderArgs};

/// Runs a GUI to perform table extractions from PDF files.
#[derive(Args, Debug)]
pub struct Command {
    /// Options relating to configuring the table reader.
    #[command(flatten)]
    table_reader: TableReaderArgs,
}

pub fn run(cmd: &Command, xdg_dirs: xdg::BaseDirectories) -> Result<()> {
    let table_reader = cmd.table_reader.build(&xdg_dirs)?;

    thread::scope(|s| {
        let worker = crate::gui::MainThreadWorker::new(table_reader.as_ref());

        let init = crate::gui::mainwin::Init {
            xdg_dirs: Arc::new(xdg_dirs),
            default_config: distpaths::config_zip(),
            worker_channel: worker.worker_channel(),
        };
        // Run the gui in a non-main thread, as the JVM will likely want to be
        // on the main thread.
        s.spawn(move || crate::gui::mainwin::run_gui(init));

        worker.run();
    });

    if let Err(err) = table_reader.close() {
        log::warn!("Failed to shut down table reader: {err}");
    }

    Ok(())
}
