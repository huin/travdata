use std::{ffi::OsStr, path::Path};

use generic_pipeline::node::Tag;

use crate::NodeId;

pub fn node_id(s: &str) -> crate::NodeId {
    NodeId::test_node_id(s)
}

pub fn output_path_buf<S: AsRef<OsStr> + ?Sized>(s: &S) -> crate::spec_types::OutputPathBuf {
    Path::new(s)
        .to_owned()
        .try_into()
        .expect("expected valid OutputPathBufValue")
}

pub fn tag(s: &str) -> generic_pipeline::node::Tag {
    Tag::test_tag(s)
}
