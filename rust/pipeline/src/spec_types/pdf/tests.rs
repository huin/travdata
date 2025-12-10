use googletest::prelude::*;
use test_casing::{TestCases, cases, test_casing};

use crate::spec_types::pdf::{PdfPoints, TabulaPdfRect};

/// Rectable used to compare for overlapping.
const CONST_RECT: Rect = Rect {
    left: 5.0,
    top: 5.0,
    right: 10.0,
    bottom: 10.0,
};
const SMALLER_RECT: Rect = Rect {
    left: 6.0,
    top: 6.0,
    right: 9.0,
    bottom: 9.0,
};

const WITHIN_WIDTH_HEIGHT: f32 = 2.0;
const OUTSIDE_WIDTH_HEIGHT: f32 = 6.0;

#[derive(Debug)]
struct PdfRectTestCase {
    other: Rect,
    expect: bool,
}

const TEST_TABULA_PDF_RECT_IS_OVERLAPPING_CASES: TestCases<PdfRectTestCase> = cases! {
    [
        // Overlapping cases.
        PdfRectTestCase {
            other: SMALLER_RECT,
            expect: true ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (WITHIN_WIDTH_HEIGHT, 0.0),
            expect: true ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (WITHIN_WIDTH_HEIGHT, WITHIN_WIDTH_HEIGHT),
            expect: true ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (0.0, WITHIN_WIDTH_HEIGHT),
            expect: true ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (-WITHIN_WIDTH_HEIGHT, WITHIN_WIDTH_HEIGHT),
            expect: true ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (-WITHIN_WIDTH_HEIGHT, 0.0),
            expect: true ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (-WITHIN_WIDTH_HEIGHT, -WITHIN_WIDTH_HEIGHT),
            expect: true ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (0.0, -WITHIN_WIDTH_HEIGHT),
            expect: true ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (WITHIN_WIDTH_HEIGHT, -WITHIN_WIDTH_HEIGHT),
            expect: true ,
        },
        // Non-overlapping cases.
        PdfRectTestCase {
            other: SMALLER_RECT + (OUTSIDE_WIDTH_HEIGHT, 0.0),
            expect: false ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (OUTSIDE_WIDTH_HEIGHT, OUTSIDE_WIDTH_HEIGHT),
            expect: false ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (0.0, OUTSIDE_WIDTH_HEIGHT),
            expect: false ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (-OUTSIDE_WIDTH_HEIGHT, OUTSIDE_WIDTH_HEIGHT),
            expect: false ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (-OUTSIDE_WIDTH_HEIGHT, 0.0),
            expect: false ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (-OUTSIDE_WIDTH_HEIGHT, -OUTSIDE_WIDTH_HEIGHT),
            expect: false ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (0.0, -OUTSIDE_WIDTH_HEIGHT),
            expect: false ,
        },
        PdfRectTestCase {
            other: SMALLER_RECT + (OUTSIDE_WIDTH_HEIGHT, -OUTSIDE_WIDTH_HEIGHT),
            expect: false ,
        },
    ]
};

#[test]
fn test_tabula_pdf_rect_is_overlapping_cases() {
    assert_eq!(
        17,
        TEST_TABULA_PDF_RECT_IS_OVERLAPPING_CASES
            .into_iter()
            .count()
    );
}

#[test_casing(17, TEST_TABULA_PDF_RECT_IS_OVERLAPPING_CASES)]
#[gtest]
fn test_tabula_pdf_rect_is_overlapping(test_case: PdfRectTestCase) {
    let a: TabulaPdfRect = CONST_RECT.into();
    let b: TabulaPdfRect = test_case.other.into();
    expect_eq!(a.is_overlapping(&b), test_case.expect);
    expect_eq!(b.is_overlapping(&a), test_case.expect);
}

#[derive(Debug)]
struct Rect {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl std::ops::Add<(f32, f32)> for Rect {
    type Output = Rect;

    fn add(self, (x, y): (f32, f32)) -> Self::Output {
        Rect {
            left: self.left + x,
            top: self.top + y,
            right: self.right + x,
            bottom: self.bottom + y,
        }
    }
}

impl From<Rect> for TabulaPdfRect {
    fn from(value: Rect) -> Self {
        Self {
            left: PdfPoints::from_f32(value.left),
            top: PdfPoints::from_f32(value.top),
            right: PdfPoints::from_f32(value.right),
            bottom: PdfPoints::from_f32(value.bottom),
        }
    }
}
