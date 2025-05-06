use crate::template;

use super::{Document, arena};

pub type TablePortionToken = arena::TypedToken<TablePortion>;

#[derive(Debug)]
pub struct TablePortionData(template::TablePortion);

pub struct TablePortion {
    doc: Document,
    token: TablePortionToken,
}

impl TablePortion {
    pub fn token(&self) -> TablePortionToken {
        self.token
    }
}

impl std::fmt::Debug for TablePortion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doc = self.doc.get_inner();
        let inner = doc.state.table_portion_arena.get_inner(self.token);
        f.debug_struct("TablePortion")
            .field("doc", &self.doc)
            .field("token", &self.token)
            .field("portion", &inner)
            .finish()
    }
}
