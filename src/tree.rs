use crate::node::Node;
use crate::store::TreeStore;
use anyhow::Result;
use std::sync::Arc;

/// Represents a Merkle-Sum Sparse Merkle Tree (MS-SMT).
pub struct FullTree<S: TreeStore> {
    store: S,
}

impl<S: TreeStore> FullTree<S> {
    /// Creates a new `FullTree` with the given store.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Returns the root node of the MS-SMT.
    pub fn root(&self) -> Result<Arc<dyn Node>> {
        self.store.root_node()
    }
}
