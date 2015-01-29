/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use cssparser::RGBA;
use legacy::{IntegerAttribute, LengthAttribute, SimpleColorAttribute, UnsignedIntegerAttribute};
use selectors::AttrSelector;
use util::str::LengthOrPercentageOrAuto;
use string_cache::{Atom, Namespace};

pub trait TNode<'a, E: TElement<'a>> : Clone + Copy {
    fn parent_node(self) -> Option<Self>;
    fn first_child(self) -> Option<Self>;
    fn last_child(self) -> Option<Self>;
    fn prev_sibling(self) -> Option<Self>;
    fn next_sibling(self) -> Option<Self>;
    fn is_document(self) -> bool;
    fn is_element(self) -> bool;
    fn as_element(self) -> E;
    fn match_attr<F>(self, attr: &AttrSelector, test: F) -> bool where F: Fn(&str) -> bool;
    fn is_html_element_in_html_document(self) -> bool;

    fn has_changed(self) -> bool;
    unsafe fn set_changed(self, value: bool);

    fn is_dirty(self) -> bool;
    unsafe fn set_dirty(self, value: bool);

    fn has_dirty_siblings(self) -> bool;
    unsafe fn set_dirty_siblings(self, value: bool);

    fn has_dirty_descendants(self) -> bool;
    unsafe fn set_dirty_descendants(self, value: bool);
}

pub trait TElement<'a> : Copy {
    fn get_attr(self, namespace: &Namespace, attr: &Atom) -> Option<&'a str>;
    fn get_attrs(self, attr: &Atom) -> Vec<&'a str>;
    fn get_link(self) -> Option<&'a str>;
    fn get_local_name(self) -> &'a Atom;
    fn get_namespace(self) -> &'a Namespace;
    fn get_hover_state(self) -> bool;
    fn get_id(self) -> Option<Atom>;
    fn get_disabled_state(self) -> bool;
    fn get_enabled_state(self) -> bool;
    fn get_checked_state(self) -> bool;
    fn get_indeterminate_state(self) -> bool;
    fn has_class(self, name: &Atom) -> bool;
    fn has_nonzero_border(self) -> bool;

    // Ordinarily I wouldn't use callbacks like this, but the alternative is
    // really messy, since there is a `JSRef` and a `RefCell` involved. Maybe
    // in the future when we have associated types and/or a more convenient
    // JS GC story... --pcwalton
    fn each_class<F>(self, callback: F) where F: FnMut(&Atom);
}

pub trait TElementAttributes : Copy {
    fn get_length_attribute(self, attribute: LengthAttribute) -> LengthOrPercentageOrAuto;
    fn get_integer_attribute(self, attribute: IntegerAttribute) -> Option<i32>;
    fn get_unsigned_integer_attribute(self, attribute: UnsignedIntegerAttribute) -> Option<u32>;
    fn get_simple_color_attribute(self, attribute: SimpleColorAttribute) -> Option<RGBA>;
}
