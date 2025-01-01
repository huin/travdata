use std::{
    sync::{mpsc, Arc},
    thread,
};

use anyhow::{Context, Result};
use clap::Args;
use relm4::RelmApp;

use crate::{
    distpaths,
    extraction::pdf::{pdfiumthread::PdfiumServer, TableReaderArgs},
    gui, mpscutil,
};

/// Runs a GUI to perform table extractions from PDF files.
#[derive(Args, Debug, Default)]
pub struct Command {
    /// Options relating to configuring the table reader.
    #[command(flatten)]
    table_reader: TableReaderArgs,

    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    gtk_options: Vec<String>,
}

pub fn run(cmd: &Command, xdg_dirs: xdg::BaseDirectories) -> Result<()> {
    let table_reader = cmd.table_reader.build(&xdg_dirs)?;

    let result: Result<()> = thread::scope(|s| {
        let worker = crate::gui::MainThreadWorker::new(table_reader.as_ref());

        // Run the PdfiumServer in its own dedicated thread, to serialise access to the
        // pdfium_render library.
        let (pdfium_client_sender, pdfium_client_receiver) = mpsc::sync_channel(0);
        s.spawn(move || {
            let pdfium_server = match PdfiumServer::new() {
                Ok(pdfium_server) => pdfium_server,
                Err(err) => {
                    mpscutil::send_or_log_warning(
                        &pdfium_client_sender,
                        "PdfiumServer error",
                        Err(err),
                    );
                    return;
                }
            };
            mpscutil::send_or_log_warning(
                &pdfium_client_sender,
                "PdfiumClient",
                Ok(pdfium_server.client()),
            );
            pdfium_server.run();
        });

        let pdfium_client = (pdfium_client_receiver
            .recv()
            .with_context(|| "receiving PdfiumClient or error")?)?;

        let init = gui::mainwin::Init {
            xdg_dirs: Arc::new(xdg_dirs),
            default_config: distpaths::config_zip(),
            pdfium_client,
            worker_channel: worker.worker_channel(),
        };

        // Run the gui in a non-main thread, as the JVM will likely want to be
        // on the main thread.
        s.spawn(move || {
            let program_invocation = std::env::args().next().unwrap();
            let mut gtk_args = vec![program_invocation];
            gtk_args.extend(cmd.gtk_options.clone());

            let app = RelmApp::new("travdata.gui").with_args(gtk_args);
            gui::install_stylesheet();
            app.run::<gui::mainwin::MainWindow>(init);
        });

        // Run the extraction worker (and the JVM that it uses) in the main thread.
        worker.run();

        Ok(())
    });

    if let Err(err) = table_reader.close() {
        log::warn!("Failed to shut down table reader: {err}");
    }

    result
}
