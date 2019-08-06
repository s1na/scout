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
        let str_len = to_u32(&data[1..(1 + len_of_str_len) as usize]);
        RLPItem::Str((1 + len_of_str_len) as usize, str_len as usize)
    } else if first_byte <= 0xf7 {
        RLPItem::List(1, (first_byte - 0xc0) as usize)
    } else {
        let len_of_list_len: u8 = first_byte - 0xf7;
        //let mut list_len_bytes = [0u8; 4]; // limit list len to u32
        /*let start_idx = 4 - len_of_list_len;
        for i in 0..len_of_list_len as usize {
            list_len_bytes[start_idx as usize + i] = data[1 + i];
        }*/
        //let list_len = u32::from_be_bytes(list_len_bytes);
        let list_len = to_u32(&data[1..(1 + len_of_list_len) as usize]);
        RLPItem::List((1 + len_of_list_len) as usize, list_len as usize)
    }
}

fn to_u32(slice: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    let mut shifts = 0;
    for b in slice.iter().rev() {
        sum += (*b as u32) << shifts;
        shifts += 8;
    }
    sum
}
