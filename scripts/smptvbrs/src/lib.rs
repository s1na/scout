extern crate elastic_array;
extern crate ewasm_api;
extern crate tiny_keccak;

mod nibbleslice;
mod node;
mod rlp;

use crate::rlp::{decode_length, RLPItem};
use ewasm_api::prelude::*;
use nibbleslice::NibbleSlice;
use node::Node;
use tiny_keccak::{keccak256, Keccak};

extern "C" {
    fn debug_startTimer();
    fn debug_endTimer();
    fn debug(v: u32);
    fn debugMem(p: *const u32, l: u32);
}

fn print_mem(slice: &[u8]) {
    unsafe { debugMem(slice.as_ptr() as *const u32, slice.len() as u32) }
}

fn decode_proof(data: &[u8]) -> Vec<&[u8]> {
    if data.len() == 0 {
        panic!("invalid rlp-encoded node");
    }

    // Proof should be a list
    let (mut offset, len) = match decode_length(data) {
        RLPItem::List(offset, len) => (offset, len),
        RLPItem::Str(_, _) => panic!("encode node should be a list"),
    };

    let data = &data[offset..offset + len];
    offset = 0;

    // Parse each node part, which should be a string
    let mut items = vec![];
    while offset < len {
        let (o, l) = match decode_length(&data[offset..]) {
            RLPItem::List(offset, len) => (offset, len),
            RLPItem::Str(offset, len) => (offset, len),
        };
        offset += o;
        items.push(&data[offset..offset + l]);
        offset += l;
    }

    items
}

fn verify_proof(root_hash: [u8; 32], key_path: NibbleSlice, proof_decoded: &Vec<&[u8]>) -> Vec<u8> {
    let mut want_hash: &[u8] = &root_hash;
    let mut path = key_path;
    let mut hash_ptr = [0u8; 32];

    for serialized_node in proof_decoded {
        Keccak::keccak256(serialized_node, &mut hash_ptr);
        /*print_mem(&h[..]);
        print_mem(want_hash);*/
        assert!(&hash_ptr == want_hash, "invalid proof node hash");

        let node = Node::from_rlp(serialized_node);
        match node {
            Node::Empty => panic!("empty node"),
            Node::Leaf(k, v) => {
                if k == path {
                    return Vec::from(v);
                } else {
                    panic!("leaf doesn't match path");
                }
            }
            Node::Extension(_, _) => {
                panic!("unimplemented");
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
                        Some(child) => {
                            path = path.mid(1);
                            want_hash = child;
                        }
                        None => panic!("Branch node empty child"),
                    }
                }
            }
        };
    }

    panic!("Shouldn't have reached here");
}

fn process_block(pre_state_root: Bytes32, block_data_bytes: &[u8]) -> Bytes32 {
    let root_hash = [
        101, 183, 185, 204, 183, 164, 216, 40, 152, 110, 72, 142, 8, 27, 193, 27, 250, 94, 246, 20,
        31, 176, 107, 10, 242, 233, 160, 189, 162, 115, 167, 83,
    ];
    let address_raw = [
        108, 223, 57, 216, 215, 85, 56, 160, 202, 215, 33, 242, 66, 118, 236, 72, 86, 46, 92, 144,
    ];
    let address_hash = keccak256(&address_raw);
    //let key_path = addressToNibbles(address_hash);
    let key_path = NibbleSlice::new(&address_hash);

    let proof_decoded = decode_proof(block_data_bytes);

    for i in 0..40 {
        /*unsafe {
            debug(i);
        }*/
        verify_proof(root_hash, key_path, &proof_decoded);
    }

    pre_state_root
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    let pre_state_root = eth2::load_pre_state_root();
    let block_data = eth2::acquire_block_data();
    let post_state_root = process_block(pre_state_root, &block_data);
    eth2::save_post_state_root(&post_state_root)
}

/*
fn addressToNibbles(address: [u8; 32]) -> [u8; 64] {
    let mut nibbles = [0u8; 64];

    let mut q = 0;
    for i in 0..32 {
        q = i * 2;
        nibbles[q] = address[i] >> 4;
        q += 1;
        nibbles[q] = address[i] % 16;
    }

    nibbles
}

fn remove_hex_prefix(nibbles: &[u8]) -> &[u8] {
    if nibbles[0] % 2 != 0 {
        &nibbles[1..]
    } else {
        &nibbles[2..]
    }
}

fn matching_nibble_length(nib1: &[u8], nib2: &[u8]) -> usize {
    let mut i = 0;
    for j in 0..nib1.len() {
        if nib1[j] != nib2[j] {
            break;
        }
        i += 1;
    }
    i
}*/
