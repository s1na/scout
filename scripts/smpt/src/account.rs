use crate::rlp::{decode_length, RLPItem};
use ethereum_types::{H256, U256};
use rlplib::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicAccount {
    pub nonce: U256,
    pub balance: U256,
    pub storage_root: H256,
    pub code_hash: H256,
}

impl BasicAccount {
    pub fn from_rlp(data: &[u8]) -> Self {
        if data.len() == 0 {
            panic!("invalid rlp-encoded node");
        }

        // Account should be a list
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
                RLPItem::Str(offset, len) => (offset, len),
                RLPItem::List(_, _) => panic!("node part should be str"),
            };
            offset += o;
            items.push(&data[offset..offset + l]);
            offset += l;
        }

        BasicAccount {
            nonce: U256::from_big_endian(&items[0]),
            balance: U256::from_big_endian(&items[1]),
            storage_root: H256::from_slice(&items[2]),
            code_hash: H256::from_slice(&items[3]),
        }
    }
}

impl Decodable for BasicAccount {
    fn decode(d: &Rlp) -> Result<Self, DecoderError> {
        if d.item_count()? != 4 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        Ok(BasicAccount {
            nonce: d.val_at(0)?,
            balance: d.val_at(1)?,
            storage_root: d.val_at(2)?,
            code_hash: d.val_at(3)?,
        })
    }
}

impl Encodable for BasicAccount {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4);
        s.append(&self.nonce);
        s.append(&self.balance);
        s.append(&self.storage_root);
        s.append(&self.code_hash);
    }
}
