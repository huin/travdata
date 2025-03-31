use serde::Deserialize;

use crate::template;

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Template(pub Vec<TemplateEntry>);

#[derive(Clone, Copy, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ExtractionMethod {
    Guess,
    Lattice,
    Stream,
}

impl From<template::TabulaExtractionMethod> for ExtractionMethod {
    fn from(value: template::TabulaExtractionMethod) -> Self {
        match value {
            template::TabulaExtractionMethod::Guess => Self::Guess,
            template::TabulaExtractionMethod::Lattice => Self::Lattice,
            template::TabulaExtractionMethod::Stream => Self::Stream,
        }
    }
}

impl From<ExtractionMethod> for template::TabulaExtractionMethod {
    fn from(value: ExtractionMethod) -> Self {
        match value {
            ExtractionMethod::Guess => Self::Guess,
            ExtractionMethod::Lattice => Self::Lattice,
            ExtractionMethod::Stream => Self::Stream,
        }
    }
}

// NOTE: In the PDF coordinate system, the origin (0,0) is at the bottom left of page. Therefore
// for a valid [TemplateEntry] the following must be true: `x1 <= x2 && y1 <= y2`.
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TemplateEntry {
    pub page: i32,
    pub extraction_method: ExtractionMethod,
    /// Position of the left side of the rectangle.
    pub x1: f32,
    /// Position of the right side of the rectangle.
    pub x2: f32,
    /// Position of the bottom side of the rectangle.
    pub y1: f32,
    /// Position of the top side of the rectangle.
    pub y2: f32,
    /// Width of the rectangle (`x2 - x1`).
    pub width: f32,
    /// Height of the rectangle (`y2 - y1`).
    pub height: f32,
}

impl From<template::TablePortion> for TemplateEntry {
    fn from(value: template::TablePortion) -> Self {
        Self {
            extraction_method: value.extraction_method.into(),
            page: value.page,
            x1: value.rect.left.to_f32(),
            x2: value.rect.right.to_f32(),
            y1: value.rect.bottom.to_f32(),
            y2: value.rect.top.to_f32(),
            width: (value.rect.right - value.rect.left).to_f32(),
            height: (value.rect.top - value.rect.bottom).to_f32(),
        }
    }
}

impl From<TemplateEntry> for template::TablePortion {
    fn from(value: TemplateEntry) -> Self {
        Self {
            key: None,
            extraction_method: value.extraction_method.into(),
            page: value.page,
            rect: template::PDFRect {
                left: value.x1.into(),
                right: value.x2.into(),
                bottom: value.y1.into(),
                top: value.y2.into(),
            },
        }
    }
}
