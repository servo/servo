/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::indexeddb_thread::IndexedDBKeyType;

// Implementation is a port of:
// https://searchfox.org/firefox-main/rev/55ec080e4a37b7ae1f89267063eccd361cdd232d/dom/indexedDB/Key.cpp#109-187

/// When encoding floats, 64bit IEEE 754 are almost sortable, except that
/// positive sort lower than negative, and negative sort descending. So we use
/// the following encoding:
///
/// value < 0 ?
/// (-to64bitInt(value)) :
/// (to64bitInt(value) | 0x8000000000000000)
pub fn serialize_number(n: f64) -> [u8; 8] {
    let mut bytes = n.to_le_bytes();
    if n.is_sign_negative() {
        for byte in &mut bytes {
            *byte = !*byte;
        }
    } else {
        bytes[7] |= 0x80;
    }
    bytes
}

pub fn deserialize_number(bytes: &[u8]) -> Option<f64> {
    if bytes.len() != 8 {
        return None;
    }
    let mut array = [0u8; 8];
    array.copy_from_slice(bytes);
    let mut bytes = array;
    if (bytes[7] & 0x80) != 0 {
        bytes[7] &= 0x7F;
    } else {
        for byte in &mut bytes {
            *byte = !*byte;
        }
    }
    Some(f64::from_le_bytes(bytes))
}

pub fn serialize(key: &IndexedDBKeyType) -> Vec<u8> {
    match key {
        IndexedDBKeyType::Number(number) => {
            [vec![0x10_u8], Vec::from(serialize_number(*number))].concat()
        },
        IndexedDBKeyType::Date(number) => {
            [vec![0x20_u8], Vec::from(serialize_number(*number))].concat()
        },
        // FIXME:(arihant2math) handle unicode encoding for string/binary, so that array can be implemented properly
        IndexedDBKeyType::String(string) => {
            [vec![0x30_u8], string.to_string().into_bytes()].concat()
        },
        IndexedDBKeyType::Binary(binary) => [vec![0x30_u8], binary.clone()].concat(),
        // FIXME:(arihant2math) don't use bincode
        IndexedDBKeyType::Array(array) => {
            [vec![0x40_u8], bincode::serialize(array).unwrap()].concat()
        },
    }
}

pub fn deserialize(data: &[u8]) -> Option<IndexedDBKeyType> {
    let (key_type, rest) = data.split_first()?;
    match key_type {
        0x10 => {
            if rest.len() != 8 {
                return None;
            }
            let mut array = [0u8; 8];
            array.copy_from_slice(rest);
            Some(IndexedDBKeyType::Number(deserialize_number(&array)?))
        },
        0x20 => {
            if rest.len() != 8 {
                return None;
            }
            let mut array = [0u8; 8];
            array.copy_from_slice(rest);
            Some(IndexedDBKeyType::Date(deserialize_number(&array)?))
        },
        0x30 => Some(IndexedDBKeyType::String(
            String::from_utf8(rest.to_vec()).ok()?,
        )),
        0x40 => Some(bincode::deserialize(rest).ok()?),
        _ => None,
    }
}
