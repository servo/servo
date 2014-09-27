/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use selectors::AttrSelector;
use servo_util::atom::Atom;
use servo_util::namespace::Namespace;


pub trait TNode<'a, E: TElement<'a>> : Clone {
    fn parent_node(&self) -> Option<Self>;
    /// Name is prefixed to avoid a conflict with TLayoutNode.
    fn tnode_first_child(&self) -> Option<Self>;
    fn prev_sibling(&self) -> Option<Self>;
    fn next_sibling(&self) -> Option<Self>;
    fn is_document(&self) -> bool;
    fn is_element(&self) -> bool;
    fn as_element(&self) -> E;
    fn match_attr(&self, attr: &AttrSelector, test: |&str| -> bool) -> bool;
    fn is_html_element_in_html_document(&self) -> bool;
}

pub trait TElement<'a> {
    fn get_attr(&self, namespace: &Namespace, attr: &str) -> Option<&'a str>;
    fn get_link(&self) -> Option<&'a str>;
    fn get_local_name<'b>(&'b self) -> &'b Atom;
    fn get_namespace<'b>(&'b self) -> &'b Namespace;
    fn get_hover_state(&self) -> bool;
    fn get_id(&self) -> Option<Atom>;
    fn get_disabled_state(&self) -> bool;
    fn get_enabled_state(&self) -> bool;
    fn has_class(&self, name: &str) -> bool;
}
