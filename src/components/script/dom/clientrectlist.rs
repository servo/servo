/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::ClientRectListBinding;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::clientrect::ClientRect;
use script_task::page_from_context;

use js::jsapi::{JSObject, JSContext};

pub struct ClientRectList {
    reflector_: Reflector,
    rects: ~[@mut ClientRect]
}

impl ClientRectList {
    pub fn new(rects: ~[@mut ClientRect], cx: *JSContext, scope: *JSObject) -> @mut ClientRectList {
        let list = @mut ClientRectList {
            reflector_: Reflector::new(),
            rects: rects
        };
        list.init_wrapper(cx, scope);
        list
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }

    pub fn Length(&self) -> u32 {
        self.rects.len() as u32
    }

    pub fn Item(&self, index: u32) -> Option<@mut ClientRect> {
        if index < self.rects.len() as u32 {
            Some(self.rects[index])
        } else {
            None
        }
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<@mut ClientRect> {
        *found = index < self.rects.len() as u32;
        self.Item(index)
    }
}

impl Reflectable for ClientRectList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        ClientRectListBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}
