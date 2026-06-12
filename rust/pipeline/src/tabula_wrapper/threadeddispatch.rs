use std::{marker::PhantomData, rc::Rc, sync::mpsc};

use crate::tabula_wrapper::{self, TabulaExtractor, singlethreaded};

/// ServingExtractor should be created and run on the main thread.
pub struct ExtractorServer<'env> {
    forbid_send: PhantomData<Rc<()>>,
    extractor: singlethreaded::SingleThreadedTabulaExtractor<'env>,
    request_sender: mpsc::SyncSender<ServeRequest>,
    request_receiver: mpsc::Receiver<ServeRequest>,
}

impl<'env> ExtractorServer<'env> {
    pub fn new(extractor: singlethreaded::SingleThreadedTabulaExtractor<'env>) -> Self {
        let (request_sender, request_receiver) = mpsc::sync_channel(0);
        Self {
            forbid_send: PhantomData,
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
        log::info!("Starting up ExtractorServer.");

        // Ensure that that we terminate the loop below when the client is dropped externally.
        drop(self.request_sender);
        let request_receiver = self.request_receiver;
        let extractor = self.extractor;

        loop {
            match request_receiver.recv() {
                Ok(ServeRequest::ExtractTables(Request {
                    request,
                    response_sender,
                })) => {
                    let result = extractor.extract_tables(request);
                    if let Err(err) = response_sender.send(result) {
                        log::warn!("Could not send extracted tables response: {err:?}.");
                    }
                }
                Err(_) => {
                    log::info!("Request channel closed; terminating ExtractorServer worker loop.");
                    return;
                }
            }
        }
    }
}

enum ServeRequest {
    ExtractTables(Request),
}

struct Request {
    request: tabula_wrapper::TabulaExtractionRequest,
    response_sender: mpsc::SyncSender<anyhow::Result<tabula_wrapper::JsonTableSet>>,
}

#[derive(Clone)]
pub struct ExtractorClient {
    request_sender: mpsc::SyncSender<ServeRequest>,
}

impl tabula_wrapper::TabulaExtractor for ExtractorClient {
    fn extract_tables(
        &self,
        request: super::TabulaExtractionRequest,
    ) -> anyhow::Result<tabula_wrapper::JsonTableSet> {
        let (response_sender, response_receiver) = mpsc::sync_channel(0);
        self.request_sender
            .send(ServeRequest::ExtractTables(Request {
                request,
                response_sender,
            }))?;

        response_receiver.recv()?
    }
}
