use anyhow::Result;
use googletest::prelude::*;

use super::*;

const INITIAL_NAME: &str = "initial name";
const NEW_NAME: &str = "new name";
const SECOND_NEW_NAME: &str = "second new name";

#[googletest::test]
fn test_undo_redo() -> Result<()> {
    let doc = DocumentRc::new();

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
fn test_edit_table_name() -> Result<()> {
    let doc = DocumentRc::new();

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
fn test_edit_table_name_merges_consecutive_edits() -> Result<()> {
    let doc = DocumentRc::new();

    let table = doc.new_table(TableData {
        name: INITIAL_NAME.to_string(),
        ..Default::default()
    });
    table.edit_name(NEW_NAME.to_string())?;
    table.edit_name(SECOND_NEW_NAME.to_string())?;
    assert_that!(table.get_name()?, eq(SECOND_NEW_NAME));

    // Should skip over NEW_NAME because the edits were merged.
    doc.undo()?;
    assert_that!(table.get_name()?, eq(INITIAL_NAME));

    Ok(())
}

#[googletest::test]
fn test_edit_group_name() -> Result<()> {
    let doc = DocumentRc::new();

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
