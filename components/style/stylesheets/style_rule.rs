/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A style rule.

use cssparser::SourceLocation;
use properties::PropertyDeclarationBlock;
use selector_parser::SelectorImpl;
use selectors::SelectorList;
use servo_arc::Arc;
use shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::ToCss;
use stylesheets::{MallocSizeOf, MallocSizeOfFn, MallocSizeOfVec, MallocSizeOfWithGuard};

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

impl MallocSizeOfWithGuard for StyleRule {
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn
    ) -> usize {
        let mut n = 0;

        // We may add measurement of things hanging off the embedded Components
        // later.
        n += self.selectors.0.malloc_shallow_size_of_vec(malloc_size_of);
        for selector in self.selectors.0.iter() {
            // It's safe to measure this ThinArc directly because it's the
            // "primary" reference. (The secondary references are on the
            // Stylist.)
            let ptr = selector.thin_arc_heap_ptr();
            n += unsafe { (malloc_size_of.0)(ptr) };
        }

        n += self.block.read_with(guard).malloc_size_of_children(malloc_size_of);

        n
    }
}

impl ToCssWithGuard for StyleRule {
    /// https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSStyleRule
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
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

