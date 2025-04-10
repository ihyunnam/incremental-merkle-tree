use crate::hasher::Hasher;
// use sha2::{Digest, Sha256};
use ark_bn254::Fr;
use crate::poseidon::PoseidonAlgorithm;
pub type Hash = Fr;

#[derive(Debug)]
pub struct MerkleTree
// where
//     T: Hasher,
{
    // hasher: H,
    pub tree: Vec<Vec<Hash>>,
    leaves_count: usize,
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
        }
    }

    pub fn insert_leaf<const N: usize>(&mut self, leaf: [Fr;N]) {
        // The leaf node should not be empty
        // assert!(!leaf.is_empty());
        // increment the leaf count by 1
        if self.tree.is_empty() {
            // self.tree.push(vec![PoseidonAlgorithm::leaf_hash(leaf)]);
            self.tree.push(vec![PoseidonAlgorithm::hash(leaf)]);
            self.leaves_count += 1;
        } else {
            let leaves = self.tree.first_mut().unwrap();
            // drop the duplicate leaf if present
            leaves.drain(self.leaves_count..);
            // leaves.push(T::leaf_hash(leaf));
            leaves.push(PoseidonAlgorithm::hash(leaf));
            self.leaves_count += 1;
        }
        Self::build_tree(&mut self.tree, self.leaves_count);
    }

    fn build_tree(tree: &mut Vec<Vec<Hash>>, leaves_count: usize) {
        tree.drain(1..);
        let mut idx = 0;
        // If there are odd number of leaves (except 1), duplicate the last leaf.
        // It makes it easier to compute the merkle proof of the last leaf in a tree with odd number of leaves
        // NOTE: This does NOT affect the tree when inserting a new leaf as the duplicated leaf is removed before a new leaf is inserted
        if leaves_count > 1 && leaves_count % 2 != 0 {
            let last = tree.get(0).unwrap().last().unwrap().clone();
            tree.get_mut(0).unwrap().push(last);
        }

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

            if row.len() > 1 && row.len() % 2 != 0 {
                let last = row.last().unwrap().clone();
                row.push(last);
            }
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

    pub fn opening(&self, mut leaf_index: usize) -> Vec<Hash> {
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
