/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use storage_traits::indexeddb::IndexedDBKeyType;

// Implementation is a port of:
// https://searchfox.org/firefox-main/rev/55ec080e4a37b7ae1f89267063eccd361cdd232d/dom/indexedDB/Key.cpp#109-187

#[repr(u8)]
enum KeyType {
    Number = 0x10,
    Date = 0x20,
    String = 0x30,
    Binary = 0x40,
    Array = 0x50,
}

impl From<KeyType> for u8 {
    fn from(key_type: KeyType) -> Self {
        key_type as u8
    }
}

impl TryFrom<u8> for KeyType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(KeyType::Number),
            0x20 => Ok(KeyType::Date),
            0x30 => Ok(KeyType::String),
            0x40 => Ok(KeyType::Binary),
            0x50 => Ok(KeyType::Array),
            _ => Err(()),
        }
    }
}

/// When encoding floats, 64bit IEEE 754 are almost sortable, except that
/// positive sort lower than negative, and negative sort descending. So we use
/// the following encoding:
///
/// value < 0 ?
/// (-to64bitInt(value)) :
/// (to64bitInt(value) | 0x8000000000000000)
pub fn serialize_number(n: f64) -> [u8; 8] {
    let bits = n.to_bits();
    let signbit = 1u64 << 63;
    let number = if (bits & signbit) != 0 {
        bits.wrapping_neg()
    } else {
        bits | signbit
    };
    number.to_be_bytes()
}

pub fn deserialize_number(bytes: &[u8]) -> Option<f64> {
    let mut array = [0u8; 8];
    let len = std::cmp::min(8, bytes.len());
    array[..len].copy_from_slice(&bytes[..len]);
    let number = u64::from_be_bytes(array);
    let signbit = 1u64 << 63;
    let bits = if (number & signbit) != 0 {
        number & !signbit
    } else {
        number.wrapping_neg()
    };
    Some(f64::from_bits(bits))
}

/// When encoding strings, we use variable-size encoding per the following table
///
///  Chars 0         - 7E           are encoded as 0xxxxxxx with 1 added
///  Chars 7F        - (3FFF+7F)    are encoded as 10xxxxxx xxxxxxxx with 7F subtracted
///  Chars (3FFF+80) - FFFF         are encoded as 11xxxxxx xxxxxxxx xx000000
fn encode_stringy<I: Iterator<Item = u16>>(iter: I, type_byte: u8, buffer: &mut Vec<u8>) {
    buffer.push(type_byte);
    for val in iter {
        if val <= 0x7E {
            buffer.push((val + 1) as u8);
        } else if val <= 0x3FFF + 0x7F {
            let c = val.wrapping_sub(0x7F).wrapping_add(0x8000);
            buffer.push((c >> 8) as u8);
            buffer.push((c & 0xFF) as u8);
        } else {
            let c = ((val as u32) << 6) | 0x00C00000;
            buffer.push((c >> 16) as u8);
            buffer.push((c >> 8) as u8);
            buffer.push(c as u8);
        }
    }
    buffer.push(0);
}

/// Tracks type offsets for arrays to ensure that we don't have collisions with type bytes. When we hit the limit, we push an extra byte to separate the levels.
fn internal_serialize(key: &IndexedDBKeyType, type_offset: u8, buffer: &mut Vec<u8>) {
    match key {
        IndexedDBKeyType::Number(n) => {
            buffer.push(KeyType::Number as u8 + type_offset);
            buffer.extend_from_slice(&serialize_number(*n));
        },
        IndexedDBKeyType::Date(n) => {
            buffer.push(KeyType::Date as u8 + type_offset);
            buffer.extend_from_slice(&serialize_number(*n));
        },
        IndexedDBKeyType::String(s) => {
            encode_stringy(
                s.encode_utf16(),
                KeyType::String as u8 + type_offset,
                buffer,
            );
        },
        IndexedDBKeyType::Binary(b) => {
            encode_stringy(
                b.iter().map(|&x| x as u16),
                KeyType::Binary as u8 + type_offset,
                buffer,
            );
        },
        IndexedDBKeyType::Array(arr) => {
            let mut offset = type_offset + KeyType::Array as u8;
            if offset == KeyType::Array as u8 * 3 {
                buffer.push(offset);
                offset = 0;
            }
            for item in arr {
                internal_serialize(item, offset, buffer);
                offset = 0;
            }
            buffer.push(offset); // Array terminator
        },
    }
}

pub fn serialize(key: &IndexedDBKeyType) -> Vec<u8> {
    let mut buffer = Vec::new();
    internal_serialize(key, 0, &mut buffer);
    while buffer.last() == Some(&0) {
        buffer.pop();
    }
    buffer
}

fn decode_stringy(buffer: &[u8], pos: &mut usize) -> Option<Vec<u16>> {
    let mut decoded = Vec::new();
    while *pos < buffer.len() && buffer[*pos] != 0 {
        let b = buffer[*pos];
        if (b & 0x80) == 0 {
            decoded.push((b - 1) as u16);
            *pos += 1;
        } else if (b & 0x40) == 0 {
            let mut c = (b as u16) << 8;
            *pos += 1;
            if *pos < buffer.len() {
                c |= buffer[*pos] as u16;
                *pos += 1;
            }
            c = c.wrapping_sub(0x8000).wrapping_add(0x7F);
            decoded.push(c);
        } else {
            let mut c = (b as u32) << 10;
            *pos += 1;
            if *pos < buffer.len() {
                c |= (buffer[*pos] as u32) << 2;
                *pos += 1;
            }
            if *pos < buffer.len() {
                c |= (buffer[*pos] as u32) >> 6;
                *pos += 1;
            }
            decoded.push(c as u16);
        }
    }
    if *pos < buffer.len() && buffer[*pos] == 0 {
        *pos += 1;
    }
    Some(decoded)
}

fn internal_deserialize(
    buffer: &[u8],
    pos: &mut usize,
    mut type_offset: u8,
) -> Option<IndexedDBKeyType> {
    if *pos >= buffer.len() {
        return None;
    }
    let key_type = buffer[*pos];
    if key_type >= KeyType::Array as u8 + type_offset {
        let mut arr = Vec::new();
        type_offset += KeyType::Array as u8;
        if type_offset == KeyType::Array as u8 * 3 {
            *pos += 1;
            type_offset = 0;
        }
        while *pos < buffer.len() && buffer[*pos] != type_offset {
            arr.push(internal_deserialize(buffer, pos, type_offset)?);
            type_offset = 0;
        }
        if *pos < buffer.len() {
            *pos += 1;
        }
        Some(IndexedDBKeyType::Array(arr))
    } else if key_type == KeyType::String as u8 + type_offset {
        *pos += 1;
        let u16s = decode_stringy(buffer, pos)?;
        Some(IndexedDBKeyType::String(String::from_utf16(&u16s).ok()?))
    } else if key_type == KeyType::Binary as u8 + type_offset {
        *pos += 1;
        let u16s = decode_stringy(buffer, pos)?;
        let u8s: Option<Vec<u8>> = u16s.into_iter().map(|x| x.try_into().ok()).collect();
        Some(IndexedDBKeyType::Binary(u8s?))
    } else if key_type == KeyType::Date as u8 + type_offset {
        *pos += 1;
        let bytes_to_copy = std::cmp::min(8, buffer.len() - *pos);
        let val = deserialize_number(&buffer[*pos..*pos + bytes_to_copy])?;
        *pos += bytes_to_copy;
        Some(IndexedDBKeyType::Date(val))
    } else if key_type == KeyType::Number as u8 + type_offset {
        *pos += 1;
        let bytes_to_copy = std::cmp::min(8, buffer.len() - *pos);
        let val = deserialize_number(&buffer[*pos..*pos + bytes_to_copy])?;
        *pos += bytes_to_copy;
        Some(IndexedDBKeyType::Number(val))
    } else {
        None
    }
}

pub fn deserialize(data: &[u8]) -> Option<IndexedDBKeyType> {
    let mut pos = 0;
    internal_deserialize(data, &mut pos, 0)
}

#[cfg(test)]
mod tests {
    use storage_traits::indexeddb::IndexedDBKeyType;

    use super::{deserialize, deserialize_number, serialize, serialize_number};

    #[test]
    fn test_number_roundtrip() {
        let numbers = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            123.456,
            -123.456,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
        ];
        for &number in &numbers {
            let serialized = serialize_number(number);
            let deserialized = deserialize_number(&serialized).unwrap();
            if number.is_nan() {
                assert!(deserialized.is_nan());
            } else {
                assert_eq!(number, deserialized);
            }
        }
    }

    #[test]
    fn test_roundtrip() {
        let keys = vec![
            IndexedDBKeyType::Number(42.0),
            IndexedDBKeyType::Date(1625077765.0),
            IndexedDBKeyType::String("hello".to_string()),
            IndexedDBKeyType::Binary(vec![1, 2, 3, 4]),
            IndexedDBKeyType::Array(vec![
                IndexedDBKeyType::Number(1.0),
                IndexedDBKeyType::String("nested".to_string()),
            ]),
        ];
        for key in &keys {
            let serialized = serialize(key);
            let deserialized = deserialize(&serialized)
                .expect(format!("Failed to deserialize key: {:?}", key).as_str());
            assert_eq!(key, &deserialized);
        }
    }

    #[test]
    fn test_sorting() {
        let keys = vec![
            // Number sorting
            IndexedDBKeyType::Number(f64::NEG_INFINITY),
            IndexedDBKeyType::Number(-100.0),
            IndexedDBKeyType::Number(-1.0),
            IndexedDBKeyType::Number(-0.0),
            IndexedDBKeyType::Number(0.0),
            IndexedDBKeyType::Number(1.0),
            IndexedDBKeyType::Number(100.0),
            IndexedDBKeyType::Number(f64::INFINITY),
            // Date sorting
            IndexedDBKeyType::Date(0.0),
            IndexedDBKeyType::Date(100.0),
            // String sorting
            IndexedDBKeyType::String("".to_string()),
            IndexedDBKeyType::String("\0".to_string()),
            IndexedDBKeyType::String("a".to_string()),
            IndexedDBKeyType::String("aa".to_string()),
            IndexedDBKeyType::String("b".to_string()),
            IndexedDBKeyType::String("ba".to_string()),
            IndexedDBKeyType::String("c".to_string()),
            IndexedDBKeyType::String("~".to_string()),
            // Binary sorting
            IndexedDBKeyType::Binary(vec![]),
            IndexedDBKeyType::Binary(vec![0]),
            IndexedDBKeyType::Binary(vec![1]),
            IndexedDBKeyType::Binary(vec![1, 0]),
            IndexedDBKeyType::Binary(vec![1, 1]),
            IndexedDBKeyType::Binary(vec![2]),
            IndexedDBKeyType::Binary(vec![255]),
            // Array sorting
            IndexedDBKeyType::Array(vec![]),
            IndexedDBKeyType::Array(vec![IndexedDBKeyType::Number(0.0)]),
            IndexedDBKeyType::Array(vec![IndexedDBKeyType::Number(1.0)]),
            IndexedDBKeyType::Array(vec![
                IndexedDBKeyType::Number(1.0),
                IndexedDBKeyType::Number(2.0),
            ]),
            IndexedDBKeyType::Array(vec![IndexedDBKeyType::String("a".to_string())]),
        ];

        let serialized: Vec<Vec<u8>> = keys.iter().map(serialize).collect();
        let mut sorted_serialized = serialized.clone();
        sorted_serialized.sort();

        // The bytes should be byte-wise sortable mirroring the original sort order
        assert_eq!(serialized, sorted_serialized);
    }
}
