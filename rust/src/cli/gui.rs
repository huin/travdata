use std::{sync::Arc, thread};

use anyhow::Result;
use clap::Args;
use relm4::RelmApp;

use crate::{distpaths, extraction::pdf::TableReaderArgs, gui::mainwin};

/// Runs a GUI to perform table extractions from PDF files.
#[derive(Args, Debug)]
pub struct Command {
    /// Options relating to configuring the table reader.
    #[command(flatten)]
    table_reader: TableReaderArgs,

    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    gtk_options: Vec<String>,
}

pub fn run(cmd: &Command, xdg_dirs: xdg::BaseDirectories) -> Result<()> {
    let table_reader = cmd.table_reader.build(&xdg_dirs)?;

    thread::scope(|s| {
        let worker = crate::gui::MainThreadWorker::new(table_reader.as_ref());

        let init = mainwin::Init {
            xdg_dirs: Arc::new(xdg_dirs),
            default_config: distpaths::config_zip(),
            worker_channel: worker.worker_channel(),
        };

        // Run the gui in a non-main thread, as the JVM will likely want to be
        // on the main thread.
        s.spawn(move || {
            let program_invocation = std::env::args().next().unwrap();
            let mut gtk_args = vec![program_invocation];
            gtk_args.extend(cmd.gtk_options.clone());

            let app = RelmApp::new("travdata.gui").with_args(gtk_args);
            app.run::<mainwin::MainWindow>(init);
        });

        worker.run();
    });

    if let Err(err) = table_reader.close() {
        log::warn!("Failed to shut down table reader: {err}");
    }

    Ok(())
}
