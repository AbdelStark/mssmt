use crate::node::{BranchNode, LeafNode, Node, NodeHash};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Represents a generic database interface for the MS-SMT.
pub trait TreeStore {
    /// Returns the root node of the tree.
    fn root_node(&self) -> Result<Arc<dyn Node>>;

    /// Gets a branch node by its hash.
    fn get_branch(&self, key: &NodeHash) -> Result<Option<Arc<BranchNode>>>;

    /// Gets a leaf node by its hash.
    fn get_leaf(&self, key: &NodeHash) -> Result<Option<Arc<LeafNode>>>;

    /// Inserts or updates a branch node.
    fn insert_branch(&mut self, branch: Arc<BranchNode>) -> Result<()>;

    /// Inserts or updates a leaf node.
    fn insert_leaf(&mut self, leaf: Arc<LeafNode>) -> Result<()>;

    /// Deletes a branch node.
    fn delete_branch(&mut self, key: &NodeHash) -> Result<()>;

    /// Deletes a leaf node.
    fn delete_leaf(&mut self, key: &NodeHash) -> Result<()>;

    /// Updates the root node.
    fn update_root(&mut self, root: Arc<dyn Node>) -> Result<()>;
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

    fn get_branch(&self, key: &NodeHash) -> Result<Option<Arc<BranchNode>>> {
        Ok(self.branches.get(key).cloned())
    }

    fn get_leaf(&self, key: &NodeHash) -> Result<Option<Arc<LeafNode>>> {
        Ok(self.leaves.get(key).cloned())
    }

    fn insert_branch(&mut self, branch: Arc<BranchNode>) -> Result<()> {
        let key = branch.node_hash();
        self.branches.insert(key, branch);
        Ok(())
    }

    fn insert_leaf(&mut self, leaf: Arc<LeafNode>) -> Result<()> {
        let key = leaf.node_hash();
        self.leaves.insert(key, leaf);
        Ok(())
    }

    fn delete_branch(&mut self, key: &NodeHash) -> Result<()> {
        self.branches.remove(key);
        Ok(())
    }

    fn delete_leaf(&mut self, key: &NodeHash) -> Result<()> {
        self.leaves.remove(key);
        Ok(())
    }

    fn update_root(&mut self, root: Arc<dyn Node>) -> Result<()> {
        self.root = Some(root);
        Ok(())
    }
}
