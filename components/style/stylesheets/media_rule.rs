/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An [`@media`][media] urle.
//!
//! [media]: https://drafts.csswg.org/css-conditional/#at-ruledef-media

use crate::media_queries::MediaList;
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::stylesheets::CssRules;
use cssparser::SourceLocation;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// An [`@media`][media] urle.
///
/// [media]: https://drafts.csswg.org/css-conditional/#at-ruledef-media
#[derive(Debug, ToShmem)]
pub struct MediaRule {
    /// The list of media queries used by this media rule.
    pub media_queries: Arc<Locked<MediaList>>,
    /// The nested rules to this media rule.
    pub rules: Arc<Locked<CssRules>>,
    /// The source position where this media rule was found.
    pub source_location: SourceLocation,
}

impl MediaRule {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        // Measurement of other fields may be added later.
        self.rules.unconditional_shallow_size_of(ops) +
            self.rules.read_with(guard).size_of(guard, ops)
    }
}

impl ToCssWithGuard for MediaRule {
    // Serialization of MediaRule is not specced.
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSMediaRule
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@media ")?;
        self.media_queries
            .read_with(guard)
            .to_css(&mut CssWriter::new(dest))?;
        self.rules.read_with(guard).to_css_block(guard, dest)
    }
}

impl DeepCloneWithLock for MediaRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        let media_queries = self.media_queries.read_with(guard);
        let rules = self.rules.read_with(guard);
        MediaRule {
            media_queries: Arc::new(lock.wrap(media_queries.clone())),
            rules: Arc::new(lock.wrap(rules.deep_clone_with_lock(lock, guard, params))),
            source_location: self.source_location.clone(),
        }
    }
}
