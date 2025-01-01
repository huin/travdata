use std::{collections::HashMap, path::PathBuf, sync::mpsc};

use anyhow::{anyhow, Result};
use image::ImageBuffer;
use pdfium_render::prelude::{PdfDocument, PdfRenderConfig, Pdfium};

use crate::mpscutil;

pub type PageImage = ImageBuffer<image::Rgb<u8>, Vec<u8>>;

/// Identity of a loaded PDF document in the [PdfiumServer].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DocumentId(usize);

/// Requests coarsely serialised PDF operations against the [Pdfium] API.
pub struct PdfiumClient {
    request_sender: mpsc::SyncSender<Request>,
}

/// Information about a loaded PDF.
#[derive(Debug)]
pub struct PdfMetadata {
    pub id: DocumentId,
    pub num_pages: u16,
}

impl PdfiumClient {
    /// Unloads a previously loaded PDF.
    pub fn unload_pdf(&self, id: DocumentId) -> Result<()> {
        let (response_sender, response_receiver) = mpsc::sync_channel(0);
        self.request_sender.send(Request::UnloadPdf {
            id,
            response_sender,
        })?;
        response_receiver.recv()?
    }

    /// Loads a PDF from the given file path.
    pub fn load_pdf(&self, path: PathBuf) -> Result<PdfMetadata> {
        let (response_sender, response_receiver) = mpsc::sync_channel(0);
        self.request_sender.send(Request::LoadPdf {
            path,
            response_sender,
        })?;
        response_receiver.recv()?
    }

    /// Renders a page from a loaded PDF.
    pub fn render_page(&self, id: DocumentId, page_index: u16) -> Result<PageImage> {
        let (response_sender, response_receiver) = mpsc::sync_channel(0);
        self.request_sender.send(Request::RenderPage {
            id,
            page_index,
            response_sender,
        })?;
        response_receiver.recv()?
    }
}

/// Serialises coarse-grained operations with the single-threaded Pdfium library.
pub struct PdfiumServer {
    pdfium: Pdfium,

    request_sender: mpsc::SyncSender<Request>,
    request_receiver: mpsc::Receiver<Request>,
}

impl PdfiumServer {
    pub fn new() -> Result<Self> {
        let pdfium = Pdfium::new(Pdfium::bind_to_statically_linked_library()?);
        let (request_sender, request_receiver) = mpsc::sync_channel(0);
        Ok(Self {
            pdfium,
            request_sender,
            request_receiver,
        })
    }

    pub fn client(&self) -> PdfiumClient {
        PdfiumClient {
            request_sender: self.request_sender.clone(),
        }
    }

    pub fn run(self) {
        // Ensure that that we terminate the loop below when the client is dropped externally.
        drop(self.request_sender);

        let request_receiver = self.request_receiver;
        let mut state = ServerState::new(&self.pdfium);

        loop {
            let request = match request_receiver.recv() {
                Ok(work) => work,
                Err(_) => {
                    log::info!("Request channel closed; terminating PdfiumServer worker loop.");
                    return;
                }
            };

            state.handle_request(request);
        }
    }
}

struct ServerState<'lib> {
    pdfium: &'lib Pdfium,
    loaded_documents: HashMap<DocumentId, LoadedDocument<'lib>>,
    next_id: DocumentId,
}

impl<'lib> ServerState<'lib> {
    fn new(pdfium: &'lib Pdfium) -> Self {
        Self {
            pdfium,
            loaded_documents: HashMap::new(),
            next_id: DocumentId(0),
        }
    }

    fn handle_request(&mut self, request: Request) {
        match request {
            Request::UnloadPdf {
                id,
                response_sender,
            } => {
                mpscutil::send_or_log_warning(
                    &response_sender,
                    "UnloadPdf response",
                    self.unload_pdf(id),
                );
            }
            Request::LoadPdf {
                path,
                response_sender,
            } => {
                mpscutil::send_or_log_warning(
                    &response_sender,
                    "LoadPdf response",
                    self.load_pdf(path),
                );
            }
            Request::RenderPage {
                id,
                page_index,
                response_sender,
            } => {
                mpscutil::send_or_log_warning(
                    &response_sender,
                    "RenderPage response",
                    self.render_page(id, page_index),
                );
            }
        }
    }

    fn unload_pdf(&mut self, id: DocumentId) -> Result<()> {
        self.loaded_documents
            .remove(&id)
            .ok_or_else(|| anyhow!("document with ID {:?} not loaded", id))
            .map(|_| ())
    }

    fn load_pdf(&mut self, path: PathBuf) -> Result<PdfMetadata> {
        let id = self.next_id;
        self.next_id.0 = self
            .next_id
            .0
            .checked_add(1)
            .ok_or_else(|| anyhow!("overflowed assigning DocumentIds"))?;
        let document = self.pdfium.load_pdf_from_file(&path, None)?;
        let metadata = PdfMetadata {
            id,
            num_pages: document.pages().len(),
        };
        let loaded_document = LoadedDocument { path, document };
        self.loaded_documents
            .entry(id)
            .insert_entry(loaded_document);
        Ok(metadata)
    }

    fn render_page(&self, id: DocumentId, page_index: u16) -> Result<PageImage> {
        let loaded_document = self
            .loaded_documents
            .get(&id)
            .ok_or_else(|| anyhow!("document with ID {:?} not loaded", id))?;
        let page = loaded_document.document.pages().get(page_index)?;

        // Render page with selected rectangle drawn.
        let config = PdfRenderConfig::new();
        let pdf_image = page.render_with_config(&config)?;
        Ok(pdf_image.as_image().into_rgb8())
    }
}

enum Request {
    UnloadPdf {
        id: DocumentId,
        response_sender: mpsc::SyncSender<Result<()>>,
    },
    LoadPdf {
        path: PathBuf,
        response_sender: mpsc::SyncSender<Result<PdfMetadata>>,
    },
    RenderPage {
        id: DocumentId,
        page_index: u16,
        response_sender: mpsc::SyncSender<Result<PageImage>>,
    },
}

struct LoadedDocument<'a> {
    // TODO: Use field, or remove.
    #[allow(dead_code)]
    path: PathBuf,
    document: PdfDocument<'a>,
}
