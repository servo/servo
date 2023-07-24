/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Code related to the invalidation of media-query-affected rules.

use crate::context::QuirksMode;
use crate::media_queries::Device;
use crate::shared_lock::SharedRwLockReadGuard;
use crate::stylesheets::{DocumentRule, ImportRule, MediaRule};
use crate::stylesheets::{NestedRuleIterationCondition, StylesheetContents, SupportsRule};
use fxhash::FxHashSet;

/// A key for a given media query result.
///
/// NOTE: It happens to be the case that all the media lists we care about
/// happen to have a stable address, so we can just use an opaque pointer to
/// represent them.
///
/// Also, note that right now when a rule or stylesheet is removed, we do a full
/// style flush, so there's no need to worry about other item created with the
/// same pointer address.
///
/// If this changes, though, we may need to remove the item from the cache if
/// present before it goes away.
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub struct MediaListKey(usize);

impl MediaListKey {
    /// Create a MediaListKey from a raw usize.
    pub fn from_raw(k: usize) -> Self {
        MediaListKey(k)
    }
}

/// A trait to get a given `MediaListKey` for a given item that can hold a
/// `MediaList`.
pub trait ToMediaListKey: Sized {
    /// Get a `MediaListKey` for this item. This key needs to uniquely identify
    /// the item.
    fn to_media_list_key(&self) -> MediaListKey {
        MediaListKey(self as *const Self as usize)
    }
}

impl ToMediaListKey for StylesheetContents {}
impl ToMediaListKey for ImportRule {}
impl ToMediaListKey for MediaRule {}

/// A struct that holds the result of a media query evaluation pass for the
/// media queries that evaluated successfully.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct EffectiveMediaQueryResults {
    /// The set of media lists that matched last time.
    set: FxHashSet<MediaListKey>,
}

impl EffectiveMediaQueryResults {
    /// Trivially constructs an empty `EffectiveMediaQueryResults`.
    pub fn new() -> Self {
        Self {
            set: FxHashSet::default(),
        }
    }

    /// Resets the results, using an empty key.
    pub fn clear(&mut self) {
        self.set.clear()
    }

    /// Returns whether a given item was known to be effective when the results
    /// were cached.
    pub fn was_effective<T>(&self, item: &T) -> bool
    where
        T: ToMediaListKey,
    {
        self.set.contains(&item.to_media_list_key())
    }

    /// Notices that an effective item has been seen, and caches it as matching.
    pub fn saw_effective<T>(&mut self, item: &T)
    where
        T: ToMediaListKey,
    {
        // NOTE(emilio): We can't assert that we don't cache the same item twice
        // because of stylesheet reusing... shrug.
        self.set.insert(item.to_media_list_key());
    }
}

/// A filter that filters over effective rules, but allowing all potentially
/// effective `@media` rules.
pub struct PotentiallyEffectiveMediaRules;

impl NestedRuleIterationCondition for PotentiallyEffectiveMediaRules {
    fn process_import(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &ImportRule,
    ) -> bool {
        true
    }

    fn process_media(_: &SharedRwLockReadGuard, _: &Device, _: QuirksMode, _: &MediaRule) -> bool {
        true
    }

    /// Whether we should process the nested rules in a given `@-moz-document` rule.
    fn process_document(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &DocumentRule,
    ) -> bool {
        use crate::stylesheets::EffectiveRules;
        EffectiveRules::process_document(guard, device, quirks_mode, rule)
    }

    /// Whether we should process the nested rules in a given `@supports` rule.
    fn process_supports(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &SupportsRule,
    ) -> bool {
        use crate::stylesheets::EffectiveRules;
        EffectiveRules::process_supports(guard, device, quirks_mode, rule)
    }
}
