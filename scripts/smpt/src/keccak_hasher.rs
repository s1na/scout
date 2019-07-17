// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Hasher implementation for the Keccak-256 hash
//! Taken from:
//! https://github.com/paritytech/parity-ethereum/blob/6bb106a784678cc2cadabfd621981371f477c48d/util/keccak-hasher/src/lib.rs
extern crate ethereum_types;
extern crate hash_db;
extern crate plain_hasher;
extern crate tiny_keccak;

use ethereum_types::H256;
use hash_db::Hasher;
use plain_hasher::PlainHasher;
use tiny_keccak::Keccak;

pub static mut hash_count: u32 = 0;

extern "C" {
    pub fn util_keccak(outputOffset: *const u32, offset: *const u32, length: u32);
}

/// Concrete `Hasher` impl for the Keccak-256 hash
#[derive(Default, Debug, Clone, PartialEq)]
pub struct KeccakHasher;
impl Hasher for KeccakHasher {
    type Out = H256;
    type StdHasher = PlainHasher;
    const LENGTH: usize = 32;
    fn hash(x: &[u8]) -> Self::Out {
        unsafe {
            hash_count += 1;
        }
        let mut out = [0u8; 32];
        /*unsafe {
            util_keccak(
                out.as_mut_ptr() as *const u32,
                x.as_ptr() as *const u32,
                x.len() as u32,
            );
        }*/
        Keccak::keccak256(x, &mut out);
        out.into()
        //H256::from_slice(&out[..])
    }
}
