use super::*;

/// A discrete change to an open [Document].
pub enum Edit {
    SetGroupName {
        group: GroupToken,
        new_name: String,
        old_name: String,
    },
    SetTableName {
        table: TableToken,
        new_name: String,
        old_name: String,
    },
}

impl undo::Edit for Edit {
    type Target = DocumentState;

    type Output = Result<(), EditError>;

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        use Edit::*;

        match &self {
            SetGroupName {
                group,
                new_name,
                old_name: _,
            } => {
                target.allocs.get_mut_group(*group)?.name = new_name.clone();
            }
            SetTableName {
                table,
                new_name,
                old_name: _,
            } => {
                target.allocs.get_mut_table(*table)?.name = new_name.clone();
            }
        }

        Ok(())
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        use Edit::*;

        match &self {
            SetGroupName {
                group,
                new_name: _,
                old_name,
            } => {
                target.allocs.get_mut_group(*group)?.name = old_name.clone();
            }
            SetTableName {
                table,
                new_name: _,
                old_name,
            } => {
                target.allocs.get_mut_table(*table)?.name = old_name.clone();
            }
        }

        Ok(())
    }
}
