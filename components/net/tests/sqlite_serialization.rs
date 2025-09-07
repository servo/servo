/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net::indexeddb::engines::sqlite::serialization::{
    deserialize, deserialize_number, serialize, serialize_number,
};
use net_traits::indexeddb_thread::IndexedDBKeyType;

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
