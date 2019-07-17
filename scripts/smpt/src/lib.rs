extern crate ethereum_types;
extern crate ewasm_api;

mod account;
mod keccak_hasher;
mod sig;
mod trie;
mod tx;

use crate::keccak_hasher::KeccakHasher;
use account::BasicAccount;
use ethereum_types::{H256, U256};
use ewasm_api::prelude::*;
use rlp::{DecoderError, Rlp};
use sig::recover_address;
use trie::Trie;
use tx::{Tx, UnsignedTx};

extern "C" {
    fn debug_startTimer();
    fn debug_endTimer();
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockData {
    txes: Vec<Tx>,
    proof_nodes: Vec<Vec<u8>>,
}

impl rlp::Decodable for BlockData {
    fn decode(d: &Rlp) -> Result<Self, DecoderError> {
        if d.item_count()? != 2 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        Ok(BlockData {
            txes: d.list_at(0)?,
            proof_nodes: d.list_at(1)?,
        })
    }
}

fn process_block(pre_state_root: Bytes32, block_data_bytes: &[u8]) -> Bytes32 {
    let block_data: BlockData = rlp::decode(&block_data_bytes).unwrap();

    let proof_nodes_count = block_data.proof_nodes.len();

    let mut trie = Trie::with_capacity(proof_nodes_count);

    for item in block_data.proof_nodes {
        trie.db_insert(item);
    }

    let mut root = H256::from_slice(&pre_state_root.bytes[..]);
    for tx in block_data.txes {
        let from_address = tx.from;
        let value = trie.get(&root, from_address);
        let from_account: BasicAccount = rlp::decode(&value).unwrap();
        let value2 = trie.get(&root, tx.to);
        let to_account: BasicAccount = rlp::decode(&value).unwrap();
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
