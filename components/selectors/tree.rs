/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency
//! between layout and style.

use crate::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use crate::matching::{ElementSelectorFlags, MatchingContext};
use crate::parser::SelectorImpl;
use std::fmt::Debug;
use std::ptr::NonNull;

/// Opaque representation of an Element, for identity comparisons.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OpaqueElement(NonNull<()>);

unsafe impl Send for OpaqueElement {}

impl OpaqueElement {
    /// Creates a new OpaqueElement from an arbitrarily-typed pointer.
    pub fn new<T>(ptr: &T) -> Self {
        unsafe {
            OpaqueElement(NonNull::new_unchecked(
                ptr as *const T as *const () as *mut (),
            ))
        }
    }
}

pub trait Element: Sized + Clone + Debug {
    type Impl: SelectorImpl;

    /// Converts self into an opaque representation.
    fn opaque(&self) -> OpaqueElement;

    fn parent_element(&self) -> Option<Self>;

    /// Whether the parent node of this element is a shadow root.
    fn parent_node_is_shadow_root(&self) -> bool;

    /// The host of the containing shadow root, if any.
    fn containing_shadow_host(&self) -> Option<Self>;

    /// The parent of a given pseudo-element, after matching a pseudo-element
    /// selector.
    ///
    /// This is guaranteed to be called in a pseudo-element.
    fn pseudo_element_originating_element(&self) -> Option<Self> {
        self.parent_element()
    }

    /// Skips non-element nodes
    fn prev_sibling_element(&self) -> Option<Self>;

    /// Skips non-element nodes
    fn next_sibling_element(&self) -> Option<Self>;

    fn is_html_element_in_html_document(&self) -> bool;

    fn local_name(&self) -> &<Self::Impl as SelectorImpl>::BorrowedLocalName;

    /// Empty string for no namespace
    fn namespace(&self) -> &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl;

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool;

    fn match_non_ts_pseudo_class<F>(
        &self,
        pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        context: &mut MatchingContext<Self::Impl>,
        flags_setter: &mut F,
    ) -> bool
    where
        F: FnMut(&Self, ElementSelectorFlags);

    fn match_pseudo_element(
        &self,
        pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        context: &mut MatchingContext<Self::Impl>,
    ) -> bool;

    /// Whether this element is a `link`.
    fn is_link(&self) -> bool;

    /// Returns whether the element is an HTML <slot> element.
    fn is_html_slot_element(&self) -> bool;

    /// Returns the assigned <slot> element this element is assigned to.
    ///
    /// Necessary for the `::slotted` pseudo-class.
    fn assigned_slot(&self) -> Option<Self> {
        None
    }

    fn has_id(
        &self,
        id: &<Self::Impl as SelectorImpl>::Identifier,
        case_sensitivity: CaseSensitivity,
    ) -> bool;

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::ClassName,
        case_sensitivity: CaseSensitivity,
    ) -> bool;

    fn is_part(&self, name: &<Self::Impl as SelectorImpl>::PartName) -> bool;

    /// Returns whether this element matches `:empty`.
    ///
    /// That is, whether it does not contain any child element or any non-zero-length text node.
    /// See http://dev.w3.org/csswg/selectors-3/#empty-pseudo
    fn is_empty(&self) -> bool;

    /// Returns whether this element matches `:root`,
    /// i.e. whether it is the root element of a document.
    ///
    /// Note: this can be false even if `.parent_element()` is `None`
    /// if the parent node is a `DocumentFragment`.
    fn is_root(&self) -> bool;

    /// Returns whether this element should ignore matching nth child
    /// selector.
    fn ignores_nth_child_selectors(&self) -> bool {
        false
    }
}
