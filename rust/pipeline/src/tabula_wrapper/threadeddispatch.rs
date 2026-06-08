use std::sync::mpsc;

use crate::tabula_wrapper::{self, TabulaExtractionRequest, TabulaExtractor, singlethreaded};

/// ServingExtractor should be created and run on the main thread.
pub struct ServingExtractor<'env> {
    // XXX think about how to shut this down.
    extractor: singlethreaded::SingleThreadedTabulaExtractor<'env>,
    request_sender: mpsc::Sender<ServeRequest>,
    request_receiver: mpsc::Receiver<ServeRequest>,
}

impl<'env> ServingExtractor<'env> {
    pub fn new(extractor: singlethreaded::SingleThreadedTabulaExtractor<'env>) -> Self {
        let (request_sender, request_receiver) = mpsc::channel();
        Self {
            extractor,
            request_sender,
            request_receiver,
        }
    }

    pub fn client(&self) -> ExtractorClient {
        ExtractorClient {
            request_sender: self.request_sender.clone(),
        }
    }

    pub fn run(self) {
        log::info!("Starting up ServingExtractor.");
        loop {
            match self.request_receiver.recv() {
                Ok(ServeRequest::ExtractTables(Request {
                    request,
                    response_sender,
                })) => {
                    let result = self.serve_extract_tables(request);
                    if let Err(err) = response_sender.send(result) {
                        log::warn!("Could not send extracted tables response: {err:?}.");
                    }
                }
                Ok(ServeRequest::Stop) | Err(_) => {
                    log::info!("Shutting down ServingExtractor.");
                    break;
                }
            }
        }
    }

    fn serve_extract_tables(
        &self,
        request: TabulaExtractionRequest,
    ) -> anyhow::Result<tabula_wrapper::JsonTableSet> {
        self.extractor.extract_tables(request)
    }
}

enum ServeRequest {
    ExtractTables(Request),
    Stop,
}

struct Request {
    request: tabula_wrapper::TabulaExtractionRequest,
    response_sender: mpsc::Sender<anyhow::Result<tabula_wrapper::JsonTableSet>>,
}

pub struct ExtractorClient {
    request_sender: mpsc::Sender<ServeRequest>,
}
