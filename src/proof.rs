use crate::node::{bit_index, BranchNode, LeafNode, Node, NodeHash};
use std::sync::Arc;

/// Represents a merkle proof for a MS-SMT.
pub struct Proof {
    pub nodes: Vec<Arc<dyn Node>>,
}

impl Proof {
    /// Creates a new `Proof`.
    pub fn new(nodes: Vec<Arc<dyn Node>>) -> Self {
        Self { nodes }
    }

    /// Computes the root from the proof and the given leaf.
    pub fn root(&self, key: [u8; 32], leaf: &LeafNode) -> Arc<dyn Node> {
        let mut current_node: Arc<dyn Node> = Arc::new(leaf.clone());

        for (height, sibling_node) in self.nodes.iter().enumerate() {
            let bit = bit_index(height, &key);
            let parent_node = if bit == 0 {
                Arc::new(BranchNode::new(current_node.clone(), sibling_node.clone()))
            } else {
                Arc::new(BranchNode::new(sibling_node.clone(), current_node.clone()))
            };
            current_node = parent_node;
        }

        current_node
    }

    /// Verifies the proof against a given root hash.
    pub fn verify(&self, key: [u8; 32], leaf: &LeafNode, root_hash: NodeHash) -> bool {
        let computed_root = self.root(key, leaf);
        computed_root.node_hash() == root_hash
    }
}
