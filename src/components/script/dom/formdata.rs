/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::DOMString;
use dom::bindings::codegen::FormDataBinding;
use dom::blob::Blob;
use dom::window::Window;

use std::hashmap::HashMap;

enum FormDatum {
    StringData(DOMString),
    BlobData { blob: @mut Blob, name: DOMString }
}

pub struct FormData {
    data: HashMap<~str, FormDatum>,
    reflector_: Reflector,
    window: @mut Window,
}

impl FormData {
    pub fn new_inherited(window: @mut Window) -> FormData {
        FormData {
            data: HashMap::new(),
            reflector_: Reflector::new(),
            window: window,
        }
    }

    pub fn new(window: @mut Window) -> @mut FormData {
        reflect_dom_object(@mut FormData::new_inherited(window), window, FormDataBinding::Wrap)
    }

    pub fn Append(&mut self, name: DOMString, value: @mut Blob, filename: Option<DOMString>) {
        let blob = BlobData {
            blob: value,
            name: filename.unwrap_or(~"default")
        };
        self.data.insert(name.clone(), blob);
    }

    pub fn Append_(&mut self, name: DOMString, value: DOMString) {
        self.data.insert(name, StringData(value));
    }
}

impl Reflectable for FormData {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
