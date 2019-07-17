pub enum RLPItem {
    Str(usize, usize),
    List(usize, usize),
}

pub fn decode_length(data: &[u8]) -> RLPItem {
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
