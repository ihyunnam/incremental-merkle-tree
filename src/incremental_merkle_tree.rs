use crate::hasher::Hasher;
// use sha2::{Digest, Sha256};
use ark_bn254::Fr;
use ark_ff::Zero;
use crate::poseidon::PoseidonAlgorithm;
pub type Hash = Fr;

#[derive(Debug, Clone)]
pub struct MerkleTree
// where
//     T: Hasher,
{
    // hasher: H,
    pub tree: Vec<Vec<Hash>>,
    leaves_count: usize,
    empty_hashes: Vec<Hash>,
}

// impl<T> Default for MerkleTree<T>
// where
//     T: Hasher,
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl MerkleTree
// where
//     T: Hasher,
{
    pub fn new() -> Self {
        Self {
            tree: Vec::new(),
            leaves_count: 0,
            empty_hashes: Vec::new(),
        }
    }

    pub fn insert_leaf<const N: usize>(&mut self, leaf: [Fr;N]) {
        let leaves = self.tree.first_mut().unwrap();
        leaves[self.leaves_count] = PoseidonAlgorithm::hash(leaf);  // Instead of draining zero nodes and pushing new, replace a zero node with the new leaf.
        self.leaves_count += 1;
        Self::build_tree(&mut self.tree, self.leaves_count);
    }

    pub fn build_empty_tree(depth: u32) -> MerkleTree {
        let mut tree = MerkleTree::new();
    
        // Start from the bottom layer (leaves)
        let num_leaves = 1usize << (depth - 1);
        // let empty_leaf = PoseidonAlgorithm::hash([Fr::zero(), Fr::zero()]);
        let mut current_level = vec![Fr::zero(); num_leaves];
    
        // Push each level from leaves to root
        for _ in 0..depth {
            tree.tree.push(current_level.clone());
            tree.empty_hashes.push(current_level[0]);   // Save empty node at each level

            // Build the next level up
            let mut next_level = Vec::with_capacity(current_level.len() / 2);
            for pair in current_level.chunks(2) {
                let h = if pair.len() == 2 {
                    PoseidonAlgorithm::hash([pair[0], pair[1]])
                } else {
                    // handle odd node (pad with zero)
                    PoseidonAlgorithm::hash([pair[0], Fr::zero()])
                };
                next_level.push(h);
            }
    
            current_level = next_level;
        }
    
        tree.leaves_count = 0;  // Note: leaves_count is for real leaves that are actually inserted (log tree roots)
        tree
    }

    // pub fn build_empty_tree(depth: u32) -> MerkleTree {
    //     // let min_num_leaves: usize = (1usize << (depth - 2)) + 1;
    //     let mut tree = MerkleTree::new();
    //     for level in 0..depth {
    //         let num_leaves = 1usize << (depth-level-1);
    //         let row = vec![Fr::zero(); num_leaves];
    //         tree.tree.push(row)
    //     }
    //     tree.leaves_count = 1usize << (depth-1);
    //     tree
    // }

    fn build_tree(tree: &mut Vec<Vec<Hash>>, leaves_count: usize) {
        tree.drain(1..);    // Remove all upper levels because they'll be rebuilt from new leaves.
        let mut idx = 0;
        // If there are odd number of leaves (except 1), append a zero node.
        // (original behavior was to duplicate the last leaf)
        // It makes it easier to compute the merkle proof of the last leaf in a tree with odd number of leaves
        // NOTE: This does NOT affect the tree when inserting a new leaf as the duplicated leaf is removed before a new leaf is inserted
        
        // if leaves_count > 1 && leaves_count % 2 != 0 {
        //     // let last = tree.get(0).unwrap().last().unwrap().clone();
        //     tree.get_mut(0).unwrap().push(Fr::zero());
        // }    // Note: all leaves have placeholders

        loop {
            // Keep looping until you reach a level with a single node
            let current_layer = tree.get(idx).unwrap().clone();
            let n = current_layer.len();
            if n == 1 {
                break;
            }

            let mut row = Vec::with_capacity((n + 1) / 2);

            let mut i = 0;

            while i < n {
                let internal_node = PoseidonAlgorithm::concat_and_hash(&current_layer[i], &current_layer[i + 1]);
                row.push(internal_node);
                i += 2;
            }

            // Note: all leaves have placeholders
            // if row.len() > 1 && row.len() % 2 != 0 {
            //     // let last = row.last().unwrap().clone();
            //     row.push(Fr::zero());
            // }
            tree.push(row);
            // Move to the next level
            idx += 1;
        }
    }

    pub fn leaves_count(&self) -> usize {
        self.leaves_count
    }

    pub fn depth(&self) -> usize {
        self.tree.len()
    }

    pub fn value(&self, leaf_index: usize) -> Option<&Hash> {
        self.tree
            .first()
            .expect("There are no leaves in the tree!")
            .get(leaf_index)
    }

    pub fn root(&self) -> Option<&Hash> {
        self.tree
            .last()
            .expect("There are no leaves in the tree!")
            .first()
    }

    pub fn opening_orig(&self, mut leaf_index: usize) -> Vec<Hash> {
        let mut opening = Vec::new();
        // Iterate over all level until the root
        for level in self.tree.split_last().unwrap().1.iter() {
            if leaf_index % 2 != 0 {
                opening.push(*level.get(leaf_index - 1).unwrap());
            } else {
                opening.push(*level.get(leaf_index + 1).unwrap());
            }
            leaf_index /= 2;
        }
        opening
    }

    pub fn opening(&self, leaf_index: u32) -> Vec<(Hash, Hash)> {
        let mut path = Vec::new();
        // let tree_index = convert_index_to_last_level(index, N); // Note: It's given index = Binary(h_i)

        // Iterate from the leaf up to the root, storing all intermediate hash values.
        let mut current_node = leaf_index;
        // let mut level_count = 0;
        // while !is_root(current_node) {
        for level in self.tree.split_last().unwrap().1.iter() {
            // println!("level {:?}", level_count);
            let sibling_node = sibling(current_node).unwrap();
            // let empty_hash = &self.empty_hashes[level_count];
            // println!("current node {:?}", current_node);
            let current = level.get(current_node as usize).cloned().expect("Expected current node to exist.");
            // println!("current {:?}", current);
            let sibling = level.get(sibling_node as usize).cloned().expect("Expected sibling node to exist.");
            // println!("sibling {:?}", sibling);
            if is_left_child(current_node) {
                // path[level] = (current, sibling);
                path.push((current, sibling));
            } else {
                // path[level] = (sibling, current);
                path.push((sibling, current));
            }
            current_node = parent(current_node).unwrap();
            // level_count += 1;
        }

        path
    }
    

    pub fn verify(&self, proof: Vec<&Hash>, mut leaf_index: usize) -> bool {
        let mut prev: ark_ff::Fp<ark_ff::MontBackend<ark_bn254::FrConfig, 4>, 4>= self.tree.first().unwrap().get(leaf_index).unwrap().clone();
        for node in proof.into_iter() {
            if leaf_index % 2 == 0 {
                prev = PoseidonAlgorithm::concat_and_hash(&prev, node);
            } else {
                prev = PoseidonAlgorithm::concat_and_hash(node, &prev);
            }
            leaf_index /= 2;
        }
        prev == self.tree.last().unwrap().first().unwrap().clone()
    }
}

// Returns true iff the given index represents a left child.
fn is_left_child(index: u32) -> bool {
    index % 2 == 0
}

// Returns the index of the sibling, given an index.
fn sibling(index: u32) -> Option<u32> {
    // if index == 0 {  // Note: Path doesn't contain the root
    //     None
    if is_left_child(index) {
        Some(index + 1)
    } else {
        Some(index - 1)
    }
}

// Given index, return parent index.
fn parent (index: u32) -> Option<u32> {
    if index > 0 {
        Some(index >> 1)  // Note: path doesn't contain leaf
    } else {
        Some(0)
    }
}

// #[derive(Clone)]
// pub struct DefaultHasher {}

// impl Hasher for DefaultHasher {
//     type Hash = Vec<u8>;

//     fn hash(bytes: &[u8]) -> Self::Hash {
//         Sha256::digest(bytes).to_vec()
//     }
// }

// #[cfg(test)]
// pub mod tests {

//     use super::*;

//     pub fn build_mock_tree<T: Hasher>(mock_merkle_tree: &mut MerkleTree<T>, leaves_count: usize) {
//         (0..leaves_count)
//             .for_each(|leaf| mock_merkle_tree.insert_leaf(leaf.to_string().as_bytes()));
//     }

//     pub fn get_leaf(index: usize) -> Vec<u8> {
//         let mut res: Vec<u8> = Vec::new();
//         res.push(0x00);
//         res.extend_from_slice(index.to_string().as_bytes());
//         Sha256::digest(res.as_slice()).to_vec()
//     }

//     #[test]

//     fn test_add_one_leaf() {
//         let mut merkle_tree = MerkleTree::<DefaultHasher>::new();
//         // Assert the merkle tree must be empty before inserting a leaf
//         assert_eq!(merkle_tree.leaves_count(), 0);
//         // Let's build a merkle tree with one node
//         merkle_tree.insert_leaf("hello".as_bytes());
//         // the leaf must be the root in this case
//         assert_eq!(merkle_tree.root(), merkle_tree.value(0));
//         assert_eq!(merkle_tree.leaves_count(), 1)
//     }

//     #[test]
//     fn test_add_two_leaves() {
//         let mut merkle_tree = MerkleTree::<DefaultHasher>::new();
//         // Assert the merkle tree must be empty before inserting a leaf
//         assert_eq!(merkle_tree.leaves_count(), 0);
//         // Build a mock merkle tree with 2 leaves
//         build_mock_tree(&mut merkle_tree, 2);

//         // The tree height is expected to be 2
//         assert_eq!(merkle_tree.depth(), 2);
//         assert_eq!(merkle_tree.leaves_count(), 2);

//         // Check if the input leaf values (hashes) are stored correctly in the tree
//         assert_eq!(merkle_tree.value(0), Some(&get_leaf(0)));
//         assert_eq!(merkle_tree.value(1), Some(&get_leaf(1)));
//     }

//     #[test]
//     fn test_add_three_leaves() {
//         let mut merkle_tree = MerkleTree::<DefaultHasher>::new();

//         // // Assert the merkle tree must be empty before inserting a leaf
//         assert_eq!(merkle_tree.leaves_count(), 0);

//         // Build a mock merkle tree with 3 leaves
//         build_mock_tree(&mut merkle_tree, 3);

//         assert_eq!(merkle_tree.leaves_count(), 3);
//         // The expected tree height should 3
//         //     root
//         //     /  \
//         //   i_0  i_1
//         //  / \   / \
//         //  0 1  2  2_copy
//         assert_eq!(merkle_tree.depth(), 3);

//         // Check if the input leaf values (hashes) are stored correctly in the tree
//         assert_eq!(merkle_tree.value(0), Some(&get_leaf(0)));
//         assert_eq!(merkle_tree.value(1), Some(&get_leaf(1)));
//         assert_eq!(merkle_tree.value(2), Some(&get_leaf(2)));

//         // get merkle proof of the leaf at index 1
//         let opening = merkle_tree.opening(1);
//         assert!(merkle_tree.verify(opening, 1));
//     }

//     #[test]
//     fn test_add_five_leaves() {
//         let mut merkle_tree = MerkleTree::<DefaultHasher>::new();

//         // // Assert the merkle tree must be empty before inserting a leaf
//         assert_eq!(merkle_tree.leaves_count(), 0);

//         // Build a mock merkle tree with 5 leaves
//         build_mock_tree(&mut merkle_tree, 5);

//         assert_eq!(merkle_tree.leaves_count(), 5);
//         // The expected tree height should 4
//         //          root
//         //       /       \
//         //     i_3       i_4
//         //    /   \     /  \
//         //  i_0  i_1  i_2 i_2_copy
//         //  / \  / \  / \
//         //  0 1  2 3  4 4_copy
//         assert_eq!(merkle_tree.depth(), 4);

//         // Check if the input leaf values (hashes) are stored correctly in the tree
//         assert_eq!(merkle_tree.value(0), Some(&get_leaf(0)));
//         assert_eq!(merkle_tree.value(1), Some(&get_leaf(1)));
//         assert_eq!(merkle_tree.value(4), Some(&get_leaf(4)));

//         // get merkle proof of the leaf at index 4
//         let opening = merkle_tree.opening(4);
//         assert!(merkle_tree.verify(opening, 4));
//     }

//     #[test]
//     fn test_add_999_leaves() {
//         let mut merkle_tree = MerkleTree::<DefaultHasher>::new();

//         // // Assert the merkle tree must be empty before inserting a leaf
//         assert_eq!(merkle_tree.leaves_count(), 0);

//         // Build a mock merkle tree with 3 leaves
//         build_mock_tree(&mut merkle_tree, 999);

//         // Check if the input leaf values (hashes) are stored correctly in the tree
//         assert_eq!(merkle_tree.value(0), Some(&get_leaf(0)));
//         assert_eq!(merkle_tree.value(1), Some(&get_leaf(1)));
//         assert_eq!(merkle_tree.value(4), Some(&get_leaf(4)));
//         assert_eq!(merkle_tree.value(101), Some(&get_leaf(101)));
//         assert_eq!(merkle_tree.value(567), Some(&get_leaf(567)));
//         assert_eq!(merkle_tree.value(789), Some(&get_leaf(789)));

//         // get merkle proof of the leaf at index 1
//         let opening = merkle_tree.opening(579);
//         assert!(merkle_tree.verify(opening, 579));
//     }

//     #[test]
//     fn test_merkle_proof_1176_leaves() {
//         let mut merkle_tree = MerkleTree::<DefaultHasher>::new();

//         // Assert the merkle tree must be empty before inserting a leaf
//         assert_eq!(merkle_tree.leaves_count(), 0);

//         // Build a mock merkle tree with 1176 leaves
//         build_mock_tree(&mut merkle_tree, 1176);

//         assert_eq!(merkle_tree.leaves_count(), 1176);
//         // get merkle proof of the leaf at index 999
//         let opening = merkle_tree.opening(999);
//         assert!(merkle_tree.verify(opening, 999));
//     }
// }
