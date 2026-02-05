use crate::node;

pub fn node_id(s: &str) -> node::NodeId {
    node::NodeId::test_node_id(s)
}
