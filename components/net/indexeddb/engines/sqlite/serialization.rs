/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::iter;

use net_traits::indexeddb_thread::IndexedDBKeyType;

// Implementation is a port of:
// https://searchfox.org/firefox-main/rev/55ec080e4a37b7ae1f89267063eccd361cdd232d/dom/indexedDB/Key.cpp#109-187

enum KeyType {
    Number = 0x10,
    Date = 0x20,
    String = 0x30,
    Array = 0x40,
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
            0x40 => Ok(KeyType::Array),
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
        IndexedDBKeyType::Number(number) => iter::once(KeyType::Number.into())
            .chain(serialize_number(*number))
            .collect(),
        IndexedDBKeyType::Date(number) => iter::once(KeyType::Date.into())
            .chain(serialize_number(*number))
            .collect(),
        // FIXME:(arihant2math) handle unicode encoding for string/binary, so that array can be implemented properly
        IndexedDBKeyType::String(string) => iter::once(KeyType::String.into())
            .chain(string.as_bytes().iter().copied())
            .collect(),
        IndexedDBKeyType::Binary(binary) => iter::once(KeyType::String.into())
            .chain(binary.iter().copied())
            .collect(),
        // FIXME:(arihant2math) don't use bincode
        IndexedDBKeyType::Array(array) => iter::once(KeyType::Array.into())
            .chain(bincode::serialize(array).unwrap())
            .collect(),
    }
}

pub fn deserialize(data: &[u8]) -> Option<IndexedDBKeyType> {
    let (&key_type, rest) = data.split_first()?;
    match key_type.try_into() {
        Ok(KeyType::Number) => {
            if rest.len() != 8 {
                return None;
            }
            Some(IndexedDBKeyType::Number(deserialize_number(rest)?))
        },
        Ok(KeyType::Date) => {
            if rest.len() != 8 {
                return None;
            }
            Some(IndexedDBKeyType::Date(deserialize_number(rest)?))
        },
        Ok(KeyType::String) => Some(IndexedDBKeyType::String(
            String::from_utf8(rest.to_vec()).ok()?,
        )),
        Ok(KeyType::Array) => bincode::deserialize(rest).ok(),
        Err(()) => None,
    }
}
