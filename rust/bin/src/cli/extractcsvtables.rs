use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use anyhow::{Context, Result};
use clap::Args;
use simple_bar::ProgressBar;

use crate::{
    extraction::{
        bookextract::{self, ExtractEvents, ExtractSpec, Extractor},
        pdf::TableReaderArgs,
    },
    filesio,
    template::loadarg,
};

/// Extracts data tables from the Mongoose Traveller 2022 core rules PDF as CSV
/// files.
#[derive(Args, Debug)]
pub struct Command {
    /// Path to input PDF.
    input_pdf: PathBuf,

    /// Path to the directory or ZIP file to output the CSV files into.
    ///
    /// Whether this is a directory or ZIP file is controlled by --output-type.
    output: PathBuf,

    /// Options relating to the extraction template.
    #[command(flatten)]
    template: loadarg::TemplateArgs,

    /// Controls how data is written to the output.
    ///
    /// By default, it guesses, based on any existing file or directory at the
    /// path or the path suffix ending in ".zip".
    #[arg(long)]
    output_type: Option<crate::filesio::IoType>,

    /// Extract CSV tables that already exist in the output. This is useful when
    /// testing larger scale changes to the configuration or code.
    #[arg(long)]
    overwrite_existing: bool,

    /// Only extract tables that have any of these tags. --without-tag takes
    /// precedence over this.
    #[arg(long, value_delimiter(','))]
    with_tags: Vec<String>,

    /// Only extract tables that do not have any of these tags. This takes
    /// precedence over --with-tag.
    #[arg(long, value_delimiter(','))]
    without_tags: Vec<String>,

    /// Show a progress bar reflecting overall extraction progress.
    #[arg(long, default_value = "false")]
    no_progress: bool,

    /// Options relating to configuring the table reader.
    #[command(flatten)]
    table_reader: TableReaderArgs,
}

/// Runs the subcommand.
pub fn run(cmd: &Command, xdg_dirs: xdg::BaseDirectories) -> Result<()> {
    v8wrapper::init_v8();
    let tls_isolate = v8wrapper::TlsIsolate::for_current_thread();

    let tmpl = cmd.template.load_template()?;

    let table_reader = cmd.table_reader.build(&xdg_dirs)?;
    let extractor = Extractor::new(&tmpl, table_reader.as_ref())?;

    let output_type = filesio::IoType::resolve_auto(cmd.output_type, &cmd.output);
    let out_writer = output_type
        .new_read_writer(&cmd.output)
        .with_context(|| format!("opening output path {:?} as {:?}", cmd.output, output_type))?;

    let spec = ExtractSpec {
        input_pdf: &cmd.input_pdf,
        overwrite_existing: cmd.overwrite_existing,
        with_tags: &cmd.with_tags,
        without_tags: &cmd.without_tags,
    };

    let continue_intent = Arc::new(AtomicBool::new(true));
    let mut events = EventDisplayer::new(!cmd.no_progress, continue_intent.clone())?;
    ctrlc::set_handler(move || continue_intent.store(false, std::sync::atomic::Ordering::SeqCst))?;

    extractor.extract_book(spec, &mut events, out_writer.as_ref());

    out_writer.close()?;

    if let Err(err) = table_reader.close() {
        log::warn!("Failed to shut down table reader: {err}");
    }

    drop(tls_isolate);

    Ok(())
}

struct EventDisplayer {
    show_progress: bool,
    progress_bar: Option<ProgressBar>,
    continue_intent: Arc<AtomicBool>,
}

impl EventDisplayer {
    fn new(show_progress: bool, continue_intent: Arc<AtomicBool>) -> Result<Self> {
        Ok(EventDisplayer {
            show_progress,
            progress_bar: None,
            continue_intent: continue_intent.clone(),
        })
    }
}

impl ExtractEvents for EventDisplayer {
    fn on_event(&mut self, event: bookextract::ExtractEvent) {
        match event {
            bookextract::ExtractEvent::Progress {
                path: _,
                completed: _,
                total,
            } => {
                if !self.show_progress {
                    return;
                }

                let progress_bar: &mut ProgressBar = match self.progress_bar.as_mut() {
                    Some(progress_bar) => progress_bar,
                    None => {
                        let progress_bar = ProgressBar::cargo_style(total as u32, 80, true);
                        self.progress_bar = Some(progress_bar);
                        self.progress_bar.as_mut().unwrap()
                    }
                };
                progress_bar.update();
            }
            bookextract::ExtractEvent::Error {
                err,
                terminal: false,
            } => {
                eprintln!("Error (continuing): {:?}.", err);
            }
            bookextract::ExtractEvent::Error {
                err,
                terminal: true,
            } => {
                eprintln!("Extraction failed: {:?}.", err);
            }
            bookextract::ExtractEvent::Completed => {
                eprintln!("Extraction complete.");
            }
            bookextract::ExtractEvent::Cancelled => {
                eprintln!("Extraction cancelled.");
            }
        }
    }

    fn do_continue(&self) -> bool {
        self.continue_intent
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
