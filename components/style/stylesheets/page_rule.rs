/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`@page`][page] rule.
//!
//! [page]: https://drafts.csswg.org/css2/page.html#page-box

use crate::properties::PropertyDeclarationBlock;
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use cssparser::SourceLocation;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt::{self, Write};

/// A [`@page`][page] rule.
///
/// This implements only a limited subset of the CSS
/// 2.2 syntax.
///
/// In this subset, [page selectors][page-selectors] are not implemented.
///
/// [page]: https://drafts.csswg.org/css2/page.html#page-box
/// [page-selectors]: https://drafts.csswg.org/css2/page.html#page-selectors
#[derive(Debug, ToShmem)]
pub struct PageRule {
    /// The declaration block this page rule contains.
    pub block: Arc<Locked<PropertyDeclarationBlock>>,
    /// The source position this rule was found at.
    pub source_location: SourceLocation,
}

impl PageRule {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        // Measurement of other fields may be added later.
        self.block.unconditional_shallow_size_of(ops) + self.block.read_with(guard).size_of(ops)
    }
}

impl ToCssWithGuard for PageRule {
    /// Serialization of PageRule is not specced, adapted from steps for
    /// StyleRule.
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@page { ")?;
        let declaration_block = self.block.read_with(guard);
        declaration_block.to_css(dest)?;
        if !declaration_block.declarations().is_empty() {
            dest.write_str(" ")?;
        }
        dest.write_str("}")
    }
}

impl DeepCloneWithLock for PageRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        _params: &DeepCloneParams,
    ) -> Self {
        PageRule {
            block: Arc::new(lock.wrap(self.block.read_with(&guard).clone())),
            source_location: self.source_location.clone(),
        }
    }
}
