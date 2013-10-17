/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::NodeListBinding;
use dom::bindings::utils::{Reflectable, BindingObject, Reflector};
use dom::node::{AbstractNode, ScriptView};
use script_task::page_from_context;

use js::jsapi::{JSObject, JSContext};

enum NodeListType {
    Simple(~[AbstractNode<ScriptView>]),
    Children(AbstractNode<ScriptView>)
}

pub struct NodeList {
    list_type: NodeListType,
    reflector_: Reflector
}

impl NodeList {
    pub fn new_simple_list(elements: ~[AbstractNode<ScriptView>], cx: *JSContext, scope: *JSObject) -> @mut NodeList {
        let list = @mut NodeList {
            list_type: Simple(elements),
            reflector_: Reflector::new()
        };

        list.init_wrapper(cx, scope);
        list
    }

    pub fn new_child_list(node: AbstractNode<ScriptView>, cx: *JSContext, scope: *JSObject) -> @mut NodeList {
        let list = @mut NodeList {
            list_type: Children(node),
            reflector_: Reflector::new()
        };

        list.init_wrapper(cx, scope);
        list
    }

    fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }

    pub fn Length(&self) -> u32 {
        match self.list_type {
            Simple(ref elems) => elems.len() as u32,
            Children(ref node) => node.children().len() as u32
        }
    }

    pub fn Item(&self, index: u32) -> Option<AbstractNode<ScriptView>> {
        match self.list_type {
            _ if index >= self.Length() => None,
            Simple(ref elems) => Some(elems[index]),
            Children(ref node) => node.children().nth(index as uint)
        }
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<AbstractNode<ScriptView>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

impl BindingObject for NodeList {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}

impl Reflectable for NodeList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        NodeListBinding::Wrap(cx, scope, self)
    }
}
