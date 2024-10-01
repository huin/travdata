use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};

use anyhow::{Context, Result};
use clap::Args;
use simple_bar::ProgressBar;

use crate::{
    extraction::{
        bookextract::{ExtractEvents, ExtractSpec, Extractor},
        pdf::{cachingreader::CachingTableReader, tabulareader::TabulaClient, TableReader},
    },
    filesio,
};

/// Extracts data tables from the Mongoose Traveller 2022 core rules PDF as CSV
/// files.
#[derive(Args, Debug)]
pub struct Command {
    /// Path to the configuration.
    book_name: String,

    /// Path to input PDF.
    input_pdf: PathBuf,

    /// Path to the directory or ZIP file to output the CSV files into.
    ///
    /// Whether this is a directory or ZIP file is controlled by --output-type.
    output: PathBuf,

    /// Path to the configuration. This must be either a directory or ZIP file,
    /// directly containing a config.yaml file, book.yaml files in directories,
    /// and its required Tabula templates. Some configurations for this should
    /// be included with this program's distribution.
    #[arg(long)]
    config: PathBuf,

    /// Path to Tabula JAR file.
    #[arg(long)]
    tabula_libpath: String,

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
    #[arg(long, default_value = "true")]
    show_progress: bool,

    /// Use the table cache.
    #[arg(long, default_value = "true")]
    table_cache: bool,
}

/// Runs the subcommand.
pub fn run(cmd: &Command) -> Result<()> {
    let tabula_reader =
        Box::new(TabulaClient::new(&cmd.tabula_libpath).with_context(|| "initialising Tabula")?);

    let mut opt_table_cache: Option<CachingTableReader<&TabulaClient>> = None;
    let table_reader: &dyn TableReader = if !cmd.table_cache {
        tabula_reader.as_ref()
    } else {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("travdata")?;
        let tables_cache_path = xdg_dirs.place_cache_file(Path::new("table-cache.json"))?;
        opt_table_cache = Some(CachingTableReader::load(
            tabula_reader.as_ref(),
            tables_cache_path,
        )?);
        opt_table_cache.as_ref().unwrap()
    };

    let cfg_type = filesio::IoType::resolve_auto(None, &cmd.config);
    let cfg_reader = cfg_type
        .new_reader(&cmd.config)
        .with_context(|| format!("opening config path {:?} as {:?}", cmd.config, cfg_type))?;

    let output_type = filesio::IoType::resolve_auto(cmd.output_type, &cmd.output);
    let out_writer = output_type
        .new_read_writer(&cmd.output)
        .with_context(|| format!("opening output path {:?} as {:?}", cmd.output, output_type))?;

    let mut extractor = Extractor::new(table_reader, cfg_reader, out_writer)?;

    let spec = ExtractSpec {
        book_name: &cmd.book_name,
        input_pdf: &cmd.input_pdf,
        overwrite_existing: cmd.overwrite_existing,
        with_tags: &cmd.with_tags,
        without_tags: &cmd.without_tags,
    };

    let continue_intent = Arc::new(AtomicBool::new(true));
    let mut events = EventDisplayer::new(cmd.show_progress, continue_intent.clone())?;
    ctrlc::set_handler(move || continue_intent.store(false, std::sync::atomic::Ordering::SeqCst))?;

    extractor.extract_book(spec, &mut events);

    extractor.close()?;

    if let Some(table_cache) = opt_table_cache {
        if let Err(err) = table_cache.store() {
            log::warn!("Failed to store table cache: {err}");
        }
    }

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
    fn on_progress(&mut self, _path: &Path, _completed: usize, total: usize) {
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

    fn on_error(&mut self, err: anyhow::Error) {
        eprintln!("Error during extraction: {:?}.", err);
    }

    fn on_end(&mut self) {}

    fn do_continue(&self) -> bool {
        self.continue_intent
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}
