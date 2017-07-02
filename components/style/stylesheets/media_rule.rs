/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An [`@media`][media] urle.
//!
//! [media]: https://drafts.csswg.org/css-conditional/#at-ruledef-media

use cssparser::SourceLocation;
use media_queries::MediaList;
use shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::ToCss;
use stylearc::Arc;
use stylesheets::CssRules;

/// An [`@media`][media] urle.
///
/// [media]: https://drafts.csswg.org/css-conditional/#at-ruledef-media
#[derive(Debug)]
pub struct MediaRule {
    /// The list of media queries used by this media rule.
    pub media_queries: Arc<Locked<MediaList>>,
    /// The nested rules to this media rule.
    pub rules: Arc<Locked<CssRules>>,
    /// The source position where this media rule was found.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for MediaRule {
    // Serialization of MediaRule is not specced.
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSMediaRule
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        dest.write_str("@media ")?;
        self.media_queries.read_with(guard).to_css(dest)?;
        dest.write_str(" {")?;
        for rule in self.rules.read_with(guard).0.iter() {
            dest.write_str(" ")?;
            rule.to_css(guard, dest)?;
        }
        dest.write_str(" }")
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
