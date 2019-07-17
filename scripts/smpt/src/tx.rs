use crate::rlp::{decode_length, RLPItem};
use crate::sig::Sig;
use ethereum_types::{Address, U256};
use rlplib::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    pub to: Address,
    pub value: U256,
    pub nonce: U256,
    pub from: Address,
    //pub sig: Vec<u8>,
    //pub sig: Sig,
}

impl Tx {
    pub fn from_rlp(data: &[u8]) -> Self {
        if data.len() == 0 {
            panic!("invalid rlp-encoded node");
        }

        // Tx should be a list
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

        Tx {
            to: Address::from_slice(&items[0]),
            value: U256::from_big_endian(&items[1]),
            nonce: U256::from_big_endian(&items[2]),
            from: Address::from_slice(&items[3]),
        }
    }
}

impl Decodable for Tx {
    fn decode(d: &Rlp) -> Result<Self, DecoderError> {
        if d.item_count()? != 4 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        Ok(Tx {
            to: d.val_at(0)?,
            value: d.val_at(1)?,
            nonce: d.val_at(2)?,
            from: d.val_at(3)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsignedTx {
    pub to: Address,
    pub value: U256,
    pub nonce: U256,
}

impl Encodable for UnsignedTx {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        s.append(&self.to);
        s.append(&self.value);
        s.append(&self.nonce);
    }
}
