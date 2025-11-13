use serde::{Deserialize, Serialize, de::Visitor};

/// Defines a page-aligned rectangular region within a page of a PDF, using the Tabula origin at
/// the top-left of the page, rather than the standard PDF origin at the bottom left.
///
/// NOTE: In the Tabula coordinate system, the origin (0,0) is at the top left of page. Therefore
/// for a valid [PdfRect] the following must be true: `left <= right && top <= bottom`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TabulaPdfRect {
    /// Horizontal coordinate of the left hand side of the rectangle.
    pub left: PdfPoints,
    /// Vertical coordinate of the top of the rectangle.
    pub top: PdfPoints,
    /// Horizontal coordinate of the right hand side of the rectangle.
    pub right: PdfPoints,
    /// Vertical coordinate of the bottom of the rectangle.
    pub bottom: PdfPoints,
}

impl TabulaPdfRect {
    fn width(&self) -> PdfPoints {
        self.right - self.left
    }

    fn height(&self) -> PdfPoints {
        self.bottom - self.top
    }

    pub fn to_tabula_rectangle(self) -> tabula::Rectangle {
        tabula::Rectangle::new(
            self.left.to_f32(),
            self.top.to_f32(),
            self.width().to_f32(),
            self.height().to_f32(),
        )
    }

    pub fn to_tabula_rectangle_page_area(&self) -> (i32, tabula::Rectangle) {
        (
            tabula::ABSOLUTE_AREA_CALCULATION_MODE,
            self.to_tabula_rectangle(),
        )
    }
}

/// Extraction algorithm for Tabula to use.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TabulaExtractionMethod {
    Guess,
    Lattice,
    Stream,
}

impl TabulaExtractionMethod {
    pub fn to_tabula_extraction_method(self) -> tabula::ExtractionMethod {
        match self {
            TabulaExtractionMethod::Stream => tabula::ExtractionMethod::Basic,
            TabulaExtractionMethod::Guess => tabula::ExtractionMethod::Decide,
            TabulaExtractionMethod::Lattice => tabula::ExtractionMethod::Spreadsheet,
        }
    }
}

/// Measurement of space within a Pdf page, 1 = 1/72 of an inch.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct PdfPoints(i64);

impl PdfPoints {
    /// Fraction of a point that can be represented.
    const PRECISION: f32 = 4096.0;

    /// Creates a [PdfPoints] with the given [f32] value.
    pub fn from_f32(value: f32) -> Self {
        let quantised = (value * Self::PRECISION).round() as i64;
        Self(quantised)
    }

    /// Returns the number of PDF points as a [f32] value.
    pub fn to_f32(self) -> f32 {
        (self.0 as f32) / Self::PRECISION
    }
}

impl std::ops::Sub for PdfPoints {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::fmt::Debug for PdfPoints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PdfPoints({} (quantised={}))", self.to_f32(), self.0)
    }
}

impl From<f32> for PdfPoints {
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

impl<'de> Deserialize<'de> for PdfPoints {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_f32(PdfPointsVisitor)
    }
}

struct PdfPointsVisitor;

impl<'de> Visitor<'de> for PdfPointsVisitor {
    type Value = PdfPoints;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a floating point number")
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PdfPoints::from_f32(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PdfPoints::from_f32(v as f32))
    }
}

impl Serialize for PdfPoints {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(self.to_f32())
    }
}
