extern crate crypto;

mod index;

use self::crypto::sha3::Sha3;
use self::index::Index;
use bytes::Bytes;
use crypto::digest::Digest;

#[derive(Debug)]
pub enum Error {
    VerifyError,
}

fn hash_leaf(value: &Bytes) -> Bytes {
    let mut hasher = Sha3::keccak256();
    let mut result = vec![0u8; hasher.output_bits() / 8];
    hasher.reset();
    hasher.input(value.as_ref());
    hasher.result(result.as_mut_slice());
    Bytes::from(result)
}

trait Hashable {
    fn hash(&self) -> Bytes;
}

/// MerkleIntervalNode is a node in merkle tree
///
///```text
///  full tree
///
///           root
///        /        \
///      Node       Node
///     /   \      /   \
///   Leaf  Leaf Leaf  Leaf
///
///  branch and proof
///
///           root
///        /        \
///      Node     ProofNode
///     /   \      
///   Leaf  Leaf
///```
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MerkleIntervalNode<I: Index> {
    Leaf {
        end: I,
        data: Bytes,
    },

    Node {
        end: I,
        left: Box<MerkleIntervalNode<I>>,
        right: Box<MerkleIntervalNode<I>>,
    },

    ProofNode {
        end: I,
        data: Bytes,
    },
}

impl<I> Hashable for MerkleIntervalNode<I>
where
    I: Index,
{
    fn hash(&self) -> Bytes {
        match self {
            MerkleIntervalNode::Leaf { data, .. } => hash_leaf(data),
            // H(H(left.end + left.data) + H(right.end + right.data))
            MerkleIntervalNode::Node { left, right, .. } => {
                let mut buf = MerkleIntervalNode::compute_node(left.get_end(), &left.hash());
                buf.extend_from_slice(&MerkleIntervalNode::compute_node(
                    right.get_end(),
                    &right.hash(),
                ));
                hash_leaf(&buf)
            }
            MerkleIntervalNode::ProofNode { data, .. } => data.clone(),
        }
    }
}

impl<I> MerkleIntervalNode<I>
where
    I: Index,
{
    /// Caluculate hash of a node
    fn compute_node(offset: I, data: &Bytes) -> Bytes {
        let mut buf = Bytes::new();
        buf.extend_from_slice(&offset.encode_as_le());
        buf.extend_from_slice(data);
        hash_leaf(&buf)
    }

    pub fn create_proof_node(node: &Self) -> Self {
        MerkleIntervalNode::ProofNode {
            end: node.get_end(),
            data: node.hash(),
        }
    }

    pub fn create_empty() -> Self {
        MerkleIntervalNode::Leaf {
            end: I::max_value(),
            data: hash_leaf(&Bytes::from_static(&[0u8])),
        }
    }

    pub fn create_leaf(end: I, data: Bytes) -> Self {
        MerkleIntervalNode::Leaf { end, data }
    }

    pub fn create_node(end: I, left: &Self, right: &Self) -> Self {
        MerkleIntervalNode::Node {
            end,
            left: Box::new(left.clone()),
            right: Box::new(right.clone()),
        }
    }

    pub fn compute_parent(left: &Self, right: &Self) -> Self {
        MerkleIntervalNode::create_node(right.get_end(), left, right)
    }

    fn get_end(&self) -> I {
        match self {
            MerkleIntervalNode::Leaf { end, .. } => *end,
            MerkleIntervalNode::Node { end, .. } => *end,
            MerkleIntervalNode::ProofNode { end, .. } => *end,
        }
    }
    fn encode(&self) -> Bytes {
        match self {
            MerkleIntervalNode::ProofNode { end, data } => {
                let mut buf = Bytes::from(end.encode_as_le());
                buf.extend_from_slice(data);
                buf.clone()
            }
            _ => panic!("Leaf and Node can't be encoded"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImplicitBounds<I: Index> {
    implicit_start: I,
    implicit_end: I,
}

impl<I> ImplicitBounds<I>
where
    I: Index,
{
    pub fn new(implicit_start: I, implicit_end: I) -> Self {
        Self {
            implicit_start,
            implicit_end,
        }
    }
    pub fn get_start(&self) -> I {
        self.implicit_start
    }
    pub fn get_end(&self) -> I {
        self.implicit_end
    }
}

#[derive(Debug)]
pub struct MerkleIntervalTree<I: Index> {
    tree: MerkleIntervalNode<I>,
}

impl<I> MerkleIntervalTree<I>
where
    I: Index,
{
    /// generate sum merkle tree
    pub fn generate(leaves: &[MerkleIntervalNode<I>]) -> Self {
        if leaves.len() <= 1 {
            return MerkleIntervalTree {
                tree: leaves[0].clone(),
            };
        }
        let mut parents = vec![];
        for chunk in leaves.chunks(2) {
            let v = chunk.to_vec();
            if chunk.len() == 1 {
                parents.push(MerkleIntervalNode::compute_parent(
                    &v[0],
                    &MerkleIntervalNode::create_empty(),
                ))
            } else {
                parents.push(MerkleIntervalNode::compute_parent(
                    &v[0].clone(),
                    &v[1].clone(),
                ))
            }
        }
        MerkleIntervalTree::generate(&parents)
    }

    /// Calculate merkle root
    pub fn get_root(&self) -> Bytes {
        self.tree.hash()
    }

    /// Returns inclusion proof for a leaf
    pub fn get_inclusion_proof(&self, idx: usize, count: usize) -> Bytes {
        let nodes = MerkleIntervalTree::get_inclusion_proof_of_tree(&self.tree, idx, count);
        Self::encode_proof(nodes)
    }

    fn get_inclusion_proof_of_tree(
        tree: &MerkleIntervalNode<I>,
        idx: usize,
        count: usize,
    ) -> Vec<MerkleIntervalNode<I>> {
        match tree {
            MerkleIntervalNode::Leaf { .. } => vec![],
            MerkleIntervalNode::Node { left, right, .. } => {
                let left_count = count.next_power_of_two() / 2;
                if idx < left_count {
                    let mut proofs = Self::get_inclusion_proof_of_tree(left, idx, left_count);
                    proofs.push(MerkleIntervalNode::create_proof_node(&right));
                    proofs
                } else {
                    let mut proofs = Self::get_inclusion_proof_of_tree(
                        right,
                        idx - left_count,
                        count - left_count,
                    );
                    proofs.push(MerkleIntervalNode::create_proof_node(&left));
                    proofs
                }
            }
            MerkleIntervalNode::ProofNode { .. } => vec![],
        }
    }

    /// get_path
    /// get_path converts index of leaf to binary.
    /// ex) 1 -> 0b0001 -(revert)> [true, false, false, false]
    /// It means right, left, left, left
    ///
    /// Another example.
    /// 3 -> 0b11 -(revert)> [true, true]
    /// It means right, right
    ///
    ///```text
    ///        root
    ///       /    \
    ///     /  \  /  \
    ///     0  1  2  3
    /// ```
    ///
    fn get_path(idx: usize, depth: usize, path: &mut Vec<bool>) {
        if depth == 0 {
            return;
        }
        path.push((idx & 0x01) != 0);
        Self::get_path(idx.rotate_right(1), depth - 1, path)
    }

    fn verify_and_get_parent(
        left: &MerkleIntervalNode<I>,
        right: &MerkleIntervalNode<I>,
        _first_left_end: I,
    ) -> Result<MerkleIntervalNode<I>, Error> {
        /*
        if left.get_end() > first_left_end {
            return Err(Error::VerifyError);
        }
        */
        if left.get_end() > right.get_end() {
            return Err(Error::VerifyError);
        }
        Ok(MerkleIntervalNode::compute_parent(left, right))
    }

    /// Verify whether leaf is included or not
    pub fn verify(
        leaf: &MerkleIntervalNode<I>,
        idx: usize,
        inclusion_proof_bytes: Bytes,
        root: &Bytes,
    ) -> Result<ImplicitBounds<I>, Error> {
        let inclusion_proof: Vec<MerkleIntervalNode<I>> = Self::decode_proof(inclusion_proof_bytes);
        let mut path: Vec<bool> = vec![];
        Self::get_path(idx, inclusion_proof.len(), path.as_mut());
        let first_left_end = path
            .iter()
            .position(|&p| p)
            .map(|pos| inclusion_proof[pos].clone())
            .map_or(I::zero(), |n| n.get_end());
        let mut computed = leaf.clone();
        for (i, item) in inclusion_proof.iter().enumerate() {
            if path[i] {
                // leaf is in right
                computed = Self::verify_and_get_parent(item, &computed, first_left_end)?
            } else {
                // leaf is in left
                computed = Self::verify_and_get_parent(&computed, item, first_left_end)?
            }
        }
        let is_last_leaf = 2u64.pow(inclusion_proof.len() as u32) - 1 == (idx as u64);
        if computed.hash() == root {
            Ok(ImplicitBounds::new(
                first_left_end,
                if is_last_leaf {
                    I::max_value()
                } else {
                    leaf.get_end()
                },
            ))
        } else {
            Err(Error::VerifyError)
        }
    }

    pub fn encode_proof(inclusion_proof_nodes: Vec<MerkleIntervalNode<I>>) -> Bytes {
        let mut inclusion_proof = Bytes::from("");
        for n in inclusion_proof_nodes.iter() {
            inclusion_proof.extend_from_slice(&n.encode().to_vec());
        }
        inclusion_proof
    }

    pub fn decode_proof(inclusion_proof: Bytes) -> Vec<MerkleIntervalNode<I>> {
        let mut nodes = vec![];
        let num_nodes = inclusion_proof.len() / 40;
        for i in 0..num_nodes {
            let index = i * 40;
            let end = I::decode_as_le(&inclusion_proof.slice(index, index + 8).to_vec());
            let data = inclusion_proof.slice(index + 8, index + 40);
            nodes.push(MerkleIntervalNode::ProofNode { end, data });
        }
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::Bytes;
    use super::MerkleIntervalNode;
    use super::MerkleIntervalTree;

    #[test]
    fn test_compute_parent() {
        let hash_message1 = Bytes::from(&b"message"[..]);
        let leaf1 = MerkleIntervalNode::Leaf {
            end: 100,
            data: hash_message1,
        };
        let hash_message2 = Bytes::from(&b"message"[..]);
        let leaf2 = MerkleIntervalNode::Leaf {
            end: 200,
            data: hash_message2,
        };
        let parent = MerkleIntervalNode::compute_parent(&leaf1, &leaf2);
        assert_eq!(parent.get_end(), 200);
    }

    #[test]
    fn test_encode_and_decode() {
        let mut leaves = vec![];
        for i in 0..100 {
            leaves.push(MerkleIntervalNode::Leaf {
                end: i * 100 + 100,
                data: Bytes::from(&b"message"[..]),
            })
        }
        let tree = MerkleIntervalTree::generate(&leaves);
        let inclusion_proof = tree.get_inclusion_proof(5, 100);
        let nodes: Vec<MerkleIntervalNode<u64>> = MerkleIntervalTree::decode_proof(inclusion_proof);
        assert_eq!(nodes.len(), 7);
    }

    #[test]
    fn test_generate_tree() {
        let hash_message1 = Bytes::from(&b"message"[..]);
        let leaf1 = MerkleIntervalNode::Leaf {
            end: 100,
            data: hash_message1,
        };
        let hash_message2 = Bytes::from(&b"message"[..]);
        let leaf2 = MerkleIntervalNode::Leaf {
            end: 200,
            data: hash_message2,
        };
        let tree = MerkleIntervalTree::generate(&[leaf1, leaf2]);
        assert_eq!(tree.get_root().len(), 32);
    }

    #[test]
    fn test_proof() {
        let hash_message1 = Bytes::from(&b"message"[..]);
        let leaf1 = MerkleIntervalNode::Leaf {
            end: 100,
            data: hash_message1,
        };
        let hash_message2 = Bytes::from(&b"message"[..]);
        let leaf2 = MerkleIntervalNode::Leaf {
            end: 200,
            data: hash_message2,
        };
        let tree = MerkleIntervalTree::generate(&[leaf1.clone(), leaf2]);
        let inclusion_proof = tree.get_inclusion_proof(0, 2);
        assert_eq!(inclusion_proof.len(), 40);
        assert_eq!(
            MerkleIntervalTree::verify(&leaf1.clone(), 0, inclusion_proof, &tree.get_root())
                .is_ok(),
            true
        );
    }

    #[test]
    fn test_large_leaves() {
        let mut leaves = vec![];
        for i in 0..100 {
            leaves.push(MerkleIntervalNode::Leaf {
                end: i * 100 + 100,
                data: Bytes::from(&b"message"[..]),
            })
        }
        let tree = MerkleIntervalTree::generate(&leaves);
        let inclusion_proof = tree.get_inclusion_proof(5, 100);
        assert_eq!(inclusion_proof.len(), 280);
        assert_eq!(
            MerkleIntervalTree::verify(&leaves[5].clone(), 5, inclusion_proof, &tree.get_root())
                .is_ok(),
            true
        );
    }

    #[test]
    fn test_failed_to_verify() {
        let mut leaves = vec![];
        for i in 0..100 {
            leaves.push(MerkleIntervalNode::Leaf {
                end: i * 100 + 100,
                data: Bytes::from(&b"message"[..]),
            })
        }
        let tree = MerkleIntervalTree::generate(&leaves);
        let inclusion_proof = tree.get_inclusion_proof(5, 100);
        assert_eq!(inclusion_proof.len(), 280);
        assert_eq!(
            MerkleIntervalTree::verify(&leaves[5].clone(), 7, inclusion_proof, &tree.get_root())
                .is_ok(),
            false
        );
    }

}
