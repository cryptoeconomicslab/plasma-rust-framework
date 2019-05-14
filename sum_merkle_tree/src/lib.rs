extern crate crypto;

use self::crypto::sha3::Sha3;
use byteorder::{LittleEndian, WriteBytesExt};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SumMerkleNode {
    Leaf {
        end: u64,
        data: Bytes,
    },

    Node {
        end: u64,
        left: Box<SumMerkleNode>,
        right: Box<SumMerkleNode>,
    },

    RightProofNode {
        end: u64,
        left: Bytes,
    },

    LeftProofNode {
        end: u64,
        right: Bytes,
    },
}

/// Caluculate hash of a node
fn compute_node(end: u64, data: &Bytes) -> Bytes {
    let mut wtr = vec![];
    wtr.write_u64::<LittleEndian>(end).unwrap();
    let mut buf = Bytes::new();
    buf.extend_from_slice(&wtr);
    buf.extend_from_slice(&data);
    hash_leaf(&buf)
}

impl Hashable for SumMerkleNode {
    fn hash(&self) -> Bytes {
        match self {
            SumMerkleNode::Leaf { end, data } => compute_node(*end, data),
            SumMerkleNode::Node { left, right, .. } => {
                let mut buf = compute_node(left.get_end(), &left.hash());
                buf.extend_from_slice(&compute_node(right.get_end(), &right.hash()));
                hash_leaf(&buf)
            }
            SumMerkleNode::RightProofNode { left, .. } => left.clone(),
            SumMerkleNode::LeftProofNode { right, .. } => right.clone(),
        }
    }
}

impl SumMerkleNode {
    pub fn create_left_proof(right: &SumMerkleNode) -> SumMerkleNode {
        SumMerkleNode::LeftProofNode {
            end: right.get_end(),
            right: right.hash(),
        }
    }

    pub fn create_right_proof(left: &SumMerkleNode) -> SumMerkleNode {
        SumMerkleNode::RightProofNode {
            end: left.get_end(),
            left: left.hash(),
        }
    }

    pub fn create_empty() -> Self {
        SumMerkleNode::Leaf {
            end: u64::max_value(),
            data: hash_leaf(&Bytes::from_static(&[0u8])),
        }
    }

    pub fn create_leaf(end: u64, data: Bytes) -> Self {
        SumMerkleNode::Leaf { end, data }
    }

    pub fn create_node(end: u64, left: &SumMerkleNode, right: &SumMerkleNode) -> Self {
        SumMerkleNode::Node {
            end,
            left: Box::new(left.clone()),
            right: Box::new(right.clone()),
        }
    }

    pub fn compute_parent(left: &SumMerkleNode, right: &SumMerkleNode) -> SumMerkleNode {
        SumMerkleNode::create_node(right.get_end(), left, right)
    }

    fn get_end(&self) -> u64 {
        match self {
            SumMerkleNode::Leaf { end, .. } => *end,
            SumMerkleNode::Node { end, .. } => *end,
            SumMerkleNode::RightProofNode { end, .. } => *end,
            SumMerkleNode::LeftProofNode { end, .. } => *end,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImplicitBounds {
    implicit_start: u64,
    implicit_end: u64,
}

impl ImplicitBounds {
    pub fn new(implicit_start: u64, implicit_end: u64) -> Self {
        ImplicitBounds {
            implicit_start,
            implicit_end,
        }
    }
}

#[derive(Debug)]
pub struct SumMerkleTree {
    tree: SumMerkleNode,
}

impl SumMerkleTree {
    /// generate sum merkle tree
    pub fn generate(leaves: &[SumMerkleNode]) -> SumMerkleTree {
        if leaves.len() <= 1 {
            return SumMerkleTree {
                tree: leaves[0].clone(),
            };
        }
        let mut parents = vec![];
        for chunk in leaves.chunks(2) {
            let v = chunk.to_vec();
            if chunk.len() == 1 {
                parents.push(SumMerkleNode::compute_parent(
                    &v[0],
                    &SumMerkleNode::create_empty(),
                ))
            } else {
                parents.push(SumMerkleNode::compute_parent(&v[0].clone(), &v[1].clone()))
            }
        }
        SumMerkleTree::generate(&parents)
    }

    /// Calculate merkle root
    pub fn get_root(&self) -> Bytes {
        self.tree.hash()
    }

    /// Returns inclusion proof for a leaf
    pub fn get_inclusion_proof(&self, idx: usize, count: usize) -> Vec<SumMerkleNode> {
        SumMerkleTree::get_inclusion_proof_of_tree(&self.tree, idx, count)
    }

    fn get_inclusion_proof_of_tree(
        tree: &SumMerkleNode,
        idx: usize,
        count: usize,
    ) -> Vec<SumMerkleNode> {
        match tree {
            SumMerkleNode::Leaf { .. } => vec![],
            SumMerkleNode::Node { left, right, .. } => {
                let left_count = count.next_power_of_two() / 2;
                if idx < left_count {
                    let mut proofs = Self::get_inclusion_proof_of_tree(left, idx, left_count);
                    proofs.push(SumMerkleNode::create_left_proof(&right));
                    proofs
                } else {
                    let mut proofs = Self::get_inclusion_proof_of_tree(
                        right,
                        idx - left_count,
                        count - left_count,
                    );
                    proofs.push(SumMerkleNode::create_right_proof(&left));
                    proofs
                }
            }
            SumMerkleNode::LeftProofNode { .. } => vec![],
            SumMerkleNode::RightProofNode { .. } => vec![],
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
        leaf: &SumMerkleNode,
        idx: usize,
        inclusion_proof: Vec<SumMerkleNode>,
        root: &Bytes,
    ) -> Result<ImplicitBounds, Error> {
        let mut path: Vec<bool> = vec![];
        Self::get_path(idx, inclusion_proof.len(), path.as_mut());
        let first_left_end = path
            .iter()
            .position(|&p| p)
            .map(|pos| inclusion_proof[pos].clone())
            .map_or(0, |n| n.get_end());
        let mut computed = leaf.clone();
        for (i, item) in inclusion_proof.iter().enumerate() {
            if path[i] {
                computed = SumMerkleNode::compute_parent(item, &computed)
            } else {
                computed = SumMerkleNode::compute_parent(&computed, item)
            }
        }
        let is_last_leaf = 2u64.pow(inclusion_proof.len() as u32) - 1 == (idx as u64);
        if computed.hash() == root {
            Ok(ImplicitBounds::new(
                first_left_end,
                if is_last_leaf {
                    u64::max_value()
                } else {
                    leaf.get_end()
                },
            ))
        } else {
            Err(Error::VerifyError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bytes;
    use super::Hashable;
    use super::SumMerkleNode;
    use super::SumMerkleTree;

    #[test]
    fn test_hash() {
        let hash_message = Bytes::from(&b"message"[..]);
        let leaf = SumMerkleNode::Leaf {
            end: 100,
            data: hash_message,
        };
        assert_eq!(leaf.hash().len(), 32);
    }

    #[test]
    fn test_compute_parent() {
        let hash_message1 = Bytes::from(&b"message"[..]);
        let leaf1 = SumMerkleNode::Leaf {
            end: 100,
            data: hash_message1,
        };
        let hash_message2 = Bytes::from(&b"message"[..]);
        let leaf2 = SumMerkleNode::Leaf {
            end: 200,
            data: hash_message2,
        };
        let parent = SumMerkleNode::compute_parent(&leaf1, &leaf2);
        assert_eq!(parent.get_end(), 200);
    }

    #[test]
    fn test_generate_tree() {
        let hash_message1 = Bytes::from(&b"message"[..]);
        let leaf1 = SumMerkleNode::Leaf {
            end: 100,
            data: hash_message1,
        };
        let hash_message2 = Bytes::from(&b"message"[..]);
        let leaf2 = SumMerkleNode::Leaf {
            end: 200,
            data: hash_message2,
        };
        let tree = SumMerkleTree::generate(&[leaf1, leaf2]);
        assert_eq!(tree.get_root().len(), 32);
    }

    #[test]
    fn test_proof() {
        let hash_message1 = Bytes::from(&b"message"[..]);
        let leaf1 = SumMerkleNode::Leaf {
            end: 100,
            data: hash_message1,
        };
        let hash_message2 = Bytes::from(&b"message"[..]);
        let leaf2 = SumMerkleNode::Leaf {
            end: 200,
            data: hash_message2,
        };
        let tree = SumMerkleTree::generate(&[leaf1.clone(), leaf2]);
        let inclusion_proof = tree.get_inclusion_proof(0, 2);
        assert_eq!(inclusion_proof.len(), 1);
        assert_eq!(
            SumMerkleTree::verify(&leaf1.clone(), 0, inclusion_proof, &tree.get_root()).is_ok(),
            true
        );
    }

    #[test]
    fn test_large_leaves() {
        let mut leaves = vec![];
        for i in 0..100 {
            leaves.push(SumMerkleNode::Leaf {
                end: i * 100 + 100,
                data: Bytes::from(&b"message"[..]),
            })
        }
        let tree = SumMerkleTree::generate(&leaves);
        let inclusion_proof = tree.get_inclusion_proof(5, 100);
        assert_eq!(inclusion_proof.len(), 7);
        assert_eq!(
            SumMerkleTree::verify(&leaves[5].clone(), 5, inclusion_proof, &tree.get_root()).is_ok(),
            true
        );
    }

}
