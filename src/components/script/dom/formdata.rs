/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Fallible, Reflectable, Reflector, reflect_dom_object};
use dom::bindings::codegen::FormDataBinding;
use dom::blob::Blob;
use dom::node::AbstractNode;
use dom::window::Window;
use servo_util::str::DOMString;

use std::hashmap::HashMap;

enum FormDatum {
    StringData(DOMString),
    BlobData { blob: @mut Blob, name: DOMString }
}

pub struct FormData {
    data: HashMap<DOMString, FormDatum>,
    reflector_: Reflector,
    window: @mut Window,
    form: Option<AbstractNode>
}

impl FormData {
    pub fn new_inherited(form: Option<AbstractNode>, window: @mut Window) -> FormData {
        FormData {
            data: HashMap::new(),
            reflector_: Reflector::new(),
            window: window,
            form: form
        }
    }

    pub fn new(form: Option<AbstractNode>, window: @mut Window) -> @mut FormData {
        reflect_dom_object(@mut FormData::new_inherited(form, window), window, FormDataBinding::Wrap)
    }

    pub fn Constructor(window: @mut Window, form: Option<AbstractNode>)
                       -> Fallible<@mut FormData> {
        Ok(FormData::new(form, window))
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
