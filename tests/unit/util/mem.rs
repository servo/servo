/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use util::mem::HeapSizeOf;


struct Four;
impl HeapSizeOf for Four {
    fn heap_size_of_children(&self) -> usize {
        4
    }
}

#[derive(HeapSizeOf)]
struct Eight(Four, Four, bool, bool, bool);

#[derive(HeapSizeOf)]
enum EightOrFour {
    Eight(Eight),
    Four(Four),
    Zero(u8)
}

#[test]
fn test_heap_size() {
    assert_eq!(Four.heap_size_of_children(), 4);
    let eight = Eight(Four, Four, true, true, true);
    assert_eq!(eight.heap_size_of_children(), 8);
    assert_eq!(EightOrFour::Eight(eight).heap_size_of_children(), 8);
    assert_eq!(EightOrFour::Four(Four).heap_size_of_children(), 4);
    assert_eq!(EightOrFour::Zero(1).heap_size_of_children(), 0);
}
