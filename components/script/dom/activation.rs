/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::HTMLInputElementCast;
use dom::bindings::js::JSRef;
use dom::element::HTMLInputElementTypeId;
use dom::htmlinputelement::HTMLInputElement;
use dom::node::{ElementNodeTypeId, Node, NodeHelpers};

pub trait Activatable {}


/// Obtain an Activatable instance for a given Node-derived object,
/// if it is activatable
pub fn activation_vtable_for<'a>(node: &'a JSRef<'a, Node>) -> Option<&'a Activatable + 'a> {
    match node.type_id() {
        ElementNodeTypeId(HTMLInputElementTypeId) => {
            let _element: &'a JSRef<'a, HTMLInputElement> = HTMLInputElementCast::to_borrowed_ref(node).unwrap();
            // Some(element as &'a VirtualMethods + 'a)
            None
        },
        _ => {
            None
        }
    }
}
