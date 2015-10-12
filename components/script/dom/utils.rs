/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::codegen::InheritTypes::HTMLElementBase;
use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::NodeBase;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::Error;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::utils::Reflectable;
use dom::element::Element;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use std::iter;

/// Used by `HTMLTableSectionElement::InsertRow` and `HTMLTableRowElement::InsertCell`
pub fn insert_cell_or_row<T, F, G, I>(this: &T, index: i32, get_items: F, new_child: G) -> Fallible<Root<HTMLElement>>
        where T: NodeBase + HTMLElementBase + Reflectable,
              F: Fn() -> Root<HTMLCollection>,
              G: Fn(&Node) -> Root<I>,
              I: NodeBase + HTMLElementBase + Reflectable,
{
    if index < -1 {
        return Err(Error::IndexSize);
    }

    let node = NodeCast::from_ref(this);
    let tr = new_child(node);

    let after_node = if index == -1 {
        None
    } else {
        match get_items().elements_iter()
                         .map(NodeCast::from_root)
                         .map(Some)
                         .chain(iter::once(None))
                         .nth(index as usize) {
            None => return Err(Error::IndexSize),
            Some(node) => node,
        }
    };

    {
        let tr_node = NodeCast::from_ref(tr.r());
        try!(node.InsertBefore(tr_node, after_node.r()));
    }

    Ok(HTMLElementCast::from_root(tr))
}

/// Used by `HTMLTableSectionElement::DeleteRow` and `HTMLTableRowElement::DeleteCell`
pub fn delete_cell_or_row<T, F, G>(this: &T, index: i32, get_items: F, is_delete_type: G) -> ErrorResult
        where T: NodeBase + Reflectable,
              F: Fn() -> Root<HTMLCollection>,
              G: Fn(&Element) -> bool
{
    let element = match index {
        index if index < -1 => return Err(Error::IndexSize),
        -1 => {
            let last_child = NodeCast::from_ref(this).GetLastChild();
            match last_child.and_then(|node| node.inclusively_preceding_siblings()
                                                 .filter_map(ElementCast::to_root)
                                                 .filter(|elem| is_delete_type(elem))
                                                 .next()) {
                Some(element) => element,
                None => return Ok(()),
            }
        },
        index => match get_items().Item(index as u32) {
            Some(element) => element,
            None => return Err(Error::IndexSize),
        },
    };

    NodeCast::from_ref(element.r()).remove_self();
    Ok(())
}
