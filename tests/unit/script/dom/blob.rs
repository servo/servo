/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::dom::blob::DataSlice;
use std::sync::Arc;

#[test]
fn test_data_slice_without_start_end_should_match_buffer_size() {
    let bytes = Arc::new(vec![1u8, 2u8, 3u8]);
    let data = DataSlice::new(bytes, None, None);
    assert_eq!(data.size(), 3);
}

#[test]
fn test_data_slice_should_prevent_reverse_bounds() {
    let bytes = Arc::new(vec![1u8, 2, 3, 4, 5]);
    let start = Some(3);
    let end = Some(1);

    let data = DataSlice::new(bytes, start, end);
    assert_eq!(data.size(), 0);
}

#[test]
fn test_data_slice_should_respect_correct_bounds() {
    let bytes = Arc::new(vec![1u8, 2, 3, 4, 5]);
    let start = Some(1);
    let end = Some(3);

    let data = DataSlice::new(bytes, start, end);
    let expected = [2u8, 3];
    assert_eq!(&expected, data.get_bytes());
}

#[test]
fn test_data_slice_negative_bound() {
    let bytes = Arc::new(vec![1u8, 2, 3, 4, 5]);
    let start = Some(-2);
    let end = Some(-1);
    let data = DataSlice::new(bytes, start, end);
    let expected = [4u8];
    assert_eq!(&expected, data.get_bytes());
}

#[test]
fn test_empty_data_slice() {
    assert_eq!(DataSlice::empty().size(), 0);
}

#[test]
fn test_data_slice_from_get_bytes() {
    let bytes = vec![1u8, 3, 4, 5, 6];
    let slice = DataSlice::from_bytes(bytes.clone());
    assert_eq!(slice.get_bytes(), &bytes[..]);
}
