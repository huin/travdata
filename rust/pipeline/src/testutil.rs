use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};

use anyhow::Result;
use generic_pipeline::node::Tag;
use googletest::{matcher::Matcher, prelude::*};
use hashbrown::{HashMap, HashSet};

use crate::{Node, NodeId, intermediates, tabula_wrapper};

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

pub struct NodeExpected<'a> {
    pub node: Node,
    pub expected: MatcherBox<&'a Result<intermediates::IntermediateValue>>,
}

/// Boxed version of a [Matcher].
#[derive(MatcherBase)]
pub struct MatcherBox<A>
where
    A: std::fmt::Debug + Copy,
{
    matcher: Box<dyn Matcher<A>>,
}

impl<A> MatcherBox<A>
where
    A: std::fmt::Debug + Copy,
{
    pub fn new<T>(matcher: T) -> Self
    where
        T: Matcher<A> + 'static,
        A: std::fmt::Debug + Copy,
    {
        Self {
            matcher: Box::new(matcher),
        }
    }
}

impl<A> Matcher<A> for MatcherBox<A>
where
    A: std::fmt::Debug + Copy,
{
    fn matches(&self, actual: A) -> googletest::matcher::MatcherResult {
        self.matcher.matches(actual)
    }

    fn describe(
        &self,
        matcher_result: googletest::matcher::MatcherResult,
    ) -> googletest::description::Description {
        self.matcher.describe(matcher_result)
    }
}

impl<A> std::fmt::Debug for MatcherBox<A>
where
    A: std::fmt::Debug + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<matcher>")
    }
}

#[track_caller]
pub fn check_results<'a, 'm>(
    actual_results_map: &'a HashMap<NodeId, Result<intermediates::IntermediateValue>>,
    node_expecteds: Vec<NodeExpected<'a>>,
) where
    'a: 'm,
{
    let actual_node_ids: HashSet<NodeId> = actual_results_map.keys().cloned().collect();
    let expected_node_ids: HashSet<NodeId> = node_expecteds
        .iter()
        .map(|node_expected| node_expected.node.id.clone())
        .collect();
    expect_that!(actual_node_ids, eq(&expected_node_ids));

    for node_expected in node_expecteds {
        match actual_results_map.get(&node_expected.node.id) {
            Some(actual_result) => {
                expect_that!(
                    actual_result,
                    node_expected.expected,
                    "for node_id {:?}",
                    node_expected.node.id,
                );
            }
            None => {
                // Failure case covered by checking equality of ID set.
            }
        }
    }
}

pub struct TabulaExtractorFixture {
    pub client: tabula_wrapper::threadeddispatch::ExtractorClient,
    // Mutex around the JoinHandle to ensure UnwindSafe for the fixture, the join handle kept for
    // blocking at drop-time.
    #[expect(unused)]
    server_thread_join: std::sync::Mutex<thread::JoinHandle<()>>,
}

impl StaticFixture for TabulaExtractorFixture {
    fn set_up_once() -> googletest::Result<Self> {
        let (client_sender, client_receiver) = mpsc::sync_channel(0);

        let server_thread_join = thread::spawn(move || {
            let vm = tabula::TabulaVM::new("../target/debug/tabula.jar", true)
                .expect("could not create TabulaVM");
            let tabula_env = vm.attach().expect("could not attach to TabulaVM");
            let extractor =
                tabula_wrapper::singlethreaded::SingleThreadedTabulaExtractor::new(tabula_env);

            let server = tabula_wrapper::threadeddispatch::ExtractorServer::new(extractor);
            if let Err(e) = client_sender.send(server.client()) {
                log::error!("Could not send ExtractorClient: {e:?}");
                return;
            }
            server.run();
        });

        let client = client_receiver.recv()?;
        Ok(Self {
            client,
            server_thread_join: server_thread_join.into(),
        })
    }
}

pub struct TestDirectory {
    dir: tempfile::TempDir,
}

impl TestDirectory {
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

impl ConsumableFixture for TestDirectory {
    fn set_up() -> googletest::Result<Self> {
        let dir = tempfile::TempDir::with_prefix("travdata-pipeline-test")?;
        Ok(Self { dir })
    }
}
