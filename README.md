# Incremental Merkle Tree
Merkle Tree implementation in Rust.
# Design
### Merkle Tree Interface
The implementation of Merkle tree has the following interface:   
* `fn insert_leaf(&self, leaf: &[u8])`: this method allows inserting new leaves incrementally to the Merkle Tree.
* `fn root(&self)`: this method returns the root of the Merkle Tree.
* `fn value(&self, leaf_index: usize)`: fetches a leaf (i.e. the hash of some value) stored at a provided index `leaf_index`.
* `fn opening(&self, leaf_index: usize)`: fetches the opening of a leaf at a provided index `leaf_index`.
* `fn verify(&self, opening: Vec<&Hash>, leaf_index: usize)`: verifies if the opening of a leaf at the provided index `leaf_index` is correct.
* `fn depth(&self)`: returns the depth of the tree at any time.
* `fn leaves_count(&self)`: returns the total number of [non-repeated] leaves in the tree at any time.


### Properties
The implementation achieves the following desired properties: 
* generic in the **hash function**.
* generic in the **tree height**.

# References
* https://w3c-ccg.github.io/Merkle-Disclosure-2021/jwp/#name-tree-construction
* https://tsc.bitcoinassociation.net/standards/merkle-proof-standardised-format/
* https://github.com/antouhou/rs-merkle