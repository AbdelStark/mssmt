//! Merkle proof structures and verification for the Merkle-Sum Sparse Merkle Tree.
//!
//! This module provides the `Proof` struct, which contains the necessary information to verify the inclusion
//! of a leaf in the tree. It includes methods to compute the root hash from the proof and verify the proof
//! against a given root hash.

use crate::node::{bit_index, BranchNode, LeafNode, Node, NodeHash, MAX_TREE_LEVELS};
use std::sync::Arc;

/// A Merkle proof for verifying the inclusion of a leaf in the Merkle-Sum Sparse Merkle Tree.
///
/// The `Proof` struct contains a vector of sibling nodes needed to reconstruct the root hash from a given leaf node.
/// It provides methods to compute the root and verify the proof.
///
/// # Fields
///
/// - `nodes`: A vector of `Arc<dyn Node>` representing the sibling nodes along the path from the leaf to the root.
///
/// # Examples
///
/// ```rust
/// use mssmt::{DefaultStore, FullTree, LeafNode};
/// use mssmt::hash_utils::to_array;
/// use sha2::{Digest, Sha256};
///
/// // Initialize the tree and insert data
/// let store = DefaultStore::new();
/// let mut tree = FullTree::new(store);
/// let key = to_array(&Sha256::digest(b"key1"));
/// let value = b"value1".to_vec();
/// let sum = 10;
/// tree.insert(key, value.clone(), sum).unwrap();
///
/// // Generate a proof
/// let proof = tree.merkle_proof(key).unwrap();
///
/// // Retrieve the root hash
/// let root_hash = tree.root().unwrap().node_hash();
///
/// // Verify the proof
/// let leaf_node = LeafNode::new(key, value, sum);
/// assert!(proof.verify(key, &leaf_node, root_hash));
/// ```

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
        let total_height = MAX_TREE_LEVELS;

        // Reverse the proof nodes to start from the leaf level
        for (height_from_leaf, sibling_node) in self.nodes.iter().rev().enumerate() {
            let height = total_height - height_from_leaf - 1;
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
    ///
    /// Reconstructs the root hash using the proof and the provided leaf node, then compares it to the given root hash.
    ///
    /// # Arguments
    ///
    /// - `key`: The key associated with the leaf node.
    /// - `leaf`: A reference to the `LeafNode` to verify.
    /// - `root_hash`: The expected root hash of the tree.
    ///
    /// # Returns
    ///
    /// - `true` if the proof is valid and the reconstructed root hash matches the given root hash.
    /// - `false` otherwise.
    ///
    pub fn verify(&self, key: [u8; 32], leaf: &LeafNode, root_hash: NodeHash) -> bool {
        let computed_root = self.root(key, leaf);
        computed_root.node_hash() == root_hash
    }
}
