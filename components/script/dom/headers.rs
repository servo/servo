use dom::bindings::reflector::{Reflector};
use dom::bindings::str::{ByteString};

#[dom_struct]
pub struct Headers {
    reflector_: Reflector,
}

impl Headers {
    pub fn Append(&self, name: ByteString, value: ByteString) {
        unimplemented!()
    }
}

/// Removes trailing and leading HTTP whitespace bytes.
pub fn normalize(value: ByteString) -> ByteString {
    let opt_first_index = index_of_first_non_whitespace(&value);
    match opt_first_index {
        None => ByteString::new(vec![]),
        Some(0) => {
            let mut value: Vec<u8> = value.into();
            loop {
                match value.last().map(|ref_byte| *ref_byte) {
                    None => panic!("Should have found non-whitespace character first."),
                    Some(byte) if is_HTTP_whitespace(byte) => value.pop(),
                    Some(_) => return ByteString::new(value),
                };
            }
        }
        Some(first_index) => {
            let opt_last_index = index_of_last_non_whitespace(&value);
            match opt_last_index {
                None => panic!("Should have found non-whitespace character first."),
                Some(last_index) => {
                    let capacity = last_index - first_index + 1;
                    let mut normalized_value = Vec::with_capacity(capacity);
                    for byte in &value[first_index..last_index + 1] {
                        normalized_value.push(*byte);
                    }
                    ByteString::new(normalized_value)
                }
            }
        }
    }
}

fn is_HTTP_whitespace(byte: u8) -> bool {
    return byte == 0x09 ||
        byte == 0x0A ||
        byte == 0x0D ||
        byte == 0x20;
}

fn index_of_first_non_whitespace(value: &ByteString) -> Option<usize> {
    for (index, &byte) in value.iter().enumerate() {
        if is_HTTP_whitespace(byte) {
            continue;
        }
        return Some(index)
    }
    None
}

fn index_of_last_non_whitespace(value: &ByteString) -> Option<usize> {
    for (index, &byte) in value.iter().enumerate().rev() {
        if is_HTTP_whitespace(byte) {
            continue;
        }
        return Some(index)
    }
    None
}
