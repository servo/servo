/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A style rule.

use cssparser::SourceLocation;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::ServoStyleRule;
use properties::PropertyDeclarationBlock;
use selector_parser::SelectorImpl;
use selectors::SelectorList;
use shared_lock::{DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::ToCss;
use stylearc::Arc;
use stylesheets::{MallocSizeOf, MallocSizeOfFn, MallocSizeOfWithGuard};

#[cfg(feature = "gecko")]
#[derive(Debug)]
pub struct GeckoStyleRule(pub *const ServoStyleRule);

// Pointers don't implement these traits automatically, but StyleRule is
// required to have them. Don't try to dereference the pointer in Rust.
#[cfg(feature = "gecko")]
unsafe impl Send for GeckoStyleRule {}
#[cfg(feature = "gecko")]
unsafe impl Sync for GeckoStyleRule {}

#[cfg(feature = "gecko")]
impl Default for GeckoStyleRule {
    fn default() -> Self {
        GeckoStyleRule(::std::ptr::null())
    }
}

#[cfg(not(feature = "gecko"))]
#[derive(Debug, Default)]
pub struct GeckoStyleRule;

/// A style rule, with selectors and declarations.
#[derive(Debug)]
pub struct StyleRule {
    /// The list of selectors in this rule.
    pub selectors: SelectorList<SelectorImpl>,
    /// The declaration block with the properties it contains.
    pub block: Arc<Locked<PropertyDeclarationBlock>>,
    /// The location in the sheet where it was found.
    pub source_location: SourceLocation,
    /// The reverse pointer to Gecko's wrapper rule object of this rule.
    /// It may be null if the corresponding wrapper object hasn't been
    /// constructed in the Gecko side.
    pub gecko_rule: GeckoStyleRule,
}

impl DeepCloneWithLock for StyleRule {
    /// Deep clones this StyleRule.
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
    ) -> StyleRule {
        StyleRule {
            selectors: self.selectors.clone(),
            block: Arc::new(lock.wrap(self.block.read_with(guard).clone())),
            source_location: self.source_location.clone(),
            gecko_rule: Default::default(),
        }
    }
}

impl MallocSizeOfWithGuard for StyleRule {
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn
    ) -> usize {
        // Measurement of other fields may be added later.
        self.block.read_with(guard).malloc_size_of_children(malloc_size_of)
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

