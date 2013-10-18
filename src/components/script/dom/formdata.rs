/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::utils::{DOMString, null_str_as_empty};
use dom::bindings::codegen::FormDataBinding;
use dom::blob::Blob;
use script_task::{page_from_context};

use js::jsapi::{JSObject, JSContext};

use std::hashmap::HashMap;

enum FormDatum {
    StringData(DOMString),
    BlobData { blob: @mut Blob, name: DOMString }
}

pub struct FormData {
    data: HashMap<~str, FormDatum>,
    reflector_: Reflector
}

impl FormData {
    pub fn new() -> @mut FormData {
        @mut FormData {
            data: HashMap::new(),
            reflector_: Reflector::new()
        }
    }

    pub fn init_wrapper(@mut self, cx: *JSContext, scope: *JSObject) {
        self.wrap_object_shared(cx, scope);
    }

    pub fn Append(&mut self, name: &DOMString, value: @mut Blob, filename: Option<DOMString>) {
        let blob = BlobData {
            blob: value,
            name: filename.unwrap_or_default(Some(~"default"))
        };
        self.data.insert(null_str_as_empty(name), blob);
    }

    pub fn Append_(&mut self, name: &DOMString, value: &DOMString) {
        self.data.insert(null_str_as_empty(name), StringData((*value).clone()));
    }
}

impl Reflectable for FormData {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        FormDataBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut Reflectable)
        }
    }
}
