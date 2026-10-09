use pdf_oxide::geometry::{Point, Rect};

#[test]
fn test_page_1_all_tables() {
    let doc = open_test_document();

    // WHEN: extract_tables is called for page 0.
    let actual_tables = doc.extract_tables(0).expect("failed to extract tables");

    // THEN: the actual tables' contents matches the expected contents.
    let actual_contents: Vec<_> = actual_tables.iter().map(table_to_vec_vec_str).collect();
    assert_eq!(
        actual_contents,
        vec![
            expected_page_0_table_0(),
            vec![
                vec!["T2 Heading 1", "T2 Heading 2", "T2 Heading 3"],
                vec!["t2r1c1", "t2r1c3", "t2r1c3"],
                vec!["t2r2c1", "t2r2c3", "t2r2c3"],
            ],
            vec![
                vec!["T3 Heading 1", "T3 Heading 2", "T3 Heading 3"],
                vec!["t3r1c1", "t3r1c3", "t3r1c3"],
                vec!["t3r2c1", "t3r2c3", "t3r2c3"],
            ],
            vec![
                vec!["t3r3c1", "t3r3c3", "t3r3c3"],
                vec!["t3r4c1", "t3r4c3", "t3r4c3"],
            ],
        ]
    );
}

#[test]
fn test_rect_bounds() {
    let around_table_0 = Rect::from_points(
        TABLE_0_OUTSIDE_TOP_LEFT.x,
        TABLE_0_OUTSIDE_TOP_LEFT.y,
        TABLE_0_OUTSIDE_BOTTOM_RIGHT.x,
        TABLE_0_OUTSIDE_BOTTOM_RIGHT.y,
    );
    let within_table_0 = Rect::from_points(
        TABLE_0_INSIDE_TOP_LEFT.x,
        TABLE_0_INSIDE_TOP_LEFT.y,
        TABLE_0_INSIDE_BOTTOM_RIGHT.x,
        TABLE_0_INSIDE_BOTTOM_RIGHT.y,
    );

    assert!(around_table_0.contains_rect(&within_table_0));
}

#[test]
fn test_extract_tables_in_rect_finds_table_with_visible_component() {
    let doc = open_test_document();

    let around_table_1 = Rect::from_points(
        TABLE_0_OUTSIDE_TOP_LEFT.x,
        TABLE_0_OUTSIDE_TOP_LEFT.y,
        TABLE_0_OUTSIDE_BOTTOM_RIGHT.x,
        TABLE_0_OUTSIDE_BOTTOM_RIGHT.y,
    );

    pdf_oxide::rendering::render_page_region(
        &doc,
        0,
        // Doc bug: crop_rect_pt is unclear as to which tuple entry is what without reading the impl
        // source code, and which corner of the rect x and y must be.
        (
            around_table_1.x,
            around_table_1.y,
            around_table_1.width,
            around_table_1.height,
        ),
        &pdf_oxide::rendering::RenderOptions::default(),
    )
    .expect("failed to render PDF page region")
    .save("page_0.png")
    .expect("failed to write rendered PDF page region to file");

    let tables = doc
        .extract_tables_in_rect(0, around_table_1)
        .expect("failed to extract tables");

    assert_eq!(tables.len(), 1);
}

#[test]
fn test_extract_tables_in_rect_page_0_table_0_is_consistent_with_extract_tables() {
    let doc = open_test_document();

    let actual_tables = doc.extract_tables(0).expect("failed to extract tables");

    let actual_all_page_0_table_0 = actual_tables
        .first()
        .expect("expected at least one table in page 0");
    let actual_all_page_0_table_0_bbox = actual_all_page_0_table_0
        .bbox
        .expect("table_0.bbox should be Some");

    let around_table_0 = Rect::from_points(
        TABLE_0_OUTSIDE_TOP_LEFT.x,
        TABLE_0_OUTSIDE_TOP_LEFT.y,
        TABLE_0_OUTSIDE_BOTTOM_RIGHT.x,
        TABLE_0_OUTSIDE_BOTTOM_RIGHT.y,
    );

    assert!(around_table_0.contains_rect(&actual_all_page_0_table_0_bbox));

    let tables = doc
        .extract_tables_in_rect(0, around_table_0)
        .expect("failed to extract tables");

    let actual_rect_page_0_table_0 = tables.first().expect("expected");

    assert_eq!(
        table_to_vec_vec_str(actual_rect_page_0_table_0),
        expected_page_0_table_0()
    );
}

fn table_to_vec_vec_str(table: &pdf_oxide::structure::Table) -> Vec<Vec<&str>> {
    table.rows.iter().map(row_to_vec_str).collect()
}

fn row_to_vec_str(row: &pdf_oxide::structure::TableRow) -> Vec<&str> {
    row.cells.iter().map(|cell| cell.text.as_str()).collect()
}

fn expected_page_0_table_0() -> Vec<Vec<&'static str>> {
    vec![
        vec!["T1 Heading 1", "T1 Heading 2", "T1 Heading 3"],
        vec!["t1r1c1", "t1r1c3", "t1r1c3"],
        vec!["t1r2c1", "t1r2c3", "t1r2c3"],
    ]
}

const TABLE_0_OUTSIDE_TOP_LEFT: Point = Point { x: 53., y: 755. };
const TABLE_0_OUTSIDE_BOTTOM_RIGHT: Point = Point { x: 539., y: 690. };
const TABLE_0_INSIDE_TOP_LEFT: Point = Point { x: 63., y: 743. };
const TABLE_0_INSIDE_BOTTOM_RIGHT: Point = Point { x: 445., y: 702. };

fn open_test_document() -> pdf_oxide::PdfDocument {
    pdf_oxide::PdfDocument::open("../pipeline/test_data/tables.pdf").expect("failed to open PDF")
}

fn f32_sub_abs_lt_tenth(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.1
}
