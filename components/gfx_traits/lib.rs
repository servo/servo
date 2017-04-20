/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "gfx_traits"]
#![crate_type = "rlib"]

#![deny(unsafe_code)]

extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
#[macro_use]
extern crate range;
#[macro_use]
extern crate serde_derive;

pub mod print_tree;

use range::RangeIndex;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};

/// A newtype struct for denoting the age of messages; prevents race conditions.
#[derive(PartialEq, Eq, Debug, Copy, Clone, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Epoch(pub u32);

impl Epoch {
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

/// A unique ID for every stacking context.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, HeapSizeOf, PartialEq, Serialize)]
pub struct StackingContextId(
    /// The identifier for this StackingContext, derived from the Flow's memory address
    /// and fragment type.  As a space optimization, these are combined into a single word.
    u64
);

impl StackingContextId {
    /// Returns the stacking context ID for the outer document/layout root.
    #[inline]
    pub fn root() -> StackingContextId {
        StackingContextId(0)
    }

    /// Returns a new sacking context id with the given numeric id.
    #[inline]
    pub fn new(id: u64) -> StackingContextId {
        StackingContextId(id)
    }
}

int_range_index! {
    #[derive(Deserialize, Serialize)]
    #[doc = "An index that refers to a byte offset in a text run. This could \
             point to the middle of a glyph."]
    #[derive(HeapSizeOf)]
    struct ByteIndex(isize)
}

/// The type of fragment that a stacking context represents.
///
/// This can only ever grow to maximum 4 entries. That's because we cram the value of this enum
/// into the lower 2 bits of the `StackingContextId`, which otherwise contains a 32-bit-aligned
/// heap address.
#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash, Deserialize, Serialize, HeapSizeOf)]
pub enum FragmentType {
    /// A StackingContext for the fragment body itself.
    FragmentBody,
    /// A StackingContext created to contain ::before pseudo-element content.
    BeforePseudoContent,
    /// A StackingContext created to contain ::after pseudo-element content.
    AfterPseudoContent,
}

/// The next ID that will be used for a special stacking context.
///
/// A special stacking context is a stacking context that is one of (a) the outer stacking context
/// of an element with `overflow: scroll`; (b) generated content; (c) both (a) and (b).
static NEXT_SPECIAL_STACKING_CONTEXT_ID: AtomicUsize = ATOMIC_USIZE_INIT;

/// If none of the bits outside this mask are set, the stacking context is a special stacking
/// context.
///
/// Note that we assume that the top 16 bits of the address space are unused on the platform.
const SPECIAL_STACKING_CONTEXT_ID_MASK: usize = 0xffff;

/// Returns a new stacking context ID for a special stacking context.
fn next_special_id() -> usize {
    // We shift this left by 2 to make room for the fragment type ID.
    ((NEXT_SPECIAL_STACKING_CONTEXT_ID.fetch_add(1, Ordering::SeqCst) + 1) << 2) &
        SPECIAL_STACKING_CONTEXT_ID_MASK
}

pub fn combine_id_with_fragment_type(id: usize, fragment_type: FragmentType) ->  usize {
    debug_assert_eq!(id & (fragment_type as usize), 0);
    if fragment_type == FragmentType::FragmentBody {
        id
    } else {
        next_special_id() | (fragment_type as usize)
    }
}

pub fn node_id_from_clip_id(id: usize) -> Option<usize> {
    if (id & !SPECIAL_STACKING_CONTEXT_ID_MASK) != 0 {
        return Some((id & !3) as usize);
    }
    None
}
