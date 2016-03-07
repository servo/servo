/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::FormDataBinding;
use dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use dom::bindings::codegen::UnionTypes::BlobOrUSVString;
use dom::bindings::error::{Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::USVString;
use dom::blob::Blob;
use dom::file::File;
use dom::htmlformelement::HTMLFormElement;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use string_cache::Atom;
use util::str::DOMString;

#[derive(JSTraceable, Clone)]
#[must_root]
#[derive(HeapSizeOf)]
pub enum FormDatum {
    StringData(String),
    BlobData(JS<Blob>)
}

#[dom_struct]
pub struct FormData {
    reflector_: Reflector,
    data: DOMRefCell<HashMap<Atom, Vec<FormDatum>>>,
    form: Option<JS<HTMLFormElement>>
}

impl FormData {
    fn new_inherited(form: Option<&HTMLFormElement>) -> FormData {
        FormData {
            reflector_: Reflector::new(),
            data: DOMRefCell::new(HashMap::new()),
            form: form.map(|f| JS::from_ref(f)),
        }
    }

    pub fn new(form: Option<&HTMLFormElement>, global: GlobalRef) -> Root<FormData> {
        reflect_dom_object(box FormData::new_inherited(form),
                           global, FormDataBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef, form: Option<&HTMLFormElement>) -> Fallible<Root<FormData>> {
        // TODO: Construct form data set for form if it is supplied
        Ok(FormData::new(form, global))
    }
}

impl FormDataMethods for FormData {
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append(&self, name: USVString, value: USVString) {
        let mut data = self.data.borrow_mut();
        match data.entry(Atom::from(name.0)) {
            Occupied(entry) => entry.into_mut().push(FormDatum::StringData(value.0)),
            Vacant  (entry) => { entry.insert(vec!(FormDatum::StringData(value.0))); }
        }
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append_(&self, name: USVString, value: &Blob, filename: Option<USVString>) {
        let blob = FormDatum::BlobData(JS::from_rooted(&self.get_file_or_blob(value, filename)));
        let mut data = self.data.borrow_mut();
        match data.entry(Atom::from(name.0)) {
            Occupied(entry) => entry.into_mut().push(blob),
            Vacant(entry) => {
                entry.insert(vec!(blob));
            }
        }
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-delete
    fn Delete(&self, name: USVString) {
        self.data.borrow_mut().remove(&Atom::from(name.0));
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-get
    fn Get(&self, name: USVString) -> Option<BlobOrUSVString> {
        self.data.borrow()
                 .get(&Atom::from(name.0))
                 .map(|entry| match entry[0] {
                     FormDatum::StringData(ref s) => BlobOrUSVString::USVString(USVString(s.clone())),
                     FormDatum::BlobData(ref b) => BlobOrUSVString::Blob(Root::from_ref(&*b)),
                 })
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-getall
    fn GetAll(&self, name: USVString) -> Vec<BlobOrUSVString> {
        self.data.borrow()
                 .get(&Atom::from(name.0))
                 .map_or(vec![], |data|
                    data.iter().map(|item| match *item {
                        FormDatum::StringData(ref s) => BlobOrUSVString::USVString(USVString(s.clone())),
                        FormDatum::BlobData(ref b) => BlobOrUSVString::Blob(Root::from_ref(&*b)),
                    }).collect()
                 )
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-has
    fn Has(&self, name: USVString) -> bool {
        self.data.borrow().contains_key(&Atom::from(name.0))
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set(&self, name: USVString, value: BlobOrUSVString) {
        let val = match value {
            BlobOrUSVString::USVString(s) => FormDatum::StringData(s.0),
            BlobOrUSVString::Blob(b) => FormDatum::BlobData(JS::from_rooted(&b))
        };
        self.data.borrow_mut().insert(Atom::from(name.0), vec!(val));
    }
}


impl FormData {
    fn get_file_or_blob(&self, value: &Blob, filename: Option<USVString>) -> Root<Blob> {
        match filename {
            Some(fname) => {
                let global = self.global();
                let name = DOMString::from(fname.0);
                Root::upcast(File::new(global.r(), value, name))
            }
            None => Root::from_ref(value)
        }
    }
}
