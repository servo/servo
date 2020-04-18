/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![forbid(unsafe_code)]

use crate::properties::PropertyDeclarationBlock;
use crate::shared_lock::{Locked, SharedRwLockReadGuard};
use crate::stylesheets::StyleRule;
use servo_arc::{Arc, ArcBorrow, ArcUnion, ArcUnionBorrow};
use std::io::Write;
use std::ptr;

/// A style source for the rule node. It can either be a CSS style rule or a
/// declaration block.
///
/// Note that, even though the declaration block from inside the style rule
/// could be enough to implement the rule tree, keeping the whole rule provides
/// more debuggability, and also the ability of show those selectors to
/// devtools.
#[derive(Clone, Debug)]
pub struct StyleSource(ArcUnion<Locked<StyleRule>, Locked<PropertyDeclarationBlock>>);

impl PartialEq for StyleSource {
    fn eq(&self, other: &Self) -> bool {
        ArcUnion::ptr_eq(&self.0, &other.0)
    }
}

impl StyleSource {
    /// Creates a StyleSource from a StyleRule.
    pub fn from_rule(rule: Arc<Locked<StyleRule>>) -> Self {
        StyleSource(ArcUnion::from_first(rule))
    }

    #[inline]
    pub(super) fn key(&self) -> ptr::NonNull<()> {
        self.0.ptr()
    }

    /// Creates a StyleSource from a PropertyDeclarationBlock.
    pub fn from_declarations(decls: Arc<Locked<PropertyDeclarationBlock>>) -> Self {
        StyleSource(ArcUnion::from_second(decls))
    }

    pub(super) fn dump<W: Write>(&self, guard: &SharedRwLockReadGuard, writer: &mut W) {
        if let Some(ref rule) = self.0.as_first() {
            let rule = rule.read_with(guard);
            let _ = write!(writer, "{:?}", rule.selectors);
        }

        let _ = write!(writer, "  -> {:?}", self.read(guard).declarations());
    }

    /// Read the style source guard, and obtain thus read access to the
    /// underlying property declaration block.
    #[inline]
    pub fn read<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> &'a PropertyDeclarationBlock {
        let block: &Locked<PropertyDeclarationBlock> = match self.0.borrow() {
            ArcUnionBorrow::First(ref rule) => &rule.get().read_with(guard).block,
            ArcUnionBorrow::Second(ref block) => block.get(),
        };
        block.read_with(guard)
    }

    /// Returns the style rule if applicable, otherwise None.
    pub fn as_rule(&self) -> Option<ArcBorrow<Locked<StyleRule>>> {
        self.0.as_first()
    }

    /// Returns the declaration block if applicable, otherwise None.
    pub fn as_declarations(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>> {
        self.0.as_second()
    }
}
