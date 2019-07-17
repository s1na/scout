use super::nibbleslice::NibbleSlice;
use crate::rlp::{decode_length, RLPItem};

/// Type of node in the trie and essential information thereof.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Node<'a> {
    /// Null trie node; could be an empty root or an empty branch entry.
    Empty,
    /// Leaf node; has key slice and value. Value may not be empty.
    Leaf(NibbleSlice<'a>, &'a [u8]),
    /// Extension node; has key slice and node data. Data may not be null.
    Extension(NibbleSlice<'a>, &'a [u8]),
    /// Branch node; has array of 16 child nodes (each possibly null) and an optional immediate node data.
    Branch([Option<&'a [u8]>; 16], Option<&'a [u8]>),
}

impl<'a> Node<'a> {
    pub fn from_rlp(data: &'a [u8]) -> Self {
        if data.len() == 0 {
            panic!("invalid rlp-encoded node");
        }

        // Node should be a list
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

        if items.len() == 2 {
            match NibbleSlice::from_encoded(items[0]) {
                (slice, true) => Node::Leaf(slice, items[1]),
                (slice, false) => Node::Extension(slice, items[1]),
            }
        } else if items.len() == 17 {
            let mut children = [None as Option<&[u8]>; 16];
            for i in 0..16 {
                if items[i].len() > 0 {
                    children[i] = Some(items[i]);
                }
            }
            if items[16].len() > 0 {
                Node::Branch(children, Some(items[16]))
            } else {
                Node::Branch(children, None)
            }
        } else {
            panic!("unknown number of node parts");
        }
    }
}
