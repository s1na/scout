use crate::tx::Tx;
use rlplib::{Decodable, DecoderError, Rlp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockData {
    pub txes: Vec<Tx>,
    pub proof_nodes: Vec<Vec<u8>>,
}

impl Decodable for BlockData {
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
