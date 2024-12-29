// XXX
#![allow(dead_code)]

use std::{collections::HashMap, path::PathBuf};

pub struct Book {
    pub scripts: Vec<PathBuf>,
    pub group: Group,
}

pub struct Group {
    pub groups: HashMap<String, Group>,
    pub tables: HashMap<String, Table>,
}

pub struct Table {
    pub segments: Vec<TableSegment>,
    pub transform: String,
}

pub struct TableSegment {
    pub page: i32,

    pub extraction_method: ExtractionMethod,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

pub enum ExtractionMethod {
    Stream,
    Guess,
    Lattice,
}
