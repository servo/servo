/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between script and
//! style.

/// FIXME(pcwalton): Should not require `Clone` and should instead return references. When this
/// happens this will need to only be implemented for `AbstractNode<LayoutView>`.
pub trait TNode<E:TElement> : Clone {
    fn parent_node(&self) -> Option<Self>;
    fn prev_sibling(&self) -> Option<Self>;
    fn next_sibling(&self) -> Option<Self>;

    fn is_document(&self) -> bool;
    fn is_element(&self) -> bool;

    /// FIXME(pcwalton): This should not use the `with` pattern.
    fn with_element<R>(&self, f: &fn(&E) -> R) -> R;
}

pub trait TElement {
    fn get_attr(&self, namespace: Option<~str>, attr: &str) -> Option<~str>;
    fn get_link(&self) -> Option<~str>;
    fn get_local_name<'a>(&'a self) -> &'a str;
    fn get_namespace_url<'a>(&'a self) -> &'a str;
}

