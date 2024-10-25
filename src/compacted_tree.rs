use crate::node::Node;
use crate::store::TreeStore;
use anyhow::Result;
use std::sync::Arc;

/// Represents a compacted MS-SMT.
pub struct CompactedTree<S: TreeStore> {
    store: S,
}

impl<S: TreeStore> CompactedTree<S> {
    /// Creates a new `CompactedTree` with the given store.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Returns the root node of the MS-SMT.
    pub fn root(&self) -> Result<Arc<dyn Node>> {
        self.store.root_node()
    }

    // Implement other methods like insert, delete, get, merkle_proof, etc.
}
