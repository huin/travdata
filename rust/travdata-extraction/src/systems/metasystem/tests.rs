use anyhow::Result;
use googletest::prelude::*;
use mockall::mock;

use super::*;
use crate::{
    intermediates,
    node::{self, core_type},
    processargs, processparams, systems,
    testutil::*,
};

mock! {
    pub System {}

    impl systems::System for System {
        fn params(&self, node: &node::Node) -> Option<processparams::NodeParams>;

        fn inputs(&self, node: &node::Node) -> Vec<core_type::NodeId>;

        fn process(
            &self,
            node: &node::Node,
            args: &processargs::ArgSet,
            intermediates: &intermediates::IntermediateSet,
        ) -> Result<intermediates::Intermediate>;

        fn process_multiple<'a>(
            &self,
            nodes: &'a [&'a node::Node],
            args: &processargs::ArgSet,
            intermediates: &intermediates::IntermediateSet,
        ) -> Vec<(core_type::NodeId, Result<intermediates::Intermediate>)>;
    }
}

#[gtest]
fn test_params() {
    let mut pdf_sys = MockSystem::new();
    let mut json_sys = MockSystem::new();

    // GIVEN: an InputPdfFile node.
    let pdf_node = Rc::new(default_node(|_| {}, default_spec_input_pdf_file(|_| {})));

    // GIVEN: an OutputFileJson node.
    let json_node = Rc::new(default_node(|_| {}, default_spec_output_file_json(|_| {})));

    // GIVEN: the pdf_sys will return the given parameters.
    let new_expected_pdf_params = || {
        Some(processparams::NodeParams {
            params: vec![processparams::Param {
                param_id: processparams::ParamId("pdf-in"),
                description: "pdf-in description.".into(),
                param_type: processparams::ParamType::InputPdf,
            }],
        })
    };
    pdf_sys
        .expect_params()
        .withf_st({
            let pdf_node = pdf_node.clone();
            move |node| node == pdf_node.as_ref()
        })
        .return_once_st(move |_| new_expected_pdf_params());

    // GIVEN: the json_sys will return the given parameters.
    let new_expected_json_params = || {
        Some(processparams::NodeParams {
            params: vec![processparams::Param {
                param_id: processparams::ParamId("json-out"),
                description: "json-out description.".into(),
                param_type: processparams::ParamType::OutputDirectory,
            }],
        })
    };
    json_sys
        .expect_params()
        .withf_st({
            let json_node = json_node.clone();
            move |node| node == json_node.as_ref()
        })
        .return_once_st(move |_| new_expected_json_params());

    // GIVEN: a meta_system that dispatches for InputPdfFile and OutputFileJson.
    let pdf_sys = Rc::new(pdf_sys);
    let json_sys = Rc::new(json_sys);
    let mut systems = hashbrown::HashMap::<spec::SpecDiscriminants, Rc<dyn systems::System>>::new();
    systems.insert(spec::SpecDiscriminants::InputPdfFile, pdf_sys.clone());
    systems.insert(spec::SpecDiscriminants::OutputFileJson, json_sys.clone());
    let meta_system = MetaSystem::new(systems);

    // WHEN: the params are requested for the InputPdfFile node.
    let got_pdf_params = meta_system.params(&pdf_node);

    // THEN: the single param should be for an input PDF file path.
    let expected_pdf_params = new_expected_pdf_params();
    expect_that!(got_pdf_params, eq(&expected_pdf_params));

    // WHEN: the params are requested for the InputPdfFile node.
    let got_json_params = meta_system.params(&json_node);

    // THEN: the single param should be for an input json file path.
    let expected_json_params = new_expected_json_params();
    expect_that!(got_json_params, eq(&expected_json_params));
}
