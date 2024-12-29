use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use anyhow::Result;
use gtk::cairo;
use gtk::gdk::RGBA;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib::property::PropertySet;
use gtk::prelude::{
    DrawingAreaExt, DrawingAreaExtManual, FrameExt, GdkCairoContextExt, OrientableExt, WidgetExt,
};
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    SimpleComponent,
};

use crate::extraction::pdf::pdfiumworker::PdfiumClient;
use crate::gui::workers::pdfiumworker;

use super::workers::pdfiumworker::PdfiumWorker;

const UNLOADED_SIZE: i32 = 100;

#[derive(Debug)]
pub enum Input {
    LoadPdf(Option<PathBuf>),

    // Internal:
    PdfiumEvent(pdfiumworker::Output),
}

/// Relm4 component that views a PDF page preview.
pub struct PageView {
    renderer: Rc<Renderer>,
    page_index: u16,
    pdf_loaded: bool,

    pdfium_worker: Controller<PdfiumWorker>,
}

#[relm4::component(pub)]
impl SimpleComponent for PageView {
    type Init = PdfiumClient;
    type Input = Input;
    type Output = ();

    view! {
        gtk::Frame {
            set_label: Some("PDF page view"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::DrawingArea {
                    set_cursor_from_name: Some("crosshair"),

                    #[watch]
                    set_content_width: model.renderer.width(),
                    #[watch]
                    set_content_height: model.renderer.height(),

                    set_draw_func: move |_drawing_area, cr, _width, _height| renderer.draw(cr),
                },

                // Page selection.
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    // TODO: Use PDF metadata to know page number range.
                    // TODO: Render selected page.
                    gtk::Label {
                        set_label: "Page",
                    },
                    gtk::SpinButton {
                    },
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::LoadPdf(path_opt) => {
                self.pdf_loaded = false;
                self.renderer.set_page_pixbuf(None);
                match path_opt {
                    Some(path) => {
                        self.pdfium_worker.emit(pdfiumworker::Input::LoadPdf(path));
                    }
                    None => {
                        self.pdfium_worker.emit(pdfiumworker::Input::UnloadPdf);
                    }
                }
            }
            Input::PdfiumEvent(event) => match event {
                pdfiumworker::Output::PdfLoaded(Ok(_metadata)) => {
                    self.pdf_loaded = true;
                    self.page_index = 0;
                    self.pdfium_worker
                        .emit(pdfiumworker::Input::RenderPage(self.page_index));
                }
                pdfiumworker::Output::PdfLoaded(Err(err)) => {
                    // TODO: Make errors visible in the GUI.
                    log::error!("Failed to load PDF: {:?}", err);
                }
                pdfiumworker::Output::PageRendered(Ok(pixbuf_data)) => {
                    self.renderer
                        .set_page_pixbuf(Some(pdfiumworker::NewPixbuf::from(pixbuf_data).0));
                }
                pdfiumworker::Output::PageRendered(Err(err)) => {
                    // TODO: Make errors visible in the GUI.
                    log::error!("Failed to load PDF: {:?}", err);
                }
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let renderer = Rc::new(Renderer::new());

        let model = Self {
            renderer: renderer.clone(),
            page_index: 0,
            pdf_loaded: false,

            pdfium_worker: pdfiumworker::PdfiumWorker::builder()
                .launch(init)
                .forward(sender.input_sender(), Input::PdfiumEvent),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}

struct Renderer {
    page_pixbuf: Mutex<Option<Pixbuf>>,
}

impl Renderer {
    fn new() -> Self {
        Self {
            page_pixbuf: Mutex::new(None),
        }
    }

    fn set_page_pixbuf(&self, page_pixbuf: Option<Pixbuf>) {
        self.page_pixbuf.set(page_pixbuf);
    }

    fn width(&self) -> i32 {
        self.page_pixbuf
            .lock()
            .unwrap()
            .as_ref()
            .map(|pb| pb.width())
            .unwrap_or(UNLOADED_SIZE)
    }

    fn height(&self) -> i32 {
        self.page_pixbuf
            .lock()
            .unwrap()
            .as_ref()
            .map(|pb| pb.height())
            .unwrap_or(UNLOADED_SIZE)
    }

    fn draw(&self, cr: &cairo::Context) {
        if let Err(err) = self.draw_inner(cr) {
            log::error!("Failed to render page view: {:?}", err);
        }
    }

    fn draw_inner(&self, cr: &cairo::Context) -> Result<()> {
        let page_pixbuf_guard = self.page_pixbuf.lock().unwrap();
        let page_pixbuf = match page_pixbuf_guard.as_ref() {
            Some(page_pixbuf) => page_pixbuf,
            None => {
                cr.set_source_color(&RGBA::BLACK);
                cr.paint()?;
                return Ok(());
            }
        };

        cr.set_source_pixbuf(page_pixbuf, 0.0, 0.0);
        cr.paint()?;

        Ok(())
    }
}
