use chrono::TimeDelta;
use clock::{FakeClock, Timestamp};
use googletest::Result;
use googletest::prelude::*;

use super::*;

const INITIAL_NAME: &str = "initial name";
const NEW_NAME: &str = "new name";
const SECOND_NEW_NAME: &str = "second new name";

// Timestamp at 2020-01-01T00:00:00Z.
const START_TIMESTAMP: Timestamp = Timestamp::from_timestamp_nanos(1_577_836_800_000_000_000);

const WITHIN_MERGE_TIME_DELTA: TimeDelta = TimeDelta::new(4, 0).unwrap();
const MAX_MERGE_TIME_DELTA: TimeDelta = TimeDelta::new(5, 0).unwrap();
const BEYOND_MERGE_TIME_DELTA: TimeDelta = TimeDelta::new(6, 0).unwrap();

struct DocumentFixture {
    doc: Document,
    fake_clock: FakeClock,
}

impl ConsumableFixture for DocumentFixture {
    fn set_up() -> Result<Self> {
        let fake_clock = FakeClock::new(START_TIMESTAMP);
        edit::set_max_merge_edit_time_delta(MAX_MERGE_TIME_DELTA);
        Ok(Self {
            doc: Document::new_with_clock(Rc::new(fake_clock.clone())),
            fake_clock,
        })
    }
}

#[googletest::test]
fn test_undo_redo(fixture: DocumentFixture) -> Result<()> {
    let doc = fixture.doc;

    let table = doc.new_table(TableData {
        name: INITIAL_NAME.to_string(),
        ..Default::default()
    });
    assert_that!(table.get_name()?, eq(INITIAL_NAME));

    table.edit_name(NEW_NAME.to_string())?;
    assert_that!(table.get_name()?, eq(NEW_NAME));

    doc.undo()?;
    assert_that!(table.get_name()?, eq(INITIAL_NAME));

    assert_that!(doc.undo(), err(eq(&EditError::NothingToUndo)));

    doc.redo()?;
    assert_that!(table.get_name()?, eq(NEW_NAME));

    assert_that!(doc.redo(), err(eq(&EditError::NothingToRedo)));

    Ok(())
}

#[googletest::test]
fn test_book(fixture: DocumentFixture) -> Result<()> {
    let doc = fixture.doc;
    let book = doc.get_book();
    let group = book.get_root_group();

    assert_that!(group.get_name()?, eq("root"));

    group.edit_name("new root name".to_string())?;
    assert_that!(group.get_name()?, eq("new root name"));

    Ok(())
}

#[googletest::test]
fn test_edit_table_name(fixture: DocumentFixture) -> Result<()> {
    let doc = fixture.doc;

    let table = doc.new_table(TableData {
        name: INITIAL_NAME.to_string(),
        ..Default::default()
    });
    assert_that!(table.get_name()?, eq(INITIAL_NAME));

    table.edit_name(NEW_NAME.to_string())?;
    assert_that!(table.get_name()?, eq(NEW_NAME));

    doc.undo()?;
    assert_that!(table.get_name()?, eq(INITIAL_NAME));

    Ok(())
}

#[googletest::test]
fn test_edit_table_name_merges_consecutive_edits_within_merge_time_delta(
    fixture: DocumentFixture,
) -> Result<()> {
    let DocumentFixture {
        doc,
        mut fake_clock,
    } = fixture;

    let table = doc.new_table(TableData {
        name: INITIAL_NAME.to_string(),
        ..Default::default()
    });
    table.edit_name(NEW_NAME.to_string())?;
    fake_clock.advance(WITHIN_MERGE_TIME_DELTA);
    table.edit_name(SECOND_NEW_NAME.to_string())?;
    assert_that!(table.get_name()?, eq(SECOND_NEW_NAME));

    // Should skip over NEW_NAME because the edits were merged.
    doc.undo()?;
    assert_that!(table.get_name()?, eq(INITIAL_NAME));

    Ok(())
}

#[googletest::test]
fn test_edit_table_name_does_not_merge_consecutive_edits_beyond_merge_time_delta(
    fixture: DocumentFixture,
) -> Result<()> {
    let DocumentFixture {
        doc,
        mut fake_clock,
    } = fixture;

    let table = doc.new_table(TableData {
        name: INITIAL_NAME.to_string(),
        ..Default::default()
    });
    table.edit_name(NEW_NAME.to_string())?;
    fake_clock.advance(BEYOND_MERGE_TIME_DELTA);
    table.edit_name(SECOND_NEW_NAME.to_string())?;
    assert_that!(table.get_name()?, eq(SECOND_NEW_NAME));

    // Should *not* skip over NEW_NAME because the edits were merged.
    doc.undo()?;
    assert_that!(table.get_name()?, eq(NEW_NAME));

    Ok(())
}

#[googletest::test]
fn test_edit_group_name(fixture: DocumentFixture) -> Result<()> {
    let doc = fixture.doc;

    let group = doc.new_group(GroupData {
        name: INITIAL_NAME.to_string(),
        ..Default::default()
    });
    assert_that!(group.get_name()?, eq(INITIAL_NAME));

    group.edit_name(NEW_NAME.to_string())?;
    assert_that!(group.get_name()?, eq(NEW_NAME));

    doc.undo()?;
    assert_that!(group.get_name()?, eq(INITIAL_NAME));

    Ok(())
}
