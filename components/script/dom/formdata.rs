/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataWrap;
use crate::dom::bindings::codegen::UnionTypes::FileOrUSVString;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::{Blob, BlobImpl};
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlformelement::{FormDatum, FormDatumValue, HTMLFormElement};
use dom_struct::dom_struct;
use html5ever::LocalName;

#[dom_struct]
pub struct FormData {
    reflector_: Reflector,
    data: DomRefCell<Vec<(LocalName, FormDatum)>>,
}

impl FormData {
    fn new_inherited(opt_form: Option<&HTMLFormElement>) -> FormData {
        let data = match opt_form {
            Some(form) => form
                .get_form_dataset(None)
                .iter()
                .map(|datum| (LocalName::from(datum.name.as_ref()), datum.clone()))
                .collect::<Vec<(LocalName, FormDatum)>>(),
            None => Vec::new(),
        };

        FormData {
            reflector_: Reflector::new(),
            data: DomRefCell::new(data),
        }
    }

    pub fn new(form: Option<&HTMLFormElement>, global: &GlobalScope) -> DomRoot<FormData> {
        reflect_dom_object(
            Box::new(FormData::new_inherited(form)),
            global,
            FormDataWrap,
        )
    }

    pub fn Constructor(
        global: &GlobalScope,
        form: Option<&HTMLFormElement>,
    ) -> Fallible<DomRoot<FormData>> {
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

        self.data
            .borrow_mut()
            .push((LocalName::from(name.0), datum));
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let datum = FormDatum {
            ty: DOMString::from("file"),
            name: DOMString::from(name.0.clone()),
            value: FormDatumValue::File(DomRoot::from_ref(&*self.create_an_entry(blob, filename))),
        };

        self.data
            .borrow_mut()
            .push((LocalName::from(name.0), datum));
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-delete
    fn Delete(&self, name: USVString) {
        self.data
            .borrow_mut()
            .retain(|(datum_name, _)| datum_name != &LocalName::from(name.0.clone()));
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-get
    fn Get(&self, name: USVString) -> Option<FileOrUSVString> {
        self.data
            .borrow()
            .iter()
            .filter(|(datum_name, _)| datum_name == &LocalName::from(name.0.clone()))
            .next()
            .map(|(_, datum)| match &datum.value {
                FormDatumValue::String(ref s) => {
                    FileOrUSVString::USVString(USVString(s.to_string()))
                },
                FormDatumValue::File(ref b) => FileOrUSVString::File(DomRoot::from_ref(&*b)),
            })
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-getall
    fn GetAll(&self, name: USVString) -> Vec<FileOrUSVString> {
        self.data
            .borrow()
            .iter()
            .filter_map(|datum| {
                if datum.0 != LocalName::from(name.0.clone()) {
                    return None;
                }

                Some(match &datum.1.value {
                    FormDatumValue::String(ref s) => {
                        FileOrUSVString::USVString(USVString(s.to_string()))
                    },
                    FormDatumValue::File(ref b) => FileOrUSVString::File(DomRoot::from_ref(&*b)),
                })
            })
            .collect()
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-has
    fn Has(&self, name: USVString) -> bool {
        self.data
            .borrow()
            .iter()
            .filter(|(datum_name, _0)| datum_name == &LocalName::from(name.0.clone()))
            .count() >
            0
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set(&self, name: USVString, str_value: USVString) {
        let mut data = self.data.borrow_mut();
        let local_name = LocalName::from(name.0.clone());

        data.retain(|(datum_name, _)| datum_name != &local_name);

        data.push((
            local_name,
            FormDatum {
                ty: DOMString::from("string"),
                name: DOMString::from(name.0),
                value: FormDatumValue::String(DOMString::from(str_value.0)),
            },
        ));
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let mut data = self.data.borrow_mut();
        let local_name = LocalName::from(name.0.clone());

        data.retain(|(datum_name, _)| datum_name != &local_name);

        data.push((
            LocalName::from(name.0.clone()),
            FormDatum {
                ty: DOMString::from("file"),
                name: DOMString::from(name.0),
                value: FormDatumValue::File(DomRoot::from_ref(
                    &*self.create_an_entry(blob, filename),
                )),
            },
        ));
    }
}

impl FormData {
    // https://xhr.spec.whatwg.org/#create-an-entry
    // Steps 3-4.
    fn create_an_entry(&self, blob: &Blob, opt_filename: Option<USVString>) -> DomRoot<File> {
        let name = match opt_filename {
            Some(filename) => DOMString::from(filename.0),
            None if blob.downcast::<File>().is_none() => DOMString::from("blob"),
            None => DOMString::from(""),
        };

        let bytes = blob.get_bytes().unwrap_or(vec![]);

        File::new(
            &self.global(),
            BlobImpl::new_from_bytes(bytes),
            name,
            None,
            &blob.type_string(),
        )
    }

    pub fn datums(&self) -> Vec<FormDatum> {
        self.data
            .borrow()
            .iter()
            .map(|(_, datum)| datum.clone())
            .collect()
    }
}

impl Iterable for FormData {
    type Key = USVString;
    type Value = FileOrUSVString;

    fn get_iterable_length(&self) -> u32 {
        self.data.borrow().len() as u32
    }

    fn get_value_at_index(&self, n: u32) -> FileOrUSVString {
        let data = self.data.borrow();
        let datum = &data.get(n as usize).unwrap().1;
        match &datum.value {
            FormDatumValue::String(ref s) => FileOrUSVString::USVString(USVString(s.to_string())),
            FormDatumValue::File(ref b) => FileOrUSVString::File(DomRoot::from_ref(b)),
        }
    }

    fn get_key_at_index(&self, n: u32) -> USVString {
        let data = self.data.borrow();
        let key = &data.get(n as usize).unwrap().0;
        USVString(key.to_string())
    }
}
