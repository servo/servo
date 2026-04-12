/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;

use malloc_size_of_derive::MallocSizeOf;
use style::selector_parser::PseudoElement;

/// A chain of pseudo-elements up to two levels deep. This is used to represent cases
/// where a pseudo-element has its own child pseudo element (for instance
/// `.div::after::marker`). If both [`Self::primary`] and [`Self::secondary`] are `None`,
/// then this chain represents the element itself. Not all combinations of pseudo-elements
/// are possible and we may not be able to calculate a style for all
/// [`PseudoElementChain`]s.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, MallocSizeOf, PartialEq)]
pub struct PseudoElementChain {
    pub primary: Option<PseudoElement>,
    pub secondary: Option<PseudoElement>,
}

impl PseudoElementChain {
    pub fn unnested(pseudo_element: PseudoElement) -> Self {
        Self {
            primary: Some(pseudo_element),
            secondary: None,
        }
    }

    pub fn innermost(&self) -> Option<PseudoElement> {
        self.secondary.or(self.primary)
    }

    /// Return a possibly nested [`PseudoElementChain`]. Currently only `::before` and
    /// `::after` only support nesting. If the primary [`PseudoElement`] on the chain is
    /// not `::before` or `::after` a single element chain is returned for the given
    /// [`PseudoElement`].
    pub fn with_pseudo(&self, pseudo_element: PseudoElement) -> Self {
        match self.primary {
            Some(primary) if primary.is_before_or_after() => Self {
                primary: self.primary,
                secondary: Some(pseudo_element),
            },
            _ => {
                assert!(self.secondary.is_none());
                Self::unnested(pseudo_element)
            },
        }
    }

    pub fn without_innermost(&self) -> Option<Self> {
        let primary = self.primary?;
        Some(
            self.secondary
                .map_or_else(Self::default, |_| Self::unnested(primary)),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.primary.is_none()
    }
}
