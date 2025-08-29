/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net::indexeddb::engines::sqlite::serialization::{
    deserialize, deserialize_number, serialize, serialize_number,
};
use net_traits::indexeddb_thread::IndexedDBKeyType;

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
        std::f64::MAX,
        std::f64::MIN,
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

fn test_number_sorting() {
    let numbers = [
        3.0,
        -1.0,
        2.0,
        0.0,
        -3.0,
        1.0,
        -2.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    let mut serialized: Vec<[u8; 8]> = numbers.iter().map(|&n| serialize_number(n)).collect();
    serialized.sort();
    let deserialized: Vec<f64> = serialized
        .iter()
        .map(|s| deserialize_number(s).unwrap())
        .collect();
    let mut expected = numbers.to_vec();
    expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(deserialized, expected);
}

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
        let deserialized = deserialize(&serialized).unwrap();
        assert_eq!(key, &deserialized);
    }
}
