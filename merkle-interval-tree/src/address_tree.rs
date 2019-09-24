use super::hash::hash_leaf;
use bytes::Bytes;
use ethereum_types::Address;

/// Address tree implementation which is described at https://docs.plasma.group/projects/spec/en/latest/src/01-core/double-layer-tree.html.
pub struct AddressTree {
    tree: AddressTreeNode,
    leaves: Vec<AddressTreeNode>,
}

impl AddressTree {
    pub fn generate(leaves: Vec<AddressTreeNode>) -> Self {
        Self {
            tree: Self::generate_part(leaves.clone()),
            leaves,
        }
    }
    pub fn generate_part(leaves: Vec<AddressTreeNode>) -> AddressTreeNode {
        if leaves.len() <= 1 {
            return leaves[0].clone();
        }
        let mut parents = vec![];
        for chunk in leaves.chunks(2) {
            let v = chunk.to_vec();
            if chunk.len() == 1 {
                parents.push(AddressTreeNode::compute_parent(
                    &v[0],
                    &AddressTreeNode::create_empty(),
                ))
            } else {
                parents.push(AddressTreeNode::compute_parent(
                    &v[0].clone(),
                    &v[1].clone(),
                ))
            }
        }
        AddressTree::generate_part(parents)
    }
    pub fn get_root(&self) -> Bytes {
        self.tree.get_hash()
    }
    pub fn get_index(&self, address: Address) -> usize {
        if let Some(index) = self.leaves.iter().position(|s| s.get_address() == address) {
            index
        } else {
            panic!("address {:?} not found in leaves.", address);
        }
    }
    pub fn get_inclusion_proof(&self, address: Address) -> Bytes {
        let index = self.get_index(address);
        Self::encode_proof(Self::get_inclusion_proof_of_tree(
            &self.tree,
            index,
            self.leaves.len(),
        ))
    }
    pub fn get_inclusion_proof_of_tree(
        tree: &AddressTreeNode,
        idx: usize,
        count: usize,
    ) -> Vec<AddressTreeNode> {
        match tree {
            AddressTreeNode::Leaf(_, _) => vec![],
            AddressTreeNode::Node(_, _, left, right) => {
                let left_count = count.next_power_of_two() / 2;
                if idx < left_count {
                    let mut proofs = Self::get_inclusion_proof_of_tree(left, idx, left_count);
                    proofs.push(AddressTreeNode::create_proof_node(&right));
                    proofs
                } else {
                    let mut proofs = Self::get_inclusion_proof_of_tree(
                        right,
                        idx - left_count,
                        count - left_count,
                    );
                    proofs.push(AddressTreeNode::create_proof_node(&left));
                    proofs
                }
            }
            AddressTreeNode::ProofNode(_, _) => vec![],
        }
    }
    fn get_path(idx: usize, depth: usize, path: &mut Vec<bool>) {
        if depth == 0 {
            return;
        }
        path.push((idx & 0x01) != 0);
        Self::get_path(idx.rotate_right(1), depth - 1, path)
    }
    /// Verify whether leaf is included or not
    pub fn verify(
        leaf: &AddressTreeNode,
        idx: usize,
        inclusion_proof_bytes: Bytes,
        root: &Bytes,
    ) -> bool {
        let inclusion_proof: Vec<AddressTreeNode> = Self::decode_proof(inclusion_proof_bytes);
        let mut path: Vec<bool> = vec![];
        Self::get_path(idx, inclusion_proof.len(), path.as_mut());
        let mut computed = leaf.clone();
        for (i, item) in inclusion_proof.iter().enumerate() {
            if path[i] {
                // leaf is in right
                computed = AddressTreeNode::compute_parent(item, &computed);
            } else {
                // leaf is in left
                computed = AddressTreeNode::compute_parent(&computed, item);
            }
        }
        computed.get_hash() == root
    }
    pub fn encode_proof(inclusion_proof_nodes: Vec<AddressTreeNode>) -> Bytes {
        let mut inclusion_proof = Bytes::from("");
        for n in inclusion_proof_nodes.iter() {
            inclusion_proof.extend_from_slice(&n.encode().to_vec());
        }
        inclusion_proof
    }

    pub fn decode_proof(inclusion_proof: Bytes) -> Vec<AddressTreeNode> {
        let mut nodes = vec![];
        let node_size = 52;
        let num_nodes = inclusion_proof.len() / node_size;
        for i in 0..num_nodes {
            let index = i * node_size;
            let hash = inclusion_proof.slice(index, index + 32);
            let address = Address::from_slice(&inclusion_proof.slice(index + 32, index + 52));
            nodes.push(AddressTreeNode::ProofNode(hash, address));
        }
        nodes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddressTreeNode {
    Leaf(Bytes, Address),
    Node(Bytes, Address, Box<AddressTreeNode>, Box<AddressTreeNode>),
    ProofNode(Bytes, Address),
}

impl AddressTreeNode {
    pub fn get_hash(&self) -> Bytes {
        match self {
            AddressTreeNode::Leaf(hash, _) => hash.clone(),
            AddressTreeNode::Node(hash, _, _, _) => hash.clone(),
            AddressTreeNode::ProofNode(hash, _) => hash.clone(),
        }
    }
    pub fn get_address(&self) -> Address {
        match self {
            AddressTreeNode::Leaf(_, address) => *address,
            AddressTreeNode::Node(_, address, _, _) => *address,
            AddressTreeNode::ProofNode(_, address) => *address,
        }
    }
    pub fn create_empty() -> Self {
        AddressTreeNode::Leaf(hash_leaf(&Bytes::from_static(&[0u8])), Address::zero())
    }
    fn compute_node(hash: Bytes, address: Address) -> Bytes {
        let mut buf = Bytes::new();
        buf.extend_from_slice(&hash);
        buf.extend_from_slice(&Bytes::from(address.as_bytes()));
        hash_leaf(&buf)
    }
    pub fn compute_parent(left: &AddressTreeNode, right: &AddressTreeNode) -> Self {
        let mut buf = Bytes::new();
        buf.extend_from_slice(&Self::compute_node(left.get_hash(), left.get_address()));
        buf.extend_from_slice(&Self::compute_node(right.get_hash(), right.get_address()));
        AddressTreeNode::Node(
            hash_leaf(&buf),
            left.get_address(),
            Box::new(left.clone()),
            Box::new(right.clone()),
        )
    }
    pub fn create_proof_node(node: &AddressTreeNode) -> Self {
        AddressTreeNode::ProofNode(node.get_hash(), node.get_address())
    }
    fn encode(&self) -> Bytes {
        match self {
            AddressTreeNode::ProofNode(hash, address) => {
                let mut buf = hash.clone();
                buf.extend_from_slice(&Bytes::from(address.as_bytes()));
                buf.clone()
            }
            _ => panic!("Leaf and Node can't be encoded"),
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
        let hash1 = hash_leaf(&Bytes::from("message1"));
        let hash2 = hash_leaf(&Bytes::from("message2"));
        let address1 = Address::random();
        let address2 = Address::random();
        let leaf1 = AddressTreeNode::Leaf(hash1, address1);
        let leaf2 = AddressTreeNode::Leaf(hash2, address2);
        let leaves = vec![leaf1, leaf2.clone()];
        let address_tree = AddressTree::generate(leaves);
        let root = address_tree.get_root();
        let inclusion_proof = address_tree.get_inclusion_proof(address2);
        assert!(AddressTree::verify(&leaf2, 1, inclusion_proof, &root));
    }
}
