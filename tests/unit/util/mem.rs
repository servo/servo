/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use libc::c_void;
use util::mem::{HeapSizeOf, heap_size_of};

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

    // Note: jemalloc often rounds up request sizes. However, it does not round up for request
    // sizes of 8 and higher that are powers of two. We take advantage of knowledge here to make
    // the sizes of various heap-allocated blocks predictable.

    //-----------------------------------------------------------------------
    // Start with basic heap block measurement.

    unsafe {
        // EMPTY is the special non-null address used to represent zero-size allocations.
        assert_eq!(heap_size_of(::std::rt::heap::EMPTY as *const c_void), 0);

        // A 64 byte request is allocated exactly.
        let x = ::std::rt::heap::allocate(64, 0);
        assert_eq!(heap_size_of(x as *const c_void), 64);
        ::std::rt::heap::deallocate(x, 64, 0);

        // A 255 byte request is rounded up to 256 bytes.
        let x = ::std::rt::heap::allocate(255, 0);
        assert_eq!(heap_size_of(x as *const c_void), 256);
        ::std::rt::heap::deallocate(x, 255, 0);

        // A 1MiB request is allocated exactly.
        let x = ::std::rt::heap::allocate(1024 * 1024, 0);
        assert_eq!(heap_size_of(x as *const c_void), 1024 * 1024);
        ::std::rt::heap::deallocate(x, 1024 * 1024, 0);
    }

    //-----------------------------------------------------------------------
    // Test HeapSizeOf implementations for various built-in types.

    // Not on the heap; 0 bytes.
    let x = 0i64;
    assert_eq!(x.heap_size_of_children(), 0);

    // An i64 is 8 bytes.
    let x = Box::new(0i64);
    assert_eq!(x.heap_size_of_children(), 8);

    // An ascii string with 16 chars is 16 bytes in UTF-8.
    assert_eq!(String::from("0123456789abcdef").heap_size_of_children(), 16);

    // … but RawVec::reserve gives twice the requested capacity.
    let mut x = String::new();
    x.push_str("0123456789abcdef");
    assert_eq!(x.heap_size_of_children(), 32);

    // Not on the heap.
    let x: Option<i32> = None;
    assert_eq!(x.heap_size_of_children(), 0);

    // Not on the heap.
    let x = Some(0i64);
    assert_eq!(x.heap_size_of_children(), 0);

    // The `Some` is not on the heap, but the Box is.
    let x = Some(Box::new(0i64));
    assert_eq!(x.heap_size_of_children(), 8);

    // Not on the heap.
    let x = ::std::sync::Arc::new(0i64);
    assert_eq!(x.heap_size_of_children(), 0);

    // The `Arc` is not on the heap, but the Box is.
    let x = ::std::sync::Arc::new(Box::new(0i64));
    assert_eq!(x.heap_size_of_children(), 8);

    // Zero elements, no heap storage.
    let x: Vec<i64> = vec![];
    assert_eq!(x.heap_size_of_children(), 0);

    // Four elements, 8 bytes per element.
    let x = vec![0i64, 1i64, 2i64, 3i64];
    assert_eq!(x.heap_size_of_children(), 32);

    //-----------------------------------------------------------------------
    // Test the HeapSizeOf auto-deriving.

    assert_eq!(Four.heap_size_of_children(), 4);
    let eight = Eight(Four, Four, true, true, true);
    assert_eq!(eight.heap_size_of_children(), 8);
    assert_eq!(EightOrFour::Eight(eight).heap_size_of_children(), 8);
    assert_eq!(EightOrFour::Four(Four).heap_size_of_children(), 4);
    assert_eq!(EightOrFour::Zero(1).heap_size_of_children(), 0);
}
