/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::NodeListBinding;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::node::AbstractNode;
use dom::window::Window;

enum NodeListType {
    Simple(~[AbstractNode]),
    Children(AbstractNode)
}

pub struct NodeList {
    list_type: NodeListType,
    reflector_: Reflector,
    window: @mut Window,
}

impl NodeList {
    pub fn new_inherited(window: @mut Window,
                         list_type: NodeListType) -> NodeList {
        NodeList {
            list_type: list_type,
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: @mut Window,
               list_type: NodeListType) -> @mut NodeList {
        reflect_dom_object(@mut NodeList::new_inherited(window, list_type),
                           window, NodeListBinding::Wrap)
    }

    pub fn new_simple_list(window: @mut Window, elements: ~[AbstractNode]) -> @mut NodeList {
        NodeList::new(window, Simple(elements))
    }

    pub fn new_child_list(window: @mut Window, node: AbstractNode) -> @mut NodeList {
        NodeList::new(window, Children(node))
    }

    pub fn Length(&self) -> u32 {
        match self.list_type {
            Simple(ref elems) => elems.len() as u32,
            Children(ref node) => node.children().len() as u32
        }
    }

    pub fn Item(&self, index: u32) -> Option<AbstractNode> {
        match self.list_type {
            _ if index >= self.Length() => None,
            Simple(ref elems) => Some(elems[index]),
            Children(ref node) => node.children().nth(index as uint)
        }
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<AbstractNode> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

impl Reflectable for NodeList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
