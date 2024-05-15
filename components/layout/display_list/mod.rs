/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

pub use self::builder::{
    BorderPaintingMode, DisplayListBuildState, IndexableText, StackingContextCollectionFlags,
    StackingContextCollectionState,
};
pub use self::conversions::{FilterToLayout, ToLayout};

mod background;
mod border;
mod builder;
pub(crate) mod conversions;
mod gradient;
pub mod items;
mod webrender_helpers;

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
