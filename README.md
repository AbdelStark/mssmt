# MS-SMT: Merkle-Sum Sparse Merkle Tree in Rust

[![Crates.io](https://img.shields.io/crates/v/mssmt.svg)](https://crates.io/crates/mssmt)
[![Documentation](https://docs.rs/mssmt/badge.svg)](https://docs.rs/mssmt)
[![Build Status](https://github.com/AbdelStark/mssmt/actions/workflows/ci.yml/badge.svg)](https://github.com/AbdelStark/mssmt/actions)
[![License](https://img.shields.io/crates/l/mssmt.svg)](https://github.com/AbdelStark/mssmt/blob/main/LICENSE)

A Rust implementation of a **Merkle-Sum Sparse Merkle Tree (MS-SMT)** data structure.

A Merkle-Sum Sparse Merkle Tree (MS-SMT) is a data structure that combines the features of a Merkle tree and a sum tree, allowing for efficient proofs of inclusion and accumulation of values. It's particularly useful for securely storing large amounts of data with the ability to verify the integrity and sum of the data efficiently.

> ⚠️ This is a work in progress and the API is subject to change. It's an experimental implementation.
>
> ⚠️ Do not use in production or for anything serious.

## Features

- **Efficient Storage**: Store and retrieve key-value pairs with associated sums efficiently.
- **Merkle Proofs**: Generate and verify Merkle proofs for inclusion and sums without accessing the entire tree.
- **Customizable Storage Backend**: Default in-memory store provided, with the ability to implement custom storage backends.
- **Easy-to-use API**: Simple and intuitive API for common tree operations like insert, get, delete, and proof generation.
- **Thread-safe**: Built with concurrency in mind using thread-safe data structures.

## Installation

Add `mssmt` to your `Cargo.toml` dependencies:

```toml
[dependencies]
mssmt = "0.0.3"
```

Or install via Cargo:

```sh
cargo add mssmt
```

## Usage

Here's a basic example of how to use the mssmt crate:

```rust
use mssmt::{DefaultStore, FullTree, LeafNode};
use mssmt::hash_utils::to_array;
use sha2::{Digest, Sha256};

fn main() -> anyhow::Result<()> {
    // Initialize a new FullTree with DefaultStore
    let store = DefaultStore::new();
    let mut tree = FullTree::new(store);

    // Generate a key by hashing a string
    let key = to_array(&Sha256::digest(b"key1"));
    let value = b"value1".to_vec();
    let sum = 10;

    // Insert the key-value-sum into the tree
    tree.insert(key, value.clone(), sum)?;

    // Retrieve value and sum for the key
    if let Some((retrieved_value, retrieved_sum)) = tree.get(key)? {
        println!("Retrieved value: {:?}", retrieved_value);
        println!("Retrieved sum: {}", retrieved_sum);
    } else {
        println!("Key not found");
    }

    // Generate a Merkle proof for the key
    let proof = tree.merkle_proof(key)?;

    // Verify the proof
    let root_node = tree.root()?;
    let root_hash = root_node.node_hash();
    let leaf_node = LeafNode::new(key, value, sum);
    let is_valid = proof.verify(key, &leaf_node, root_hash);
    println!("Proof verification result: {}", is_valid);

    Ok(())
}
```

## Documentation

For more detailed information on the API and usage, please refer to the [API documentation](https://docs.rs/mssmt).

## Examples

For more examples, please refer to the [examples](./examples) directory.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
