/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`@page`][page] rule.
//!
//! [page]: https://drafts.csswg.org/css2/page.html#page-box

use crate::parser::{Parse, ParserContext};
use crate::properties::PropertyDeclarationBlock;
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::values::{AtomIdent, CustomIdent};
use cssparser::{Parser, SourceLocation};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// Type of a single [`@page`][page selector]
///
/// We do not support pseudo selectors yet.
/// [page-selectors]: https://drafts.csswg.org/css2/page.html#page-selectors
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct PageSelector(pub AtomIdent);

impl PageSelector {
    /// Checks if the ident matches a page-name's ident.
    ///
    /// This does not currently take pseudo selectors into account.
    #[inline]
    pub fn ident_matches(&self, other: &CustomIdent) -> bool {
        self.0 .0 == other.0
    }
}

impl Parse for PageSelector {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let s = input.expect_ident()?;
        Ok(PageSelector(AtomIdent::from(&**s)))
    }
}

/// A list of [`@page`][page selectors]
///
/// [page-selectors]: https://drafts.csswg.org/css2/page.html#page-selectors
#[derive(Clone, Debug, Default, MallocSizeOf, ToCss, ToShmem)]
#[css(comma)]
pub struct PageSelectors(#[css(iterable)] pub Box<[PageSelector]>);

impl PageSelectors {
    /// Creates a new PageSelectors from a Vec, as from parse_comma_separated
    #[inline]
    pub fn new(s: Vec<PageSelector>) -> Self {
        PageSelectors(s.into())
    }
    /// Returns true iff there are any page selectors
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    /// Get the underlying PageSelector data as a slice
    #[inline]
    pub fn as_slice(&self) -> &[PageSelector] {
        &*self.0
    }
}

impl Parse for PageSelectors {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(PageSelectors::new(input.parse_comma_separated(|i| {
            PageSelector::parse(context, i)
        })?))
    }
}

/// A [`@page`][page] rule.
///
/// This implements only a limited subset of the CSS
/// 2.2 syntax.
///
/// [page]: https://drafts.csswg.org/css2/page.html#page-box
/// [page-selectors]: https://drafts.csswg.org/css2/page.html#page-selectors
#[derive(Clone, Debug, ToShmem)]
pub struct PageRule {
    /// Selectors of the page-rule
    pub selectors: PageSelectors,
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
        self.block.unconditional_shallow_size_of(ops) +
            self.block.read_with(guard).size_of(ops) +
            self.selectors.size_of(ops)
    }
}

impl ToCssWithGuard for PageRule {
    /// Serialization of PageRule is not specced, adapted from steps for
    /// StyleRule.
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@page ")?;
        if !self.selectors.is_empty() {
            self.selectors.to_css(&mut CssWriter::new(dest))?;
            dest.write_char(' ')?;
        }
        dest.write_str("{ ")?;
        let declaration_block = self.block.read_with(guard);
        declaration_block.to_css(dest)?;
        if !declaration_block.declarations().is_empty() {
            dest.write_char(' ')?;
        }
        dest.write_char('}')
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
            selectors: self.selectors.clone(),
            block: Arc::new(lock.wrap(self.block.read_with(&guard).clone())),
            source_location: self.source_location.clone(),
        }
    }
}
