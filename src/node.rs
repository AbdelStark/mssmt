use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use std::fmt;
use std::sync::Arc;

use crate::hash_utils::to_array;

pub const HASH_SIZE: usize = 32;
pub const MAX_TREE_LEVELS: usize = HASH_SIZE * 8;
pub const LAST_BIT_INDEX: usize = MAX_TREE_LEVELS - 1;

/// Represents the hash of a node in the MS-SMT.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHash([u8; HASH_SIZE]);

impl NodeHash {
    /// Creates a new `NodeHash` from a byte array.
    pub fn new(bytes: [u8; HASH_SIZE]) -> Self {
        NodeHash(bytes)
    }

    /// Returns a `NodeHash` with all zeros.
    pub fn zero() -> Self {
        NodeHash([0u8; HASH_SIZE])
    }
}

impl fmt::Debug for NodeHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        hex::encode(self.0).fmt(f)
    }
}

/// Represents a node in the MS-SMT.
pub trait Node: Send + Sync {
    /// Returns the hash of the node.
    fn node_hash(&self) -> NodeHash;

    /// Returns the sum of the node.
    fn node_sum(&self) -> u64;

    /// Creates a deep copy of the node.
    fn copy(&self) -> Box<dyn Node>;
}

/// Checks whether two nodes are equal based on their hash and sum.
pub fn is_equal_node(a: &dyn Node, b: &dyn Node) -> bool {
    a.node_hash() == b.node_hash() && a.node_sum() == b.node_sum()
}

/// Represents a leaf node in the MS-SMT.
#[derive(Clone)]
pub struct LeafNode {
    node_hash: Option<NodeHash>,
    value: Vec<u8>,
    sum: u64,
}

impl LeafNode {
    /// Creates a new `LeafNode`.
    pub fn new(value: Vec<u8>, sum: u64) -> Self {
        Self {
            node_hash: None,
            value,
            sum,
        }
    }

    /// Checks if the leaf node is empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty() && self.sum == 0
    }
}

impl Node for LeafNode {
    fn node_hash(&self) -> NodeHash {
        if let Some(node_hash) = self.node_hash {
            return node_hash;
        }

        let mut hasher = Sha256::new();
        hasher.update(&self.value);
        hasher.update(self.sum.to_be_bytes());
        let hash = hasher.finalize();

        NodeHash::new(to_array(&hash))
    }

    fn node_sum(&self) -> u64 {
        self.sum
    }

    fn copy(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}

/// Represents an empty leaf node.
pub static EMPTY_LEAF_NODE: Lazy<LeafNode> = Lazy::new(|| LeafNode::new(Vec::new(), 0));

/// Represents a branch node in the MS-SMT.
#[derive(Clone)]
pub struct BranchNode {
    node_hash: Option<NodeHash>,
    sum: Option<u64>,
    pub left: Arc<dyn Node>,
    pub right: Arc<dyn Node>,
}

impl BranchNode {
    /// Creates a new `BranchNode`.
    pub fn new(left: Arc<dyn Node>, right: Arc<dyn Node>) -> Self {
        Self {
            node_hash: None,
            sum: None,
            left,
            right,
        }
    }
}

impl Node for BranchNode {
    fn node_hash(&self) -> NodeHash {
        if let Some(node_hash) = self.node_hash {
            return node_hash;
        }

        let left_hash = self.left.node_hash();
        let right_hash = self.right.node_hash();

        let mut hasher = Sha256::new();
        hasher.update(left_hash.0);
        hasher.update(right_hash.0);
        hasher.update(self.node_sum().to_be_bytes());
        let hash = hasher.finalize();

        NodeHash::new(to_array(&hash))
    }

    fn node_sum(&self) -> u64 {
        if let Some(sum) = self.sum {
            return sum;
        }

        self.left.node_sum() + self.right.node_sum()
    }

    fn copy(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}

/// Represents a compacted leaf node.
#[derive(Clone)]
pub struct CompactedLeafNode {
    leaf_node: LeafNode,
    key: [u8; HASH_SIZE],
    compacted_node_hash: NodeHash,
}

impl CompactedLeafNode {
    /// Creates a new `CompactedLeafNode`.
    pub fn new(height: usize, key: [u8; HASH_SIZE], leaf: LeafNode) -> Self {
        let mut current: Arc<dyn Node> = Arc::new(leaf.clone());
        for i in (height..MAX_TREE_LEVELS).rev() {
            let bit = bit_index(i as u8, &key);
            if bit == 0 {
                current = Arc::new(BranchNode::new(current, EMPTY_TREE[i + 1].clone()));
            } else {
                current = Arc::new(BranchNode::new(EMPTY_TREE[i + 1].clone(), current));
            }
        }
        let node_hash = current.node_hash();

        Self {
            leaf_node: leaf,
            key,
            compacted_node_hash: node_hash,
        }
    }

    /// Extracts the subtree represented by this compacted leaf.
    pub fn extract(&self, height: usize) -> Arc<dyn Node> {
        let mut current: Arc<dyn Node> = Arc::new(self.leaf_node.clone());
        for j in ((height + 1)..=MAX_TREE_LEVELS).rev() {
            let bit = bit_index((j - 1) as u8, &self.key);
            if bit == 0 {
                current = Arc::new(BranchNode::new(current, EMPTY_TREE[j].clone()));
            } else {
                current = Arc::new(BranchNode::new(EMPTY_TREE[j].clone(), current));
            }
        }
        current
    }
}

impl Node for CompactedLeafNode {
    fn node_hash(&self) -> NodeHash {
        self.compacted_node_hash
    }

    fn node_sum(&self) -> u64 {
        self.leaf_node.node_sum()
    }

    fn copy(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}

/// Represents a precomputed node.
#[derive(Clone)]
pub struct ComputedNode {
    hash: NodeHash,
    sum: u64,
}

impl ComputedNode {
    /// Creates a new `ComputedNode`.
    pub fn new(hash: NodeHash, sum: u64) -> Self {
        Self { hash, sum }
    }
}

impl Node for ComputedNode {
    fn node_hash(&self) -> NodeHash {
        self.hash
    }

    fn node_sum(&self) -> u64 {
        self.sum
    }

    fn copy(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}

/// Initializes the empty tree nodes.
pub static EMPTY_TREE: Lazy<Vec<Arc<dyn Node>>> = Lazy::new(|| {
    let mut empty_tree: Vec<Arc<dyn Node>> = Vec::with_capacity(MAX_TREE_LEVELS + 1);
    empty_tree.resize_with(MAX_TREE_LEVELS + 1, || Arc::new(EMPTY_LEAF_NODE.clone()));

    empty_tree[MAX_TREE_LEVELS] = Arc::new(EMPTY_LEAF_NODE.clone());

    for i in (0..=LAST_BIT_INDEX).rev() {
        let branch = BranchNode::new(empty_tree[i + 1].clone(), empty_tree[i + 1].clone());
        empty_tree[i] = Arc::new(branch);
    }

    empty_tree
});

/// Returns the bit at a given index in the key.
pub fn bit_index(idx: u8, key: &[u8; HASH_SIZE]) -> u8 {
    let byte_val = key[(idx / 8) as usize];
    (byte_val >> (idx % 8)) & 1
}
