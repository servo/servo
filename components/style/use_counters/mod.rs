/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various stuff for CSS property use counters.

#[cfg(feature = "gecko")]
use gecko_bindings::sugar::ownership::{HasBoxFFI, HasFFI, HasSimpleFFI};
use properties::{NonCustomPropertyId, NON_CUSTOM_PROPERTY_ID_COUNT};
use std::cell::Cell;

#[cfg(target_pointer_width = "64")]
const BITS_PER_ENTRY: usize = 64;

#[cfg(target_pointer_width = "32")]
const BITS_PER_ENTRY: usize = 32;

/// One bit per each non-custom CSS property.
#[derive(Default)]
pub struct NonCustomPropertyUseCounters {
    storage: [Cell<usize>; (NON_CUSTOM_PROPERTY_ID_COUNT - 1 + BITS_PER_ENTRY) / BITS_PER_ENTRY],
}

impl NonCustomPropertyUseCounters {
    /// Returns the bucket a given property belongs in, and the bitmask for that
    /// property.
    #[inline(always)]
    fn bucket_and_pattern(id: NonCustomPropertyId) -> (usize, usize) {
        let bit = id.bit();
        let bucket = bit / BITS_PER_ENTRY;
        let bit_in_bucket = bit % BITS_PER_ENTRY;
        (bucket, 1 << bit_in_bucket)
    }

    /// Record that a given non-custom property ID has been parsed.
    #[inline]
    pub fn record(&self, id: NonCustomPropertyId) {
        let (bucket, pattern) = Self::bucket_and_pattern(id);
        let bucket = &self.storage[bucket];
        bucket.set(bucket.get() | pattern)
    }

    /// Returns whether a given non-custom property ID has been recorded
    /// earlier.
    #[inline]
    pub fn recorded(&self, id: NonCustomPropertyId) -> bool {
        let (bucket, pattern) = Self::bucket_and_pattern(id);
        self.storage[bucket].get() & pattern != 0
    }

    /// Merge `other` into `self`.
    #[inline]
    fn merge(&self, other: &Self) {
        for (bucket, other_bucket) in self.storage.iter().zip(other.storage.iter()) {
            bucket.set(bucket.get() | other_bucket.get())
        }
    }
}

/// The use-counter data related to a given document we want to store.
#[derive(Default)]
pub struct UseCounters {
    /// The counters for non-custom properties that have been parsed in the
    /// document's stylesheets.
    pub non_custom_properties: NonCustomPropertyUseCounters,
}

impl UseCounters {
    /// Merge the use counters.
    ///
    /// Used for parallel parsing, where we parse off-main-thread.
    #[inline]
    pub fn merge(&self, other: &Self) {
        self.non_custom_properties.merge(&other.non_custom_properties)
    }
}

#[cfg(feature = "gecko")]
unsafe impl HasFFI for UseCounters {
    type FFIType = ::gecko_bindings::structs::StyleUseCounters;
}

#[cfg(feature = "gecko")]
unsafe impl HasSimpleFFI for UseCounters {}

#[cfg(feature = "gecko")]
unsafe impl HasBoxFFI for UseCounters {}
