use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use anyhow::{anyhow, Context, Result};
use relm4::Worker;

use crate::{
    extraction::{bookextract, pdf::TableReader},
    gui::util::{self, SelectedFileIo},
};

/// Initialisation data for [ExtractorWorker].
pub struct Init {
    pub worker_channel: WorkChannel,
}

/// Specifies an extraction for [ExtractorWorker] to perform.
#[derive(Debug)]
pub struct Request {
    pub cfg_io: SelectedFileIo,
    pub input_pdf: PathBuf,
    pub book_id: String,
    pub out_io: SelectedFileIo,
}

/// Input messages for [ExtractorWorker].
#[derive(Debug)]
pub enum Input {
    // External:
    Start(Request),
    Cancel,
    // Internal:
    Completed,
}

/// Output messages for [ExtractorWorker].
#[derive(Debug)]
pub enum Output {
    /// Relays events from [bookextract::ExtractEvent].
    Event(bookextract::ExtractEvent),
    /// Indicates a failure to start the extraction process. This will be the only event emitted
    /// for the work.
    Failure(anyhow::Error),
}

pub struct ExtractorWorker {
    worker_channel: WorkChannel,
    work_handle: Option<WorkHandle>,
}

impl ExtractorWorker {
    pub fn is_running(&self) -> bool {
        self.work_handle.is_some()
    }
}

impl Worker for ExtractorWorker {
    type Init = Init;
    type Input = Input;
    type Output = Output;

    fn init(init: Self::Init, _sender: relm4::ComponentSender<Self>) -> Self {
        Self {
            worker_channel: init.worker_channel,
            work_handle: None,
        }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        match (message, &mut self.work_handle) {
            (Input::Start(_), Some(_)) => {
                util::send_output_or_log(
                    Output::Failure(anyhow!(
                        "Cannot start requested extraction. Work already in progress."
                    )),
                    "failure to start message",
                    sender,
                );
            }
            (Input::Start(request), work_handle_opt @ None) => {
                let work = Work::new(request, sender.clone());
                *work_handle_opt = Some(work.handle());
                if let Err(err) = self.worker_channel.sender.send(work) {
                    util::send_output_or_log(
                        Output::Failure(anyhow!(
                            "Could not request extraction - has the worker died? {:?}",
                            err
                        )),
                        "failure to start message",
                        sender,
                    );
                }
            }
            (Input::Cancel, Some(work_handle)) => {
                work_handle.cancel();
            }
            (Input::Cancel, None) => {
                util::send_output_or_log(
                    Output::Failure(anyhow!(
                        "Received extraction cancelled message, but was not running."
                    )),
                    "failure to cancel message",
                    sender,
                );
            }
            (Input::Completed, None) => {
                log::warn!("Received extraction completed message, but was not running.");
            }
            (Input::Completed, work_handle_opt) => {
                *work_handle_opt = None;
            }
        }
    }
}

pub struct MainThreadWorker<'a> {
    table_reader: &'a dyn TableReader,

    request_sender: mpsc::SyncSender<Work>,
    request_receiver: mpsc::Receiver<Work>,
}

impl<'a> MainThreadWorker<'a> {
    pub fn new(table_reader: &'a dyn TableReader) -> Self {
        let (request_sender, request_receiver) = mpsc::sync_channel(0);
        Self {
            table_reader,
            request_sender,
            request_receiver,
        }
    }

    pub fn worker_channel(&self) -> WorkChannel {
        WorkChannel {
            sender: self.request_sender.clone(),
        }
    }

    /// Should be called from the main thread once the GUI thread has been started.
    /// Blocks until shut down. Consumes `self`.
    pub fn run(self) {
        let table_reader = self.table_reader;
        drop(self.request_sender);
        let request_receiver = self.request_receiver;

        loop {
            let mut work = match request_receiver.recv() {
                Ok(work) => work,
                Err(_) => {
                    log::info!("Worker request channel closed; terminating worker loop.");
                    return;
                }
            };

            work.run(table_reader);
        }
    }
}

pub struct WorkChannel {
    sender: mpsc::SyncSender<Work>,
}

struct Work {
    request: Request,
    sender: WorkEventSender,
}

impl Work {
    fn new(request: Request, component_sender: relm4::ComponentSender<ExtractorWorker>) -> Self {
        Self {
            request,
            sender: WorkEventSender {
                component_sender,
                continue_intent: Arc::new(AtomicBool::new(true)),
            },
        }
    }

    fn handle(&self) -> WorkHandle {
        WorkHandle {
            continue_intent: self.sender.continue_intent.clone(),
        }
    }

    fn run(&mut self, table_reader: &dyn TableReader) {
        if let Err(err) = self.run_inner(table_reader) {
            self.sender.send(Output::Failure(err));
        }
    }

    fn run_inner(&mut self, table_reader: &dyn TableReader) -> Result<()> {
        let cfg_reader = self
            .request
            .cfg_io
            .new_reader()
            .with_context(|| "Opening configuration reader.")?;
        let out_writer = self
            .request
            .out_io
            .new_read_writer()
            .with_context(|| "Opening output writer.")?;
        let mut extractor = bookextract::Extractor::new(table_reader, cfg_reader, out_writer)
            .with_context(|| "Preparing extractor.")?;

        let spec = bookextract::ExtractSpec {
            book_name: &self.request.book_id,
            input_pdf: &self.request.input_pdf,
            overwrite_existing: true,
            with_tags: &[],
            without_tags: &[],
        };
        extractor.extract_book(spec, &mut self.sender);

        Ok(())
    }
}

struct WorkEventSender {
    component_sender: relm4::ComponentSender<ExtractorWorker>,
    continue_intent: Arc<AtomicBool>,
}

impl WorkEventSender {
    fn send(&self, event: Output) {
        if let Err(err) = self.component_sender.output(event) {
            log::warn!("Failed to send work event message: {:?}", err);
        }
    }
}

impl bookextract::ExtractEvents for WorkEventSender {
    fn on_event(&mut self, event: bookextract::ExtractEvent) {
        if let &bookextract::ExtractEvent::Completed = &event {
            self.component_sender.input(Input::Completed);
        }
        if let Err(err) = self.component_sender.output(Output::Event(event)) {
            log::warn!("Failed to send work event message: {:?}", err);
        }
    }

    fn do_continue(&self) -> bool {
        self.continue_intent
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

struct WorkHandle {
    continue_intent: Arc<AtomicBool>,
}

impl WorkHandle {
    fn cancel(&self) {
        self.continue_intent
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}
