//! Defines the internal representation of extraction templates.

#![allow(dead_code)]

pub mod loadarg;
pub mod serialised;

use std::collections::{HashMap, HashSet};

use borsh::BorshSerialize;

use crate::extraction::tableextract;

/// Extraction template for a single book.
#[derive(Clone, Debug)]
pub struct Book {
    /// Scripts providing helper code for code in [Table]`.transform`.
    pub scripts: Vec<Script>,
    /// Top-level group of templated extractions.
    pub group: Group,
    /// Offset of visible page numbers from the index.
    pub page_offset: i32,
}

/// Provides code for the v8 VM isolate.
#[derive(Clone, Debug)]
pub struct Script {
    /// Identifying name of the script, often the basis of a filename.
    pub name: String,
    /// Source code of the script.
    pub code: String,
}

/// Hierarchy of content to extract from a [Book]. The hierarchy has two effects:
/// - Tag inheritance, identifying extracted content (for filtering and in the output).
/// - Output directory structure for extracted data.
#[derive(Clone, Debug)]
pub struct Group {
    /// Set of tags defined on the group and all its descendent [Table]s.
    pub tags: HashSet<String>,
    /// Child groups.
    pub groups: HashMap<String, Group>,
    /// Child tables.
    pub tables: HashMap<String, Table>,
}

/// Template for extracting a single table.
#[derive(Clone, Debug)]
pub struct Table {
    /// Set of tags defined on the table (not including those from the parent [Group].
    pub tags: HashSet<String>,
    /// Individual raw regions of table to extract.
    pub portions: Vec<TablePortion>,
    /// Defines the transformation to apply to the raw extracted table portions to define the final
    /// tabular data.
    pub transform: Option<tableextract::TableTransform>,
}

/// Single raw region of tabular data to extract.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TablePortion {
    pub extraction_method: TabulaExtractionMethod,
    /// Zero-based index of the page to extract data from.
    pub page: i32,
    /// Region of the PDF page.
    pub rect: PDFRect,
}

/// Measurement of space within a PDF page, 1 = 1/72 of an inch.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct PDFPoints(i64);

impl PDFPoints {
    /// Fraction of a point that can be represented.
    const PRECISION: f32 = 4096.0;

    #[cfg(test)]
    pub const fn from_quantised(quantised: i64) -> Self {
        Self(quantised)
    }

    pub fn from_f32(value: f32) -> Self {
        let quantised = (value * Self::PRECISION).round() as i64;
        Self(quantised)
    }

    pub fn to_f32(self) -> f32 {
        (self.0 as f32) / Self::PRECISION
    }
}

impl std::ops::Sub for PDFPoints {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::fmt::Debug for PDFPoints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PDFPoints")
            .field(&self.0)
            .field(&self.to_f32())
            .finish()
    }
}

impl From<f32> for PDFPoints {
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

// NOTE: In the PDF coordinate system, the origin (0,0) is at the bottom left of page. Therefore
// for a valid [PDFRect] the following must be true: `left <= right && bottom <= top`.
//
// Each dimension is measured in points (1/72 of an inch).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PDFRect {
    /// Horizontal coordinate of the left hand side of the rectangle.
    pub left: PDFPoints,
    /// Vertical coordinate of the top of the rectangle.
    pub top: PDFPoints,
    /// Horizontal coordinate of the right hand side of the rectangle.
    pub right: PDFPoints,
    /// Vertical coordinate of the bottom of the rectangle.
    pub bottom: PDFPoints,
}

/// Extraction algorithm for Tabula to use.
#[derive(BorshSerialize, Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum TabulaExtractionMethod {
    Guess,
    Lattice,
    Stream,
}
