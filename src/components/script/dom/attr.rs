/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::AttrBinding;
use dom::bindings::utils::{null_string, str};
use dom::bindings::utils::{BindingObject, CacheableWrapper, DOMString, WrapperCache};
use dom::namespace;
use dom::namespace::Namespace;
use script_task::{page_from_context};

use js::jsapi::{JSObject, JSContext};

use std::cast;
use std::str::eq_slice;

pub struct Attr {
    wrapper: WrapperCache,
    priv local_name: Option<~str>,
    value: DOMString,
    name: ~str,
    namespace: Namespace,
    prefix: DOMString
}

impl CacheableWrapper for Attr {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&mut self.wrapper) }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        AttrBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for Attr {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        let page = page_from_context(cx);
        unsafe {
            Some((*page).frame.get_ref().window as @mut CacheableWrapper)
        }
    }
}

impl Attr {
    pub fn new(name: ~str, value: ~str) -> Attr {
        Attr {
            wrapper: WrapperCache::new(),
            local_name: None, //Only store local_name if it is different from name
            value: str(value),
            name: name, //TODO: Atomise attribute names
            namespace: namespace::Null,
            prefix: null_string
        }
    }

    pub fn new_ns(local_name: ~str, value: ~str,  name: ~str, namespace: Namespace, prefix: Option<~str>) -> Attr {
        
        Attr {
            wrapper: WrapperCache::new(),
            local_name: if eq_slice(local_name, name) {None} else {Some(local_name)},
            value: str(value),
            name: name,
            namespace: namespace,
            prefix: match prefix {Some(x) => str(x), None => null_string}
        }
    }

    pub fn local_name(&self) -> ~str {
        match self.local_name {
            Some(ref x) => x.to_owned(),
            None => self.name.clone()
        }
    }

    pub fn LocalName(&self) -> DOMString {
        str(self.local_name())
    }

    pub fn Value(&self) -> DOMString {
        self.value.clone()
    }

    pub fn SetValue(&mut self, value: &DOMString) {
        self.value = (*value).clone()
    }

    pub fn Name(&self) -> DOMString {
        str(self.name.clone())
    }

    pub fn GetNamespaceURI(&self) -> DOMString {
        self.namespace.to_str()
    }

    pub fn GetPrefix(&self) -> DOMString {
        self.prefix.clone()
    }
}
