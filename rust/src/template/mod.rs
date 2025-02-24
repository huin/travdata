#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

/// Extraction template for a single book.
pub struct Book {
    /// Source code of scripts.
    pub scripts: Vec<String>,
    /// Groups of templated extractions.
    pub groups: HashMap<String, Group>,
}

/// Hierarchy of content to extract from a [Book]. The hierarchy has two effects:
/// - Tag inheritance, identifying extracted content (for filtering and in the output).
/// - Output directory structure for extracted data.
pub struct Group {
    /// Set of tags defined on the group and all its descendent [Table]s.
    pub tags: HashSet<String>,
    /// Child groups.
    pub groups: HashMap<String, Group>,
    /// Child tables.
    pub tables: HashMap<String, Table>,
}

/// Template for extracting a single table.
pub struct Table {
    /// Set of tags defined on the table (not including those from the parent [Group].
    pub tags: HashSet<String>,
    /// Individual raw regions of table to extract.
    pub segments: Vec<TablePortion>,
    /// ECMAScript function body, defining the transformation to apply to the raw extracted table
    /// portions to define the final tabular data.
    pub transform: String,
}

/// Single raw region of tabular data to extract.
pub struct TablePortion {
    /// Optional key to include to identify the portion when being processed by the
    /// [Table] `transform` function.
    pub key: Option<String>,
    pub extraction_method: ExtractionMethod,
    /// Zero-based index of the page to extract data from.
    pub page: i32,
    /// Horizontal coordinate of the left hand side of the rectangle.
    pub left: f32,
    /// Vertical coordinate of the top of the rectangle.
    pub top: f32,
    /// Horizontal coordinate of the right hand side of the rectangle.
    pub right: f32,
    /// Vertical coordinate of the bottom of the rectangle.
    pub bottom: f32,
}

/// Extraction algorithm for Tabula to use.
pub enum ExtractionMethod {
    Guess,
    Lattice,
    Stream,
}
