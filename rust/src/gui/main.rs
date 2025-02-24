use std::{
    sync::{mpsc, Arc},
    thread,
};

use anyhow::{Context, Result};
use gtk::prelude::*;
use gtk::Application;
use relm4::RelmApp;

use crate::{
    distpaths,
    extraction::pdf::pdfiumthread::PdfiumServer,
    gui::{self, mainmenu, workers},
    mpscutil,
};

/// Runs the GUI. Must be called from the main thread.
pub fn run(
    table_reader: Box<dyn crate::extraction::pdf::TableReader>,
    gtk_options: &[String],
    xdg_dirs: xdg::BaseDirectories,
) -> Result<()> {
    let result: Result<()> = thread::scope(|s| {
        let worker = workers::extractor::MainThreadWorker::new(table_reader.as_ref());

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
            gtk_args.extend(gtk_options.iter().cloned());

            let gtk_app = Application::builder()
                .application_id("github.com/huin/travdata")
                .register_session(true)
                .build();
            mainmenu::install_on_startup(&gtk_app);
            install_css_on_startup(&gtk_app);
            let app = RelmApp::from_app(gtk_app.clone()).with_args(gtk_args);
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

/// Installs the GUI's CSS stylesheet on [Application] startup.
fn install_css_on_startup(app: &Application) {
    app.connect_startup(|_| {
        relm4::set_global_css(include_str!("styles.css"));
    });
}
