/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Visitor traits for selectors.

#![deny(missing_docs)]

use crate::attr::NamespaceConstraint;
use crate::parser::{Combinator, Component, Selector, SelectorImpl};

/// A trait to visit selector properties.
///
/// All the `visit_foo` methods return a boolean indicating whether the
/// traversal should continue or not.
pub trait SelectorVisitor: Sized {
    /// The selector implementation this visitor wants to visit.
    type Impl: SelectorImpl;

    /// Visit an attribute selector that may match (there are other selectors
    /// that may never match, like those containing whitespace or the empty
    /// string).
    fn visit_attribute_selector(
        &mut self,
        _namespace: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        _local_name: &<Self::Impl as SelectorImpl>::LocalName,
        _local_name_lower: &<Self::Impl as SelectorImpl>::LocalName,
    ) -> bool {
        true
    }

    /// Visit a simple selector.
    fn visit_simple_selector(&mut self, _: &Component<Self::Impl>) -> bool {
        true
    }

    /// Visit a nested selector list. The caller is responsible to call visit
    /// into the internal selectors if / as needed.
    ///
    /// The default implementation does this.
    fn visit_selector_list(&mut self, list: &[Selector<Self::Impl>]) -> bool {
        for nested in list {
            if !nested.visit(self) {
                return false;
            }
        }
        true
    }

    /// Visits a complex selector.
    ///
    /// Gets the combinator to the right of the selector, or `None` if the
    /// selector is the rightmost one.
    fn visit_complex_selector(&mut self, _combinator_to_right: Option<Combinator>) -> bool {
        true
    }
}
