/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::indexeddb_thread::IndexedDBKeyType;

/// A module for encoding and decoding various IndexedDBKeyType types according to the basic strategy:
///
/// - Numbers:  0x10 + 8-byte sortable float
/// - Dates:    0x20 + 8-byte sortable float
/// - Strings:  0x30 + encoded chars + 0 terminator
/// - Binaries: 0x40 + encoded octets + 0 terminator
/// - Arrays:   0x50 + encoded items + 0 terminator

/// Internal helper: convert f64 to sortable u64 per IEEE754.
fn f64_to_sortable(value: f64) -> u64 {
    let bits = value.to_bits();
    if value.is_sign_negative() {
        bits.wrapping_neg()
    } else {
        bits | 0x8000_0000_0000_0000
    }
}

/// Internal helper: reverse sortable u64 back to f64.
fn sortable_to_f64(bits: u64) -> f64 {
    let orig = if bits & 0x8000_0000_0000_0000 != 0 {
        bits & 0x7FFF_FFFF_FFFF_FFFF
    } else {
        bits.wrapping_neg()
    };
    f64::from_bits(orig)
}

/// Encode a number (f64) with prefix 0x10.
pub fn encode_number(item: f64) -> Vec<u8> {
    let mut out = Vec::with_capacity(9);
    out.push(0x10);
    out.extend(&f64_to_sortable(item).to_be_bytes());
    out
}

/// Decode a number from a byte slice, returning (IndexedDBKeyType, bytes_consumed).
pub fn decode_number(input: &[u8]) -> Option<(f64, usize)> {
    if input.get(0)? != &0x10 || input.len() < 9 {
        return None;
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&input[1..9]);
    let bits = u64::from_be_bytes(arr);
    Some((sortable_to_f64(bits), 9))
}

/// Encode a date with prefix 0x20 and terminator 0x00.
pub fn encode_date(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 2 + 2);
    out.push(0x20);
    for &byte in data {
        if byte <= 0x7E {
            out.push(byte.wrapping_add(1));
        } else {
            let v = (byte as u32).wrapping_sub(0x7F);
            out.push(0x80 | (((v >> 8) as u8) & 0x3F));
            out.push((v & 0xFF) as u8);
        }
    }
    out.push(0x00);
    out
}

/// Decode date, returning (Vec<u8>, bytes_consumed).
pub fn decode_date(input: &[u8]) -> Option<(Vec<u8>, usize)> {
    if input.get(0)? != &0x20 {
        return None;
    }
    let mut idx = 1;
    let mut out = Vec::new();
    while idx < input.len() {
        let b = input[idx];
        if b == 0x00 {
            idx += 1;
            break;
        }
        if b & 0x80 == 0 {
            out.push(b.wrapping_sub(1));
            idx += 1;
        } else if b & 0xC0 == 0x80 {
            let hi = (b & 0x3F) as u32;
            let lo = *input.get(idx + 1)? as u32;
            let v = ((hi << 8) | lo).wrapping_add(0x7F);
            out.push(v as u8);
            idx += 2;
        } else {
            return None;
        }
    }
    Some((out, idx))
}

/// Encode a string with prefix 0x30 and terminator 0x00.
pub fn encode_string(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() + 2);
    out.push(0x30);
    for ch in s.chars() {
        let cp = ch as u32;
        if cp <= 0x7E {
            out.push((cp as u8).wrapping_add(1));
        } else if cp <= (0x3FFF + 0x7F) {
            let v = cp - 0x7F;
            out.push(0x80 | (((v >> 8) as u8) & 0x3F));
            out.push((v & 0xFF) as u8);
        } else if cp <= 0xFFFF {
            let v = cp - (0x3FFF + 0x80);
            out.push(0xC0 | (((v >> 16) as u8) & 0x3F));
            out.push(((v >> 8) & 0xFF) as u8);
            out.push(((v & 0xFF) << 6) as u8);
        } else {
            panic!("Code point out of range: {}", cp);
        }
    }
    out.push(0x00);
    out
}

/// Decode a string, returning (String, bytes_consumed).
pub fn decode_string(input: &[u8]) -> Option<(String, usize)> {
    if input.get(0)? != &0x30 {
        return None;
    }
    let mut idx = 1;
    let mut buf = String::new();
    while idx < input.len() {
        let b = input[idx];
        if b == 0x00 {
            idx += 1;
            break;
        }
        if b & 0x80 == 0 {
            // 1-byte
            let cp = b.wrapping_sub(1) as u32;
            buf.push(std::char::from_u32(cp)?);
            idx += 1;
        } else if b & 0xC0 == 0x80 {
            // 2-byte
            let hi = (b & 0x3F) as u32;
            let lo = input.get(idx + 1)?;
            let v = ((hi << 8) | (*lo as u32)).wrapping_add(0x7F);
            buf.push(std::char::from_u32(v)?);
            idx += 2;
        } else if b & 0xC0 == 0xC0 {
            // 3-byte
            let hi = (b & 0x3F) as u32;
            let mid = *input.get(idx + 1)? as u32;
            let lo = *input.get(idx + 2)? as u32;
            let v = ((hi << 16) | (mid << 8) | (lo >> 6)).wrapping_add(0x3FFF + 0x80);
            buf.push(std::char::from_u32(v)?);
            idx += 3;
        } else {
            return None;
        }
    }
    Some((buf, idx))
}

/// Encode binary data with prefix 0x40 and terminator 0x00.
pub fn encode_binary(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 2 + 2);
    out.push(0x40);
    for &byte in data {
        if byte <= 0x7E {
            out.push(byte.wrapping_add(1));
        } else {
            let v = (byte as u32).wrapping_sub(0x7F);
            out.push(0x80 | (((v >> 8) as u8) & 0x3F));
            out.push((v & 0xFF) as u8);
        }
    }
    out.push(0x00);
    out
}

/// Decode binary, returning (Vec<u8>, bytes_consumed).
pub fn decode_binary(input: &[u8]) -> Option<(Vec<u8>, usize)> {
    if input.get(0)? != &0x40 {
        return None;
    }
    let mut idx = 1;
    let mut out = Vec::new();
    while idx < input.len() {
        let b = input[idx];
        if b == 0x00 {
            idx += 1;
            break;
        }
        if b & 0x80 == 0 {
            out.push(b.wrapping_sub(1));
            idx += 1;
        } else if b & 0xC0 == 0x80 {
            let hi = (b & 0x3F) as u32;
            let lo = *input.get(idx + 1)? as u32;
            let v = ((hi << 8) | lo).wrapping_add(0x7F);
            out.push(v as u8);
            idx += 2;
        } else {
            return None;
        }
    }
    Some((out, idx))
}

/// Encode an array of already-encoded items with prefix 0x50 and terminator 0x00.
pub fn encode_array(items: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x50);
    for item in items {
        out.extend(item);
    }
    // array terminator
    out.push(0x00);
    out
}

/// Decode an array, returning (Vec<IndexedDBKeyType>, bytes_consumed).
pub fn decode_array(input: &[u8]) -> Option<(Vec<IndexedDBKeyType>, usize)> {
    if input.get(0)? != &0x50 {
        return None;
    }
    let mut idx = 1;
    let mut items = Vec::new();
    while idx < input.len() {
        if input[idx] == 0x00 {
            idx += 1;
            break;
        }
        let (indexed_db_key_type, consumed) = match input[idx] {
            0x10 => {
                let (v, n) = decode_number(&input[idx..])?;
                (IndexedDBKeyType::Number(v), n)
            },
            0x20 => {
                let (v, n) = decode_date(&input[idx..])?;
                (IndexedDBKeyType::Date(v), n)
            },
            0x30 => {
                let (v, n) = decode_string(&input[idx..])?;
                (IndexedDBKeyType::String(v), n)
            },
            0x40 => {
                let (v, n) = decode_binary(&input[idx..])?;
                (IndexedDBKeyType::Binary(v), n)
            },
            0x50 => {
                let (v, n) = decode_array(&input[idx..])?;
                (IndexedDBKeyType::Array(v), n)
            },
            _ => return None,
        };
        items.push(indexed_db_key_type);
        idx += consumed;
    }
    Some((items, idx))
}

pub fn encode(item: &IndexedDBKeyType) -> Vec<u8> {
    match item {
        IndexedDBKeyType::Number(v) => encode_number(*v),
        IndexedDBKeyType::Date(f) => encode_date(f),
        IndexedDBKeyType::String(v) => encode_string(v),
        IndexedDBKeyType::Binary(v) => encode_binary(v),
        IndexedDBKeyType::Array(v) => {
            let items: Vec<_> = v.iter().map(encode).collect();
            encode_array(&items)
        },
    }
}

pub fn decode(bytes: &[u8]) -> Option<IndexedDBKeyType> {
    match bytes[0] {
        0x10 => {
            let (v, _n) = decode_number(&bytes[1..])?;
            Some(IndexedDBKeyType::Number(v))
        },
        0x20 => {
            let (v, _n) = decode_date(&bytes[1..])?;
            Some(IndexedDBKeyType::Date(v))
        },
        0x30 => {
            let (v, _n) = decode_string(&bytes[1..])?;
            Some(IndexedDBKeyType::String(v))
        },
        0x40 => {
            let (v, _n) = decode_binary(&bytes[1..])?;
            Some(IndexedDBKeyType::Binary(v))
        },
        0x50 => {
            let (v, _n) = decode_array(&bytes[1..])?;
            Some(IndexedDBKeyType::Array(v))
        },
        _ => None,
    }
}

// Tests for the codec module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_number() {
        let v = 42.125;
        let enc = encode_number(v);
        let (dec, _) = decode_number(&enc).unwrap();
        assert_eq!(v, dec);
    }

    #[test]
    fn roundtrip_date() {
        let t = 1_625_097_600.0;
        let enc = encode_date(t);
        let (dec, _) = decode_date(&enc).unwrap();
        assert_eq!(t, dec);
    }

    #[test]
    fn roundtrip_string() {
        let s = "Hello test";
        let enc = encode_string(s);
        let (dec, _) = decode_string(&enc).unwrap();
        assert_eq!(s, dec);
    }

    #[test]
    fn roundtrip_binary() {
        let b = vec![0x00, 0x7E, 0xFF];
        let enc = encode_binary(&b);
        let (dec, _) = decode_binary(&enc).unwrap();
        assert_eq!(b, dec);
    }

    #[test]
    fn roundtrip_array() {
        let items = vec![
            IndexedDBKeyType::Number(1.0),
            IndexedDBKeyType::String("A".into()),
            IndexedDBKeyType::Binary(vec![1, 2, 3]),
        ];
        let enc_items: Vec<_> = items
            .iter()
            .map(|item| match item {
                IndexedDBKeyType::Number(v) => encode_number(*v),
                IndexedDBKeyType::String(s) => encode_string(s),
                IndexedDBKeyType::Binary(b) => encode_binary(b),
                _ => Vec::new(),
            })
            .collect();
        let enc = encode_array(&enc_items);
        let (dec, _) = decode_array(&enc).unwrap();
        assert_eq!(items, dec);
    }
}
