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

use crate::extraction::pdf::pdfiumthread::PdfiumClient;
use crate::gui::workers::pdfiumworker;

use super::workers::pdfiumworker::PdfiumWorker;

const UNLOADED_SIZE: i32 = 100;

/// Input messages for [PageView].
#[derive(Debug)]
pub enum Input {
    SelectPdf { path: Option<PathBuf> },

    // Internal:
    PdfiumEvent(pdfiumworker::Output),
}

/// Relm4 component that views a PDF page preview.
pub struct PageView {
    renderer: Rc<Renderer>,
    page_index: u16,
    /// ID of the loaded PDF, if any.
    document_id: Option<pdfiumworker::DocumentId>,

    pdfium_worker: Controller<PdfiumWorker>,
}

impl PageView {
    fn unload_current_pdf(&mut self) {
        if let Some(id) = self.document_id {
            self.pdfium_worker
                .emit(pdfiumworker::Input::UnloadPdf { id });
        }
        self.document_id = None;
    }
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

                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Automatic,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,
                    set_propagate_natural_width: false,
                    set_propagate_natural_height: false,

                    gtk::DrawingArea {
                        set_cursor_from_name: Some("crosshair"),

                        #[watch]
                        set_content_width: model.renderer.width(),
                        #[watch]
                        set_content_height: model.renderer.height(),

                        set_draw_func: move |_drawing_area, cr, _width, _height| renderer.draw(cr),
                    }
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
            Input::SelectPdf { path: path_opt } => {
                self.unload_current_pdf();

                if let Some(path) = path_opt {
                    self.pdfium_worker
                        .emit(pdfiumworker::Input::LoadPdf { path });
                    self.renderer.set_page_pixbuf(None);
                }
            }
            Input::PdfiumEvent(event) => match event {
                pdfiumworker::Output::PdfLoaded { id_result: Ok(id) } => {
                    self.document_id = Some(id);
                    self.page_index = 0;
                    self.pdfium_worker.emit(pdfiumworker::Input::RenderPage {
                        id,
                        page_index: self.page_index,
                    });
                }
                pdfiumworker::Output::PdfLoaded {
                    id_result: Err(err),
                } => {
                    // TODO: Make errors visible in the GUI.
                    log::error!("Failed to load PDF: {:?}", err);
                }
                pdfiumworker::Output::PageRendered {
                    id,
                    page_index,
                    image_result: Ok(pixbuf_data),
                } => {
                    if Some(id) != self.document_id || page_index != self.page_index {
                        // Selection has changed since request was made.
                        return;
                    }
                    self.renderer
                        .set_page_pixbuf(Some(pdfiumworker::NewPixbuf::from(pixbuf_data).0));
                }
                pdfiumworker::Output::PageRendered {
                    id: _,
                    page_index: _,
                    image_result: Err(err),
                } => {
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
            document_id: None,

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
