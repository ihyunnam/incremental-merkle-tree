use ark_bn254::Fr;

pub trait Hasher: Clone {
    // type Hash: Clone + PartialEq + Into<Vec<u8>> + TryFrom<Vec<u8>>;
    type Hash: Clone + PartialEq;

    fn hash<const N: usize>(data: [Fr;N]) -> Fr;

    fn leaf_hash(leaf: Fr) -> Fr {
        // let mut concatenated: Vec<u8> = Vec::new();
        // concatenated.push(0x00); // TODO: domain separation?
        // concatenated.extend_from_slice(leaf);
        // Fr(concatenated.as_slice())
        Self::hash([leaf])
    }

    // fn concat_and_hash(left: &Fr, right: &Fr) -> Fr {
    //     let mut concatenated: Vec<u8> = left.clone().into();
    //     concatenated.push(0x01);
    //     concatenated.extend_from_slice(left.clone().into().as_slice());
    //     concatenated.extend_from_slice(right.clone().into().as_slice());

    //     Fr(concatenated.as_slice())
    // }

    fn concat_and_hash(left: &Fr, right: &Fr) -> Fr {
        // let mut concatenated: Vec<u8> = (*left).into();
        // let mut concatenated: Vec<u8> = (left).into_bigint().to_bytes_be();
        Self::hash([*left, *right])
        // match right {
        //     Some(right_node) => {
        //         // let mut right_node_clone: Vec<u8> = (right_node).into_bigint().to_bytes_be();
        //         // concatenated.append(&mut right_node_clone);
        //         Fr([left, right_node])
        //     }
        //     None => left,
        // }
    }
}
