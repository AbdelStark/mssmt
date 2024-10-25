use anyhow::Result;
use mssmt::hash_utils::to_array;
use mssmt::{DefaultStore, FullTree, LeafNode};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
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
    } else {
        println!("key1 not found");
    }

    // Retrieve value and sum for key2
    if let Some((value, sum)) = tree.get(key2)? {
        println!("Retrieved key2 with value {:?} and sum {}", value, sum);
    } else {
        println!("key2 not found");
    }

    // Retrieve value and sum for key3
    if let Some((value, sum)) = tree.get(key3)? {
        println!("Retrieved key3 with value {:?} and sum {}", value, sum);
    } else {
        println!("key3 not found");
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

    // Step 6: Delete keys and log steps
    println!("\nDeleting key2 from the tree...");
    tree.delete(key2)?;
    println!("Deleted key2");

    // Step 7: Check the tree after deletions
    if let Some((value, sum)) = tree.get(key2)? {
        println!("Retrieved key2 with value {:?} and sum {}", value, sum);
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

    Ok(())
}
