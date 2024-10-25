use crate::node::{bit_index, BranchNode, LeafNode, Node, EMPTY_LEAF_NODE, MAX_TREE_LEVELS};
use crate::proof::Proof;
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

    /// Inserts a key-value pair into the tree.
    pub fn insert(&mut self, key: [u8; 32], value: Vec<u8>, sum: u64) -> Result<()> {
        let leaf_node = Arc::new(LeafNode::new(key, value, sum));

        let root = self.store.root_node()?;
        let new_root = self.insert_at_node(root, 0, &key, leaf_node.clone())?;
        self.store.update_root(new_root)?;

        Ok(())
    }
    fn insert_at_node(
        &mut self,
        node: Arc<dyn Node>,
        height: usize,
        key: &[u8; 32],
        leaf_node: Arc<LeafNode>,
    ) -> Result<Arc<dyn Node>> {
        if height == MAX_TREE_LEVELS {
            self.store.insert_leaf(leaf_node.clone())?;
            return Ok(leaf_node);
        }

        let bit = bit_index(height, key);

        if let Some(branch_node) = node.as_any().downcast_ref::<BranchNode>() {
            let left = branch_node.left.clone();
            let right = branch_node.right.clone();

            let new_left;
            let new_right;

            if bit == 0 {
                new_left = self.insert_at_node(left, height + 1, key, leaf_node)?;
                new_right = right;
            } else {
                new_left = left;
                new_right = self.insert_at_node(right, height + 1, key, leaf_node)?;
            }

            let new_branch = Arc::new(BranchNode::new(new_left, new_right));
            self.store.insert_branch(new_branch.clone())?;
            Ok(new_branch)
        } else if let Some(leaf_node_existing_ref) = node.as_any().downcast_ref::<LeafNode>() {
            let leaf_node_existing = leaf_node_existing_ref.clone();

            if leaf_node_existing.key == *key {
                // Replace the existing leaf node
                self.store.insert_leaf(leaf_node.clone())?;
                Ok(leaf_node)
            } else {
                // Need to split and create a branch
                let existing_key = leaf_node_existing.key;
                let mut current_height = height;

                let mut current_node = node;
                let new_leaf_node = leaf_node;

                loop {
                    if current_height >= MAX_TREE_LEVELS {
                        break;
                    }

                    let existing_bit = bit_index(current_height, &existing_key);
                    let new_bit = bit_index(current_height, key);

                    if existing_bit != new_bit {
                        let left_node;
                        let right_node;

                        if new_bit == 0 {
                            left_node = self.insert_at_node(
                                Arc::new(EMPTY_LEAF_NODE.clone()),
                                current_height + 1,
                                key,
                                new_leaf_node.clone(),
                            )?;
                            right_node = self.insert_at_node(
                                Arc::new(EMPTY_LEAF_NODE.clone()),
                                current_height + 1,
                                &existing_key,
                                Arc::new(leaf_node_existing.clone()),
                            )?;
                        } else {
                            left_node = self.insert_at_node(
                                Arc::new(EMPTY_LEAF_NODE.clone()),
                                current_height + 1,
                                &existing_key,
                                Arc::new(leaf_node_existing.clone()),
                            )?;
                            right_node = self.insert_at_node(
                                Arc::new(EMPTY_LEAF_NODE.clone()),
                                current_height + 1,
                                key,
                                new_leaf_node.clone(),
                            )?;
                        }

                        let new_branch = Arc::new(BranchNode::new(left_node, right_node));
                        self.store.insert_branch(new_branch.clone())?;
                        return Ok(new_branch);
                    } else {
                        current_height += 1;
                        current_node = Arc::new(BranchNode::new(
                            Arc::new(EMPTY_LEAF_NODE.clone()),
                            Arc::new(EMPTY_LEAF_NODE.clone()),
                        ));
                    }
                }

                Ok(current_node)
            }
        } else {
            Ok(leaf_node)
        }
    }

    /// Retrieves a value and sum associated with a key.
    pub fn get(&self, key: [u8; 32]) -> Result<Option<(Vec<u8>, u64)>> {
        let node = self.store.root_node()?;
        self.get_at_node(node, 0, &key)
    }

    fn get_at_node(
        &self,
        node: Arc<dyn Node>,
        height: usize,
        key: &[u8; 32],
    ) -> Result<Option<(Vec<u8>, u64)>> {
        if height == MAX_TREE_LEVELS {
            if let Some(leaf_node) = node.as_any().downcast_ref::<LeafNode>() {
                if leaf_node.key == *key {
                    return Ok(Some((leaf_node.value.clone(), leaf_node.sum)));
                }
            }
            return Ok(None);
        }

        let bit = bit_index(height, key);

        if let Some(branch_node) = node.as_any().downcast_ref::<BranchNode>() {
            if bit == 0 {
                self.get_at_node(branch_node.left.clone(), height + 1, key)
            } else {
                self.get_at_node(branch_node.right.clone(), height + 1, key)
            }
        } else {
            Ok(None)
        }
    }

    /// Deletes a key from the tree.
    pub fn delete(&mut self, key: [u8; 32]) -> Result<()> {
        let root = self.store.root_node()?;
        let new_root = self.delete_at_node(root, 0, &key)?;
        self.store.update_root(new_root)?;

        Ok(())
    }

    fn delete_at_node(
        &mut self,
        node: Arc<dyn Node>,
        height: usize,
        key: &[u8; 32],
    ) -> Result<Arc<dyn Node>> {
        if height == MAX_TREE_LEVELS {
            if let Some(leaf_node) = node.as_any().downcast_ref::<LeafNode>() {
                if leaf_node.key == *key {
                    self.store.delete_leaf(&leaf_node.node_hash())?;
                    return Ok(Arc::new(EMPTY_LEAF_NODE.clone()));
                }
            }
            return Ok(node);
        }

        let bit = bit_index(height, key);

        if let Some(branch_node) = node.as_any().downcast_ref::<BranchNode>() {
            let new_left;
            let new_right;

            if bit == 0 {
                new_left = self.delete_at_node(branch_node.left.clone(), height + 1, key)?;
                new_right = branch_node.right.clone();
            } else {
                new_left = branch_node.left.clone();
                new_right = self.delete_at_node(branch_node.right.clone(), height + 1, key)?;
            }

            let new_branch = Arc::new(BranchNode::new(new_left.clone(), new_right.clone()));
            self.store.insert_branch(new_branch.clone())?;

            // If both children are empty, return empty node
            if new_left.node_hash() == EMPTY_LEAF_NODE.node_hash()
                && new_right.node_hash() == EMPTY_LEAF_NODE.node_hash()
            {
                Ok(Arc::new(EMPTY_LEAF_NODE.clone()))
            } else {
                Ok(new_branch)
            }
        } else {
            Ok(node)
        }
    }

    /// Generates a Merkle proof for a key.
    pub fn merkle_proof(&self, key: [u8; 32]) -> Result<Proof> {
        let node = self.store.root_node()?;
        let mut proof_nodes = Vec::new();
        self.generate_proof(node, 0, &key, &mut proof_nodes)?;
        Ok(Proof::new(proof_nodes))
    }

    fn generate_proof(
        &self,
        node: Arc<dyn Node>,
        height: usize,
        key: &[u8; 32],
        proof_nodes: &mut Vec<Arc<dyn Node>>,
    ) -> Result<()> {
        if height == MAX_TREE_LEVELS {
            return Ok(());
        }

        let bit = bit_index(height, key);

        if let Some(branch_node) = node.as_any().downcast_ref::<BranchNode>() {
            if bit == 0 {
                proof_nodes.push(branch_node.right.clone());
                self.generate_proof(branch_node.left.clone(), height + 1, key, proof_nodes)?;
            } else {
                proof_nodes.push(branch_node.left.clone());
                self.generate_proof(branch_node.right.clone(), height + 1, key, proof_nodes)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash_utils::to_array;
    use crate::store::DefaultStore;
    use anyhow::Result;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_full_tree_operations() -> Result<()> {
        // Step 1: Initialize a new FullTree with DefaultStore
        println!("Initializing a new Merkle-Sum Sparse Merkle Tree...");
        let store = DefaultStore::new();
        let mut tree = FullTree::new(store);

        // Step 2: Insert some key-value pairs into the tree
        println!("\nInserting key-value pairs into the tree...");

        // Generate keys by hashing strings
        let key1 = to_array(&Sha256::digest(b"key1"));
        let value1 = b"value1".to_vec();
        let sum1 = 10;

        // Insert the key-value-sum into the tree
        tree.insert(key1, value1.clone(), sum1)?;
        println!("Inserted key1 with value1 and sum1");

        let key2 = to_array(&Sha256::digest(b"key2"));
        let value2 = b"value2".to_vec();
        let sum2 = 20;

        tree.insert(key2, value2.clone(), sum2)?;
        println!("Inserted key2 with value2 and sum2");

        let key3 = to_array(&Sha256::digest(b"key3"));
        let value3 = b"value3".to_vec();
        let sum3 = 30;

        tree.insert(key3, value3.clone(), sum3)?;
        println!("Inserted key3 with value3 and sum3");

        // Step 3: Fetch values and log the results
        println!("\nFetching values from the tree...");

        // Retrieve value and sum for key1
        if let Some((value, sum)) = tree.get(key1)? {
            println!("Retrieved key1 with value {:?} and sum {}", value, sum);
            assert_eq!(value, value1);
            assert_eq!(sum, sum1);
        } else {
            println!("key1 not found");
            panic!("key1 should be found");
        }

        // Retrieve value and sum for key2
        if let Some((value, sum)) = tree.get(key2)? {
            println!("Retrieved key2 with value {:?} and sum {}", value, sum);
            assert_eq!(value, value2);
            assert_eq!(sum, sum2);
        } else {
            println!("key2 not found");
            panic!("key2 should be found");
        }

        // Retrieve value and sum for key3
        if let Some((value, sum)) = tree.get(key3)? {
            println!("Retrieved key3 with value {:?} and sum {}", value, sum);
            assert_eq!(value, value3);
            assert_eq!(sum, sum3);
        } else {
            println!("key3 not found");
            panic!("key3 should be found");
        }

        // Step 4: Generate Merkle proofs for keys
        println!("\nGenerating Merkle proofs for keys...");

        let proof1 = tree.merkle_proof(key1)?;
        println!("Generated proof for key1");

        // Step 5: Verify proofs
        println!("\nVerifying proofs...");

        let root_node = tree.root()?;
        let root_hash = root_node.node_hash();

        // Create a leaf node for key1 to use in verification
        let leaf_node1 = LeafNode::new(key1, value1.clone(), sum1);

        // Verify the proof for key1
        let is_valid = proof1.verify(key1, &leaf_node1, root_hash);
        println!("Proof verification for key1: {}", is_valid);
        assert!(is_valid);

        // Step 6: Delete keys and log steps
        println!("\nDeleting key2 from the tree...");
        tree.delete(key2)?;
        println!("Deleted key2");

        // Step 7: Check the tree after deletions
        if let Some((_value, _sum)) = tree.get(key2)? {
            panic!("key2 should have been deleted");
        } else {
            println!("key2 not found after deletion");
        }

        // Step 8: Generate and verify proof after deletion
        let proof1_after = tree.merkle_proof(key1)?;
        println!("Generated proof for key1 after deletion");

        let root_node_after = tree.root()?;
        let root_hash_after = root_node_after.node_hash();

        let is_valid_after = proof1_after.verify(key1, &leaf_node1, root_hash_after);
        println!(
            "Proof verification for key1 after deletion: {}",
            is_valid_after
        );
        assert!(is_valid_after);

        Ok(())
    }
}
