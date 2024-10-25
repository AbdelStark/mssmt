use crate::node::{BranchNode, LeafNode, Node, NodeHash};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Represents a generic database interface for the MS-SMT.
pub trait TreeStore {
    /// Returns the root node of the tree.
    fn root_node(&self) -> Result<Arc<dyn Node>>;

    // Other methods...
}

/// Default in-memory implementation of `TreeStore`.
#[derive(Default)]
pub struct DefaultStore {
    pub branches: HashMap<NodeHash, Arc<BranchNode>>,
    pub leaves: HashMap<NodeHash, Arc<LeafNode>>,
    pub root: Option<Arc<dyn Node>>,
}

impl DefaultStore {
    /// Creates a new `DefaultStore`.
    pub fn new() -> Self {
        Self {
            branches: HashMap::new(),
            leaves: HashMap::new(),
            root: None,
        }
    }
}

impl TreeStore for DefaultStore {
    fn root_node(&self) -> Result<Arc<dyn Node>> {
        if let Some(root) = &self.root {
            Ok(root.clone())
        } else {
            // Return empty tree root
            Ok(crate::node::EMPTY_TREE[0].clone())
        }
    }

    // Implement other methods...
}

/// Represents a view-only transaction.
pub trait TreeStoreViewTx {
    fn get_children(&self, height: usize, key: NodeHash) -> Result<(Arc<dyn Node>, Arc<dyn Node>)>;
    fn root_node(&self) -> Result<Arc<dyn Node>>;
}

/// Represents an update transaction.
pub trait TreeStoreUpdateTx: TreeStoreViewTx {
    fn update_root(&mut self, root: Arc<dyn Node>) -> Result<()>;
    fn insert_branch(&mut self, branch: Arc<BranchNode>) -> Result<()>;
    fn insert_leaf(&mut self, leaf: Arc<LeafNode>) -> Result<()>;
    fn delete_branch(&mut self, key: NodeHash) -> Result<()>;
    fn delete_leaf(&mut self, key: NodeHash) -> Result<()>;
}
