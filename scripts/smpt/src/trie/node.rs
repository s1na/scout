use super::nibbleslice::NibbleSlice;
use rlp::{DecoderError, Prototype, Rlp};

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

pub enum RLPItem {
    Str(usize, usize),
    List(usize, usize),
}

impl<'a> Node<'a> {
    pub fn from(data: &'a [u8]) -> Self {
        if data.len() == 0 {
            panic!("invalid rlp-encoded node");
        }

        // Node should be a list
        let (mut offset, len) = match Node::decode_length(data) {
            RLPItem::List(offset, len) => (offset, len),
            RLPItem::Str(_, _) => panic!("encode node should be a list"),
        };

        let data = &data[offset..offset + len];
        offset = 0;

        // Parse each node part, which should be a string
        let mut items = vec![];
        while offset < len {
            let (o, l) = match Node::decode_length(&data[offset..]) {
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

    fn decode_length(data: &[u8]) -> RLPItem {
        let len = data.len();
        if len == 0 {
            panic!("rlp len is zero");
        }
        let first_byte: u8 = data[0];

        if first_byte <= 0x7f {
            RLPItem::Str(0, 1)
        } else if first_byte <= 0xb7 {
            RLPItem::Str(1, (first_byte - 0x80) as usize)
        } else if first_byte <= 0xbf {
            let len_of_str_len = first_byte - 0xb7;
            let mut str_len_bytes = [0u8; 4]; // limit list len to u32
            let start_idx = 4 - len_of_str_len;
            for i in 0..len_of_str_len as usize {
                str_len_bytes[start_idx as usize + i] = data[1 + i];
            }
            let str_len = u32::from_be_bytes(str_len_bytes);
            RLPItem::Str((1 + len_of_str_len) as usize, str_len as usize)
        } else if first_byte <= 0xf7 {
            RLPItem::List(1, (first_byte - 0xc0) as usize)
        } else {
            let len_of_list_len: u8 = first_byte - 0xf7;
            let mut list_len_bytes = [0u8; 4]; // limit list len to u32
            let start_idx = 4 - len_of_list_len;
            for i in 0..len_of_list_len as usize {
                list_len_bytes[start_idx as usize + i] = data[1 + i];
            }
            let list_len = u32::from_be_bytes(list_len_bytes);
            RLPItem::List((1 + len_of_list_len) as usize, list_len as usize)
        }
    }

    /*
    pub fn from_rlp(data: &'a [u8]) -> Result<Self, DecoderError> {
        let r = Rlp::new(data);
        match r.prototype()? {
            // either leaf or extension - decode first item with NibbleSlice::???
            // and use is_leaf return to figure out which.
            // if leaf, second item is a value (is_data())
            // if extension, second item is a node (either SHA3 to be looked up and
            // fed back into this function or inline RLP which can be fed back into this function).
            Prototype::List(2) => match NibbleSlice::from_encoded(r.at(0)?.data()?) {
                (slice, true) => Ok(Node::Leaf(slice, r.at(1)?.data()?)),
                (slice, false) => Ok(Node::Extension(slice, r.at(1)?.data()?)),
            },
            // branch - first 16 are nodes, 17th is a value (or empty).
            Prototype::List(17) => {
                let mut nodes = [None as Option<&[u8]>; 16];
                for i in 0..16 {
                    let v = r.at(i)?;
                    if v.is_empty() {
                        nodes[i] = None;
                    } else {
                        nodes[i] = Some(v.data()?);
                    }
                }
                Ok(Node::Branch(
                    nodes,
                    if r.at(16)?.is_empty() {
                        None
                    } else {
                        Some(r.at(16)?.data()?)
                    },
                ))
            }
            // an empty branch index.
            Prototype::Data(0) => Ok(Node::Empty),
            // something went wrong.
            _ => Err(DecoderError::Custom("Rlp is not valid.")),
        }
    }*/
}
