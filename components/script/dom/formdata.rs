/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::FormDataBinding;
use dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use dom::bindings::codegen::UnionTypes::FileOrUSVString;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::blob::{Blob, BlobImpl};
use dom::file::File;
use dom::htmlformelement::{HTMLFormElement, FormDatumValue, FormDatum};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use string_cache::Atom;

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
    fn Append(&self, name: USVString, str_value: USVString) {
        let datum = FormDatum {
            ty: DOMString::from("string"),
            name: DOMString::from(name.0.clone()),
            value: FormDatumValue::String(DOMString::from(str_value.0)),
        };

        let mut data = self.data.borrow_mut();
        match data.entry(Atom::from(name.0)) {
            Occupied(entry) => entry.into_mut().push(datum),
            Vacant(entry) => { entry.insert(vec!(datum)); }
        }
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let datum = FormDatum {
            ty: DOMString::from("file"),
            name: DOMString::from(name.0.clone()),
            value: FormDatumValue::File(Root::from_ref(&*self.get_file(blob, filename))),
        };

        let mut data = self.data.borrow_mut();

        match data.entry(Atom::from(name.0)) {
            Occupied(entry) => entry.into_mut().push(datum),
            Vacant(entry) => { entry.insert(vec!(datum)); },
        }
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-delete
    fn Delete(&self, name: USVString) {
        self.data.borrow_mut().remove(&Atom::from(name.0));
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-get
    fn Get(&self, name: USVString) -> Option<FileOrUSVString> {
        self.data.borrow()
                 .get(&Atom::from(name.0))
                 .map(|entry| match entry[0].value {
                     FormDatumValue::String(ref s) => FileOrUSVString::USVString(USVString(s.to_string())),
                     FormDatumValue::File(ref b) => FileOrUSVString::File(Root::from_ref(&*b)),
                 })
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-getall
    fn GetAll(&self, name: USVString) -> Vec<FileOrUSVString> {
        self.data.borrow()
                 .get(&Atom::from(name.0))
                 .map_or(vec![], |data|
                    data.iter().map(|item| match item.value {
                        FormDatumValue::String(ref s) => FileOrUSVString::USVString(USVString(s.to_string())),
                        FormDatumValue::File(ref b) => FileOrUSVString::File(Root::from_ref(&*b)),
                    }).collect()
                 )
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-has
    fn Has(&self, name: USVString) -> bool {
        self.data.borrow().contains_key(&Atom::from(name.0))
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set(&self, name: USVString, str_value: USVString) {
        self.data.borrow_mut().insert(Atom::from(name.0.clone()), vec![FormDatum {
            ty: DOMString::from("string"),
            name: DOMString::from(name.0),
            value: FormDatumValue::String(DOMString::from(str_value.0)),
        }]);
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        self.data.borrow_mut().insert(Atom::from(name.0.clone()), vec![FormDatum {
            ty: DOMString::from("file"),
            name: DOMString::from(name.0),
            value: FormDatumValue::File(Root::from_ref(&*self.get_file(blob, filename))),
        }]);
    }

}


impl FormData {
    fn get_file(&self, blob: &Blob, opt_filename: Option<USVString>) -> Root<File> {
        let global = self.global();

        let name = match opt_filename {
            Some(filename) => DOMString::from(filename.0),
            None => DOMString::from(""),
        };

        let bytes = blob.get_bytes().unwrap_or(vec![]);

        File::new(global.r(), BlobImpl::new_from_bytes(bytes), name, None, "")
    }

    pub fn datums(&self) -> Vec<FormDatum> {
        let mut ret = vec![];
        for values in self.data.borrow().values() {
            ret.append(&mut values.clone());
        }

        ret
    }
}
