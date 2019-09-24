use super::address_tree::{AddressTree, AddressTreeNode};
use super::interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use abi_derive::{AbiDecodable, AbiEncodable};
use abi_utils::{Decodable, Encodable, Integer};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use std::collections::HashMap;

#[derive(Clone, Debug, AbiEncodable, AbiDecodable)]
pub struct InclusionProof {
    pub address_idx: Integer,
    pub interval_idx: Integer,
    pub address_tree_inclusion_proof: Bytes,
    pub interval_tree_inclusion_proof: Bytes,
}

impl InclusionProof {
    pub fn new(
        address_idx: Integer,
        interval_idx: Integer,
        address_tree_inclusion_proof: Bytes,
        interval_tree_inclusion_proof: Bytes,
    ) -> Self {
        Self {
            address_idx,
            interval_idx,
            address_tree_inclusion_proof,
            interval_tree_inclusion_proof,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DoubleLayerTreeLeaf {
    pub data: Bytes,
    pub end: u64,
    pub address: Address,
}

/// Double Layer Merkle Tree implementation which is described at https://docs.plasma.group/projects/spec/en/latest/src/01-core/double-layer-tree.html.
pub struct DoubleLayerTree {
    tree: AddressTree,
    interval_trees: HashMap<Address, MerkleIntervalTree<u64>>,
}

impl From<&DoubleLayerTreeLeaf> for MerkleIntervalNode<u64> {
    fn from(tree: &DoubleLayerTreeLeaf) -> MerkleIntervalNode<u64> {
        MerkleIntervalNode::Leaf {
            data: tree.data.clone(),
            end: tree.end,
        }
    }
}

impl DoubleLayerTree {
    pub fn generate(double_layer_tree_leaves: &[DoubleLayerTreeLeaf]) -> Self {
        let leaves_set: HashMap<Address, Vec<MerkleIntervalNode<u64>>> = double_layer_tree_leaves
            .iter()
            .fold(HashMap::new(), |mut acc, l| {
                let interval_leaf = MerkleIntervalNode::Leaf {
                    data: l.data.clone(),
                    end: l.end,
                };
                let mut leaves: Vec<MerkleIntervalNode<u64>> =
                    acc.get(&l.address).unwrap_or(&vec![]).to_vec();
                leaves.push(interval_leaf);
                acc.insert(l.address, leaves.to_vec());
                acc
            });
        let mut interval_trees: HashMap<Address, MerkleIntervalTree<u64>> = Default::default();
        let mut address_tree_leaves: Vec<AddressTreeNode> = vec![];
        for (address, leaves) in leaves_set {
            let tree = MerkleIntervalTree::generate(&leaves);
            let root = tree.get_root();
            interval_trees.insert(address, tree);
            address_tree_leaves.push(AddressTreeNode::Leaf(root, address));
        }
        let address_tree = AddressTree::generate(address_tree_leaves);
        Self {
            tree: address_tree,
            interval_trees,
        }
    }
    pub fn get_root(&self) -> Bytes {
        self.tree.get_root()
    }
    pub fn get_address_index(&self, address: Address) -> usize {
        self.tree.get_index(address)
    }
    pub fn get_inclusion_proof(&self, address: Address, idx: usize) -> Bytes {
        let address_tree_inclusion_proof = self.tree.get_inclusion_proof(address);
        let interval_tree_inclusion_proof = self
            .interval_trees
            .get(&address)
            .unwrap()
            .get_inclusion_proof(idx);
        Bytes::from(
            InclusionProof {
                address_idx: Integer(self.tree.get_index(address) as u64),
                interval_idx: Integer(idx as u64),
                address_tree_inclusion_proof,
                interval_tree_inclusion_proof,
            }
            .to_abi(),
        )
    }
    pub fn verify(leaf: &DoubleLayerTreeLeaf, inclusion_proof_bytes: Bytes, root: &Bytes) -> bool {
        let inclusion_proof = InclusionProof::from_abi(&inclusion_proof_bytes.to_vec()).unwrap();
        if let Ok((computed_root, _)) = MerkleIntervalTree::compute_root(
            &leaf.into(),
            inclusion_proof.interval_idx.0 as usize,
            inclusion_proof.interval_tree_inclusion_proof,
        ) {
            AddressTree::verify(
                &AddressTreeNode::Leaf(computed_root, leaf.address),
                inclusion_proof.address_idx.0 as usize,
                inclusion_proof.address_tree_inclusion_proof,
                root,
            )
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use ethereum_types::Address;

    #[test]
    fn test_generate() {
        let address1 = Address::random();
        let address2 = Address::random();
        let mut leaves = vec![];
        for i in 0..100 {
            leaves.push(DoubleLayerTreeLeaf {
                end: i * 100 + 100,
                data: Bytes::from(&b"message1"[..]),
                address: address1,
            })
        }
        for i in 0..100 {
            leaves.push(DoubleLayerTreeLeaf {
                end: i * 100 + 100,
                data: Bytes::from(&b"message2"[..]),
                address: address2,
            })
        }
        let tree = DoubleLayerTree::generate(&leaves);
        let root = tree.get_root();

        let inclusion_proof = tree.get_inclusion_proof(address2, 2);
        assert!(DoubleLayerTree::verify(
            &DoubleLayerTreeLeaf {
                data: Bytes::from(&b"message2"[..]),
                end: 300,
                address: address2,
            },
            inclusion_proof,
            &root
        ));
    }
}
