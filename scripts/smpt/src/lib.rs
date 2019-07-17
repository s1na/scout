extern crate ethereum_types;
extern crate ewasm_api;
extern crate rlp as rlplib;

mod account;
mod blockdata;
mod keccak_hasher;
mod rlp;
mod sig;
mod trie;
mod tx;

use crate::keccak_hasher::KeccakHasher;
use account::BasicAccount;
use blockdata::BlockData;
use ethereum_types::{H256, U256};
use ewasm_api::prelude::*;
use rlplib::{Decodable, DecoderError, Rlp};
use sig::recover_address;
use trie::Trie;
use tx::{Tx, UnsignedTx};

extern "C" {
    fn debug_startTimer();
    fn debug_endTimer();
}

fn process_block(pre_state_root: Bytes32, block_data_bytes: &[u8]) -> Bytes32 {
    // Takes ~25ms
    let block_data: BlockData = rlplib::decode(&block_data_bytes).unwrap();

    let proof_nodes_count = block_data.proof_nodes.len();

    let mut trie = Trie::with_capacity(proof_nodes_count);

    for item in block_data.proof_nodes {
        trie.db_insert(item);
    }

    let mut root = H256::from(pre_state_root.bytes);
    // Up to this point: 53ms keccak256, 26ms memcpy

    for tx in block_data.txes {
        let from_address = tx.from;
        let value = trie.get(&root, from_address);
        let from_account = BasicAccount::from_rlp(&value);
        let value2 = trie.get(&root, tx.to);
        let to_account = BasicAccount::from_rlp(&value);
        /*let new_nodes = trie.verify_and_update(&root, from_address, tx.value, true);
        for n in new_nodes {
            root = trie.db_insert(&n);
        }
        let new_nodes = trie.verify_and_update(&root, tx.to, tx.value, false);
        for n in new_nodes {
            root = trie.db_insert(&n);
        }*/
    }

    Bytes32::from(root.as_fixed_bytes())
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    let pre_state_root = eth2::load_pre_state_root();
    let block_data = eth2::acquire_block_data();
    let post_state_root = process_block(pre_state_root, &block_data);
    eth2::save_post_state_root(&post_state_root)
}
