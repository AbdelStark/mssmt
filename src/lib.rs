//! # mssmt: Merkle-Sum Sparse Merkle Tree in Rust
//!
//! A Rust implementation of a Merkle-Sum Sparse Merkle Tree (MS-SMT).
//!
//! A Merkle-Sum Sparse Merkle Tree (MS-SMT) is a data structure that combines the features of a Merkle tree and a sum tree,
//! allowing for efficient proofs of inclusion and accumulation of values. It's particularly useful for securely storing large
//! amounts of data with the ability to verify the integrity and sum of the data efficiently.
//!
//! ## Features
//!
//! - **Efficient Storage**: Store and retrieve key-value pairs with associated sums efficiently.
//! - **Merkle Proofs**: Generate and verify Merkle proofs for inclusion and sums without accessing the entire tree.
//! - **Customizable Storage Backend**: Default in-memory store provided, with the ability to implement custom storage backends.
//! - **Easy-to-use API**: Simple and intuitive API for common tree operations like insert, get, delete, and proof generation.
//!
//! ## Example
//!
//! ```rust
//! use mssmt::{DefaultStore, FullTree, LeafNode};
//! use mssmt::hash_utils::to_array;
//! use sha2::{Digest, Sha256};
//!
//! fn main() -> anyhow::Result<()> {
//!     // Initialize a new FullTree with DefaultStore
//!     let store = DefaultStore::new();
//!     let mut tree = FullTree::new(store);
//!
//!     // Generate a key by hashing a string
//!     let key = to_array(&Sha256::digest(b"key1"));
//!     let value = b"value1".to_vec();
//!     let sum = 10;
//!
//!     // Insert the key-value-sum into the tree
//!     tree.insert(key, value.clone(), sum)?;
//!
//!     // Retrieve value and sum for the key
//!     if let Some((retrieved_value, retrieved_sum)) = tree.get(key)? {
//!         println!("Retrieved value: {:?}", retrieved_value);
//!         println!("Retrieved sum: {}", retrieved_sum);
//!     } else {
//!         println!("Key not found");
//!     }
//!
//!     // Generate a Merkle proof for the key
//!     let proof = tree.merkle_proof(key)?;
//!
//!     // Verify the proof
//!     let root_node = tree.root()?;
//!     let root_hash = root_node.node_hash();
//!     let leaf_node = LeafNode::new(key, value, sum);
//!     let is_valid = proof.verify(key, &leaf_node, root_hash);
//!     println!("Proof verification result: {}", is_valid);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - [`hash_utils`]: Utility functions for hashing.
//! - [`node`]: Node definitions and implementations.
//! - [`proof`]: Merkle proof structures and verification.
//! - [`store`]: Storage interfaces and default implementations.
//! - [`tree`]: The main MS-SMT tree implementation.
//!
//! ## Crate Exports
//!
//! The most important types and traits are re-exported at the crate root for convenience:
//!
//! - [`FullTree`]: The main tree structure.
//! - [`DefaultStore`]: The default in-memory storage backend.
//! - [`LeafNode`], [`BranchNode`]: Node types in the tree.
//! - [`Proof`]: Merkle proof structure.
//!
//! ## License
//!
//! This project is licensed under the MIT License.
//!
//! [`hash_utils`]: crate::hash_utils
//! [`node`]: crate::node
//! [`proof`]: crate::proof
//! [`store`]: crate::store
//! [`tree`]: crate::tree
//! [`FullTree`]: crate::tree::FullTree
//! [`DefaultStore`]: crate::store::DefaultStore
//! [`LeafNode`]: crate::node::LeafNode
//! [`BranchNode`]: crate::node::BranchNode
//! [`Proof`]: crate::proof::Proof

pub mod hash_utils;
pub mod node;
pub mod proof;
pub mod store;
pub mod tree;

pub use crate::node::{BranchNode, LeafNode, Node, NodeHash};
pub use crate::proof::Proof;
pub use crate::store::{DefaultStore, TreeStore};
pub use crate::tree::FullTree;
