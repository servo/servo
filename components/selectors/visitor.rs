/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Visitor traits for selectors.

#![deny(missing_docs)]

use attr::NamespaceConstraint;
use parser::{Combinator, Component, SelectorImpl};

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

    /// Visits a complex selector.
    ///
    /// Gets the combinator to the right of the selector, or `None` if the
    /// selector is the rightmost one.
    fn visit_complex_selector(&mut self,
                              _combinator_to_right: Option<Combinator>)
                              -> bool {
        true
    }
}

/// Enables traversing selector components stored in various types
pub trait Visit {
    /// The type parameter of selector component types.
    type Impl: SelectorImpl;

    /// Traverse selector components inside `self`.
    ///
    /// Implementations of this method should call `SelectorVisitor` methods
    /// or other impls of `Visit` as appropriate based on the fields of `Self`.
    ///
    /// A return value of `false` indicates terminating the traversal.
    /// It should be propagated with an early return.
    /// On the contrary, `true` indicates that all fields of `self` have been traversed:
    ///
    /// ```rust,ignore
    /// if !visitor.visit_simple_selector(&self.some_simple_selector) {
    ///     return false;
    /// }
    /// if !self.some_component.visit(visitor) {
    ///     return false;
    /// }
    /// true
    /// ```
    fn visit<V>(&self, visitor: &mut V) -> bool
    where
        V: SelectorVisitor<Impl = Self::Impl>;
}
