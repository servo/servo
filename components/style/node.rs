/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use selectors::AttrSelector;
use servo_util::atom::Atom;
use servo_util::namespace::Namespace;


pub trait TNode<E:TElement> : Clone {
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
    fn is_dirty(&self) -> bool;
    unsafe fn set_is_dirty(&self, value: bool);
    fn has_dirty_descendants(&self) -> bool;
    unsafe fn set_has_dirty_descendants(&self, value: bool);
    fn is_fragment(&self) -> bool;
    unsafe fn set_is_fragment(&self, value: bool);
    fn has_fragment_children(&self) -> bool;
    unsafe fn set_has_fragment_children(&self, value: bool);
}

pub trait TElement {
    fn get_attr(&self, namespace: &Namespace, attr: &str) -> Option<&'static str>;
    fn get_link(&self) -> Option<&'static str>;
    fn get_local_name<'a>(&'a self) -> &'a Atom;
    fn get_namespace<'a>(&'a self) -> &'a Namespace;
    fn get_hover_state(&self) -> bool;
    fn get_id(&self) -> Option<Atom>;
    fn get_disabled_state(&self) -> bool;
    fn get_enabled_state(&self) -> bool;
    fn has_class(&self, name: &str) -> bool;
}
