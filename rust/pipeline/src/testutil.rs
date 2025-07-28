use std::{ffi::OsStr, path::Path};

pub fn node_id(s: &str) -> crate::NodeId {
    s.to_string().try_into().expect("expected valid Id value")
}

pub fn output_path_buf<S: AsRef<OsStr> + ?Sized>(s: &S) -> crate::spec_types::OutputPathBuf {
    Path::new(s)
        .to_owned()
        .try_into()
        .expect("expected valid OutputPathBufValue")
}

pub fn tag(s: &str) -> generic_pipeline::node::Tag {
    s.to_string().try_into().expect("expected valid Tag value")
}
