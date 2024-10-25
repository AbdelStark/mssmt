use once_cell::sync::Lazy;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::hash_utils::to_array;

pub const HASH_SIZE: usize = 32;
pub const MAX_TREE_LEVELS: usize = HASH_SIZE * 8; // 256 for 32 bytes
pub const LAST_BIT_INDEX: usize = MAX_TREE_LEVELS - 1;

/// Represents the hash of a node in the MS-SMT.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHash(pub [u8; HASH_SIZE]);

impl NodeHash {
    /// Creates a new `NodeHash` from a byte array.
    pub fn new(bytes: [u8; HASH_SIZE]) -> Self {
        NodeHash(bytes)
    }

    /// Returns a `NodeHash` with all zeros.
    pub fn zero() -> Self {
        NodeHash([0u8; HASH_SIZE])
    }

    /// Returns the inner byte array.
    pub fn as_bytes(&self) -> &[u8; HASH_SIZE] {
        &self.0
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

    /// Returns a reference to Any, for downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// Represents a leaf node in the MS-SMT.
#[derive(Clone)]
pub struct LeafNode {
    node_hash: Arc<RwLock<Option<NodeHash>>>,
    pub key: [u8; HASH_SIZE],
    pub value: Vec<u8>,
    pub sum: u64,
}

impl LeafNode {
    /// Creates a new `LeafNode`.
    pub fn new(key: [u8; HASH_SIZE], value: Vec<u8>, sum: u64) -> Self {
        Self {
            node_hash: Arc::new(RwLock::new(None)),
            key,
            value,
            sum,
        }
    }

    /// Checks if the leaf node is empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty() && self.sum == 0
    }

    /// Returns the value of the leaf node.
    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }
}

impl Node for LeafNode {
    fn node_hash(&self) -> NodeHash {
        {
            let node_hash = self.node_hash.read();
            if let Some(node_hash) = *node_hash {
                return node_hash;
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(self.key);
        hasher.update(&self.value);
        hasher.update(self.sum.to_be_bytes());
        let hash = hasher.finalize();

        let node_hash = NodeHash::new(to_array(&hash));
        {
            let mut node_hash_lock = self.node_hash.write();
            *node_hash_lock = Some(node_hash);
        }
        node_hash
    }

    fn node_sum(&self) -> u64 {
        self.sum
    }

    fn copy(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Represents an empty leaf node.
pub static EMPTY_LEAF_NODE: Lazy<LeafNode> =
    Lazy::new(|| LeafNode::new([0u8; HASH_SIZE], Vec::new(), 0));

/// Represents a branch node in the MS-SMT.
#[derive(Clone)]
pub struct BranchNode {
    node_hash: Arc<RwLock<Option<NodeHash>>>,
    sum: Arc<RwLock<Option<u64>>>,
    pub left: Arc<dyn Node>,
    pub right: Arc<dyn Node>,
}

impl BranchNode {
    /// Creates a new `BranchNode`.
    pub fn new(left: Arc<dyn Node>, right: Arc<dyn Node>) -> Self {
        Self {
            node_hash: Arc::new(RwLock::new(None)),
            sum: Arc::new(RwLock::new(None)),
            left,
            right,
        }
    }
}

impl Node for BranchNode {
    fn node_hash(&self) -> NodeHash {
        {
            let node_hash = self.node_hash.read();
            if let Some(node_hash) = *node_hash {
                return node_hash;
            }
        }

        let left_hash = self.left.node_hash();
        let right_hash = self.right.node_hash();

        let mut hasher = Sha256::new();
        hasher.update(left_hash.0);
        hasher.update(right_hash.0);
        hasher.update(self.node_sum().to_be_bytes());
        let hash = hasher.finalize();

        let node_hash = NodeHash::new(to_array(&hash));
        {
            let mut node_hash_lock = self.node_hash.write();
            *node_hash_lock = Some(node_hash);
        }
        node_hash
    }

    fn node_sum(&self) -> u64 {
        {
            let sum = self.sum.read();
            if let Some(sum) = *sum {
                return sum;
            }
        }

        let sum = self.left.node_sum() + self.right.node_sum();
        {
            let mut sum_lock = self.sum.write();
            *sum_lock = Some(sum);
        }
        sum
    }

    fn copy(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Initializes the empty tree nodes.
pub static EMPTY_TREE: Lazy<Vec<Arc<dyn Node>>> = Lazy::new(|| {
    let mut empty_tree: Vec<Arc<dyn Node>> = Vec::with_capacity(MAX_TREE_LEVELS + 1);
    empty_tree.resize_with(MAX_TREE_LEVELS + 1, || Arc::new(EMPTY_LEAF_NODE.clone()));

    for i in (0..MAX_TREE_LEVELS).rev() {
        let branch = BranchNode::new(empty_tree[i + 1].clone(), empty_tree[i + 1].clone());
        empty_tree[i] = Arc::new(branch);
    }

    empty_tree
});

/// Returns the bit at a given index in the key.
pub fn bit_index(idx: usize, key: &[u8; HASH_SIZE]) -> u8 {
    let byte_val = key[idx / 8];
    (byte_val >> (7 - (idx % 8))) & 1
}
