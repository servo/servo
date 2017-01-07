/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![plugin(plugins)]

#![crate_name = "gfx_traits"]
#![crate_type = "rlib"]

#![deny(unsafe_code)]

extern crate heapsize;
#[macro_use] extern crate heapsize_derive;
#[macro_use]
extern crate range;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod print_tree;

use range::RangeIndex;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};

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

// Units for use with euclid::length and euclid::scale_factor.

/// One hardware pixel.
///
/// This unit corresponds to the smallest addressable element of the display hardware.
#[derive(Copy, Clone, RustcEncodable, Debug)]
pub enum DevicePixel {}

/// One pixel in layer coordinate space.
///
/// This unit corresponds to a "pixel" in layer coordinate space, which after scaling and
/// transformation becomes a device pixel.
#[derive(Copy, Clone, RustcEncodable, Debug)]
pub enum LayerPixel {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerKind {
    NoTransform,
    HasTransform,
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Deserialize, Serialize, HeapSizeOf)]
pub enum LayerType {
    /// A layer for the fragment body itself.
    FragmentBody,
    /// An extra layer created for a DOM fragments with overflow:scroll.
    OverflowScroll,
    /// A layer created to contain ::before pseudo-element content.
    BeforePseudoContent,
    /// A layer created to contain ::after pseudo-element content.
    AfterPseudoContent,
}

/// The scrolling policy of a layer.
#[derive(Clone, PartialEq, Eq, Copy, Deserialize, Serialize, Debug, HeapSizeOf)]
pub enum ScrollPolicy {
    /// These layers scroll when the parent receives a scrolling message.
    Scrollable,
    /// These layers do not scroll when the parent receives a scrolling message.
    FixedPosition,
}

/// A newtype struct for denoting the age of messages; prevents race conditions.
#[derive(PartialEq, Eq, Debug, Copy, Clone, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Epoch(pub u32);

impl Epoch {
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct FrameTreeId(pub u32);

impl FrameTreeId {
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

/// A unique ID for every stacking context.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, HeapSizeOf, PartialEq, Serialize)]
pub struct StackingContextId(
    /// The identifier for this StackingContext, derived from the Flow's memory address
    /// and fragment type.  As a space optimization, these are combined into a single word.
    usize
);

impl StackingContextId {
    #[inline]
    pub fn new(id: usize) -> StackingContextId {
        StackingContextId::new_of_type(id, FragmentType::FragmentBody)
    }

    /// Returns a new stacking context ID for a special stacking context.
    fn next_special_id() -> usize {
        // We shift this left by 2 to make room for the fragment type ID.
        ((NEXT_SPECIAL_STACKING_CONTEXT_ID.fetch_add(1, Ordering::SeqCst) + 1) << 2) &
            SPECIAL_STACKING_CONTEXT_ID_MASK
    }

    #[inline]
    pub fn new_of_type(id: usize, fragment_type: FragmentType) -> StackingContextId {
        debug_assert_eq!(id & (fragment_type as usize), 0);
        if fragment_type == FragmentType::FragmentBody {
            StackingContextId(id)
        } else {
            StackingContextId(StackingContextId::next_special_id() | (fragment_type as usize))
        }
    }

    /// Returns an ID for the stacking context that forms the outer stacking context of an element
    /// with `overflow: scroll`.
    #[inline(always)]
    pub fn new_outer(fragment_type: FragmentType) -> StackingContextId {
        StackingContextId(StackingContextId::next_special_id() | (fragment_type as usize))
    }

    #[inline]
    pub fn fragment_type(&self) -> FragmentType {
        FragmentType::from_usize(self.0 & 3)
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.0 & !3
    }

    /// Returns the stacking context ID for the outer document/layout root.
    #[inline]
    pub fn root() -> StackingContextId {
        StackingContextId(0)
    }

    /// Returns true if this is a special stacking context.
    ///
    /// A special stacking context is a stacking context that is one of (a) the outer stacking
    /// context of an element with `overflow: scroll`; (b) generated content; (c) both (a) and (b).
    #[inline]
    pub fn is_special(&self) -> bool {
        (self.0 & !SPECIAL_STACKING_CONTEXT_ID_MASK) == 0
    }
}

/// A unique ID for every scrolling root.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, HeapSizeOf, PartialEq, Serialize)]
pub struct ScrollRootId(
    /// The identifier for this StackingContext, derived from the Flow's memory address
    /// and fragment type.  As a space optimization, these are combined into a single word.
    pub usize
);

impl ScrollRootId {
    /// Returns a new stacking context ID for a special stacking context.
    fn next_special_id() -> usize {
        // We shift this left by 2 to make room for the fragment type ID.
        ((NEXT_SPECIAL_STACKING_CONTEXT_ID.fetch_add(1, Ordering::SeqCst) + 1) << 2) &
            SPECIAL_STACKING_CONTEXT_ID_MASK
    }

    #[inline]
    pub fn new_of_type(id: usize, fragment_type: FragmentType) -> ScrollRootId {
        debug_assert_eq!(id & (fragment_type as usize), 0);
        if fragment_type == FragmentType::FragmentBody {
            ScrollRootId(id)
        } else {
            ScrollRootId(ScrollRootId::next_special_id() | (fragment_type as usize))
        }
    }

    /// Returns the stacking context ID for the outer document/layout root.
    #[inline]
    pub fn root() -> ScrollRootId {
        ScrollRootId(0)
    }

    /// Returns true if this is a special stacking context.
    ///
    /// A special stacking context is a stacking context that is one of (a) the outer stacking
    /// context of an element with `overflow: scroll`; (b) generated content; (c) both (a) and (b).
    #[inline]
    pub fn is_special(&self) -> bool {
        (self.0 & !SPECIAL_STACKING_CONTEXT_ID_MASK) == 0
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.0 & !3
    }

    #[inline]
    pub fn fragment_type(&self) -> FragmentType {
        FragmentType::from_usize(self.0 & 3)
    }

    #[inline]
    pub fn to_stacking_context_id(&self) -> StackingContextId {
        StackingContextId(self.0)
    }
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

impl FragmentType {
    #[inline]
    pub fn from_usize(n: usize) -> FragmentType {
        debug_assert!(n < 3);
        match n {
            0 => FragmentType::FragmentBody,
            1 => FragmentType::BeforePseudoContent,
            _ => FragmentType::AfterPseudoContent,
        }
    }
}

int_range_index! {
    #[derive(Deserialize, Serialize, RustcEncodable)]
    #[doc = "An index that refers to a byte offset in a text run. This could \
             point to the middle of a glyph."]
    #[derive(HeapSizeOf)]
    struct ByteIndex(isize)
}
