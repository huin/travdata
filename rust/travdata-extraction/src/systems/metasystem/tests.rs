use googletest::prelude::*;

use super::*;
use crate::{processparams, testutil::*};

#[gtest]
fn test_params() {
    let mut foo_sys = MockFakeSystem::new();
    let mut bar_sys = MockFakeSystem::new();

    // GIVEN: a Foo node.
    let foo_node = Rc::new(FakeNode::default_with_spec(FakeSpec::default_foo()));

    // GIVEN: a Bar node.
    let bar_node = Rc::new(FakeNode::default_with_spec(FakeSpec::default_bar()));

    // GIVEN: the foo_sys will return the given parameters.
    let new_expected_foo_params = || {
        Some(processparams::NodeParams {
            params: vec![processparams::Param {
                param_id: processparams::ParamId("foo-param"),
                description: "foo-param description.".into(),
                param_type: processparams::ParamType::InputPdf,
            }],
        })
    };
    foo_sys
        .expect_params()
        .withf_st({
            let foo_node = foo_node.clone();
            move |node| node == foo_node.as_ref()
        })
        .return_once_st(move |_| new_expected_foo_params());

    // GIVEN: the bar_sys will return the given parameters.
    let new_expected_bar_params = || {
        Some(processparams::NodeParams {
            params: vec![processparams::Param {
                param_id: processparams::ParamId("bar-param"),
                description: "bar-param description.".into(),
                param_type: processparams::ParamType::OutputDirectory,
            }],
        })
    };
    bar_sys
        .expect_params()
        .withf_st({
            let bar_node = bar_node.clone();
            move |node| node == bar_node.as_ref()
        })
        .return_once_st(move |_| new_expected_bar_params());

    // GIVEN: a meta_system that dispatches for Foo and Bar.
    let foo_sys = Rc::new(foo_sys);
    let bar_sys = Rc::new(bar_sys);
    let mut systems =
        hashbrown::HashMap::<FakeSpecDiscriminants, Rc<dyn GenericSystem<FakeSpec>>>::new();
    systems.insert(FakeSpecDiscriminants::Foo, foo_sys.clone());
    systems.insert(FakeSpecDiscriminants::Bar, bar_sys.clone());
    let meta_system = GenericMetaSystem::new(systems);

    // WHEN: the params are requested for the Foo node.
    let got_foo_params = meta_system.params(&foo_node);

    // THEN: the single param should be for an input PDF file path.
    let expected_foo_params = new_expected_foo_params();
    expect_that!(got_foo_params, eq(&expected_foo_params));

    // WHEN: the params are requested for the Bar node.
    let got_bar_params = meta_system.params(&bar_node);

    // THEN: the single param should be for an output JSON file path.
    let expected_bar_params = new_expected_bar_params();
    expect_that!(got_bar_params, eq(&expected_bar_params));
}
