use std::path::PathBuf;

use anyhow::Result;
use gtk::{
    gdk_pixbuf::{Colorspace, Pixbuf},
    glib::Bytes,
};
use relm4::Worker;

use crate::{
    extraction::pdf::pdfiumworker::{PageImage, PdfMetadata, PdfiumClient},
    gui::util,
};

#[derive(Debug)]
pub enum Input {
    UnloadPdf,
    LoadPdf(PathBuf),
    RenderPage(u16),
}

#[derive(Debug)]
pub enum Output {
    PdfLoaded(Result<PdfMetadata>),
    PageRendered(Result<PixbufData>),
}

pub struct PdfiumWorker {
    pdfium_client: PdfiumClient,
}

impl Worker for PdfiumWorker {
    type Init = PdfiumClient;
    type Input = Input;
    type Output = Output;

    fn init(init: Self::Init, _sender: relm4::ComponentSender<Self>) -> Self {
        Self {
            pdfium_client: init,
        }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        match message {
            Input::UnloadPdf => {
                if let Err(err) = self.pdfium_client.unload_pdf() {
                    log::warn!("Error unloading PDF: {:?}", err);
                }
            }
            Input::LoadPdf(path) => {
                let result = self.pdfium_client.load_pdf(path);
                util::send_output_or_log(Output::PdfLoaded(result), "PdfLoaded", &sender);
            }
            Input::RenderPage(page_index) => {
                let result = self.pdfium_client.render_page(page_index);
                util::send_output_or_log(
                    Output::PageRendered(result.map(PixbufData::from)),
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

impl From<PageImage> for PixbufData {
    fn from(value: PageImage) -> Self {
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
