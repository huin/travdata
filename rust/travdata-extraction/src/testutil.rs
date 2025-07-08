use std::{ffi::OsStr, path::Path};

use crate::node::{self, core_type, spec};

pub fn node_id(s: &str) -> node::NodeId {
    s.to_string().try_into().expect("expected valid Id value")
}

pub fn output_path_buf<S: AsRef<OsStr> + ?Sized>(s: &S) -> core_type::OutputPathBuf {
    Path::new(s)
        .to_owned()
        .try_into()
        .expect("expected valid OutputPathBufValue")
}

pub fn tag(s: &str) -> core_type::Tag {
    s.to_string().try_into().expect("expected valid Tag value")
}

pub fn default_node<F>(f: F, spec: spec::Spec) -> node::Node
where
    F: FnOnce(&mut node::Node),
{
    let mut n = node::Node {
        id: node_id("foo"),
        tags: Default::default(),
        public: false,
        spec,
    };
    f(&mut n);
    n
}

pub fn default_spec_input_pdf_file<F>(f: F) -> spec::Spec
where
    F: FnOnce(&mut spec::input_pdf_file::InputPdfFile),
{
    let mut s = spec::input_pdf_file::InputPdfFile;
    f(&mut s);
    spec::Spec::InputPdfFile(s)
}

pub fn default_spec_output_file_json<F>(f: F) -> spec::Spec
where
    F: FnOnce(&mut spec::output_file_json::OutputFileJson),
{
    let mut s = spec::output_file_json::OutputFileJson {
        input_data: node_id("input-data"),
        filename: output_path_buf("output.json"),
    };
    f(&mut s);
    spec::Spec::OutputFileJson(s)
}
