/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use matching::{ElementSelectorFlags, StyleRelations};
use parser::{AttrSelector, SelectorImpl};
use std::ascii::AsciiExt;

/// The definition of whitespace per CSS Selectors Level 3 § 4.
pub static SELECTOR_WHITESPACE: &'static [char] = &[' ', '\t', '\n', '\r', '\x0C'];

// Attribute matching routines. Consumers with simple implementations can implement
// MatchAttrGeneric instead.
pub trait MatchAttr {
    type Impl: SelectorImpl;

    fn match_attr_has(
        &self,
        attr: &AttrSelector<Self::Impl>) -> bool;

    fn match_attr_equals(
        &self,
        attr: &AttrSelector<Self::Impl>,
        value: &<Self::Impl as SelectorImpl>::AttrValue) -> bool;

    fn match_attr_equals_ignore_ascii_case(
        &self,
        attr: &AttrSelector<Self::Impl>,
        value: &<Self::Impl as SelectorImpl>::AttrValue) -> bool;

    fn match_attr_includes(
        &self,
        attr: &AttrSelector<Self::Impl>,
        value: &<Self::Impl as SelectorImpl>::AttrValue) -> bool;

    fn match_attr_dash(
        &self,
        attr: &AttrSelector<Self::Impl>,
        value: &<Self::Impl as SelectorImpl>::AttrValue) -> bool;

    fn match_attr_prefix(
        &self,
        attr: &AttrSelector<Self::Impl>,
        value: &<Self::Impl as SelectorImpl>::AttrValue) -> bool;

    fn match_attr_substring(
        &self,
        attr: &AttrSelector<Self::Impl>,
        value: &<Self::Impl as SelectorImpl>::AttrValue) -> bool;

    fn match_attr_suffix(
        &self,
        attr: &AttrSelector<Self::Impl>,
        value: &<Self::Impl as SelectorImpl>::AttrValue) -> bool;
}

pub trait MatchAttrGeneric {
    type Impl: SelectorImpl;
    fn match_attr<F>(&self, attr: &AttrSelector<Self::Impl>, test: F) -> bool where F: Fn(&str) -> bool;
}

impl<T> MatchAttr for T where T: MatchAttrGeneric, T::Impl: SelectorImpl<AttrValue = String> {
    type Impl = T::Impl;

    fn match_attr_has(&self, attr: &AttrSelector<Self::Impl>) -> bool {
        self.match_attr(attr, |_| true)
    }

    fn match_attr_equals(&self, attr: &AttrSelector<Self::Impl>, value: &String) -> bool {
        self.match_attr(attr, |v| v == value)
    }

    fn match_attr_equals_ignore_ascii_case(&self, attr: &AttrSelector<Self::Impl>,
                                           value: &String) -> bool {
        self.match_attr(attr, |v| v.eq_ignore_ascii_case(value))
    }

    fn match_attr_includes(&self, attr: &AttrSelector<Self::Impl>, value: &String) -> bool {
        self.match_attr(attr, |attr_value| {
            attr_value.split(SELECTOR_WHITESPACE).any(|v| v == value)
        })
    }

    fn match_attr_dash(&self, attr: &AttrSelector<Self::Impl>, value: &String) -> bool {
        self.match_attr(attr, |attr_value| {
            // The attribute must start with the pattern.
            if !attr_value.starts_with(value) {
                return false
            }

            // If the strings are the same, we're done.
            if attr_value.len() == value.len() {
                return true
            }

            // The attribute is long than the pattern, so the next character must be '-'.
            attr_value.as_bytes()[value.len()] == '-' as u8
        })
    }

    fn match_attr_prefix(&self, attr: &AttrSelector<Self::Impl>, value: &String) -> bool {
        self.match_attr(attr, |attr_value| {
            attr_value.starts_with(value)
        })
    }

    fn match_attr_substring(&self, attr: &AttrSelector<Self::Impl>, value: &String) -> bool {
        self.match_attr(attr, |attr_value| {
            attr_value.contains(value)
        })
    }

    fn match_attr_suffix(&self, attr: &AttrSelector<Self::Impl>, value: &String) -> bool {
        self.match_attr(attr, |attr_value| {
            attr_value.ends_with(value)
        })
    }
}

pub trait Element: MatchAttr + Sized {
    fn parent_element(&self) -> Option<Self>;

    // Skips non-element nodes
    fn first_child_element(&self) -> Option<Self>;

    // Skips non-element nodes
    fn last_child_element(&self) -> Option<Self>;

    // Skips non-element nodes
    fn prev_sibling_element(&self) -> Option<Self>;

    // Skips non-element nodes
    fn next_sibling_element(&self) -> Option<Self>;

    fn is_html_element_in_html_document(&self) -> bool;
    fn get_local_name(&self) -> &<Self::Impl as SelectorImpl>::BorrowedLocalName;
    fn get_namespace(&self) -> &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl;

    fn match_non_ts_pseudo_class<F>(&self,
                                    pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
                                    relations: &mut StyleRelations,
                                    flags_setter: &mut F) -> bool
        where F: FnMut(&Self, ElementSelectorFlags);

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

    // Ordinarily I wouldn't use callbacks like this, but the alternative is
    // really messy, since there is a `JSRef` and a `RefCell` involved. Maybe
    // in the future when we have associated types and/or a more convenient
    // JS GC story... --pcwalton
    fn each_class<F>(&self, callback: F) where F: FnMut(&<Self::Impl as SelectorImpl>::ClassName);
}
