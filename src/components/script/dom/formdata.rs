/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FormDataBinding;
use dom::bindings::error::{Fallible};
use dom::bindings::js::{JS, JSRef, Temporary, OptionalUnrootable};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::blob::Blob;
use dom::htmlformelement::HTMLFormElement;
use dom::window::Window;
use servo_util::str::DOMString;
use collections::hashmap::HashMap;
use std::cell::RefCell;

#[deriving(Encodable)]
pub enum FormDatum {
    StringData(DOMString),
    BlobData { blob: JS<Blob>, name: DOMString }
}

#[deriving(Encodable)]
pub struct FormData {
    pub data: Traceable<RefCell<HashMap<DOMString, FormDatum>>>,
    pub reflector_: Reflector,
    pub window: JS<Window>,
    pub form: Option<JS<HTMLFormElement>>
}

impl FormData {
    pub fn new_inherited(form: Option<JSRef<HTMLFormElement>>, window: &JSRef<Window>) -> FormData {
        FormData {
            data: Traceable::new(RefCell::new(HashMap::new())),
            reflector_: Reflector::new(),
            window: JS::from_rooted(window),
            form: form.unrooted(),
        }
    }

    pub fn new(form: Option<JSRef<HTMLFormElement>>, window: &JSRef<Window>) -> Temporary<FormData> {
        reflect_dom_object(box FormData::new_inherited(form, window), window, FormDataBinding::Wrap)
    }

    pub fn Constructor(window: &JSRef<Window>, form: Option<JSRef<HTMLFormElement>>) -> Fallible<Temporary<FormData>> {
        Ok(FormData::new(form, window))
    }
}

pub trait FormDataMethods {
    fn Append(&self, name: DOMString, value: &JSRef<Blob>, filename: Option<DOMString>);
    fn Append_(&self, name: DOMString, value: DOMString);
}

impl<'a> FormDataMethods for JSRef<'a, FormData> {
    fn Append(&self, name: DOMString, value: &JSRef<Blob>, filename: Option<DOMString>) {
        let blob = BlobData {
            blob: JS::from_rooted(value),
            name: filename.unwrap_or("default".to_string())
        };
        self.data.deref().borrow_mut().insert(name.clone(), blob);
    }

    fn Append_(&self, name: DOMString, value: DOMString) {
        self.data.deref().borrow_mut().insert(name, StringData(value));
    }
}

impl Reflectable for FormData {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
