pub trait Hasher: Clone {
    type Hash: Clone + PartialEq + Into<Vec<u8>> + TryFrom<Vec<u8>>;

    fn hash(data: &[u8]) -> Self::Hash;

    fn leaf_hash(leaf: &[u8]) -> Self::Hash {
        let mut concatenated: Vec<u8> = Vec::new();
        concatenated.push(0x00);
        concatenated.extend_from_slice(leaf);
        Self::hash(concatenated.as_slice())
    }

    fn concat_and_hash(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
        let mut concatenated: Vec<u8> = left.clone().into();
        concatenated.push(0x01);
        concatenated.extend_from_slice(left.clone().into().as_slice());
        concatenated.extend_from_slice(right.clone().into().as_slice());

        Self::hash(concatenated.as_slice())
    }
}
