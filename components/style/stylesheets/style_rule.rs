/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A style rule.

use cssparser::SourceLocation;
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
#[cfg(feature = "gecko")]
use malloc_size_of::MallocUnconditionalShallowSizeOf;
use properties::PropertyDeclarationBlock;
use selector_parser::SelectorImpl;
use selectors::SelectorList;
use servo_arc::Arc;
use shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt::{self, Write};
use str::CssStringWriter;

/// A style rule, with selectors and declarations.
#[derive(Debug)]
pub struct StyleRule {
    /// The list of selectors in this rule.
    pub selectors: SelectorList<SelectorImpl>,
    /// The declaration block with the properties it contains.
    pub block: Arc<Locked<PropertyDeclarationBlock>>,
    /// The location in the sheet where it was found.
    pub source_location: SourceLocation,
}

impl DeepCloneWithLock for StyleRule {
    /// Deep clones this StyleRule.
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        _params: &DeepCloneParams,
    ) -> StyleRule {
        StyleRule {
            selectors: self.selectors.clone(),
            block: Arc::new(lock.wrap(self.block.read_with(guard).clone())),
            source_location: self.source_location.clone(),
        }
    }
}

impl StyleRule {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = 0;
        n += self.selectors.0.size_of(ops);
        n += self.block.unconditional_shallow_size_of(ops) +
            self.block.read_with(guard).size_of(ops);
        n
    }
}

impl ToCssWithGuard for StyleRule {
    /// https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSStyleRule
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        use cssparser::ToCss;

        // Step 1
        self.selectors.to_css(dest)?;
        // Step 2
        dest.write_str(" { ")?;
        // Step 3
        let declaration_block = self.block.read_with(guard);
        declaration_block.to_css(dest)?;
        // Step 4
        if !declaration_block.declarations().is_empty() {
            dest.write_str(" ")?;
        }
        // Step 5
        dest.write_str("}")
    }
}
