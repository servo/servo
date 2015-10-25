/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::FormDataBinding;
use dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use dom::bindings::codegen::UnionTypes::FileOrString;
use dom::bindings::codegen::UnionTypes::FileOrString::{eFile, eString};
use dom::bindings::conversions::Castable;
use dom::bindings::error::{Fallible};
use dom::bindings::global::{GlobalField, GlobalRef};
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::blob::Blob;
use dom::file::File;
use dom::htmlformelement::HTMLFormElement;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use util::str::DOMString;

#[derive(JSTraceable, Clone)]
#[must_root]
#[derive(HeapSizeOf)]
pub enum FormDatum {
    StringData(DOMString),
    FileData(JS<File>)
}

#[dom_struct]
pub struct FormData {
    reflector_: Reflector,
    data: DOMRefCell<HashMap<DOMString, Vec<FormDatum>>>,
    global: GlobalField,
    form: Option<JS<HTMLFormElement>>
}

impl FormData {
    fn new_inherited(form: Option<&HTMLFormElement>, global: GlobalRef) -> FormData {
        FormData {
            reflector_: Reflector::new(),
            data: DOMRefCell::new(HashMap::new()),
            global: GlobalField::from_rooted(&global),
            form: form.map(|f| JS::from_ref(f)),
        }
    }

    pub fn new(form: Option<&HTMLFormElement>, global: GlobalRef) -> Root<FormData> {
        reflect_dom_object(box FormData::new_inherited(form, global),
                           global, FormDataBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef, form: Option<&HTMLFormElement>) -> Fallible<Root<FormData>> {
        Ok(FormData::new(form, global))
    }
}

impl FormDataMethods for FormData {
    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append(&self, name: DOMString, value: &Blob, filename: Option<DOMString>) {
        let file = FormDatum::FileData(JS::from_rooted(&self.get_file_from_blob(value, filename)));
        let mut data = self.data.borrow_mut();
        match data.entry(name) {
            Occupied(entry) => entry.into_mut().push(file),
            Vacant(entry) => {
                entry.insert(vec!(file));
            }
        }
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append_(&self, name: DOMString, value: DOMString) {
        let mut data = self.data.borrow_mut();
        match data.entry(name) {
            Occupied(entry) => entry.into_mut().push(FormDatum::StringData(value)),
            Vacant  (entry) => { entry.insert(vec!(FormDatum::StringData(value))); },
        }
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-delete
    fn Delete(&self, name: DOMString) {
        self.data.borrow_mut().remove(&name);
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-get
    fn Get(&self, name: DOMString) -> Option<FileOrString> {
        self.data.borrow()
                 .get(&name)
                 .map(|entry| match entry[0] {
                     FormDatum::StringData(ref s) => eString(s.clone()),
                     FormDatum::FileData(ref f) => eFile(f.root()),
                 })
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-has
    fn Has(&self, name: DOMString) -> bool {
        self.data.borrow().contains_key(&name)
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set_(&self, name: DOMString, value: DOMString) {
        self.data.borrow_mut().insert(name, vec!(FormDatum::StringData(value)));
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set(&self, name: DOMString, value: &Blob, filename: Option<DOMString>) {
        let file = FormDatum::FileData(JS::from_rooted(&self.get_file_from_blob(value, filename)));
        self.data.borrow_mut().insert(name, vec!(file));
    }
}


impl FormData {
    fn get_file_from_blob(&self, value: &Blob, filename: Option<DOMString>) -> Root<File> {
        let global = self.global.root();
        let f = value.downcast::<File>();
        let name = filename.unwrap_or(f.map(|inner| inner.name().clone()).unwrap_or("blob".to_owned()));
        File::new(global.r(), value, name)
    }
}
