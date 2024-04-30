/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "gfx_traits"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

pub mod print_tree;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;
use range::{int_range_index, RangeIndex};
use serde::{Deserialize, Serialize};
use webrender_api::{
    Epoch as WebRenderEpoch, FontInstanceFlags, FontInstanceKey, FontKey, NativeFontHandle,
};

/// A newtype struct for denoting the age of messages; prevents race conditions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Epoch(pub u32);

impl Epoch {
    pub fn next(&mut self) {
        self.0 += 1;
    }
}

impl From<Epoch> for WebRenderEpoch {
    fn from(val: Epoch) -> Self {
        WebRenderEpoch(val.0)
    }
}

pub trait WebRenderEpochToU16 {
    fn as_u16(&self) -> u16;
}

impl WebRenderEpochToU16 for WebRenderEpoch {
    /// The value of this [`Epoch`] as a u16 value. Note that if this Epoch's
    /// value is more than u16::MAX, then the return value will be modulo
    /// u16::MAX.
    fn as_u16(&self) -> u16 {
        (self.0 % u16::MAX as u32) as u16
    }
}

/// A unique ID for every stacking context.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct StackingContextId(
    /// The identifier for this StackingContext, derived from the Flow's memory address
    /// and fragment type.  As a space optimization, these are combined into a single word.
    pub u64,
);

impl StackingContextId {
    /// Returns the stacking context ID for the outer document/layout root.
    #[inline]
    pub fn root() -> StackingContextId {
        StackingContextId(0)
    }

    pub fn next(&self) -> StackingContextId {
        let StackingContextId(id) = *self;
        StackingContextId(id + 1)
    }
}

int_range_index! {
    #[derive(Deserialize, MallocSizeOf, Serialize)]
    /// An index that refers to a byte offset in a text run. This could
    /// the middle of a glyph.
    struct ByteIndex(isize)
}

/// The type of fragment that a scroll root is created for.
///
/// This can only ever grow to maximum 4 entries. That's because we cram the value of this enum
/// into the lower 2 bits of the `ScrollRootId`, which otherwise contains a 32-bit-aligned
/// heap address.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum FragmentType {
    /// A StackingContext for the fragment body itself.
    FragmentBody,
    /// A StackingContext created to contain ::before pseudo-element content.
    BeforePseudoContent,
    /// A StackingContext created to contain ::after pseudo-element content.
    AfterPseudoContent,
}

/// The next ID that will be used for a special scroll root id.
///
/// A special scroll root is a scroll root that is created for generated content.
static NEXT_SPECIAL_SCROLL_ROOT_ID: AtomicU64 = AtomicU64::new(0);

/// If none of the bits outside this mask are set, the scroll root is a special scroll root.
/// Note that we assume that the top 16 bits of the address space are unused on the platform.
const SPECIAL_SCROLL_ROOT_ID_MASK: u64 = 0xffff;

/// Returns a new scroll root ID for a scroll root.
fn next_special_id() -> u64 {
    // We shift this left by 2 to make room for the fragment type ID.
    ((NEXT_SPECIAL_SCROLL_ROOT_ID.fetch_add(1, Ordering::SeqCst) + 1) << 2) &
        SPECIAL_SCROLL_ROOT_ID_MASK
}

pub fn combine_id_with_fragment_type(id: usize, fragment_type: FragmentType) -> u64 {
    debug_assert_eq!(id & (fragment_type as usize), 0);
    if fragment_type == FragmentType::FragmentBody {
        id as u64
    } else {
        next_special_id() | (fragment_type as u64)
    }
}

pub fn node_id_from_scroll_id(id: usize) -> Option<usize> {
    if (id as u64 & !SPECIAL_SCROLL_ROOT_ID_MASK) != 0 {
        return Some(id & !3);
    }
    None
}

pub trait WebrenderApi {
    fn add_font_instance(
        &self,
        font_key: FontKey,
        size: f32,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey;
    fn add_font(&self, data: Arc<Vec<u8>>, index: u32) -> FontKey;
    fn add_system_font(&self, handle: NativeFontHandle) -> FontKey;
}
