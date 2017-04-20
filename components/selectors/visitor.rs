/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Visitor traits for selectors.

#![deny(missing_docs)]

use parser::{AttrSelector, Combinator, Component};
use parser::{SelectorImpl, SelectorIter};

/// A trait to visit selector properties.
///
/// All the `visit_foo` methods return a boolean indicating whether the
/// traversal should continue or not.
pub trait SelectorVisitor {
    /// The selector implementation this visitor wants to visit.
    type Impl: SelectorImpl;

    /// Visit an attribute selector that may match (there are other selectors
    /// that may never match, like those containing whitespace or the empty
    /// string).
    fn visit_attribute_selector(&mut self, _: &AttrSelector<Self::Impl>) -> bool {
        true
    }

    /// Visit a simple selector.
    fn visit_simple_selector(&mut self, _: &Component<Self::Impl>) -> bool {
        true
    }

    /// Visits a complex selector.
    ///
    /// Gets the combinator to the right of the selector, or `None` if the
    /// selector is the leftmost one.
    fn visit_complex_selector(&mut self,
                              _: SelectorIter<Self::Impl>,
                              _combinator_to_right: Option<Combinator>)
                              -> bool {
        true
    }
}
