// Copyright 2026 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::rc::Rc;
use std::sync::Arc;

use servo_malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};

// Large enough to ensure rounding up occurs.
const PAYLOAD: usize = 100;
// The allocation also contains two reference counts.
const MINIMUM: usize = size_of::<usize>() * 2 + PAYLOAD;
// Leave enough slack for allocator rounding without crossing a page.
const SLACK: usize = 4096;

fn ops() -> MallocSizeOfOps {
    MallocSizeOfOps::new(
        servo_allocator::usable_size,
        servo_allocator::enclosing_size,
        None,
    )
}

#[test]
fn arc_shallow_size_is_the_allocation_size() {
    let arc = Arc::new([0u8; PAYLOAD]);
    let shallow = arc.unconditional_shallow_size_of(&mut ops());
    assert!(
        (MINIMUM..MINIMUM + SLACK).contains(&shallow),
        "expected roughly {MINIMUM} bytes for the ArcInner allocation, got {shallow}"
    );
}

#[test]
fn rc_shallow_size_is_the_allocation_size() {
    let rc = Rc::new([0u8; PAYLOAD]);
    let shallow = rc.unconditional_shallow_size_of(&mut ops());
    assert!(
        (MINIMUM..MINIMUM + SLACK).contains(&shallow),
        "expected roughly {MINIMUM} bytes for the RcInner allocation, got {shallow}"
    );
}

#[test]
fn arc_shallow_size_honors_alignment() {
    // Alignment greater than two words increases the data offset:
    // the counts are padded to 64 bytes, so this ZST's allocation is 64 bytes.
    #[repr(align(64))]
    struct Aligned;

    let arc = Arc::new(Aligned);
    let shallow = arc.unconditional_shallow_size_of(&mut ops());
    let minimum = align_of::<Aligned>();
    assert!(
        (minimum..minimum + SLACK).contains(&shallow),
        "expected roughly {minimum} bytes for the aligned ArcInner allocation, got {shallow}"
    );
}
