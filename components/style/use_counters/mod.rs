/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Various stuff for CSS property use counters.

use properties::{NonCustomPropertyId, NON_CUSTOM_PROPERTY_ID_COUNT};
// FIXME(emilio): We need AtomicU32 on stable ASAP...
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[cfg(target_pointer_width = "64")]
const BITS_PER_ENTRY: usize = 64;

#[cfg(target_pointer_width = "32")]
const BITS_PER_ENTRY: usize = 32;

/// One bit per each non-custom CSS property.
#[derive(Default)]
pub struct NonCustomPropertyUseCounters {
    storage: [AtomicUsize; (NON_CUSTOM_PROPERTY_ID_COUNT - 1 + BITS_PER_ENTRY) / BITS_PER_ENTRY],
}

impl NonCustomPropertyUseCounters {
    /// Record that a given non-custom property ID has been parsed.
    #[inline]
    pub fn record(&self, id: NonCustomPropertyId) {
        let bit = id.bit();
        let bucket = bit / BITS_PER_ENTRY;
        let bit_in_bucket = bit % BITS_PER_ENTRY;
        self.storage[bucket].fetch_or(1 << bit_in_bucket, Ordering::Relaxed);
    }
}

/// The use-counter data related to a given document we want to store.
#[derive(Default)]
pub struct UseCounters {
    /// The counters for non-custom properties that have been parsed in the
    /// document's stylesheets.
    pub non_custom_properties: NonCustomPropertyUseCounters,
}
