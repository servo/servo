/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::tree::OpaqueElement;
use fxhash::FxHashMap;

/// A cache to speed up matching of nth-index-like selectors.
///
/// See [1] for some discussion around the design tradeoffs.
///
/// [1] https://bugzilla.mozilla.org/show_bug.cgi?id=1401855#c3
#[derive(Default)]
pub struct NthIndexCache {
    nth: NthIndexCacheInner,
    nth_last: NthIndexCacheInner,
    nth_of_type: NthIndexCacheInner,
    nth_last_of_type: NthIndexCacheInner,
}

impl NthIndexCache {
    /// Gets the appropriate cache for the given parameters.
    pub fn get(&mut self, is_of_type: bool, is_from_end: bool) -> &mut NthIndexCacheInner {
        match (is_of_type, is_from_end) {
            (false, false) => &mut self.nth,
            (false, true) => &mut self.nth_last,
            (true, false) => &mut self.nth_of_type,
            (true, true) => &mut self.nth_last_of_type,
        }
    }
}

/// The concrete per-pseudo-class cache.
#[derive(Default)]
pub struct NthIndexCacheInner(FxHashMap<OpaqueElement, i32>);

impl NthIndexCacheInner {
    /// Does a lookup for a given element in the cache.
    pub fn lookup(&mut self, el: OpaqueElement) -> Option<i32> {
        self.0.get(&el).map(|x| *x)
    }

    /// Inserts an entry into the cache.
    pub fn insert(&mut self, element: OpaqueElement, index: i32) {
        self.0.insert(element, index);
    }

    /// Returns whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
