use accesskit::{Node as AxNode, NodeId as AxNodeId, Tree as AxTree};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[derive(Debug)]
pub struct AccessibilityTree {
    pub ax_nodes: FxHashMap<AxNodeId, AxNode>,
    pub ax_tree: AxTree,
}

impl AccessibilityTree {
    pub fn descendants(&self) -> AxDescendants<'_> {
        AxDescendants(self, vec![self.ax_tree.root])
    }
}

pub struct AxDescendants<'tree>(&'tree AccessibilityTree, Vec<AxNodeId>);
impl<'tree> Iterator for AxDescendants<'tree> {
    type Item = (AxNodeId, &'tree AxNode);
    fn next(&mut self) -> Option<Self::Item> {
        let Some(result_id) = self.1.pop() else {
            return None;
        };
        let result_node = self.0.ax_nodes.get(&result_id).unwrap();
        for child_id in result_node.children().iter().rev() {
            self.1.push(*child_id);
        }
        Some((result_id, result_node))
    }
}