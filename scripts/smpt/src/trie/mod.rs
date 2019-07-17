extern crate trie_db as trie;

mod nibbleslice;
mod node;
//mod rlp_node_codec;

use super::BasicAccount;
use super::KeccakHasher;
use ethereum_types::{Address, H256, U256};
use kvdb::DBValue;
use nibbleslice::NibbleSlice;
use node::Node;
//use rlp_node_codec::{RlpNodeCodec, HASHED_NULL_NODE_BYTES};
use std::collections::HashMap;
use tiny_keccak::keccak256;

pub const HASHED_NULL_NODE_BYTES: [u8; 32] = [
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
];

pub struct Trie {
    db: HashMap<H256, DBValue>,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            db: HashMap::default(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Trie {
            db: HashMap::with_capacity(cap),
        }
    }

    pub fn db_get(&self, key: &H256) -> Option<&DBValue> {
        self.db.get(key)
        /*match self.db.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }*/
    }

    pub fn db_insert(&mut self, value: &[u8]) -> H256 {
        //let mut out = [0u8; 32];
        //tiny_keccak::Keccak::keccak256(value, &mut out);
        let key: H256 = H256::from(keccak256(value));
        //let key: H256 = out.into();
        self.db.insert(key, value.into());
        key
    }

    fn get_node<'a>(&'a self, node_hash: &'a [u8]) -> Node<'a> {
        if node_hash == HASHED_NULL_NODE_BYTES || node_hash == rlp::NULL_RLP {
            return Node::Empty;
        }

        if node_hash.len() < 32 {
            Node::from(node_hash)
        //Node::from_rlp(node_hash).expect("ok decode")
        } else {
            let encoded = self.db_get(&H256::from_slice(node_hash)).unwrap();
            Node::from(&encoded)
            //Node::from_rlp(&encoded).expect("ok decode")
        }
    }

    pub fn get(&self, root: &H256, key: Address) -> Vec<u8> {
        let key = keccak256(key.as_bytes());
        let mut node = self.get_node(root.as_bytes());
        let mut path = NibbleSlice::new(&key);
        loop {
            let (mid, child) = match node {
                Node::Empty => panic!("empty node"),
                Node::Leaf(k, v) => {
                    if k == path {
                        return Vec::from(v);
                    } else {
                        panic!("leaf doesn't match path");
                    }
                }
                Node::Extension(k, child) => {
                    if path.starts_with(&k) {
                        (k.len(), self.get_node(child))
                    } else {
                        panic!("not matching extension prefix");
                    }
                }
                Node::Branch(children, value) => {
                    if path.is_empty() {
                        match value {
                            Some(v) => {
                                return Vec::from(v);
                            }
                            None => panic!("No value in branch"),
                        }
                    } else {
                        let idx = path.at(0);
                        match children[idx as usize] {
                            Some(child) => (1, self.get_node(child)),
                            None => panic!("Branch node empty child"),
                        }
                    }
                }
            };

            path = path.mid(mid);
            node = child;
        }
    }

    pub fn verify_and_update(
        &mut self,
        root: &H256,
        key: Address,
        value: U256,
        sub: bool,
    ) -> Vec<Vec<u8>> {
        //fn get_trace(&self, root: &H256, key: Address) -> (&[u8], Vec<Node>, Vec<u8>) {
        let key = keccak256(key.as_bytes());
        //println!("hashed key: {:?}", key);
        let mut node = self.get_node(root.as_bytes());
        //println!("root node: {:?}", node);
        let mut path = NibbleSlice::new(&key);
        let encoded;
        //let mut trace = vec![];
        let mut branch_indices = vec![];
        loop {
            //println!("verify loop, path: {:?}", path);
            //println!("node: {:?}", node);
            //trace.push(node.clone());
            let (mid, child) = match node {
                Node::Empty => panic!("empty node"),
                Node::Leaf(k, v) => {
                    //println!(">>Leaf node: {:?}", v);
                    if k == path {
                        encoded = v;
                        break;
                    } else {
                        panic!("leaf doesn't match path");
                    }
                }
                Node::Extension(k, child) => {
                    //println!(">>Extension node: {:?}", k);
                    if path.starts_with(&k) {
                        (k.len(), self.get_node(child))
                    } else {
                        panic!("not matching extension prefix");
                    }
                }
                Node::Branch(children, value) => {
                    if path.is_empty() {
                        match value {
                            Some(v) => {
                                branch_indices.push(16);
                                //println!(">>Branch with value: {:?}", v);
                                encoded = v;
                                break;
                            }
                            None => panic!("No value in branch"),
                        }
                    } else {
                        let idx = path.at(0);
                        branch_indices.push(idx);
                        //println!(">>branch nibble: {:?}", idx);
                        match children[idx as usize] {
                            Some(child) => {
                                //println!("branch child({}): {:?}", child.len(), child);
                                (1, self.get_node(child))
                            }
                            None => panic!("Branch node empty child"),
                        }
                    }
                }
            };

            path = path.mid(mid);
            node = child;
        }

        vec![]
        /*trace.reverse();

        let mut account: BasicAccount = rlp::decode(encoded).unwrap();
        //println!("got account: {:?}", account);
        let mut new_encoded;
        if sub {
            // Sender
            assert!(account.balance >= value, "sender has enough balance");
            account.balance -= value;
            account.nonce += U256::one();
            new_encoded = rlp::encode(&account);
        } else {
            account.balance += value;
            new_encoded = rlp::encode(&account);
        }
        //println!("updated account: {:?}", account);
        //println!(
        //    "serialized update account: {:?}",
        //    to_hex_string(&new_encoded)
        //);
        let mut new_nodes = vec![];
        for node in trace.iter() {
            new_encoded = match node {
                Node::Empty => panic!("no empty supported"),
                Node::Leaf(k, v) => {
                    //println!("%%Leaf node: {:?}", new_encoded);
                    RlpCodec::leaf_node(&k.encoded(true), &new_encoded)
                }
                Node::Extension(k, v) => {
                    //println!("Huhhhh??????");
                    if new_encoded.len() >= 32 {
                        rlp_node_codec::ext_node(&k.encoded(false), &keccak256(&new_encoded))
                    } else {
                        rlp_node_codec::ext_node(&k.encoded(false), &new_encoded)
                    }
                }
                Node::Branch(children, v) => {
                    let branch_idx = branch_indices.pop().unwrap();
                    //println!("%%Branch node: {:?}", branch_idx);
                    if branch_idx == 16 {
                        rlp_node_codec::branch_node(
                            *children,
                            Some(elastic_array::ElasticArray128::from_slice(&new_encoded)),
                        )
                    } else {
                        let mut children = children.clone();
                        let mut h = [0; 32];
                        if new_encoded.len() >= 32 {
                            h = keccak256(&new_encoded.clone());
                            children[branch_idx as usize] = Some(&h);
                        } else {
                            children[branch_idx as usize] = Some(&new_encoded);
                        }
                        if let Some(v) = v {
                            rlp_node_codec::branch_node(
                                children,
                                Some(elastic_array::ElasticArray128::from_slice(v)),
                            )
                        } else {
                            rlp_node_codec::branch_node(children, None)
                        }
                    }
                }
            };
            new_nodes.push(Vec::from(new_encoded.clone()));
            //self.db_insert(&new_encoded);
        }

        // Consumed all branch indices
        assert!(branch_indices.len() == 0);

        new_nodes*/
    }
}

/*struct MemoryDB {
    db: HashMap<H256, DBValue>,
}

impl MemoryDB {
    fn new() -> Self {
        MemoryDB {
            db: HashMap::default(),
        }
    }

    fn with_capacity(cap: usize) -> Self {
        MemoryDB {
            db: HashMap::with_capacity(cap),
        }
    }

    fn get(&self, key: &H256) -> Option<&DBValue> {
        self.db.get(key)
    }

    fn insert(&mut self, value: &[u8]) -> H256 {
        let key = H256::from(keccak256(value));
        self.db.insert(key, value.into());
        key
    }
}*/
