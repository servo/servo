/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency
//! between layout and style.

use attr::{AttrSelectorOperation, NamespaceConstraint};
use matching::{ElementSelectorFlags, LocalMatchingContext, MatchingContext, RelevantLinkStatus};
use parser::SelectorImpl;
use std::fmt::Debug;

pub trait Element: Sized + Debug {
    type Impl: SelectorImpl;

    fn parent_element(&self) -> Option<Self>;

    /// The parent of a given pseudo-element, after matching a pseudo-element
    /// selector.
    ///
    /// This is guaranteed to be called in a pseudo-element.
    fn pseudo_element_originating_element(&self) -> Option<Self> {
        self.parent_element()
    }

    /// Skips non-element nodes
    fn first_child_element(&self) -> Option<Self>;

    /// Skips non-element nodes
    fn last_child_element(&self) -> Option<Self>;

    /// Skips non-element nodes
    fn prev_sibling_element(&self) -> Option<Self>;

    /// Skips non-element nodes
    fn next_sibling_element(&self) -> Option<Self>;

    fn is_html_element_in_html_document(&self) -> bool;

    fn get_local_name(&self) -> &<Self::Impl as SelectorImpl>::BorrowedLocalName;

    /// Empty string for no namespace
    fn get_namespace(&self) -> &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl;

    fn attr_matches(&self,
                    ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
                    local_name: &<Self::Impl as SelectorImpl>::LocalName,
                    operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>)
                    -> bool;

    fn match_non_ts_pseudo_class<F>(&self,
                                    pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
                                    context: &mut LocalMatchingContext<Self::Impl>,
                                    relevant_link: &RelevantLinkStatus,
                                    flags_setter: &mut F) -> bool
        where F: FnMut(&Self, ElementSelectorFlags);

    fn match_pseudo_element(&self,
                            pe: &<Self::Impl as SelectorImpl>::PseudoElement,
                            context: &mut MatchingContext)
                            -> bool;

    /// Whether this element is a `link`.
    fn is_link(&self) -> bool;

    fn get_id(&self) -> Option<<Self::Impl as SelectorImpl>::Identifier>;

    fn has_class(&self, name: &<Self::Impl as SelectorImpl>::ClassName) -> bool;

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
}
