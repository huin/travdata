use std::{path::PathBuf, time::Instant};

use anyhow::Result;
use clap::Args;

use pdfium_render::prelude::*;

#[derive(Args, Debug)]
pub struct Command {
    pdf_path: PathBuf,
    page: u16,
    output: PathBuf,
}

pub fn run(cmd: &Command) -> Result<()> {
    let pdfium = Pdfium::new(Pdfium::bind_to_statically_linked_library()?);

    let document = pdfium.load_pdf_from_file(&cmd.pdf_path, None)?;
    let md = document.metadata();
    for tag in md.iter() {
        println!("{:?} :: {}", tag.tag_type(), tag.value());
    }

    let page = document.pages().get(cmd.page)?;
    let rect = ImageRect {
        left: 73.0575,
        top: 170.9775,
        width: 337.365,
        height: 310.59000000000003,
    };

    // Render page with selected rectangle drawn.
    let config = PdfRenderConfig::new();
    let mut page_image = page.render_with_config(&config)?.as_image().into_rgb8();
    draw_rect(&mut page_image, &rect, image::Rgb::<u8>([0, 0, 0]));
    page_image.save_with_format(&cmd.output, image::ImageFormat::Jpeg)?;

    // Test text extraction speed.
    const ITERATIONS: u32 = 100;
    let before = Instant::now();
    for _i in 0..ITERATIONS {
        print_text_in_rect(&page, &rect)?;
    }
    let total_time = before.elapsed();
    let per_time = total_time / ITERATIONS;
    println!("Total time: {:?}", total_time);
    println!("Per time: {:?}", per_time);

    Ok(())
}

struct ImageRect {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl ImageRect {
    fn pdf_rect(&self, page: &PdfPage) -> PdfRect {
        let page_rect = page.page_size();
        PdfRect {
            left: PdfPoints {
                value: self.left as f32,
            },
            top: PdfPoints {
                value: page_rect.top.value - (self.top as f32),
            },
            right: PdfPoints {
                value: (self.left + self.width) as f32,
            },
            bottom: PdfPoints {
                value: page_rect.top.value - (self.top + self.height) as f32,
            },
        }
    }
}

fn draw_rect<I: image::GenericImage>(img: &mut I, rect: &ImageRect, pixel: I::Pixel) {
    let l = rect.left.round() as u32;
    let t = rect.top.round() as u32;
    let r = (rect.left + rect.width).round() as u32;
    let b = (rect.top + rect.height).round() as u32;

    for x in l..r {
        img.put_pixel(x, t, pixel);
        img.put_pixel(x, b, pixel);
    }
    for y in t..b {
        img.put_pixel(l, y, pixel);
        img.put_pixel(r, y, pixel);
    }
}

fn print_text_in_rect(page: &PdfPage<'_>, rect: &ImageRect) -> Result<()> {
    let pdf_rect = rect.pdf_rect(page);
    let texts = page.text()?;
    let segments = texts.segments();
    for segment in segments.iter() {
        let bounds = segment.bounds();
        if !bounds.is_inside(&pdf_rect) {
            continue;
        }
        println!("bounds: {} text: {}", bounds, segment.text());
    }
    Ok(())
}
