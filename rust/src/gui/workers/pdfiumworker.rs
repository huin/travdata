use std::path::PathBuf;

use anyhow::Result;
use gtk::{
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::Bytes,
};
use relm4::Worker;

use crate::{extraction::pdf::pdfiumthread, gui::util};

pub type DocumentId = pdfiumthread::DocumentId;
pub type PdfMetadata = pdfiumthread::PdfMetadata;

/// Input messages for [PdfiumWorker].
#[derive(Debug)]
pub enum Input {
    /// Requests that the identified document is unloaded.
    UnloadPdf { id: DocumentId },
    /// Requests loading the PDF at the given file path.
    LoadPdf { path: PathBuf },
    /// Requests rendering a page of the identified document.
    RenderPage { id: DocumentId, page_index: u16 },
}

/// Output messages for [PdfiumWorker].
#[derive(Debug)]
pub enum Output {
    /// Requested attempt to load the requested PDF has completed.
    PdfLoaded {
        metadata_result: Result<PdfMetadata>,
    },
    /// Requested attempt to render a page of the document has completed.
    PageRendered {
        id: DocumentId,
        page_index: u16,
        image_result: Result<PixbufData>,
    },
}

/// Relm4 [Worker] component for manipulating PDF documents in a worker thread.
pub struct PdfiumWorker {
    pdfium_client: pdfiumthread::PdfiumClient,
}

impl Worker for PdfiumWorker {
    type Init = pdfiumthread::PdfiumClient;
    type Input = Input;
    type Output = Output;

    fn init(init: Self::Init, _sender: relm4::ComponentSender<Self>) -> Self {
        Self {
            pdfium_client: init,
        }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        match message {
            Input::UnloadPdf { id } => {
                if let Err(err) = self.pdfium_client.unload_pdf(id) {
                    log::warn!("Error unloading PDF: {:?}", err);
                }
            }
            Input::LoadPdf { path } => {
                let metadata_result = self.pdfium_client.load_pdf(path);
                util::send_output_or_log(
                    Output::PdfLoaded { metadata_result },
                    "PdfLoaded",
                    &sender,
                );
            }
            Input::RenderPage { id, page_index } => {
                let result = self.pdfium_client.render_page(id, page_index);
                util::send_output_or_log(
                    Output::PageRendered {
                        id,
                        page_index,
                        image_result: result.map(PixbufData::from),
                    },
                    "PageRendered",
                    &sender,
                );
            }
        }
    }
}

/// [Send]able data to construct a [Pixbuf] (which does not implement [Send].
#[derive(Debug)]
pub struct PixbufData {
    data: Bytes,
    colorspace: Colorspace,
    has_alpha: bool,
    bits_per_sample: i32,
    width: i32,
    height: i32,
    rowstride: i32,
}

impl From<pdfiumthread::PageImage> for PixbufData {
    fn from(value: pdfiumthread::PageImage) -> Self {
        let width = value.width() as i32;
        let height = value.height() as i32;
        let sample_layout = value.sample_layout();
        let rowstride = sample_layout.height_stride as i32;
        let buffer = value.into_raw();
        Self {
            data: Bytes::from_owned(buffer),
            colorspace: Colorspace::Rgb,
            has_alpha: false,
            bits_per_sample: 8,
            width,
            height,
            rowstride,
        }
    }
}

/// Simple wrapper around [Pixbuf] that supports implementing [From].
pub struct NewPixbuf(pub Pixbuf);

impl From<PixbufData> for NewPixbuf {
    fn from(value: PixbufData) -> Self {
        Self(Pixbuf::from_bytes(
            &value.data,
            value.colorspace,
            value.has_alpha,
            value.bits_per_sample,
            value.width,
            value.height,
            value.rowstride,
        ))
    }
}
