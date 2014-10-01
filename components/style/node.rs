/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use selectors::AttrSelector;
use string_cache::{Atom, Namespace};


pub trait TNode<'a, E: TElement<'a>> : Clone + Copy {
    fn parent_node(self) -> Option<Self>;
    fn first_child(self) -> Option<Self>;
    fn prev_sibling(self) -> Option<Self>;
    fn next_sibling(self) -> Option<Self>;
    fn is_document(self) -> bool;
    fn is_element(self) -> bool;
    fn as_element(self) -> E;
    fn match_attr(self, attr: &AttrSelector, test: |&str| -> bool) -> bool;
    fn is_html_element_in_html_document(self) -> bool;
}

pub trait TElement<'a> : Copy {
    fn get_attr(self, namespace: &Namespace, attr: &str) -> Option<&'a str>;
    fn get_link(self) -> Option<&'a str>;
    fn get_local_name(self) -> &'a Atom;
    fn get_namespace(self) -> &'a Namespace;
    fn get_hover_state(self) -> bool;
    fn get_id(self) -> Option<Atom>;
    fn get_disabled_state(self) -> bool;
    fn get_enabled_state(self) -> bool;
    fn has_class(self, name: &str) -> bool;
}
