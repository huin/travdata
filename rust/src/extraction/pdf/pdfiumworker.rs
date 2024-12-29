use std::{path::PathBuf, sync::mpsc};

use anyhow::{anyhow, Result};
use image::ImageBuffer;
use pdfium_render::prelude::{PdfDocument, PdfRenderConfig, Pdfium};

use crate::mpscutil;

pub type PageImage = ImageBuffer<image::Rgb<u8>, Vec<u8>>;

pub struct PdfiumClient {
    request_sender: mpsc::SyncSender<Request>,
}

impl PdfiumClient {
    pub fn unload_pdf(&self) -> Result<()> {
        let (response_sender, response_receiver) = mpsc::sync_channel(0);
        self.request_sender
            .send(Request::UnloadPdf(response_sender))?;
        Ok(response_receiver.recv()?)
    }

    pub fn load_pdf(&self, path: PathBuf) -> Result<PdfMetadata> {
        let (response_sender, response_receiver) = mpsc::sync_channel(0);
        self.request_sender
            .send(Request::LoadPdf(path, response_sender))?;
        response_receiver.recv()?
    }

    pub fn render_page(&self, page_index: u16) -> Result<PageImage> {
        let (response_sender, response_receiver) = mpsc::sync_channel(0);
        self.request_sender
            .send(Request::RenderPage(page_index, response_sender))?;
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
        let mut state = ServerState {
            pdfium: &self.pdfium,
            loaded_document: None,
        };

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
    loaded_document: Option<LoadedDocument<'lib>>,
}

impl ServerState<'_> {
    fn handle_request(&mut self, request: Request) {
        match request {
            Request::UnloadPdf(resp_send) => {
                self.loaded_document = None;
                mpscutil::send_or_log_warning(&resp_send, "UnloadPdf response", ());
            }
            Request::LoadPdf(path, resp_send) => {
                mpscutil::send_or_log_warning(&resp_send, "LoadPdf response", self.load_pdf(path));
            }
            Request::RenderPage(page_number, resp_send) => {
                mpscutil::send_or_log_warning(
                    &resp_send,
                    "RenderPage response",
                    self.render_page(page_number),
                );
            }
        }
    }

    fn load_pdf(&mut self, path: PathBuf) -> Result<PdfMetadata> {
        let document = self.pdfium.load_pdf_from_file(&path, None)?;
        let metadata = PdfMetadata {
            path: path.clone(),
            num_pages: document.pages().len(),
        };
        self.loaded_document = Some(LoadedDocument { path, document });
        Ok(metadata)
    }

    fn render_page(&self, page_index: u16) -> Result<PageImage> {
        let loaded_document = self
            .loaded_document
            .as_ref()
            .ok_or_else(|| anyhow!("no PDF loaded to render"))?;
        let page = loaded_document.document.pages().get(page_index)?;

        // Render page with selected rectangle drawn.
        let config = PdfRenderConfig::new();
        let pdf_image = page.render_with_config(&config)?;
        Ok(pdf_image.as_image().into_rgb8())
    }
}

#[derive(Debug)]
pub struct PdfMetadata {
    // TODO: Use field, or remove.
    #[allow(dead_code)]
    pub path: PathBuf,
    // TODO: Use field, or remove.
    #[allow(dead_code)]
    pub num_pages: u16,
}

enum Request {
    UnloadPdf(mpsc::SyncSender<()>),
    LoadPdf(PathBuf, mpsc::SyncSender<Result<PdfMetadata>>),
    RenderPage(u16, mpsc::SyncSender<Result<PageImage>>),
}

struct LoadedDocument<'a> {
    // TODO: Use field, or remove.
    #[allow(dead_code)]
    path: PathBuf,
    document: PdfDocument<'a>,
}
