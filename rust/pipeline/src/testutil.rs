use std::path::{Path, PathBuf};

use generic_pipeline::node::Tag;

use crate::NodeId;

pub fn node_id(s: &str) -> crate::NodeId {
    NodeId::test_node_id(s)
}

pub fn output_path_buf<P>(s: P) -> crate::spec_types::OutputPathBuf
where
    P: Into<PathBuf> + AsRef<Path>,
{
    crate::spec_types::OutputPathBuf::new(s).expect("expected valid OutputPathBufValue")
}

pub fn tag(s: &str) -> generic_pipeline::node::Tag {
    Tag::test_tag(s)
}
